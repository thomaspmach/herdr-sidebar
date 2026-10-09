<div align="center">

# Herdr Sidebar

### The sidebar your terminal was missing — inspired by VS Code.

A file explorer and a full source-control panel in one dockable
[herdr](https://github.com/ogulcancelik/herdr) pane — activity-bar switching,
mouse-driven controls, AI-drafted commit messages, and file previews that open as editor
tabs — ephemeral until you double-click to pin one.

<img alt="Rust" src="https://img.shields.io/badge/Rust-self--contained_crate-orange?logo=rust&logoColor=white">
<img alt="herdr" src="https://img.shields.io/badge/herdr-%E2%89%A5%200.8-5865a3">
<img alt="Platforms" src="https://img.shields.io/badge/Windows%20%C2%B7%20macOS%20%C2%B7%20Linux-supported-2ea44f">
<img alt="CI" src="https://github.com/thomaspmach/herdr-sidebar/actions/workflows/ci.yml/badge.svg">
<img alt="License" src="https://img.shields.io/badge/license-MIT-blue">

<br><br>

<img src="plugins/herdr-sidebar/docs/media/hero.png" alt="The sidebar docked beside a 2x2 fleet of Claude Code and Codex agents" width="920">

</div>

If you've ever alt-tabbed out of your terminal just to *look* at the tree, the diff, or
what's staged, this closes that loop.

```sh
herdr plugin install thomaspmach/herdr-sidebar/plugins/herdr-sidebar
```

Tagged releases use SHA-256-verified binaries on supported platforms and fall back to a
source build when needed.

## Three Views

The activity bar switches Explorer, Search, and Source Control instantly in one process.
Use the mouse or press `1`, `2`, and `3`.

### Explorer & Preview

<div align="center">
<img src="plugins/herdr-sidebar/docs/media/preview.png" alt="Explorer and file preview" width="920">
</div>

- Navigate a real expandable tree with file icons, hover actions, Git decorations, and
  `m` / Ctrl+right-click context menus.
- Click a file to reuse an ephemeral preview tab; double-click to pin it. Preview in the
  same tab with **Preview opens in: pane**, or opt into experimental `replace` mode:
  working panes move to a temporary tab while you preview; Esc / `q` brings them back.
- Preview text, Markdown, images, and—when `ffmpeg` is available—video poster frames.
  Read-only previews support mouse selection and clipboard copy, including OSC 52 over SSH,
  and reload on their own when the file changes on disk.
- Find files with `Ctrl+P`; search project contents with `Ctrl+F` or `Ctrl+Shift+F`.
  Search supports case, whole-word, regex, and include/exclude filters.
- Stage files or folders from the tree without crossing nested-repository boundaries.
- Opt into a terminal editor for mouse clicks, while Enter keeps the built-in preview.
- Press `e` in a text preview for the experimental editor with selection, find,
  clipboard actions, explicit save, and external-change protection.

**Takeover caveat:** after a herdr restart or an incomplete restore, move any remaining
working panes back manually. Third-party TUIs are not yet verified with takeover;
`tab` remains the default.

### Source Control

<div align="center">
<img src="plugins/herdr-sidebar/docs/media/source-control.png" alt="Source Control view" width="920">
</div>

- Stage, unstage, discard, commit, inspect diffs, and sync with the upstream.
- Source Control refreshes external file and staging changes every 1.5 seconds, including
  while focus stays in a neighbouring terminal; commit-message drafts are preserved.
- Click the branch name—in the panel header, a repository row, or the Git footer—to
  switch branches, create one with **New branch…**, or track a remote branch.
  Deleting an unmerged branch requires a separate force-delete confirmation.
- Toggle changed files between list and folder tree with `t`, the view button, or Settings.
  In tree view, use `←→` to fold folders and `m` to stage or unstage a folder.
- Use one commit box per repository in multi-repo folders.
- Draft a commit message with the ✧ button through the local `claude` or `codex` CLI, with a
  filename-based fallback when Claude is unavailable.
- Browse commits, file history, branches, worktrees, remotes, stashes, and tags.
- Keep branch and sync controls visible in every sidebar view with the compact Git footer;
  hide it from Settings if you prefer the extra row.
- Status refreshes while the pane is focused. To keep a Source Control pane updating as a
  passive monitor (for example beside an agent), launch it with
  `HERDR_SIDEBAR_BACKGROUND_REFRESH=1` in its environment.

## Settings

<div align="center">
<img src="plugins/herdr-sidebar/docs/media/settings.png" alt="Sidebar settings" width="920">
</div>

The current version appears at the top. Official GitHub installations offer **Update & refresh**
when a newer stable release is available; linked development checkouts stay untouched.

Settings persist across tabs and restarts. Configure:

- Unified or separate Explorer and Source Control panes
- Left/right docking and preferred width
- Material/emoji icons and VS Code/light/terminal colors
- Tab (default), split-pane, or temporary takeover previews and optional custom editor
- Source Control list/tree view (shared across sidebars)
- Hidden files, Git decorations, Git footer, and footer hotkeys
- Auto-open, strict open/close toggle, focus-on-open, and live folder following

The sidebar follows a neighbouring pane's working directory by default. A manually chosen
folder stays put until that pane changes directory again.

## Keys

| Explorer / Search | Action | Source Control | Action |
|---|---|---|---|
| `↑↓` / `jk` | move | `Enter` | stage / unstage file; fold folder |
| `←→` / `hl` | fold / unfold | `a` / `u` | stage all / none |
| `Enter` | toggle / preview | `c` | commit message |
| `Ctrl+P` | quick open | `A` | draft message |
| `Ctrl+F` | content search | `S` | sync |
| `.` | hidden files | `o` | open diff |
| `r` | refresh | `r` | refresh |
| `m` | context menu | `m` | context menu |
| `s` | settings | `s` | settings |
| `b` | hide | `b` | hide |
| `1` / `2` / `3` | change view | `1` / `2` / `3` | change view |

Preview: drag to select (releasing copies, following herdr's `copy_on_select`),
`Ctrl/Cmd+C` to copy, arrows/PageUp/PageDown or
Space/`b` to scroll, `w` to toggle wrapping, and `q` or Esc to close.

Host keybindings can invoke the direct `show-explorer`, `show-search`, `show-git`, and
`quick-open` actions. For example, bind `cmd+p` to:

```toml
[[keys.command]]
key = "cmd+p"
type = "shell"
command = "herdr plugin action invoke quick-open --plugin herdr-sidebar"
```

## Install & Develop

**Requirements:** herdr 0.8+. Source builds require Rust 1.89+.
A Nerd Font is recommended for material icons; the emoji theme works everywhere.

```sh
herdr plugin install thomaspmach/herdr-sidebar/plugins/herdr-sidebar
```

Local checkout:

```sh
cd plugins/herdr-sidebar
cargo build --release
herdr plugin link .
```

Open or toggle it:

```sh
herdr plugin action invoke herdr-sidebar.open-sidebar-windows   # Windows
herdr plugin action invoke herdr-sidebar.open-sidebar           # Linux / macOS
```

Useful development actions:

| Action | Purpose |
|---|---|
| `open-sidebar` / `open-sidebar-windows` | open, focus, or hide the sidebar |
| `open-git` / `open-git-windows` | toggle separate Source Control |
| `show-explorer`, `show-search`, `show-git` | open/focus one activity without toggling |
| `quick-open` | open/focus the sidebar and show the file picker |
| `redeploy` / `redeploy-windows` | refresh running sidebars after a rebuild |

Use the `-windows` suffix for each direct action on Windows.

All docking, metadata, pane creation, and preview control use herdr's socket API directly.
The plugin is one Rust crate; optional external tools only enhance Markdown (`glow`), video
posters (`ffmpeg`), and AI commit drafts (`claude` or `codex`).

<div align="center">
<sub>Screenshots: herdr on Windows Terminal with a Nerd Font.</sub>
</div>

## Commit AI configuration

Commit drafts default to Claude CLI with the `haiku` model. To use Codex, create
`commit-ai.json` in `HERDR_PLUGIN_STATE_DIR` (normally
`~/.local/state/herdr/plugins/herdr-sidebar` on macOS/Linux, or
`%LOCALAPPDATA%\herdr\plugins\herdr-sidebar` on Windows):

```json
{
  "cli": "codex",
  "model": "gpt-6-luna",
  "reasoning_effort": "medium"
}
```

The file is read for every suggestion, so changing it needs no restart. Supported CLI
values are `claude` and `codex`; omitting the file preserves Claude/Haiku. Codex defaults
to `gpt-6-luna` and `medium` when the optional model/effort fields are absent. The chosen
CLI must be installed on PATH and authenticated. Codex runs ephemeral, read-only, without
user configuration or project instructions; shell tools are disabled. It receives only
the pending diff (capped at 16 KiB) as task input. Only a completed agent message from a
successful Codex turn becomes a draft. No commit is made by the AI subprocess.

Missing CLI, authentication/model failures, malformed configuration, or a 60-second timeout
use the existing filename-based fallback. Diagnostic stderr excludes diff/message content.

### Maintaining a local fork

Build and register from `plugins/herdr-sidebar` with `cargo build --release` and
`herdr plugin link . --enabled`. Use the `refresh-sidebars` action to reload running
sidebars while preserving preview/editor panes and saving commit drafts.
Local links do not offer the official automatic update action. This fork builds from source
with Rust: its manifest does not download original release binaries that lack the modification.

Keep your change on a branch, with `upstream` pointing to the original repository.
To bring in updates, run `git fetch upstream`, then `git merge upstream/main` on your
branch, resolve any conflicts, run `cargo test` and `cargo clippy -- -D warnings`, rebuild,
and refresh. These commands update your fork; installing the official plugin replaces
the local registration. To revert to the original registered version, link its original
plugin directory and refresh again.

### Local fork: quick AI commit

With the Sidebar focused, `Option+C` stages all changes in the selected repository, generates a message using the configured commit AI, and commits without pushing. In Explorer it switches to Source Control first. It ignores the shortcut while a dialog is open and prevents overlapping Git actions while running. If the index, branch, or HEAD changes during generation, it stops and leaves changes staged. Generation uses the existing filename-based fallback when the AI CLI fails.

On macOS, forward the chord through Ghostty: `keybind = alt+c=esc:c`. `Cmd+C` remains the terminal copy shortcut.

`Option+S` replaces `Shift+S` for Sync Changes (`pull --rebase`, then `push`) with the focus in the Source Control list. Ghostty forwards it with `keybind = alt+s=esc:s`. Plain `s` still opens settings; Shift+S types `S` in the message field.

`Option+A` combines the quick AI commit and Sync Changes: stage all → generate message → commit → pull --rebase --autostash → push, on the same worker and repository. It stops at the first failure; if sync fails, the completed local commit remains available to retry with Option+S. Explorer switches to SCM first. Ghostty must forward it with `keybind = alt+a=esc:a`.

This fork keeps Source Control refreshing while unfocused by default. Set `HERDR_SIDEBAR_BACKGROUND_REFRESH=0` before launch to opt out, or `1` to explicitly enable it. Version 0.16.1 includes upstream v0.15.1 preview reload, selection-copy, workspace-root, symlink-discovery, and Windows installer improvements while preserving Option+C/S/A and configurable commit AI.
