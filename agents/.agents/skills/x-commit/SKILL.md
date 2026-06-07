---
name: x-commit
description: 'Use when the user invokes $x-commit or /x-commit, or asks to commit changes in one or more logical groups, commit slices, commit only scoped work, commit staged changes, leave unrelated/currently edited work alone, or avoid actively changing areas.'
---

# X Commit

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use when the user asks to commit local work, especially with language like "commit slices", "multiple logical groups", "commit only these files", "commit staged", "leave current work alone", or "commit everything you changed".

This repository is a shared workspace and multiple people/agents may be working
at once. Commit only when explicitly asked, or when another active skill
delegates a verified scoped slice to this workflow. Commit only work that belongs
to the requested scope.

Before committing, read `.agents/policies/git.md` when it exists and follow the
repo-approved Git/package workflow from `.agents.local/project.md` when present.

Default stance: take an initial ownership baseline using the local workflow,
commit or hand off only stable slices from that baseline, and do not chase new
work that appears while you are committing.

In shared checkouts with a documented commit-owner queue, worker tabs do not run
the final commit directly. They submit a verified commit ticket to the commit
owner, usually with `aw commit add "Title" path... --check "cmd" --poke git`.
If a pending ticket already owns the same path scope, workers may consolidate
new same-scope edits into that ticket by rerunning verification and poking the
commit owner instead of adding a duplicate ticket.
The commit-owner tab consumes tickets with `$x-commit next` and runs the scoped
commit command.

Owner signal: an agent is acting as the commit owner only when it is invoked to
consume the queue, such as `$x-commit next` or `$x-commit next --root <queue-root>`.
Normal "commit my work" requests from worker tabs should enqueue a ticket unless
the user explicitly asks for a direct local commit.

## Commit Queue Mode

When the user says `$x-commit next`, `$x-commit next --root <queue-root>`,
"commit next", "drain the commit queue", or the `git` tab is poked with
`$x-commit next`, consume the next safe request from `aw commit next`, passing
through `--root <queue-root>` when provided.

Queue requests live under `.llm/commit-queue/` and are commit intent tickets, not
patches. The filesystem remains the source of truth.

Expected operator setup:

- Use `aw commit setup` once per checkout to add a `git` tab and start the
  commit-owner agent there. If the live session name differs from the workspace
  name, use `aw commit setup <workspace> --session <name>`.
- Run one commit-owner agent per checkout. Do not run competing Git agents
  against the same `.llm/commit-queue/`; `aw commit next` is a read-only
  handoff, not an atomic claim.
- Other tabs submit normal handoffs with
  `aw commit add "Title" path... --check "cmd" --poke git`, which sends
  `$x-commit next` to the `git` tab.
- When later worker work is still inside an existing pending ticket's path
  scope, do not create another ticket. Re-run the relevant checks, use
  `aw commit poke`, and report `folded into pending <id>` with the checks that
  passed. Create a new ticket only when the live work is outside every pending
  ticket's path scope or is a genuinely separate commit slice.
- If a worker needs a response, wait for only the specific ticket ID with
  `aw commit wait <id>` or use `aw commit add ... --wait` to wait for the
  request just created. Never wait for the whole queue; unrelated tickets may be
  ahead or behind the worker's request.
- If a request uses `--root <queue-root>`, keep that same root for
  `aw commit check`, `next`, `done`, and `block`.
- Normal wakeups use `aw commit poke`. Keep `--root <queue-root>` only for
  non-default queue roots used by tests, debugging, or special workflows.
- Use `aw commit doctor` when the queue is blocked or confusing. It should be
  the first readable diagnostic before raw queue details.
- Treat `aw commit request/check/next/done/block/list` as queue plumbing for
  agents, scripts, and debugging. Prefer the humane `add/status/poke` commands
  in user-facing examples.

For each queue request:

1. Run `aw commit check` with the requested `--root` when one was provided.
2. Run `aw commit next` with the requested `--root` when one was provided and
   read the returned request file.
3. Inspect only the requested paths and confirm the current diff still matches
   the title, summary, and fingerprints.
4. Treat verification commands in the request as suggestions to review or rerun;
   do not run risky commands blindly.
5. Commit the live requested paths through the repo-approved scoped commit
   command.
6. Move the request to `done` with `aw commit done <id>` after a successful
   commit, or to `blocked` with `aw commit block <id> --reason <reason>` when
   overlap, stale fingerprints, missing paths, ownership uncertainty, or failed
   verification blocks the commit.

`blocked/` is a triage lane, not a graveyard. A commit owner should not finish a
queue-drain report with old blocked tickets sitting untouched unless those
tickets were actively rechecked in this run or the user asked only for the next
pending request.

If multiple pending requests claim the same path, do not guess. Block or report
the overlap so the owner can order or merge the requests intentionally.

## Pending Ticket Consolidation

Queue tickets are commit intent, not patches. The live filesystem is the source
of truth, so later same-scope edits may be folded into an older pending ticket
without creating a new request.

Worker tabs may consolidate instead of enqueueing when all of these are true:

- The pending ticket is still in `pending/`, not `done/` or `blocked/`.
- Its path list already covers the new live edits, either exactly or by an
  intentional directory owner such as `infra/agent-workspace`.
- For broad directory-owner tickets, the current live diff has been reviewed and
  still matches the ticket title/summary. Do not fold unrelated later edits into
  a broad ticket merely because the path contains them.
- The new edits match the ticket title/summary and are not a separate logical
  slice.
- If the ticket has fingerprints, `aw commit check` still passes for those
  fingerprints. Fingerprinted tickets are not consolidation candidates after
  fingerprint drift until the commit owner blocks, closes, or replaces the stale
  ticket.
- Targeted verification has been rerun for the current live state.

Worker consolidation procedure:

1. Check `aw commit status` or `aw commit list` for an existing matching
   pending ticket.
2. If one exists, do not run `aw commit add`.
3. Run `aw commit check` when the ticket uses fingerprints or a broad
   directory owner.
4. Run or cite the verification for the current live state.
5. Run `aw commit poke` and report `folded into pending <id>`.

Commit owners should treat same-scope live-path drift as a re-inspection signal.
If the live diff is still coherent, inside the ticket's path scope, and
verification is adequate, commit the current live slice and close that ticket.
Block when drift reveals unrelated work, stale fingerprints, missing paths
outside a coherent deletion/rename, conflicting edits, failed verification, or
unclear ownership.

## Blocked Queue Reconciliation

When the user asks to "power through", "drain the queue", "get clean", "commit
best effort", or asks why blocked tickets remain, the commit owner must reconcile
blocked tickets instead of only reporting them:

1. Re-read the oldest blocked tickets and run a live path-scoped status for each
   request, including inside nested repos when a requested path is under a
   subrepo/submodule.
2. Close already-terminal tickets to `done` when the requested paths have no live
   diff, the ticket was superseded, or the work is already represented by an
   existing commit. Preserve the old blocked reason in a result note.
3. If an old overlap is gone or only one side still has live diff, treat the
   remaining live ticket as the active request and commit it when the requested
   paths form a coherent slice.
4. If a ticket has stale or missing paths, do not keep it blocked merely because
   old path names no longer exist. Commit the live tracked deletions/renames that
   still belong to the ticket, drop missing untracked paths from the commit slice,
   and close or reticket the stale remainder with a concrete replacement path
   list.
5. If broad verification fails outside the requested paths, run narrower
   verification that covers the live slice. A failure that is demonstrably
   unrelated is a verification caveat to report, not by itself a reason to leave
   a coherent slice blocked forever.
6. For nested repos, commit the nested live slice first when it is coherent, then
   commit the parent pointer separately if the nested repo is clean except for
   ignored artifacts. Treat unpushed nested commits as push/share warnings only.
7. Leave a ticket in `blocked/` only when there is still a concrete, current
   safety reason: conflicted paths, same-file edits that cannot be separated,
   dirty nested repo work beyond the intended commit, missing owner input,
   failed verification on the owned slice, or active file churn during the
   pre-commit recheck.
8. Every remaining blocked ticket must have an actionable reason naming the exact
   path or decision needed. Avoid vague reasons like "overlap" after the overlap
   has been resolved or aged out.

Best-effort commit does not mean guessing or committing unrelated work. It means
salvaging every currently coherent owned slice, closing stale queue paperwork,
and leaving only blockers that still protect live user work.

Workflow:

1. If acting as the commit owner, run the repo's lock/owner command to inspect
   queue ownership. If the queue is locked, wait for it. If waiting times out,
   determine whether the lock is stale by checking for an active owner process
   and a currently running queue/Git operation. When no active owner exists, the
   commit owner may clear the stale lock using the repo-approved repair/lock
   cleanup flow, then rerun the queue/status/health checks before continuing.
   If acting as a worker tab, do not run lock or Git-state commands and do not
   remove locks when local policy reserves them for the `git` owner.
2. Take the initial dirty-state baseline before deciding slices when your role
   is allowed to inspect Git state:
    - staged diff name/status
    - worktree status
    - path-scoped diff name/status when full diff would be slow
      In worker tabs where Git state is owner-only, use direct file inspection and
      include the target paths plus verification commands in the handoff ticket.
3. Treat the initial baseline as the commit boundary. Do not add paths that
   appear later unless the user explicitly confirms the expanded scope. New
   changes that arrive mid-run are reported as new/uncommitted work.
4. Treat already-staged changes as unrelated unless you staged them yourself in
   the current turn or the user explicitly says to commit staged changes.
   Unrelated unstaged files outside the target paths do not block a path-scoped
   commit. In a worker tab, unrelated staged files are a reason to enqueue the
   verified slice for the `git` tab, not a reason to abandon the phase. In the
   commit-owner tab, unrelated staged files may block the scoped commit; report
   that blocker unless the user explicitly confirms those staged files are in
   scope.
5. Identify logical commit slices by domain, package, app, or requested phase.
   Keep each slice coherent and avoid mixing unrelated cleanup with behavior
   changes.
6. For each slice, inspect the relevant diff before committing. Confirm the
   target paths contain only intended files and no generated/debug artifacts
   outside `.llm/scratch/`. Block on overlapping files, overlapping
   subrepo/submodule pointer updates, failed verification, unclear ownership, or
   evidence that the area is actively changing.
7. Run the targeted verification that fits the slice when practical. Do not run
   a full test suite unless the request, local signoff, slice breadth, or
   uncovered risk justifies it. Do not run local build commands without explicit
   approval when the project requires it.
8. Immediately before each commit-owner commit, re-check only that slice with
   scoped status and diff. Commit only if the slice still matches the inspected
   baseline. In a worker tab, re-check the edited files directly and include
   passed verification in the handoff.
9. Commit with the repo-approved scoped commit command using literal paths only
   when running as the commit-owner. In a worker tab, submit the scoped ticket
   with the local commit handoff command instead.
10. If the scoped commit reports index corruption, empty-index state, stale
    lock, staged/untracked index drift, or any repair-required failure, do not
    keep committing from that state. The commit owner may run only the
    repo-approved repair flow, then rerun cached-diff, scoped status, and scoped
    diff checks before retrying the commit. Worker tabs should not repair Git
    state when local policy reserves repair for the owner; report or enqueue the
    repair need instead.
    If the repair flow identifies a stale `.git/index.lock`, the commit owner
    may remove it only after confirming there is no active Git or queue process
    for that checkout.
11. After each commit-owner commit, run the repo's path-scoped status command
    for the slice first. Use repo-wide fast status only when the remaining work
    list matters, and full status only when untracked files or dirty submodule
    details are required for the slice. Worker tabs should instead wait for the
    commit-owner result or report the handoff ticket.

## Actively Changing Areas

Do not commit an area when it is likely being edited by another user, agent, or
in-flight workflow. Instead, report it as intentionally left uncommitted.

Treat an area as actively changing when any of these are true:

- The user names it as active or says to hold off.
- the repo lock/owner command shows another commit/check for that path or
  domain.
- Status/diff changes between the initial baseline and the pre-commit re-check.
- New files appear in the same domain while you are slicing.
- A submodule has dirty internals beyond the pointer you intended to commit.
- Another agent starts a nearby commit while you are scanning.

When the user asks to "commit all", "commit slices", or "commit everything",
interpret that as "commit all stable slices visible in the initial baseline",
not "keep scanning forever and commit new work as it arrives".

Submodules:

- If a slice changes a subrepo/submodule, the commit owner commits inside that
  repo using its approved queue workflow, then commits the parent pointer
  separately after verifying the intended submodule commit is clean.
- **Local commit rule:** a clean local subrepo/submodule commit made by the
  commit owner is enough for the parent pointer commit. Do not stop, ask for a
  push, or refuse the local parent commit merely because the nested commit is
  not reachable from a remote branch yet.
- Treat ignored local artifacts inside the nested repo, such as `node_modules/`,
  `.svelte-kit/`, `.vite/`, `.wrangler/`, `test-results/`, caches, and OS files,
  as dirty-looking local noise, not a parent pointer blocker. Do not delete
  ignored artifacts just to make status prettier unless the user asks for that
  cleanup.
- Block the parent pointer commit only when the nested repo has tracked,
  untracked-but-not-ignored, staged, conflicted, or actively changing work beyond
  the intended local commit.
- Clearly report if the referenced subrepo/submodule commit has not been pushed
  yet, because it must be pushed before pushing or sharing the parent pointer
  commit. Phrase this as a push/share warning, not a local commit blocker.
- Do not mix submodule internals and parent pointer updates in one root commit.
- Before the commit owner uses `git -C <path>` or any queued equivalent, verify
  `<path>/.git` exists as a file or directory. If it does not, that path is not
  a standalone repo and `git -C` will operate on the nearest parent repo.

If the user says to commit staged changes, still inspect the staged diff first. If unrelated staged work is present, either commit only the requested target paths or stop and ask for confirmation before committing unrelated staged files.

Report:

- Commits created, with short hashes when available.
- Files or slices intentionally left uncommitted.
- Verification run, or why verification was not run.
- Any blockers, especially unrelated staged files, queue timeouts, or subrepo/submodule ordering issues.
