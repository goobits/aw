# Git Policy

This policy defines the reusable Git safety bar. Local command names and queue
tools belong in `.agents.local/project.md`.

## Shared Checkout Safety

Treat Git state as shared unless the local project policy proves otherwise.
Prefer one worktree or clone per agent. If multiple workers share a checkout,
treat the index as a critical section and use the local project's approved queue
or lock workflow.

When the local project has a single commit-owner queue, normal worker tabs should
submit verified commit intent to that owner instead of committing directly. The
commit-owner is the only agent that should run the final scoped commit command.

## Commits

Commit only when explicitly asked, or when an invoked skill explicitly says to
commit verified scoped work. Never commit when the user says not to.

Commit only your own changes; leave unrelated changes unstaged.

Before final commit or commit handoff:

- inspect staged changes
- inspect unstaged changes for the target paths
- identify a coherent logical slice
- run targeted verification when practical
- use the local project's scoped commit workflow

If you are not the local commit-owner and the project documents a commit handoff
command, enqueue the verified slice through that handoff instead of treating
unrelated staged files as a reason to abandon the work.

When creating commit requests for agent-authored work, set the request owner to
your chosen session agent name, usually with `--owner "<agent-name>"`.

When creating commit requests or direct commits for agent-authored work, include
this trailer in the request summary or commit message:

```text
Co-authored-by: Miko Meow <101564+mudcube@users.noreply.github.com>
```

In a worker tab, "inspect staged/unstaged changes" means use only the local
project's worker-approved inspection path. If Git state inspection is reserved
for a commit owner, hand the verified paths and checks to that owner instead of
running Git or queue internals yourself.

Treat already-staged changes as unrelated unless you staged them yourself in the
current turn or the user explicitly says to commit staged changes.

Create new commits; never amend unless the user explicitly requests it.

## Forbidden Defaults

Never run destructive or history-changing Git operations unless the user has
explicitly approved the exact operation:

- reset
- checkout/restore of user work
- stash/pop
- amend/rebase/history rewrite
- raw index plumbing
- manual index lock removal from worker tabs
- raw push/fetch/worktree/submodule update operations when the local project
  provides queued commands for them

If the local project has a documented repair flow, use it only when your role is
allowed to run it. If repair is commit-owner-only, report the problem or hand it
to the owner rather than improvising raw Git plumbing.

When a nested repo or submodule shows tracked files as both staged deleted and
untracked, use the local recursive index repair flow only when your role is
allowed to run it. Do not repair nested indexes with ad hoc raw reset or index
plumbing commands in a shared checkout.

The commit owner may clear a stale queue or index lock only after proving no
active owner process, Git process, or package-manager process is still using
that checkout. After clearing the stale lock, rerun the repo-approved
queue/status/health checks before staging, repairing, or committing. Worker tabs
must hand stale-lock repair to the commit owner.

## Submodules And Nested Repos

When a root commit depends on submodule or nested-repo changes, commit the nested
repo first, then commit the parent pointer/reference separately after verifying
the intended nested commit.

Local commits do not require pushing first. A clean local nested commit may be
referenced by a parent pointer/reference commit even when that nested commit is
not reachable from a remote branch yet.

Clearly report when a referenced nested commit has not been pushed, but treat
that as a push/share warning only, not a blocker for local development commits.

Do not mix nested repo internals and parent pointer/reference updates in one
commit.

Parent pointer commits may reference a clean local nested commit that has not
been pushed yet. In that case, report that the nested commit must be pushed
before pushing or sharing the parent pointer commit.
