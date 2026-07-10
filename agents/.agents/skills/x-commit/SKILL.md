---
name: x-commit
description: 'Use when the user invokes $x-commit or /x-commit, or asks to commit changes in one or more logical groups, commit slices, commit only scoped work, commit staged changes, leave unrelated/currently edited work alone, or avoid actively changing areas.'
---

# X Commit

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local
output rules.

Use when the user asks to commit local work, especially "commit slices",
"multiple logical groups", "commit only these files", "commit staged", "leave
current work alone", or "commit everything you changed".

This is a shared workspace. Commit only when explicitly asked, or when another
active skill delegates a verified scoped slice. Commit only requested-scope work.

Before committing, read `.agents/policies/git.md` when it exists and follow the
repo-approved Git/package workflow from `.agents.local/project.md` when present.

Default: take an initial ownership baseline, commit or hand off only stable
slices from that baseline, and do not chase new work that appears mid-commit.

## Shelly Commit Queue

When the repository config enables Shelly commit ownership, workers submit a
ticket instead of running the final Git commit. The first-party client is:

```bash
pnpm shelly:commit request --title "Title" --path path/to/file --owner "<agent-name>" --check "command"
```

The Shelly server uses a launch profile with `metadata.role=commit-owner`. It
opens that Git tab with `yocodex` when absent, or sends the running agent a
normal message when present. `SHELLY_COMMIT_OWNER=auto` is the default: the
queue activates only when that launch profile exists. Use `enabled` to require
it or `disabled` to turn it off entirely.

`--owner` is the stable agent name chosen at startup. It describes the request
owner only; it is not a lock or authorization grant.

Workers should include each changed file or an intentional directory owner,
plus the focused verification command when practical. A same-scope live edit
belongs in the existing queued ticket only when the title, scope, and checks
still match. Otherwise create a separate ticket.

The Shelly queue serializes overlapping path scopes per repository. Its
`blockers` list reports which earlier queued or active ticket owns an overlap.
Do not create a second ticket merely to bypass a blocker.

## Commit-owner mode

When the user says `$x-commit next`, "commit next", "drain the commit queue",
or Shelly wakes the Git tab, process safe tickets until none remain runnable:

1. Run `pnpm shelly:commit next` and read the returned ticket.
2. Inspect only its requested paths. Confirm the live diff still matches the
   title and summary, and review or run the listed verification commands.
3. Run `pnpm shelly:commit commit <ticket-id>`. The client acquires a local Git
   lock, refuses unrelated staged changes, stages only the ticket paths, checks
   the staged diff, commits, and marks the ticket complete.
4. If the slice cannot safely commit, run
   `pnpm shelly:commit fail <ticket-id> --result "concrete reason"`. A failed
   ticket is terminal and unblocks later work. Do not fail a ticket just because
   another ticket is ahead of it.
5. Repeat from step 1. Use `pnpm shelly:commit status` to inspect active,
   queued, completed, failed, and blocked work.

Do not run competing Git owners against the same checkout. The lock protects
the final Git operation; the queue provides the human-readable ordering and
blocker record.

Normal worker "commit my work" requests enqueue when the Shelly queue is
enabled. Direct scoped Git commits are appropriate only when the user explicitly
asks for them or the capability is disabled.

## Workflow

1. Take the initial dirty-state baseline before deciding slices when your role
   is allowed to inspect Git state:
   - staged diff name/status
   - worktree status
   - path-scoped diff name/status when full diff would be slow
2. Treat the initial baseline as the commit boundary. Do not add later paths
   unless the user confirms expanded scope. Report mid-run arrivals as new
   uncommitted work.
3. Identify logical commit slices by domain, package, app, or requested phase.
   Keep each slice coherent and avoid mixing unrelated cleanup with behavior
   changes.
4. For each slice, inspect the diff. Confirm target paths contain only intended
   files and no generated/debug artifacts outside `.llm/scratch/`.
5. Run targeted verification when practical. Use full suites only when request,
   local signoff, slice breadth, or uncovered risk justifies it.
6. Before each final commit, re-check only that slice with scoped status and
   diff. Commit only if it still matches the inspected baseline.
7. If Git reports index corruption or a stale lock, stop. Follow the local
   repair procedure, re-check the staged diff and the owned paths, then retry.
8. After each commit, run path-scoped status for the slice first. Use repo-wide
   status only when remaining work matters.

## Actively changing areas

Do not commit an area likely edited by another user, agent, or in-flight
workflow. Report it as intentionally left uncommitted.

Treat an area as actively changing when status or the scoped diff changes
between the initial baseline and the pre-commit re-check, new files appear in
the same domain, or a submodule has dirty internals beyond the intended slice.

For submodules, commit the nested live slice first, then commit the parent
pointer separately after verifying the nested repository is clean except for
ignored artifacts. Treat unpushed nested commits as push/share warnings, not
local blockers.

Report commits created, checks run, intentionally uncommitted slices, and any
remaining blocker with its exact path or decision.
