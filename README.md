# Agent Workspace

Agent Workspace (`aw`) provides portable agent adapters, serialized Git and
package-manager ownership, repository maintenance commands, and a shared
commit-owner queue. Interactive terminal ownership belongs to Shelly.

The design keeps one boundary between the queue and the terminal UI:

```text
worker -> aw commit request -> existing queue -> configured owner hook -> Shelly
```

There is no queue daemon, watcher, lease, claim protocol, or second session
registry. The owner hook only wakes the existing commit owner. Queue safety,
overlap detection, blockers, verification, and Git locking remain owned by
`aw`.

## Install

From a checkout that contains this repository:

```bash
cargo run -- install
```

This installs `aw` in `~/.local/bin` and completion files in
`~/.aw/completions`. It also removes the old managed shell block and obsolete
private terminal-multiplexer helpers from earlier Agent Workspace installs.

To install or repair repository adapters:

```bash
aw install --repo
aw install --repo --dry-run
```

Repository installation preserves existing files and creates the standard
links only when safe:

```text
.agents        -> infra/aw/agents/.agents
CLAUDE.md      -> AGENTS.md
.claude/skills -> ../.agents/skills
```

## Commit Owner Configuration

The repository profile is `config/aw/profile.conf`:

```ini
name=workspace
root=/workspace
commit_owner=enabled
```

`commit_owner` accepts `enabled` or `disabled`. It is enabled when omitted so
existing local workflows continue to use queue ownership after upgrading.
`SHELLY_COMMIT_OWNER` can override the file for one process and accepts common
boolean forms such as `1`, `0`, `true`, `false`, `enabled`, and `disabled`.

When disabled:

- `aw commit status` reports `Status disabled`.
- new commit requests and pokes stop with a direct-workflow message.
- no ticket or owner session is created.
- low-level queue inspection commands remain available for recovery.

## Shelly Owner Hook

`aw` invokes one configurable program to ensure or message the commit owner:

```bash
export AW_COMMIT_POKE_PROGRAM=/path/to/shelly-commit-owner
export AW_COMMIT_POKE_ARG=optional-first-argument
```

The requested message is appended as the final argument. For example,
`aw commit poke` sends `$x-commit next`. A repository can point the hook at its
Shelly adapter without making `aw` depend on Shelly packages or protocol
details.

The adapter is responsible for these terminal concerns:

1. Target the active Shelly workspace exactly.
2. Reuse the single live `git` launch when present.
3. Refuse ambiguous duplicate live Git sessions.
4. Send normal input to idle, done, or working agents so work can queue, but
   refuse to answer an agent that needs human attention.
5. Remove stale Git tabs, open the stable `git` launch when absent, then use its
   first input line as the initial agent prompt.
6. Use Shelly's existing authenticated status, open, input, and detach path.

A missing hook does not damage a successfully created queue ticket. The CLI
reports that the request is queued but the Shelly owner was not reached.

## Commit Queue

Workers submit exact path ownership and useful checks:

```bash
aw commit request "Add queue docs" README.md \
  --owner Ledger \
  --check "cargo test" \
  --poke
```

Common commands:

```bash
aw commit setup
aw commit status
aw commit doctor
aw commit poke
aw commit list
aw commit check
aw commit next
aw commit wait <request-id> --timeout 10m
```

The commit owner processes requests with the `x-commit` workflow. Tickets are
JSON files under `.llm/commit-queue/` and move through `pending`, `done`, and
`blocked`. Each ticket can include:

- exact paths and an owner name
- a commit title and summary
- verification commands
- required or forbidden diff text
- completion metadata, commit hash, verification result, and notes

The queue rejects unsafe overlaps and invalid tickets. The commit owner still
inspects current Git status and diffs before staging, leaves unrelated edits
alone, and uses the serialized Git owner command for mutations.

## Serialized Owners

Run Git mutations through:

```bash
aw owner git -- status --short
aw owner git owned-commit --spec /path/to/spec.json
aw owner git health
```

Run package-manager mutations through:

```bash
aw owner pkg -- install --lockfile-only
aw owner pkg -- add <package> --filter <workspace>
```

These commands preserve the existing Git and package locks. The Shelly hook
does not replace or duplicate them.

## Repository Maintenance

```bash
aw repo doctor
aw repo migrate --dry-run
aw repo clean
aw repo measure-git
aw repo probe-git-config
aw repo routes doctor
aw repo worktree <path> --branch <name>
```

`aw repo doctor` checks the adapter files, links, `config/aw`, and the explicit
commit-owner setting.

## Development

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Focused contracts live under `tests/`. Queue behavior is tested independently
from the configured terminal hook, and the hook contract uses a small fake
executable rather than a terminal UI simulator.
