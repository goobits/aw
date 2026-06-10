---
name: x-consolidate
description: 'Use when the user invokes $x-consolidate or /x-consolidate, asks for a broad consolidation pass, asks to run all consolidate skills, or asks to find duplicated, scattered, stale, overlapping, misplaced, low-value, or redundant code, docs, tests, and todos together for one scope.'
---

# X Consolidate

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill as the broad consolidation router for one package, app, server,
feature, domain, recent slice, or explicitly named scope. It coordinates:

- `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`)
- `x-consolidate-tests` (`.agents/skills/x-consolidate-tests/SKILL.md`)
- `x-consolidate-docs` (`.agents/skills/x-consolidate-docs/SKILL.md`)
- `x-consolidate-todos` (`.agents/skills/x-consolidate-todos/SKILL.md`)

Default mode is audit/proposal-first. Do not edit files unless the user
explicitly asks to apply the consolidation, proceed with a named phase, or clean
up a clear low-risk docs/todos overlap.

Consolidation should reduce durable surface area by default: fewer files, fewer
concepts, fewer helpers, fewer duplicate tests/docs, and lower long-term
maintenance. Prefer `~` edits and `-` deletions over `+` additions. A new file,
helper, tracker, fixture, or abstraction is an exception, and is only acceptable
when it removes or simplifies a larger existing surface.

Read `.agents/policies/quality.md`, `.agents/policies/testing.md`,
`.agents/policies/docs.md`, `.agents/policies/code-standards.md`,
`.agents/policies/git.md`, and `.agents.local/project.md` when present. Keep
project-specific paths, commands, and commit rules in those files.

## Scope Rules

1. Identify the target scope. Do not run a repo-wide consolidation unless the
   user explicitly asks for the whole repo.
2. Take a scoped dirty-state baseline with the repo-approved Git workflow from
   `.agents.local/project.md` when present.
3. Search for similar existing owners before proposing new code, tests, docs,
   helpers, fixtures, or trackers.
4. Aim for a net-smaller proposal. If total LOC is positive, explicitly explain
   why the added LOC still reduces long-term maintenance.
5. Treat unclear ownership, active changes, missing verification, and deletion
   uncertainty as blockers, not reasons to guess.

## Workflow

1. **Recover scope.** Name the area, likely owners, entrypoints, tests, docs, and
   task sources.
2. **Run code consolidation.** Use `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`) logic to find duplicate,
   scattered, stale, misplaced, low-value, or overlapping implementation
   surfaces.
3. **Run test consolidation.** Use `x-consolidate-tests` (`.agents/skills/x-consolidate-tests/SKILL.md`) logic to find duplicate,
   misplaced, brittle, slow, poorly named, or low-value tests. In default mode,
   report what should be merged/rehome/renamed/deleted rather than editing.
4. **Run docs consolidation.** Use `x-consolidate-docs` (`.agents/skills/x-consolidate-docs/SKILL.md`) logic to find duplicate,
   stale, conflicting, scattered, misplaced, or low-value docs and identify the
   canonical owner.
5. **Run todos consolidation.** Use `x-consolidate-todos` (`.agents/skills/x-consolidate-todos/SKILL.md`) logic to find
   scattered trackers/checklists/roadmaps and identify whether one canonical
   ordered tracker exists.
6. **Unify the plan.** Deduplicate findings across the four passes. If one
   finding spans code/tests/docs/todos, report it once with the affected
   surfaces.
7. **Sequence by operations.** Put prerequisites first: canonical owners,
   boundary moves, code cleanup, test consolidation, docs/todos cleanup, then
   verification and commit.

## Safety Rules

- Do not create a new shared owner until existing owners have been searched and
  ruled out.
- Do not propose additive consolidation by default. New owners must replace a
  larger duplicated/scattered surface, not sit beside it.
- Do not delete code, tests, docs, or todos unless an equivalent or better
  canonical owner remains.
- Do not let an editing child skill commit automatically during a broad default
  audit. If the user asks to apply, consolidate in scoped phases and use
  `x-commit` (`.agents/skills/x-commit/SKILL.md`) only after verification passes.
- If the user asks only for one consolidation type, use the specific child skill
  instead of this broad router.
- If performance is the main issue, route implementation findings to
  `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`).
- If package/API ownership is the main issue, reference `x-boundary-audit` (`.agents/skills/x-boundary-audit/SKILL.md`).

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

Lead with cross-surface findings. When implementation paths are known, the
consolidation plan must use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format.
If `Total LOC` is positive, add one short `Why net-positive is still
consolidation:` line before `Layman's wins`.

Use ownership markers only where they clarify who acts next: `🤖` for
agent-owned work and `🫵` for user inputs, approvals, credentials, business
decisions, or external evidence. If a phase has both, split it into A/B
subphases or put the user need on a `Blocked input:` line.

```text
Findings
- Severity: surface - duplicate, scattered, stale, overlapping, misplaced, or low-value item. Risk. Consolidation direction.

Consolidation proposal
- Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

Out of scope
- Areas intentionally not consolidated.

Open questions
- Ownership, deletion safety, behavior, verification, or user-owned inputs.
```

If no material consolidation is found, say `No material consolidation findings`
and list any unverified surfaces.
