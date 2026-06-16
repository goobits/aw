---
name: x-do
description: 'Use when the user invokes $x-do or /x-do, or says a go-ahead such as proceed, proceed with your plans, proceed with all phases, continue, make it so, do it, keep going, until done, or common typos like proceed wtih, proced, proeed, yoru, or you rplans. Execute all approved proposed phases in a loop until complete or genuinely blocked.'
---

# X Do

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill as the no-friction execution command:

Run every approved phase until done. Verify each phase and either commit it as
the commit owner or hand it to the commit owner from a worker tab. Do not ask
again. Do not expand scope. Stop only when done or blocked.

This is not a new implementation engine. Use `x-next` (`.agents/skills/x-next/SKILL.md`) execute mode for the
phase loop and `x-commit` (`.agents/skills/x-commit/SKILL.md`) for each verified scoped commit.

## Behavior

1. Recover current state with the repo-approved worker workflow from
   `.agents.local/project.md` when present. If Git state is commit-owner-only,
   use direct file inspection and leave Git state checks to the handoff owner.
2. Identify the approved proposal, phase list, or active plan.
3. Execute the next approved phase.
   When the phase creates, moves, or renames code files, apply local file naming policy
   before verification.
4. Run that phase's targeted verification. Do not run a full suite unless the
   user asked for it, local signoff requires it, targeted checks cannot cover
   the risk, or the phase changed a broad/shared surface.
   When the repo provides a test selector and the smallest useful check is not
   obvious, run the selector in dry-run mode first. In this repo, use
   `pnpm run test:select -- --path <changed-path>`.
5. Commit the verified phase through `x-commit` (`.agents/skills/x-commit/SKILL.md`). In shared checkouts with a
   commit-owner queue, this means submitting the verified phase to the `git` tab
   instead of running the final commit from the worker tab.
6. Continue immediately to the next approved phase.
7. Repeat until all approved phases are done or a concrete blocker prevents safe
   progress.

Do not ask the user to approve each phase again. The user's go-ahead is
authorization to run the approved proposal loop.

## Scope Guard

- Execute approved scope only.
- Do not invent phases, expand the project, or turn suggested phases into
  approved phases.
- If no approved proposal or phase list exists, stop and produce a short
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) plan instead of guessing.
- If approved phases are complete and more work is possible, stop and report
  suggested next phases with `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) formatting. Do not continue into
  unapproved follow-up work.
- If user-owned inputs, secrets, credentials, approvals, or business decisions
  are required, stop and report them clearly. Use `x-owner-checklist` (`.agents/skills/x-owner-checklist/SKILL.md`) when a
  form-like owner input list would be clearer.

## Stop Conditions

Stop only when:

- all approved phases are complete
- verification fails
- ownership is unclear
- target paths overlap unrelated or actively changing work
- `x-commit` (`.agents/skills/x-commit/SKILL.md`) reports that a submodule/subrepo pointer cannot be safely committed
- continuing would exceed the approved scope
- the user explicitly says to pause, stop, or only report status

A clean local nested/subrepo commit made by the commit owner is enough to
continue the local phase loop. Do not stop, ask for a push, or mark the phase
blocked merely because that nested commit has not been pushed. Do not stop
merely because ignored nested artifacts make the parent display a dirty-looking
submodule marker. Delegate the exact commit decision to `x-commit` (`.agents/skills/x-commit/SKILL.md`): the commit
owner commits the parent pointer when the nested repo has no tracked, staged,
conflicted, unignored, or actively changing work beyond the intended local
commit, then reports any unpushed nested commit as a push/share warning only.

## Reporting

Style completed phases, remaining work, verification, commits, and blockers
with shared colors when useful.

Keep status terse while working:

- **Phase N: done**
- **Phase N: blocked** because ...
- **Remaining:** ...
- **All approved phases done.**

Final reports must include commits created, verification run, remaining approved
phases, and any blockers.
