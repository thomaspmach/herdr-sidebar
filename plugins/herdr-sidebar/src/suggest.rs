//! The ✨ commit-message suggestion: ask a configured local AI CLI to summarize
//! the pending diff (like VS Code's sparkle button), falling back to a
//! filename-based heuristic when the CLI is missing, slow, or fails. Runs on a
//! background thread so the TUI stays responsive; the app polls the returned
//! channel from its refresh tick.

use serde::Deserialize;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// Cap the diff sent to the model — huge diffs only slow generation down, and
/// the file list already names everything that changed.
const MAX_DIFF_BYTES: usize = 16 * 1024;

/// How long to wait for an AI CLI before killing it and falling back.
const TIMEOUT: Duration = Duration::from_secs(120);

const PROMPT: &str = "Write a git commit message for the diff on stdin: one imperative \
                      subject line under 72 characters, no quotes, no trailing period. \
                      Reply with ONLY the message line.";

/// Spawn generation for `diff`/`files`; the result arrives on the channel.
/// Always yields exactly one message (the fallback is used on any failure).
pub fn spawn(diff: String, files: Vec<String>) -> Receiver<String> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let message = generate(&diff, &files);
        let _ = tx.send(message);
    });
    rx
}

/// Optional per-user settings in the plugin state directory, re-read for each draft.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    cli: Cli,
    model: Option<String>,
    reasoning_effort: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Cli {
    #[default]
    Claude,
    Codex,
}

impl Settings {
    fn load() -> Result<Self, String> {
        let Some(dir) = crate::state::state_dir() else {
            return Ok(Self::default());
        };
        match std::fs::read(dir.join("commit-ai.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn command(&self) -> Result<Command, String> {
        match self.cli {
            Cli::Claude => {
                let mut command = Command::new("claude");
                command.args([
                    "-p",
                    "--model",
                    self.model.as_deref().unwrap_or("haiku"),
                    "--strict-mcp-config",
                    PROMPT,
                ]);
                Ok(command)
            }
            Cli::Codex => {
                let effort = self.reasoning_effort.as_deref().unwrap_or("medium");
                if !["minimal", "low", "medium", "high", "xhigh"].contains(&effort) {
                    return Err("Unsupported reasoning_effort".into());
                }
                let mut command = Command::new("codex");
                command.args([
                    "exec",
                    "--ignore-user-config",
                    "--ignore-rules",
                    "--ephemeral",
                    "--sandbox",
                    "read-only",
                    "--skip-git-repo-check",
                    "--color",
                    "never",
                    "--json",
                    "--model",
                    self.model.as_deref().unwrap_or("gpt-6-luna"),
                    "-c",
                    &format!("model_reasoning_effort=\"{effort}\""),
                    "-c",
                    "project_doc_max_bytes=0",
                    "-c",
                    "features.shell_tool=false",
                    "-c",
                    "features.multi_agent=false",
                    PROMPT,
                ]);
                Ok(command)
            }
        }
    }
}

fn generate(diff: &str, files: &[String]) -> String {
    let settings = match Settings::load() {
        Ok(settings) => settings,
        Err(error) => {
            // Never log the diff, paths or generated message.
            eprintln!("Commit AI configuration failed: {error}");
            return fallback(files);
        }
    };
    let Ok(command) = settings.command() else {
        eprintln!("Commit AI configuration failed: unsupported reasoning effort");
        return fallback(files);
    };
    let message = ask_cli(command, &settings.cli, diff, TIMEOUT);
    #[cfg(windows)]
    let message = message.or_else(|| {
        if matches!(settings.cli, Cli::Claude) {
            let original = settings.command().ok()?;
            let mut shim = Command::new("claude.cmd");
            shim.args(original.get_args());
            ask_cli(shim, &settings.cli, diff, TIMEOUT)
        } else {
            None
        }
    });
    match message {
        Some(message) => message,
        None => {
            eprintln!("Commit AI generation failed or timed out; using filename fallback");
            fallback(files)
        }
    }
}

/// Drain stdout concurrently: Codex JSON events may exceed the pipe capacity.
fn ask_cli(mut command: Command, cli: &Cli, diff: &str, timeout: Duration) -> Option<String> {
    let mut input = String::with_capacity(diff.len().min(MAX_DIFF_BYTES));
    for c in diff.chars() {
        if input.len() + c.len_utf8() > MAX_DIFF_BYTES {
            input.push_str("\n[diff truncated]");
            break;
        }
        input.push(c);
    }
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (output_tx, output_rx) = channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut out = String::new();
        let result = stdout.read_to_string(&mut out).ok().map(|_| out);
        let _ = output_tx.send(result);
    });
    let mut stdin = child.stdin.take()?;
    std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => return None,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let out = output_rx
        .recv_timeout(timeout.saturating_sub(start.elapsed()))
        .ok()??;
    match cli {
        Cli::Claude => clean_reply(&out),
        Cli::Codex => codex_reply(&out),
    }
}

/// Only completed agent messages are eligible; never use events or tool output.
fn codex_reply(raw: &str) -> Option<String> {
    let mut message = None;
    let mut completed = false;
    for line in raw.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match event["type"].as_str() {
            Some("error" | "turn.failed") => return None,
            Some("turn.completed") => completed = true,
            Some("item.completed") if event["item"]["type"] == "agent_message" => {
                message = event["item"]["text"].as_str().and_then(clean_reply);
            }
            _ => {}
        }
    }
    if completed { message } else { None }
}

/// The reply line, stripped of the quoting/fencing chat models sometimes add
/// despite instructions; `None` when nothing usable came back. Startup log
/// noise (MCP warnings and the like) can precede the reply on stdout, so this
/// takes the LAST usable line and drops warning-looking lines outright.
fn clean_reply(raw: &str) -> Option<String> {
    let line = raw.lines().map(str::trim).rfind(|l| {
        let lower = l.to_lowercase();
        !l.is_empty()
            && !l.starts_with("```")
            && !lower.contains("warn")
            && !lower.contains("error")
    })?;
    let line = line
        .trim_matches(['"', '\'', '`'])
        .trim_end_matches('.')
        .trim();
    (!line.is_empty()).then(|| line.to_string())
}

/// Filename-based fallback: good enough to save retyping, honest about scope.
fn fallback(files: &[String]) -> String {
    let name = |path: &String| path.rsplit('/').next().unwrap_or(path).to_string();
    match files {
        [] => "Update".to_string(),
        [only] => format!("Update {}", name(only)),
        [first, rest @ ..] => format!("Update {} and {} more", name(first), rest.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_configuration_preserves_claude_default_and_builds_codex_arguments() {
        let default = Settings::default().command().unwrap();
        assert_eq!(default.get_program(), "claude");
        assert!(default.get_args().any(|a| a == "haiku"));
        let settings: Settings = serde_json::from_str(
            r#"{"cli":"codex","model":"gpt-6-luna","reasoning_effort":"medium"}"#,
        )
        .unwrap();
        let command = settings.command().unwrap();
        assert_eq!(command.get_program(), "codex");
        let args: Vec<_> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert!(args.contains(&"gpt-6-luna"));
        assert!(args.contains(&"model_reasoning_effort=\"medium\""));
        assert!(args.contains(&"read-only"));
        assert!(args.contains(&"--ignore-user-config"));
        assert!(serde_json::from_str::<Settings>(r#"{"cli":"unknown"}"#).is_err());
        let invalid: Settings =
            serde_json::from_str(r#"{"cli":"codex","reasoning_effort":"typo"}"#).unwrap();
        assert!(invalid.command().is_err());
    }

    #[test]
    fn codex_output_uses_only_successful_final_agent_message() {
        let events = concat!(
            "{\"type\":\"item.completed\",\"item\":{\"type\":\"command_execution\",\"text\":\"wrong\"}}\n",
            "{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"Add configurable commit AI\"}}\n"
        );
        assert_eq!(codex_reply(events), None);
        assert_eq!(
            codex_reply(&format!("{events}{{\"type\":\"turn.completed\"}}\n")),
            Some("Add configurable commit AI".into())
        );
        assert_eq!(
            codex_reply(&format!("{events}{{\"type\":\"turn.failed\"}}\n")),
            None
        );
        assert_eq!(codex_reply("{\"type\":\"turn.completed\"}"), None);
    }

    #[cfg(unix)]
    #[test]
    fn codex_process_drains_large_event_stream_and_ignores_tool_output() {
        let mut command = Command::new("sh");
        command.args(["-c", r#"cat >/dev/null; head -c 100000 /dev/zero | tr '\000' x; printf '\n%s\n' '{"type":"item.completed","item":{"type":"command_execution","text":"wrong"}}' '{"type":"item.completed","item":{"type":"agent_message","text":"Add Codex commit drafts"}}' '{"type":"turn.completed"}'"#]);
        assert_eq!(
            ask_cli(command, &Cli::Codex, "diff", Duration::from_secs(5)),
            Some("Add Codex commit drafts".into())
        );
    }

    #[cfg(unix)]
    #[test]
    fn cli_process_handles_success_failure_and_timeout() {
        let mut success = Command::new("sh");
        success.args(["-c", "cat >/dev/null; printf 'Add commit AI settings\\n'"]);
        assert_eq!(
            ask_cli(success, &Cli::Claude, "diff", Duration::from_secs(2)),
            Some("Add commit AI settings".into())
        );
        let mut failure = Command::new("sh");
        failure.args(["-c", "exit 1"]);
        assert_eq!(
            ask_cli(failure, &Cli::Claude, "diff", Duration::from_secs(2)),
            None
        );
        let mut slow = Command::new("sh");
        slow.args(["-c", "exec sleep 5"]);
        assert_eq!(
            ask_cli(slow, &Cli::Claude, "diff", Duration::from_millis(50)),
            None
        );
    }

    #[test]
    fn reply_cleanup_strips_quotes_fences_and_periods() {
        assert_eq!(
            clean_reply("Add sidebar merge\n"),
            Some("Add sidebar merge".into())
        );
        assert_eq!(
            clean_reply("\"Fix the thing.\""),
            Some("Fix the thing".into())
        );
        assert_eq!(
            clean_reply("```\nRefactor launch flow\n```"),
            Some("Refactor launch flow".into())
        );
        assert_eq!(clean_reply("   \n\n"), None);
        // Log noise before (or instead of) the reply must never win.
        assert_eq!(
            clean_reply("RendererWarning resource UID duplicate\nAdd auth docs\n"),
            Some("Add auth docs".into())
        );
        assert_eq!(clean_reply("[WARN] something\nERROR: nope\n"), None);
    }

    #[test]
    fn fallback_names_the_files() {
        assert_eq!(fallback(&[]), "Update");
        assert_eq!(fallback(&["src/app.rs".into()]), "Update app.rs");
        assert_eq!(
            fallback(&["src/app.rs".into(), "b".into(), "c".into()]),
            "Update app.rs and 2 more"
        );
    }
}
