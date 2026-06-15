# aw

Zellij workspace CLI for repos with many tabs, agents, and shared Git
coordination.

`aw` keeps project workspaces in plain text, installs portable agent adapters,
opens repeatable Zellij sessions, and provides queue/owner tools for shared
checkouts.

## Key Features

| Area | What `aw` handles |
|---|---|
| Workspaces | Open, create, rename, remove, refresh, and list Zellij workspaces. |
| Tabs | Add, move, rename, remove, and refresh live tabs from saved `*.tabs` files. |
| Repo setup | Install shared agent adapters and repo-owned `config/aw` profiles. |
| Agent work | Coordinate worker requests through a commit-owner queue. |
| Maintenance | Doctor checks, generated cleanup, Git timing probes, and worktree helpers. |

## Quick Start

```bash
aw install
aw main=app,server,infra,scratch
aw main
```

In a repo that already vendors AW at `infra/aw`, use the package script when
available:

```bash
pnpm run aw:install
aw front
```

## Install Globally

Install the shared tooling once from this repository:

```bash
aw install
pnpm run aw -- install
```

If the consuming repo does not have package scripts, run the Rust binary
directly:

```bash
cargo run --manifest-path infra/aw/Cargo.toml -- install
```

If `zellij` is missing, the installer downloads pinned Zellij `0.44.3` for
supported platforms. Set `ZELLIJ_INSTALL_BINARY=0` to skip that step.

AW-owned state is installed under `~/.aw` by default. Set `AW_HOME` to use a
custom home.

Refresh an installed profile directly when needed:

```bash
aw setup --config config/aw
```

## Add To A Repo

Consuming repos pin this project as a submodule at `infra/aw`.

```bash
pnpm run aw:install
```

Equivalent direct command:

```bash
cargo run --manifest-path infra/aw/Cargo.toml -- install --repo
```

Use `--dry-run` before writing adapter files:

```bash
pnpm run aw:install --dry-run
cargo run --manifest-path infra/aw/Cargo.toml -- install --repo --dry-run
```

Repo install creates or validates:

```text
AGENTS.md
CLAUDE.md -> AGENTS.md
.agents -> infra/aw/agents/.agents
.agents.local/project.md
.claude/skills -> ../.agents/skills
config/aw/
```

Shared agent behavior belongs in `infra/aw/agents/.agents`. Repo-specific
commands, ports, and policies belong in `.agents.local/project.md`.
Codex discovers repo skills from `.agents/skills`; `.claude/skills` is only the
Claude compatibility adapter.

## Update

Use the consuming repo's update script when available:

```bash
pnpm run aw:update
git add infra/aw
git commit -m "chore: update aw"
```

Manual equivalent:

```bash
git -C infra/aw pull --ff-only
pnpm run aw:install
pnpm run aw -- doctor
pnpm run aw -- repo doctor
git add infra/aw
git commit -m "chore: update aw"
```

## Daily Use

Create the first workspace. If `config/aw/` is missing, `aw` creates it.

```bash
aw main=app,server,infra,scratch
aw main
```

Common workspace commands:

```bash
aw                         # show help
aw list                    # list workspaces and saved tabs
aw main                    # open workspace in a checkout-scoped session
aw main -s shared-dev      # open or resume an explicit named session
aw main -r /tmp/worktree   # open with a different cwd in the checkout's default session
aw create docs guide api scratch
aw refresh main
aw rename main work
aw remove work
```

When `config/aw/profile.conf` exists, `aw` auto-detects the project profile.

## Tabs

Manage saved and live tabs with the `tab` namespace:

```bash
aw main tab list
aw main tab add keyboard
aw main tab add keyboard@1
aw main tab move keyboard@1
aw main tab rename keyboard keys
aw main tab rename keyboard keys@1
aw main tab remove keyboard
aw main tab refresh
aw main tab refresh --session shared-dev
```

When a profile has exactly one workspace, `aw tab` can infer it:

```bash
aw tab list
aw tab add keyboard@1
aw tab move keyboard@2
aw tab rename keyboard keys
aw tab rename keyboard keys@1
aw tab remove keys
aw tab refresh
```

Power-user shortcut:

```bash
aw main=tools,ui,scratch  # create or replace workspace tabs
```

`aw refresh <workspace>` converges a live session back to its `*.tabs` file:
missing tabs are created, duplicate or out-of-profile tabs are removed, and
configured tabs return to saved order.

Session commands:

```bash
aw session name
aw session name main
aw ps
aw kill <session>
```

## Repo Maintenance

Validate setup:

```bash
aw doctor       # global install, profile, and runtime checks
aw repo doctor  # repo adapters, config/aw, and lowercase git tab
aw repo migrate # repair old adapter symlinks into the current layout
```

Clean generated files with a dry run first:

```bash
aw repo clean
aw repo clean --generated
aw repo clean --rust-targets
aw repo clean --nested-node-modules
aw repo clean --preprocessed
aw repo clean --all-safe --delete
```

Cleanup categories:

| Flag | Deletes when combined with `--delete` |
|---|---|
| `--generated` | `.turbo`, `.svelte-kit` |
| `--rust-targets` | Rust/Cargo `target` directories |
| `--nested-node-modules` | Package-local `node_modules`, not root `node_modules` |
| `--preprocessed` | Old `_preprocessed` and code-watcher cache folders |
| `--all-safe` | All safe categories above |
| `--build-outputs` | `dist` and `build`; review dry-run output first |

Measure Git performance and candidate config:

```bash
aw repo measure-git
aw repo measure-git infra/aw
aw repo probe-git-config
aw repo probe-git-config --path infra/aw
aw repo probe-git-config --path infra/aw --apply
aw repo routes
aw repo routes doctor
aw repo routes --config config/aw/routes.conf
```

Create an isolated worktree for scratch work:

```bash
aw repo worktree /tmp/brush-v8-fluid
aw repo worktree /tmp/brush-v8-fluid --branch brush-v8 --base main
```

The worktree helper creates a branch-backed worktree through owner internals,
hydrates available submodules/generated artifacts/dependency links, and avoids
fetching private submodules over SSH.

Routes are optional named local URLs for repo services. Store them in
`config/aw/routes.conf`:

```text
main=http://localhost:3240
dev=http://dev.localhost:3240 http://dev.localtest.me:3240
```

## Commit Queue

Use the commit queue when multiple workers share one checkout. Workers request
scoped work; a commit-owner tab performs Git and package mutations.

Set up the owner tab:

```bash
aw commit setup main --tab git --agent codex
aw commit setup main --session shared-dev --tab git --agent codex
aw commit setup main --tab git --no-agent
```

The setup command prepares the tab and can start the agent, but it does not
consume queue items by itself. The commit-owner agent in the `git` tab becomes
active when it receives `$x-commit next`; `aw commit poke git` sends that text
to the tab in this checkout's resolved default session. Pass
`--session <name>` to setup, request, or poke only when intentionally using an
explicit shared/resumed session name. Pass `--workspace <workspace>` to
request or poke when the commit tab lives in a non-default workspace.

Worker flow:

```bash
aw commit request "Update docs" README.md \
  --owner "AgentName" \
  --check "cargo test" \
  --summary "Short context" \
  --poke git

aw commit status
aw commit doctor
aw commit wait <request-id>
aw commit poke git
aw commit poke git --workspace main
```

After `--poke git`, the `git` tab should run `$x-commit next`, inspect the
ticket, commit the scoped live paths when safe, and mark the ticket done or
blocked. Worker tabs should not run `$x-commit next`; they submit requests,
check status, and wait only for their own request when needed.

Useful request flags:

| Flag | Use |
|---|---|
| `--queue-root <path>` | Share a custom queue path between request/status/owner commands. |
| `--owner <name>` | Record the requesting agent's chosen session name. |
| `--workspace <workspace>` | Target the default session for a specific workspace when poking. |
| `--session <name>` | Target an explicit shared/resumed Zellij session. |
| `--wait --timeout 10m` | Wait for the request just created. |
| `--must-contain <text>` | Block stale tickets until expected text exists. |
| `--must-not-contain <text>` | Block stale tickets until unwanted text is gone. |

Avoid fingerprint flags for broad directory tickets that will likely receive
follow-up edits in the same scope.

## Owner Commands

`aw owner git` and `aw owner pkg` are commit-owner internals. Worker tabs should
not run final commits, staging, repair, or package mutations directly.

Git owner examples:

```bash
aw owner git status
aw owner git status-fast
aw owner git health --deep
aw owner git repair-index
aw owner git repair-index --recursive
aw owner git chmod +x -- scripts/run.sh
aw owner git fetch origin
aw owner git push origin main
aw owner git lfs-push origin main
aw owner git worktree list
aw owner git clone <args...>
aw owner git submodule-sync <args...>
aw owner git submodule-update <args...>
aw owner git maintenance
aw owner git submodule-status
aw owner git commit-owned -m "message" -- path/to/file
aw owner git -- status --short
```

Package owner examples:

```bash
aw owner pkg lock-info
aw owner pkg -- install --lockfile-only
aw owner pkg -- add <package> --filter <workspace>
```

Only clear stale queue or `.git/index.lock` files after checking that no active
Git, package-manager, or queue owner process is using the checkout. Rerun
queue/status/health checks before committing.

## Configuration

Profiles are inert project files:

```text
config/aw/
  profile.conf
  main.tabs
  docs.tabs
  routes.conf
```

`profile.conf`:

```text
name=my-project
root=/workspace
default_workspace=main
default_workspaces=main docs
```

Set `tab_bar=aw` to use the optional AW tab bar plugin when its WASM artifact
is installed. The stock Zellij tab bar remains the default.

```text
tab_bar=aw
```

Build the plugin before `aw install` when you want installer-managed placement:

```bash
rustup target add wasm32-wasip1
cargo build --release --manifest-path infra/aw/plugins/aw-tab-bar/Cargo.toml --target wasm32-wasip1
aw install
```

Set `AW_TAB_BAR_WASM_SOURCE=/path/to/aw-tab-bar.wasm` before `aw install` to
copy a non-default build artifact. Set `AW_TAB_BAR_PLUGIN_PATH=/path/to/aw-tab-bar.wasm`
when rendering layouts against an explicit plugin path.

`*.tabs` files list tabs in order. Add a tab-specific working directory after a
tab character:

```text
app
server	/workspace/server
git
scratch
```

Workspace names are just file names. `aw docs` opens `config/aw/docs.tabs`.

Default session names include the profile name, workspace name, and a stable
fingerprint of the local checkout that owns `config/aw`, so the same workspace
name can run concurrently from different repos or worktrees even when
`profile.conf` has the same checked-in `root=` value. For installed profiles
without a local `config/aw`, AW falls back to the profile root. Use
`--session <name>` only when you intentionally want to share or resume that
exact Zellij session.

`aw <workspace> -r <path>` changes the cwd used for the layout, but it does not
change the implicit session identity. Pass `-s <session>` when you intentionally
want a second session from the same checkout.

`AW_HOME` changes where AW installs profiles, helpers, completions, and plugins.
It is useful for install/profile isolation, but live Zellij session isolation is
controlled by the session name.

## Shell And Session Behavior

Shell completions are installed for zsh and bash. They complete commands,
workspace names, tab names, and commit queue flags from the current profile.

Inspect the active AW locations when debugging setup:

```bash
aw paths
```

Inside Zellij, the watcher marks background activity:

| Marker | Meaning |
|---|---|
| `🤖` | An agent or script is working. |
| `🔔` | Work finished on a background tab; it clears when viewed. |

Watcher controls:

```bash
ZELLIJ_AGENT_TAB_WATCHER_DISABLE=1 aw main
ZELLIJ_AGENT_TAB_WATCHER_POLL_SECONDS=0.5 aw main
ZELLIJ_SESSION_NAME=<resolved-session> ~/.aw/bin/.zellij-agent-tab-watcher --status
ZELLIJ_SESSION_NAME=<resolved-session> ~/.aw/bin/.zellij-agent-tab-watcher --restart
ZELLIJ_SESSION_NAME=<resolved-session> ~/.aw/bin/.zellij-agent-tab-watcher --stop
ZELLIJ_SESSION_NAME=<resolved-session> ~/.aw/bin/.zellij-agent-tab-watcher --log 40
```

Session behavior:

- Existing sessions with the same resolved session name are resumed instead of
  recreated.
- Default session names are checkout-scoped. Old sessions named only after a
  workspace, such as `main` or `docs`, can remain until you remove them with
  `aw kill <session>`.
- Running `aw` inside Zellij switches sessions in place.
- Serialized sessions restore panes as shells so `Ctrl+C` exits foreground
  tools without killing the tab.
- `Ctrl+T` opens the next scratch tab.
- macOS-style delete and word/line movement keybinds are configured when the
  terminal sends Apple/Meta keys.
- Mouse scrolling and focus-follows-mouse are enabled; Ctrl-wheel pane resizing
  is disabled.

## Development

The public binary is installed as:

```text
~/.local/bin/aw
```

Private helper binaries live under:

```text
~/.aw/bin/
```

Helpers include `zwork`, `zellij-workspace-init`,
`zellij-workspace-doctor`, `zellij-new-scratch-tab`,
`zellij-launch-session`, `zellij-open-session`, `zellij-render-layout`,
`zellij-saved-session-order`, `zellij-live-tab-order`,
`zellij-session-tab-order`, and `.zellij-agent-tab-watcher`.

The installer also writes:

```text
~/.aw/config.kdl
~/.aw/default-profile
~/.aw/profiles/
~/.aw/completions/
~/.aw/plugins/
marked shell blocks in ~/.zshrc and ~/.bashrc
```

Run these after changing AW:

```bash
cargo fmt --manifest-path Cargo.toml --check
cargo test --manifest-path Cargo.toml
cargo outdated --manifest-path Cargo.toml
```
