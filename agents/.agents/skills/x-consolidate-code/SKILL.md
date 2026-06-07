---
name: x-consolidate-code
description: 'Use when the user invokes $x-consolidate-code or /x-consolidate-code, asks to find duplicated, scattered, overlapping, stale, misplaced, or low-value code, asks what code should be merged, deleted, rehomed, renamed, or simplified, or asks for a consolidation plan for helpers, modules, types, exports, utilities, adapters, or package internals.'
---

# X Consolidate Code

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to identify code that should be merged, deleted, rehomed,
renamed, or simplified. This is audit/proposal-first. Do not edit files unless
the user explicitly asks to proceed after the consolidation plan.

Consolidation should reduce durable code surface by default. Prefer editing
existing owners, deleting stale code, moving code to its proper owner, and
collapsing repeated call-site logic before adding a helper or module. New code
is only consolidation when it replaces a larger duplicated or scattered surface.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/git.md`, and `.agents.local/project.md` when present. Keep
repo-specific package and command details in those files instead of this skill.

## Scope Recovery

1. Identify the package, app, server, module group, feature, or recent slice.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present: path-scoped status, unstaged diff, and staged diff.
3. Map entrypoints, exports, callers, tests, and nearby helpers before judging
   whether code is duplicated or misplaced.
4. Search by behavior/domain terms, not only filenames, so parallel helpers and
   stale abstractions are visible.

## What To Consolidate

- Duplicate helpers, adapters, utilities, literals, validation, parsing, config,
  or type definitions that should have one owner.
- Overlapping modules or abstractions that express the same responsibility.
- Misplaced code that belongs in a different package, app, server, or domain.
- Stale compatibility wrappers, legacy adapters, dead exports, unused files, or
  temporary terminology.
- Public/private boundary drift caused by helpers living in public-looking paths
  or private details leaking through exports.
- Repeated call-site logic that should use an existing stable API.

## Safety Rules

- Prefer a proposal over edits. Consolidation can change behavior, package
  boundaries, public APIs, migrations, or cross-package callers.
- Aim for net-negative LOC. If the proposal is net-positive, explicitly justify
  why the added code lowers future maintenance.
- Do not preserve old APIs with compatibility wrappers unless the user explicitly
  asks for a staged migration.
- Do not move code across ownership boundaries without naming the new owner and
  affected callers.
- Do not delete code unless usage and coverage make the deletion safe.
- If tests need consolidation too, call that out and route implementation to
  `x-consolidate-tests` (`.agents/skills/x-consolidate-tests/SKILL.md`).
- If the finding is primarily runtime, memory, IO, query, rendering, or bundle
  performance, route implementation to `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`).
- If the finding is mostly package/API ownership, reference `x-boundary-audit` (`.agents/skills/x-boundary-audit/SKILL.md`).
- If the finding is mostly release-readiness cleanup near the active slice,
  reference `x-hardening-audit` (`.agents/skills/x-hardening-audit/SKILL.md`).

## Output

Lead with findings. When implementation paths are known, the consolidation plan
must use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order
phases by operations/dependencies.
Before proposing `+` new helpers, modules, or shared owners, search for existing
equivalents and prefer merging into, moving, or renaming an existing owner over
creating another surface.
If `Total LOC` is positive, add one short `Why net-positive is still
consolidation:` line before `Layman's wins`.
Apply the shared colorful output vocabulary directly. Keep the section labels
scan-friendly: `▌ Code Consolidation`, `▌ Consolidation Proposal`, `▌ Out Of
Scope`, and `▌ Open Questions`.

```text
▌ Code Consolidation
! Severity  file:line - duplicated, scattered, stale, misplaced, or low-value code. Risk. Consolidation direction.
✓ Keep       current canonical owner or healthy area.

▌ Consolidation Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Out Of Scope
◇ Areas intentionally not consolidated.

▌ Open Questions
· Ownership, API, behavior, migration, or verification decisions.
```

If no safe consolidation is found, say `No material code consolidation findings`
and list any unverified surfaces.
