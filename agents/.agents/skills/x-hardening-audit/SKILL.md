---
name: x-hardening-audit
description: 'Use when the user invokes $x-hardening-audit or /x-hardening-audit, asks for a final cleanup audit before shipping, or asks to deeply audit current work and nearby code for release-readiness cleanup, fewer lines, less cruft, scope creep, public/private boundary drift, duplication, or cleanup opportunities before implementation.'
---

# X Hardening Audit

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user wants a final cleanup audit before shipping, especially to
confirm the slice is release-readiness cleanup only: fewer lines, clearer
boundaries, less duplication, less cruft, and no accidental feature expansion.

Treat line-count reduction as a primary goal. Prefer deleting code, collapsing duplicate paths, removing stale adapters, and simplifying local structure when behavior stays the same and ownership is clear.

This is a proposal/audit skill. Do not edit files unless the user explicitly asks to proceed after the audit.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/testing.md`, and `.agents/policies/git.md` when present.

Start by recovering the local context:

1. Identify the active work from the conversation, scoped diffs, and relevant proposal notes.
2. Use the repo-approved path-scoped status/diff workflow when target paths are known. Use repo-wide fast status only when ownership is unclear, and full status only when untracked files or submodule dirtiness matter.
3. Read only the nearby files needed to evaluate the active slice, its call sites, exports, package boundaries, tests, and demos.
4. Separate the user's/current agent's work from unrelated dirty work. Do not audit broad unrelated work unless it directly affects the slice.

Audit for:

- Public/private boundary drift: accidental exports, package `src/` imports
  across boundaries, private classes/helpers living in public-looking filenames
  or paths, local file naming policy violations, missing package entrypoint
  updates, or app-specific wiring leaking into reusable packages.
- Scope creep: new product behavior, expanded feature surface, compatibility wrappers, legacy bridges, optional modes, or demo-only features not required for hardening.
- Cruft and duplication: duplicate helpers, parallel catalogs, stale names, dead exports, unused files, repeated literals, needless adapters, or temporary terminology.
- LOC reduction: code that can be deleted, folded into an existing helper, centralized behind an existing API, or simplified without changing behavior.
- Release risk near the slice: missing caller updates, stale tests, stale docs/proposals, unverified demos, migration holes, or dirty-file ownership conflicts that would make a commit unsafe.
- Performance opportunities only when they are local, behavior-preserving, and
  backed by evidence. Route broader optimization work to `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`).

Report with this compact structure. When proposing file changes, the proposal
must use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order
phases by operations/dependencies.
Before proposing `+` new code, helpers, tests, docs, or tools, search for
existing equivalents and prefer editing, rehoming, or consolidating the existing
owner over creating a parallel surface.
Apply the shared colorful output vocabulary directly. Keep the section labels
scan-friendly: `▌ Hardening Audit`, `▌ Proposal`, `▌ Out Of Scope`, and `▌
Verified / Gaps` when verification context matters.

```text
▌ Hardening Audit
Scope
· paths/domains intentionally audited

! Severity  file/path - concrete issue, why it matters
✓ Solid     checked area with no hardening concern

▌ Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Out Of Scope
◇ unrelated dirty work or broad codebase areas not audited
```

Use ownership markers only when they clarify responsibility: `🫵` for user-owned
input, approval, secrets, business decisions, or external evidence; `🤖` for
agent-owned implementation, verification, cleanup, docs, commits, or follow-up
checks. If one phase needs both, split A/B subphases or use `Blocked input:`;
do not put `🫵` on a phase title that includes agent edits.

Keep findings specific and actionable. If no hardening work remains in the audited slice, say that directly and list any residual verification gaps.

When proposing edits, prefer behavior-preserving cleanup. Do not propose compatibility wrappers, staged migrations, public API expansion, or feature growth unless the user explicitly asks for that direction.
