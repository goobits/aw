# Agent Workspace Tools

Shared-checkout hygiene tools for Git safety, generated-file cleanup, and
performance measurement. Agent Workspace owns both the implementation and this manual.

## Commit Queue

`.agents/policies/git.md` owns the reusable safety policy. This section is the
tool manual for the shared `aw commit` queue and the commit-owner internals.

Use this queue when multiple workers share the same checkout. It serializes Git
and package-manager mutations so nobody commits from a stale status view or
races over package metadata.

Worker tabs use only the humane front door:

```sh
aw commit add "Describe the scoped change" path/to/file \
  --check "targeted verification command" \
  --poke git

aw commit status
aw commit doctor
aw commit wait <request-id>
```

Advanced request flags are available when a ticket needs stronger metadata or
fingerprint checks:

```sh
aw commit add "Update docs" README.md \
  --check "pnpm test" \
  --must-contain "Expected text" \
  --must-not-contain "Stale text" \
  --poke git
```

Fingerprint flags intentionally make stale tickets block until the commit owner
rechecks, blocks, closes, or replaces them. Avoid fingerprint flags for broad
directory tickets that will likely receive later same-scope follow-up edits.

Use `aw commit wait <request-id>` only for one ticket. There is intentionally
no global wait command, because that would make a worker wait for unrelated
queue items from other agents. `aw commit add ... --wait` waits only for the
request it just created.

Use `aw commit doctor` when the queue feels stuck. It gives the readable
reason before anyone reaches for internal queue details.

The lower-level Git and package queue commands are commit-owner internals exposed
through `aw gitq` and `aw pkgq`. They are not worker instructions and should
not be copied into normal agent guidance.

## Brush API Worktrees

Use the Brush API worktree helper when brush work needs isolation from the
shared checkout's Git index, HMR state, or active user-facing dev server:

```sh
aw brush-api worktree /tmp/brush-v8-fluid
cd /tmp/brush-v8-fluid
pnpm --filter @sketchapi/brush-api run check:types
```

The helper creates a branch-backed worktree through the commit-owner internals,
clones populated submodules from the current checkout when local copies exist,
copies generated brush WASM artifacts, and symlinks the dependency link
structure needed for Brush API package checks. It does not fetch private
submodules over SSH.

Use `--copy-deps` only when a scratch worktree must be isolated from the parent
checkout's `node_modules`; copying dependencies is much slower and uses
substantially more disk.

For browser debugging inside the worktree, run the Brush API-only dev server
instead of the full workspace dev profile:

```sh
pnpm run brush-api:dev
```

It serves `http://localhost:3338/tools/brush-api/` by default and can be moved
with `BRUSH_API_PORT=<port>`.

The commit owner may use the internal scoped Git workflow after reading the live
repo policy. Workers should not run final commit, staging, repair, or package
mutation commands directly.

## Repair

Use the repo-approved repair flow when Git reports tracked files as both staged
deleted and untracked, or when the commit owner detects an index/HEAD entry
mismatch. Use the recursive form only when a submodule or nested repo shows the
same pattern.

`aw gitq repair-index` rebuilds `.git/index` from `HEAD`, backs up the
existing index, and does not update worktree files.
`aw gitq repair-index --recursive` applies the same repair to initialized
submodules declared in `.gitmodules`, recursively declared nested submodules,
and extra initialized nested Git repos with their own `.git` file or directory.

## Cleanup

Dry-run generated cleanup before deleting anything:

```sh
aw workspace cleanup-generated
aw workspace cleanup-generated --generated
aw workspace cleanup-generated --rust-targets
aw workspace cleanup-generated --nested-node-modules
aw workspace cleanup-generated --preprocessed
```

The command reports candidates by default. It deletes only when `--delete` is
passed with an explicit category flag.

Safe categories:

- `--generated`: `.turbo` and `.svelte-kit`
- `--rust-targets`: Rust/Cargo `target` directories
- `--nested-node-modules`: package-local `node_modules`, excluding root
  `node_modules`
- `--all-safe`: all categories above
- `--preprocessed`: legacy `_preprocessed` folders and code-watcher cache

Manual-review category:

- `--build-outputs`: `dist` and `build` directories. These names are broad, so
  review the dry-run output before deleting them.

## Measurement

Use the measurement tools to prove workspace changes helped instead of guessing.

```sh
aw workspace measure-git
aw workspace measure-git infra/agent-workspace
aw workspace probe-git-config
aw workspace probe-git-config --path infra/agent-workspace
aw workspace probe-git-config --path infra/agent-workspace --apply
```

`measure-git` prints full and path-scoped Git timings. Pass a path to measure a
specific slice. Without a path, it measures `infra/agent-workspace`.

`probe-git-config` measures candidate Git config values without writing them.
It writes only winning values when explicitly run with `--apply`.

## Raw Git Policy

The reusable rule is: raw Git passthrough is inspection-only. Worker tabs should
not use raw Git or package-manager mutation commands. They should submit scoped
requests with `aw commit add ... --poke git` and wait for one request ID only
when they need a response.

The commit owner may use `aw gitq` for scoped commits, repair, chmod,
fetch/push, and submodule setup after reading the live local policy. Those
commands are intentionally not listed here as worker-facing copy.

The commit owner may clear stale queue or `.git/index.lock` files only after
checking that no active Git, package-manager, or queue owner process is still
using the checkout. Worker tabs must not remove locks manually. After clearing a
stale lock, rerun queue/status/health checks before committing.
