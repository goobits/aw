---
name: x-commit
description: 'Use when the user invokes $x-commit or /x-commit, or asks to commit changes in one or more logical groups, commit slices, commit only scoped work, commit staged changes, leave unrelated/currently edited work alone, or avoid actively changing areas.'
---

# X Commit

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user asks to commit local work, especially "commit slices",
"multiple logical groups", "commit only these files", "commit staged", "leave
current work alone", or "commit everything you changed".

This is a shared workspace. Commit only when explicitly asked, or when another
active skill delegates a verified scoped slice. Commit only requested-scope work.

Before committing, read `.agents/policies/git.md` when it exists and follow the
repo-approved Git/package workflow from `.agents.local/project.md` when present.

Default: take an initial ownership baseline, commit or hand off only stable
slices from that baseline, and do not chase new work that appears mid-commit.

In shared checkouts with a documented commit-owner queue, worker tabs do not run
the final commit directly. They submit a verified commit ticket to the commit
owner, usually with
`aw commit request "Title" path... --owner "<agent-name>" --check "cmd" --poke git`.
If a pending ticket already owns the same path scope, workers may fold in
same-scope edits by rerunning verification and poking the owner instead of
adding a duplicate. The commit-owner tab consumes tickets with `$x-commit next`.

`--owner` is the stable agent name chosen at startup: a human-readable request
owner, not a uniqueness guarantee, queue lock, or authorization mechanism.

An agent is commit owner only when invoked to consume the queue, such as
`$x-commit next` or `$x-commit next --root <queue-root>`. Normal worker-tab
"commit my work" requests enqueue unless the user explicitly asks for direct
local commit.

## Commit Queue Mode

When the user says `$x-commit next`, "commit next", "drain the commit queue", or
the `git` tab is poked, drain by default: consume safe `aw commit next` requests
in a loop, passing through `--queue-root` when provided. Do not stop at the first
"no safe pending" while blocked tickets remain. After safe requests are
exhausted, run Blocked Queue Reconciliation, re-check for newly unblocked
requests, and repeat until the queue is empty or only current blockers remain.

Only do a single-item pass when the user explicitly scopes it that way, such as
"just the next ticket", "next one only", or "don't touch blocked".

Queue requests under `.llm/commit-queue/` are intent tickets, not patches. The
filesystem remains the source of truth.

Expected operator setup:

- `aw commit setup` runs once per checkout to add a `git` tab and start the
  commit owner. Use explicit workspace/session args only for intentionally
  shared/resumed sessions.
- Run one commit-owner agent per checkout. Do not run competing Git agents
  against the same queue; `aw commit next` is a read-only handoff, not a claim.
- Worker tabs submit handoffs with
  `aw commit request "Title" path... --owner "<agent-name>" --check "cmd" --poke git`,
  sending `$x-commit next` to the checkout's resolved default `git` tab. Include
  required trailers in the request summary. Use `--workspace` or `--session`
  only for intentional non-default targets.
- When later worker work is still inside an existing pending ticket's path
  scope, do not create another ticket. Re-run the relevant checks, use
  `aw commit poke`, and report `folded into pending <id>` with the checks that
  passed. Create a new ticket only when the live work is outside every pending
  ticket's path scope or is a genuinely separate commit slice.
- If a worker needs a response, wait only for its ticket ID with
  `aw commit wait <id>` or `aw commit request ... --wait`. Never wait for the
  whole queue.
- If a request uses `--queue-root <path>`, keep that same queue root for
  `aw commit check`, `next`, `done`, and `block`.
- Normal wakeups use `aw commit poke`, which resolves the checkout's default
  session. Keep `--queue-root`, `--workspace`, and `--session` only for
  non-default queue roots, workspaces, or intentional explicit sessions.
- Use `aw commit doctor` when the queue is blocked or confusing. It should be
  the first readable diagnostic before raw queue details.
- Treat `aw commit raw-request/check/next/done/block/list` as agent/script/debug
  plumbing. Prefer `request/status/poke` in user-facing examples.

For each queue request:

1. Run `aw commit check` with the requested `--queue-root` when one was provided.
2. Run `aw commit next` with the requested `--queue-root` when one was provided and
   read the returned request file.
3. Inspect only the requested paths and confirm the current diff still matches
   the title, summary, and fingerprints.
4. Treat verification commands in the request as suggestions to review or rerun;
   do not run risky commands blindly.
5. Commit the live requested paths through the repo-approved scoped commit
   command. Use a short imperative title plus a brief description/body that
   says what changed and why it matters.
6. Move the request to `done` with `aw commit done <id>` after a successful
   commit, or to `blocked` with `aw commit block <id> --reason <reason>` when
   overlap, stale fingerprints, missing paths, ownership uncertainty, or failed
   verification blocks the commit.

`blocked/` is a triage lane, not a graveyard. A commit owner should not finish a
queue-drain report with old blocked tickets sitting untouched unless those
tickets were actively rechecked in this run or the user explicitly scoped the
run to a single ticket.

If multiple pending requests claim the same path, do not guess. Block or report
the overlap so the owner can order or merge the requests intentionally.

## Pending Ticket Consolidation

Queue tickets are intent, not patches. The live filesystem is source of truth,
so later same-scope edits may fold into an older pending ticket.

Worker tabs may consolidate instead of enqueueing when all of these are true:

- The pending ticket is still in `pending/`, not `done/` or `blocked/`.
- Its path list already covers the new live edits, exactly or by intentional
  directory owner such as `infra/agent-workspace`.
- For broad directory-owner tickets, the live diff was reviewed and still
  matches the title/summary. Do not fold unrelated edits in merely because the
  path contains them.
- The new edits match the ticket title/summary and are not a separate logical
  slice.
- If the ticket has fingerprints, `aw commit check` still passes. Fingerprinted
  tickets are not consolidation candidates after drift until the owner blocks,
  closes, or replaces them.
- Targeted verification has been rerun for the current live state.

Worker consolidation procedure:

1. Check `aw commit status` or `aw commit list` for an existing matching
   pending ticket.
2. If one exists, do not run `aw commit request`.
3. Run `aw commit check` when the ticket uses fingerprints or a broad
   directory owner.
4. Run or cite the verification for the current live state.
5. Run `aw commit poke` and report `folded into pending <id>`.

Commit owners treat same-scope live-path drift as a re-inspection signal. If the
live diff is coherent, inside scope, and verified, commit it and close the
ticket. Block on unrelated work, stale fingerprints, missing paths outside a
coherent deletion/rename, conflicts, failed verification, or unclear ownership.

## Blocked Queue Reconciliation

When asked to "power through", "drain the queue", "get clean", "commit best
effort", or explain blocked tickets, reconcile blocked tickets instead of only
reporting them:

1. Re-read oldest blocked tickets and run live path-scoped status, including
   inside nested repos for subrepo/submodule paths.
2. Close already-terminal tickets to `done` when the requested paths have no live
   diff, the ticket was superseded, or the work is already represented by an
   existing commit. Preserve the old blocked reason in a result note.
3. If an old overlap is gone or only one side still has live diff, treat the
   remaining live ticket as the active request and commit it when the requested
   paths form a coherent slice.
4. For stale/missing paths, do not keep blocking because old names vanished.
   Commit live tracked deletions/renames that still belong, drop missing
   untracked paths from the slice, and close or reticket stale remainder paths.
5. If broad verification fails outside requested paths, run narrower slice
   verification. Demonstrably unrelated failures are caveats, not a reason to
   leave a coherent slice blocked.
6. For nested repos, commit the nested live slice first when it is coherent, then
   commit the parent pointer separately if the nested repo is clean except for
   ignored artifacts. Treat unpushed nested commits as push/share warnings only.
7. Leave a ticket in `blocked/` only for concrete current safety reasons:
   conflicts, inseparable same-file edits, dirty nested repo work beyond the
   intended commit, missing owner input, owned-slice verification failure, or
   active churn during pre-commit recheck.
8. Every remaining blocked ticket must have an actionable reason naming the exact
   path or decision needed. Avoid vague reasons like "overlap" after the overlap
   has been resolved or aged out.

Best effort means salvaging coherent owned slices, closing stale paperwork, and
leaving only blockers that still protect live user work. It does not mean
guessing or committing unrelated work.

Workflow:

1. Commit owner: run the repo lock/owner command. If locked, wait. If waiting
   times out, check for active owner and queue/Git processes. With no active
   owner, use the repo-approved stale-lock cleanup flow, then rerun
   queue/status/health checks. Worker tabs do not run lock/Git-state commands or
   remove locks when local policy reserves them for `git`.
2. Take the initial dirty-state baseline before deciding slices when your role
   is allowed to inspect Git state:
    - staged diff name/status
    - worktree status
    - path-scoped diff name/status when full diff would be slow
      In worker tabs where Git state is owner-only, use direct file inspection and
      include the target paths plus verification commands in the handoff ticket.
3. Treat the initial baseline as the commit boundary. Do not add later paths
   unless the user confirms expanded scope. Report mid-run arrivals as new
   uncommitted work.
4. Treat already-staged changes as unrelated unless you staged them this turn or
   the user asked to commit staged changes. Unrelated unstaged files outside the
   target paths do not block path-scoped commit. In worker tabs, unrelated staged
   files mean enqueue for `git`; in the commit-owner tab, they may block unless
   the user confirms they are in scope.
5. Identify logical commit slices by domain, package, app, or requested phase.
   Keep each slice coherent and avoid mixing unrelated cleanup with behavior
   changes.
6. For each slice, inspect the diff. Confirm target paths contain only intended
   files and no generated/debug artifacts outside `.llm/scratch/`. Block on
   overlaps, subrepo/submodule pointer overlap, failed verification, unclear
   ownership, or active changes.
7. Run targeted verification when practical. Use full suites only when request,
   local signoff, slice breadth, or uncovered risk justifies it. Do not run
   local builds without required approval.
8. Before each commit-owner commit, re-check only that slice with scoped status
   and diff. Commit only if it still matches the inspected baseline. Worker tabs
   re-check edited files directly and include passed checks in the handoff.
9. Commit with the repo-approved scoped commit command using literal paths only
   when running as the commit-owner. In a worker tab, submit the scoped ticket
   with the local commit handoff command instead, including your chosen agent
   name as the request owner.
10. If commit reports index corruption, empty-index state, stale lock,
    staged/untracked index drift, or repair-required failure, stop committing
    from that state. The commit owner may run only the repo-approved repair flow,
    then rerun cached-diff, scoped status, and scoped diff before retrying.
    Worker tabs report or enqueue repair needs when repair is owner-only.
    If the repair flow identifies a stale `.git/index.lock`, the commit owner
    may remove it only after confirming there is no active Git or queue process
    for that checkout.
11. After each commit-owner commit, run path-scoped status for the slice first.
    Use repo-wide fast status only when remaining work matters, and full status
    only for untracked files or dirty submodule details. Worker tabs wait for
    owner result or report the handoff ticket.

## Actively Changing Areas

Do not commit an area likely edited by another user, agent, or in-flight
workflow. Report it as intentionally left uncommitted.

Treat an area as actively changing when any of these are true:

- The user names it as active or says to hold off.
- the repo lock/owner command shows another commit/check for that path or
  domain.
- Status/diff changes between the initial baseline and the pre-commit re-check.
- New files appear in the same domain while you are slicing.
- A submodule has dirty internals beyond the pointer you intended to commit.
- Another agent starts a nearby commit while you are scanning.

Interpret "commit all", "commit slices", or "commit everything" as "commit all
stable slices visible in the initial baseline", not "keep scanning forever".

Submodules:

- If a slice changes a subrepo/submodule, the commit owner commits inside it
  using its approved queue workflow, then commits the parent pointer separately
  after verifying the intended submodule commit is clean.
- **Local commit rule:** a clean local subrepo/submodule commit by the commit
  owner is enough for the parent pointer commit. Do not stop, ask for a push, or
  refuse the local parent commit because the nested commit is not remote yet.
- Treat ignored nested artifacts such as `node_modules/`, `.svelte-kit/`,
  `.vite/`, `.wrangler/`, `test-results/`, caches, and OS files as local noise,
  not a parent pointer blocker. Do not delete them unless asked.
- Block the parent pointer commit only when the nested repo has tracked,
  untracked-but-not-ignored, staged, conflicted, or actively changing work beyond
  the intended local commit.
- Report unpushed nested commits as push/share warnings, not local blockers.
- Do not mix submodule internals and parent pointer updates in one root commit.
- Before `git -C <path>` or queued equivalent, verify `<path>/.git` exists. If
  not, the path is not standalone and Git will operate on the nearest parent.

If asked to commit staged changes, inspect staged diff first. If unrelated
staged work is present, commit only requested target paths or ask before
committing unrelated staged files.

Report:

Style commits, checks, skipped slices, warnings, and blockers with shared colors
when useful.

- Commits created, with short hashes when available.
- Include both the commit title and a short description/body for each commit.
- Files or slices intentionally left uncommitted.
- Verification run, or why verification was not run.
- Any blockers, especially unrelated staged files, queue timeouts, or subrepo/submodule ordering issues.
