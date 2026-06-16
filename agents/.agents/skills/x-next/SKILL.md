---
name: x-next
description: 'Use when the user invokes $x-next or /x-next, asks what is next/remaining, asks whether the work is done, or asks to proceed with an approved proposal, next phase, next slice/no-brainers, or continue until done.'
---

# X Next

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user says variants of "what's next", "what's remaining", "are you
done", "proceed with your proposal", "proceed with next phase", "continue with
next slice", "continue until done", or "continue with next no-brainers".

There are three modes:

- Report mode: when the user asks what is next, what remains, or whether the work
  is done, report the remaining phases or current done/blocker status without
  implementing.
- Suggested-phase mode: when the user asks for "what's next phases", "next
  phases", "order of ops", or actionable next work, do not answer with a loose
  phase bullet list. Produce a `Suggested phases` proposal using the canonical
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format. If file paths are not yet known, mark paths as candidates or
  make the first phase a scoped discovery/check phase instead of inventing files.
  For code file paths, apply local file naming policy before proposing new,
  moved, or renamed files.
  Before proposing `+` new files, helpers, or tools, do the `x-proposal` (`.agents/skills/x-proposal/SKILL.md`)
  existing-first check so suggested phases do not create duplicate surfaces.
- Execute mode: when the user says continue, proceed, do the next phase, or
  continue until done, resume the approved proposal and implement the next
  in-scope phase.

In execute mode, if an approved proposal or active phase list exists, do not ask
for reconfirmation. Resume the proposal and complete every remaining item that
is in scope. Stop only when all approved items are done or a concrete blocker
prevents progress.

Read the relevant `.agents/policies/*.md` files and `.agents/policies/git.md`
when present before executing a phase. When context may be stale, compacted, or
split across prior work, first recover the current state with the repo-approved
worker workflow. In the current repo, use `.agents.local/project.md`: if Git
state is reserved for the commit owner, use direct file inspection and hand
verified paths/checks to the `git` tab instead of running status/diff commands
from a worker tab.

Do not invent new phases or expand scope beyond the approved proposal. If no
proposal exists, ask once for the target proposal or create a short phase
proposal using `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) before continuing.

If approved phases are done and the user asks what should come next, label the
answer as suggested, not approved, then use proposal formatting. Example:
`Suggested phases, not approved yet`.

When the user asks for the next "no-brainers", treat that as behavior-preserving, low-risk cleanup unless an approved proposal explicitly broadens the scope.

## Commit Mode

When this skill edits files and the phase verifies successfully, commit the scoped phase by default using `x-commit` (`.agents/skills/x-commit/SKILL.md`).
In shared checkouts with a commit-owner queue, worker tabs should submit the
verified phase to the `git` tab with the local handoff command instead of
running the final commit directly.

Run a phase loop with this cadence:

1. Recover current state and identify the next approved phase.
2. Implement only that phase.
   When the phase creates, moves, or renames code files, apply the local file
   naming policy before verification.
3. Run the phase's targeted verification and any required audits. Do not run a
   full suite unless the user asked for it, local signoff requires it, targeted
   checks cannot cover the risk, or the phase changed a broad/shared surface.
   If the repo has a test selector and the right focused command is unclear, run
   the selector in dry-run mode before choosing a broad command. In this repo,
   use `pnpm run test:select -- --path <changed-path>`.
4. Commit the verified phase through `x-commit` (`.agents/skills/x-commit/SKILL.md`).
5. Continue to the next approved phase when implementation scope is still safe.

Each completed phase must be a valid stopping point. Do not leave the repo knowingly broken while depending on a later phase to make the current phase safe.

For normal root slices, a commit blocker does not automatically block further
implementation. For subrepo/submodule slices, the commit owner commits the
subrepo/submodule work first, then commits the parent pointer separately through
`x-commit` (`.agents/skills/x-commit/SKILL.md`) after verifying the intended submodule commit. A clean local nested
commit made by the commit owner is enough to continue; do not stop, ask for a
push, or mark the phase blocked merely because it has not been pushed, and do
not treat ignored nested artifacts as a local phase blocker. Clearly report
unpushed nested commits as push/share warnings only: they must be pushed before
pushing or sharing the parent pointer commit, but they do not block local parent
commits.

Do not commit if verification fails, ownership is unclear, target paths overlap unrelated work, or the user explicitly says not to commit. Stop only when continuing would mix unrelated files, make verification misleading, or violate the approved scope.

When creating or refreshing a phase proposal, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`)
phase format when file changes are known. When sequencing matters, order phases
by operations/dependencies.

When reporting progress, keep it short:

Style done phases, blockers, remaining work, suggested phases, verification,
and ownership markers with shared colors when useful.

- **Phase N: done**
- **Phase N: blocked** because ...
- **Remaining:** ...

Use that short progress format only for pure status. When recommending next
phases, use `Suggested phases` with `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) formatting instead of bullets
such as `Phase 7: ...`.

Use ownership markers only when they clarify responsibility: `🫵` for user-owned
input, approval, secrets, business decisions, or external evidence; `🤖` for
agent-owned implementation, verification, cleanup, docs, commits, or follow-up
checks. If one phase needs both, split A/B subphases or use `Blocked input:`;
do not put `🫵` on a phase title that includes agent edits.

If the user asks for `tree diff`, use `x-proposal` (`.agents/skills/x-proposal/SKILL.md`).
