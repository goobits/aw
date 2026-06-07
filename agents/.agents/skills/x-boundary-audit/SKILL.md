---
name: x-boundary-audit
description: 'Use when the user invokes $x-boundary-audit or /x-boundary-audit, asks whether code lives in the right package/API place, or asks to audit package boundaries, imports, exports, public/private API drift, circular dependencies, workspace membership, module manifests, or domain ownership.'
---

# X Boundary Audit

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to check whether code lives in the right package/API place. It
reviews dependency direction, package ownership, imports/exports, and
public/private boundaries. This is audit/proposal-first. Do not edit files
unless the user explicitly asks to proceed.

## Context To Load

Always follow `AGENTS.md`. Read `.agents/policies/quality.md`,
`.agents/policies/code-standards.md`, `.agents/policies/git.md`, and
`.agents.local/project.md` when present. Load deeper docs only when the scope needs
them:

- Relevant dependency-tier docs for package dependency direction.
- Relevant package-convention docs for package export shape and structure.
- Relevant module-dependency docs when auditing module structure.
- `.llm/docs/reference/typescript-strictness-roadmap.md` when boundary issues are type-surface related.

## Scope Recovery

1. Identify the target package, app, server, or module group.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present.
3. Map files and imports:
    - `rg --files <target>`
    - `rg "^import .* from " <target>`
    - inspect package manifests, exports, TypeScript config, workspace manifests,
      and module manifests when present.
4. Read callers when exports or APIs changed.

## What To Audit

- Same-package internals use relative imports with TypeScript source extensions.
- Cross-package imports use package entrypoints or intentional package subpaths, never another package's `src/`.
- Dependency direction follows stable-to-unstable rules and documented tiers.
- Public exports are intentional, documented when required, and suitable for generated API docs.
- Private helpers live in private-looking files (`_underscoreCamelCase.ts`) or internal folders.
- No compatibility wrappers, global namespace shims, lazy circular-dependency workarounds, or legacy bridges unless explicitly approved.
- Shared logic is extracted only when genuinely reusable and placed in the stable owning domain.
- Workspace membership belongs in the local project's authoritative workspace
  manifest, not duplicated in secondary manifests.
- App/tool wiring stays in app/tool domains, not generic utility packages.
- Tests follow the same boundary rules as source.

## Output

Lead with findings. When recommending file changes, the boundary proposal must
use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order
phases by operations/dependencies.
Before proposing `+` new packages, entrypoints, adapters, helpers, or modules,
search for existing owners and prefer moving, re-exporting intentionally, or
editing the current owner over creating a parallel boundary.
Apply the shared colorful output vocabulary directly. Keep the section labels
scan-friendly: `▌ Boundary Findings`, `▌ Boundary Proposal`, `▌ Healthy`, and
`▌ Open Questions`.

```text
▌ Boundary Findings
! Severity  file:line - boundary issue. Impact. Fix direction.

▌ Boundary Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Healthy
✓ Areas checked with no concern.

▌ Open Questions
· ownership decisions or package placement uncertainty
```

If no material issues are found, say `No material boundary findings` and list any unverified surfaces.
