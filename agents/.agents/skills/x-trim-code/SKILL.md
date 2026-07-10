---
name: x-trim-code
description: 'Use when the user invokes $x-trim-code or /x-trim-code, asks to aggressively reduce LOC/code surface in a named target while preserving behavior or visual parity, asks for ideal organization, strict public/private boundaries, no duplicate code, no new packages/modules/Go modules, and wants the result as an x-proposal before implementation.'
---

# X Trim Code

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this for target-scoped code-surface reduction: net-smaller,
better-organized, behavior-preserving, strict public/private boundaries, and no
duplicate durable surface.

This is audit/proposal-first. Do not edit files unless the user explicitly asks
to proceed after the trim proposal.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/testing.md`, `.agents/policies/git.md`, and
`.agents.local/project.md` when present. Keep repo-specific commands and module
rules there.

## Target Recovery

1. Treat the phrase after the skill invocation as the target: app, package,
   feature, route, tool, module group, directory, or recent slice.
2. If the target is ambiguous, infer the smallest likely target from repo search
   and state the assumption before proposing changes.
3. Use repo-approved scoped state checks from `.agents.local/project.md`.
4. Map entrypoints, exports, callers, tests, routes, demos, manifests, and
   nearby helpers before judging what can shrink.
5. Search by behavior/domain terms, not only filenames, so parallel helpers,
   stale abstractions, and misplaced owners are visible.

## What To Trim

- Dead, stale, unused, legacy, compatibility, demo-only, or temporary code that
  is no longer required.
- Duplicate helpers, adapters, literals, validation, parsing, config, type
  definitions, fixtures, route wiring, or rendering logic that should have one
  owner.
- Misplaced code that can move to the existing owning package, app, server,
  feature, or private helper without creating a new durable surface.
- Public/private boundary drift: accidental exports, public-looking private
  helpers, cross-package `src/` imports, app wiring inside reusable packages, or
  reusable logic hidden in app-local code.
- One-use or low-value abstractions that can be inlined or folded into an
  existing owner without reducing clarity.
- Repeated call-site logic that should use an existing stable API.
- Local performance waste only when trimming removes work while preserving
  behavior. Route primary performance work to `x-optimize-code`
  (`.agents/skills/x-optimize-code/SKILL.md`).

## Hard Rules

- Aim for net-negative LOC. If the proposal is net-positive, explicitly justify
  why the added code lowers future maintenance.
- Do not create packages, workspace entries, package subpaths, public modules,
  Go modules, helper libraries, registries, or broad shared owners unless the
  user asks or the new surface replaces a larger scattered one.
- Do not preserve old APIs with compatibility wrappers unless the user
  explicitly asks for a staged migration.
- When trimming creates, moves, or renames code files, apply the local file
  naming policy; private TypeScript classes use `_PascalCase.ts` and private
  helpers use `_camelCase.ts` in this repo.
- Do not rename, move, or narrow public API without naming affected callers and
  migration impact.
- Do not move code across ownership boundaries without naming the new owner and
  affected callers.
- Do not delete code unless usage and coverage make the deletion safe.
- Do not trade correctness, readability, boundary clarity, or visual parity for
  fewer lines.
- Do not combine unrelated cleanup with the trim target.

## Visual Parity

For frontend, canvas, rendering, route, game, demo, or UI-adjacent targets:

1. Identify the visible surfaces that can change.
2. Prefer existing Playwright, screenshot, canvas-pixel, visual-regression,
   route-smoke, or manual comparison checks named in `.agents.local/project.md`.
3. If direct visual verification is not practical during the audit, state the
   parity risk and propose the lightest sufficient check in the phase that would
   change visuals.
4. Do not propose style, layout, interaction, asset, animation, or rendering
   changes unless they are required to preserve behavior while shrinking code.

## Routing

- Use `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`) when
  the primary work is duplicate/scattered/stale code consolidation.
- Use `x-boundary-audit` (`.agents/skills/x-boundary-audit/SKILL.md`) when the
  primary risk is package/API ownership, imports, exports, or public/private
  boundary drift.
- Use `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`) when the
  primary finding is runtime, memory, IO, query, rendering, startup, or bundle
  performance and needs evidence-first optimization.
- Use `x-hardening-audit` (`.agents/skills/x-hardening-audit/SKILL.md`) when
  the user wants final release-readiness cleanup for an active slice rather than
  a target-wide trim plan.
- Use the canonical `x-proposal`
  (`.agents/skills/x-proposal/SKILL.md`) phase format for all recommended file
  changes. When sequencing matters, order phases by operations/dependencies.

## Output

Lead with findings. Before proposing `+` new helpers, modules, tests, docs,
tools, packages, or public surfaces, search for existing equivalents and prefer
deleting, editing, rehoming, renaming, or consolidating an existing owner.

If `Total LOC` is positive, add one short `Why net-positive is still trimming:`
line before `Layman's wins`.

Apply the shared colorful output vocabulary directly. Keep the section labels
scan-friendly: `▌ Trim Audit`, `▌ Trim Proposal`, `▌ Visual Parity`, `▌ Out Of
Scope`, and `▌ Open Questions`.

```text
▌ Trim Audit
Scope
· target paths/domains intentionally audited

! Severity  file:line - excess, duplicate, misplaced, boundary, or parity risk. Trim direction.
✓ Keep       healthy owner or code path that should not move.

▌ Trim Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Visual Parity
✓ existing checks or proposed checks
· not applicable: reason

▌ Out Of Scope
◇ areas intentionally left alone

▌ Open Questions
· ownership, API, behavior, visual parity, migration, or verification decisions.
```

If no safe trim is found, say `No safe code trim found` and explain whether
`x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`),
`x-boundary-audit` (`.agents/skills/x-boundary-audit/SKILL.md`),
`x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`), or better visual
verification should run next.
