# 🧭 aw: Zellij Workspaces

Reusable, zero-friction Zellij workspace tooling.

`aw` provides sane defaults for managing complex terminal environments. It lets
you define project layouts in plain text, then handles session management,
auto-linking, and layout generation for you.

The goal: **Clone a repo, type `aw`, and get to work.**

## 🚀 Getting Started

### 1. Install

Install the shared tooling once. From this repository directory, run:

```bash
pnpm run aw -- install
```

If `zellij` is missing, this downloads pinned Zellij `0.44.3` for your
architecture. Set `ZELLIJ_INSTALL_BINARY=0` to skip binary installation. In
repos without the package script, use `cargo run --manifest-path
infra/agent-workspace/Cargo.toml -- install` instead.

For a consuming repo that includes `infra/agent-workspace`, make Agent Workspace the
one-stop setup owner:

```bash
pnpm run aw:install
```

`--repo` creates the portable agent adapters and installs the repo-owned
`config/aw` profile. Use `pnpm run aw:install --dry-run` to preview adapter
changes without writing files. Non-dry-run repo installs and migrations finish
with `aw doctor repo`.

Agent Workspace owns the shared agent bundle too:

```text
repo/
  AGENTS.md                    root adapter
  CLAUDE.md -> AGENTS.md        Claude-compatible adapter
  .agents -> infra/agent-workspace/agents/.agents
  .agents.local/project.md      repo-specific commands, ports, and policies
  .claude/skills -> ../.agents/skills
```

Shared behavior belongs in `infra/agent-workspace/agents/.agents`. Repo-specific facts stay
in `.agents.local/project.md`.

### 2. Create A Workspace

In any project directory, assign a workspace name to a comma-separated tab list.
If `config/aw/` does not exist, `aw` creates the profile first.

```bash
# Create a project profile with one workspace named main
aw main=app,server,infra,scratch

# Add another workspace
aw front=tools,components,scratch

# Rename a workspace
aw rename front app-ui

# Remove a workspace
aw remove app-ui

# Add or replace a workspace in an existing project
aw back=infra,api,db

# Open the workspace when you want a shell
aw back
```

### 3. Daily Usage

When a project has `config/aw/`, `aw` auto-detects it. You do not need to
manually link or install profiles for normal repos.

```bash
# Open the default workspace
aw

# Open a specific workspace
aw front

# Show available commands
aw help

# Create, add, or replace a local workspace, then sync a matching session
aw now=tools,components,scratch

# Open a workspace in a named session
aw front -s sketch-api

# Open a workspace with a different root directory
aw front -r /custom/workspace/path

# Combine flags; order does not matter
aw front -s sketch-api -r /custom/workspace/path
```

### 4. Visibility And Management

```bash
aw list         # List available workspaces in the current project
aw ps           # List running Zellij sessions
aw kill <name>  # Kill a specific session
aw create docs guide api scratch
aw refresh front
aw rename <old> <new>
aw remove <workspace>
aw doctor       # Validate install, profile config, and runtime tab order
aw doctor repo  # Validate repo adapters, config/aw, and git tab
aw migrate repo # Repair old repo adapter paths into the Agent Workspace layout
```

Shell completions are installed for zsh and bash. They complete commands,
workspace names, known tab names, and commit queue flags from the current
`config/aw` profile.

Advanced commit request flags such as `--owner`, `--must-contain`, and
`--must-not-contain` are public because the queue uses them for scoped owner
metadata and fingerprint checks. Do not use fingerprint flags for tickets that
you expect to fold later unless those fingerprints will remain true.

### 5. Live Tab Management

Workspace names can also manage their live Zellij tabs. Indexed tab specs use
zero-based positions, so `keyboard@1` places `keyboard` at the second tab.

```bash
aw tab list front
aw tab add front keyboard
aw tab add front keyboard@1
aw tab remove front keyboard
aw tab move front keyboard@1
aw tab rename front keyboard keys
aw refresh front
```

`aw refresh <workspace>` converges the live and saved session back to the
workspace's `*.tabs` file: missing tabs are created, duplicate or out-of-profile
tabs are removed, and configured tabs return to their saved order.

### 6. Commit Queue

Use one lowercase `git` tab when multiple agents share a checkout.

```bash
aw commit setup front --tab git --agent codex
aw commit setup front --session sketch-api --tab git --agent codex
aw commit add "Update docs" README.md --check "pnpm test" --poke git
aw commit status
aw commit poke git
```

If you use a custom queue root, pass `--root <queue-root>` to both the producer
and the `git` tab command.

```bash
aw commit add "Update docs" README.md --root /tmp/commit-queue --poke git
aw commit poke git --root /tmp/commit-queue
```

See `docs/workspace-tools.md` for the commit-owner queue, repair, cleanup,
measurement, and Brush API worktree manual.

Shared-agent workspace examples should include the same lowercase `git` tab so
the commit owner has a predictable place to run.

## 📁 How Profiles Work

A profile is a directory of inert config data that `aw` reads to build your
environment.

```text
my-project/config/aw/
  profile.conf
  frontend.tabs
  backend.tabs
```

`profile.conf` sets project defaults:

```text
name=my-project
root=/workspace
default_workspace=frontend
default_workspaces=frontend backend
```

`*.tabs` files define workspace layouts. Each line is a tab name. You can
optionally set a tab working directory with a tab-separated second column:

```text
app
server	/workspace/server
scratch
```

`aw <workspace>` works for any `<workspace>.tabs` file. Workspace names such
as `frontend` and `backend` are conventions, not special cases.

Creating the first workspace writes:

```text
name=<current-directory-name>
root=<current-directory-path>
default_workspace=main
default_workspaces=main
```

with `main.tabs`:

```text
app
server
infra
scratch
```

## ✨ Quality Of Life Features

### 🤖 Agent Tab Status

`aw` includes a watcher that can mark tabs while background agents or scripts
are working:

- `🤖` means an agent or script is actively working.
- `🔔` means work finished on a background tab; it disappears when you view the
  tab.

The watcher starts automatically inside Zellij. Disable it for a launch when you
want no status markers:

```bash
ZELLIJ_AGENT_TAB_WATCHER_DISABLE=1 aw front
```

Reset, stop, or inspect the watcher for a session:

```bash
ZELLIJ_SESSION_NAME=front ~/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher --restart
ZELLIJ_SESSION_NAME=front ~/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher --stop
ZELLIJ_SESSION_NAME=front ~/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher --status
ZELLIJ_SESSION_NAME=front ~/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher --log 40
```

Tune polling:

```bash
ZELLIJ_AGENT_TAB_WATCHER_POLL_SECONDS=0.5 aw front
```

### 🧠 Smart Session Resumption

Existing sessions are preserved. When you resume a session, `aw` moves core
profile tabs back into their configured order and removes duplicate or
out-of-profile tabs so the session matches its `*.tabs` file.

If you run `aw` from inside an existing Zellij client, it switches sessions in
place instead of nesting a second client.

Serialized sessions restore panes as shells instead of foreground apps. This
keeps tabs alive when `Ctrl+C` exits tools such as Codex.

### 🍎 macOS Notes And Keybinds

For Mac-like editing, let your terminal app handle standard shortcuts such as
Command+C, Command+V, and Command+L. The config maps Apple/Meta arrows to shell
line movement.

`Ctrl+T` creates a new scratch tab. If `scratch` already exists, it creates the
next available name such as `scratch1`, `scratch2`, and so on.

The config maps standard Mac delete behaviors:

- `Alt + Backspace`: Delete previous word
- `Alt + Left/Right`: Move to start/end of line when Apple is sent as Meta
- `Super + Backspace`: Delete current line
- `Super + Left/Right`: Move to start/end of line when Apple is sent as Super

Text selection does not copy automatically. Use `Super c`, `Alt c`, or `Ctrl y`
to copy the active Zellij selection.

The config keeps mouse wheel scrolling enabled, disables Ctrl-wheel pane
resizing, and uses focus-follows-mouse so the pane under the pointer receives
scroll focus.

## 🧰 Under The Hood

`aw` is built from the Rust crate in `infra/agent-workspace` and
installed with the Zellij helper bundle. The installer puts only the public
command into `~/.local/bin/`. Treat this as the public interface:

- `aw`

The Rust `aw` binary is also installed under private helper names in
`~/.local/share/agent-workspace/bin/`:

- `zwork <profile> <workspace> [session] [workdir]`
- `zellij-workspace-init`
- `zellij-workspace-doctor`
- `zellij-new-scratch-tab`
- `zellij-launch-session`
- `zellij-open-session`
- `zellij-render-layout`
- `zellij-saved-session-order`
- `zellij-live-tab-order`
- `zellij-session-tab-order`
- `.zellij-agent-tab-watcher`

The installer also writes:

- `~/.config/aw/config.kdl`
- a marked shell block in `~/.zshrc` and `~/.bashrc`

## ✅ Maintenance Checks

Run these after changing the Zellij setup:

```bash
cargo test --manifest-path Cargo.toml
```
