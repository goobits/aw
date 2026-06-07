---
name: x-consolidate-tests
description: 'Use when the user invokes $x-consolidate-tests or /x-consolidate-tests, asks to merge, rehome, rename, simplify, or delete duplicate, scattered, stale, low-value, misplaced, slow, or poorly layered tests while preserving or improving behavioral coverage.'
---

# X Consolidate Tests

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to actively improve test-suite shape after a test audit, proposal,
or clear local discovery shows scattered or duplicate coverage. The goal is
higher confidence with less durable test code. This is an editing skill.

Consolidated tests should usually be shorter, clearer, and less repetitive.
Prefer merging, rehoming, renaming, deleting duplicate coverage, and improving
existing helpers before adding new test files or fixtures. New tests are allowed
only after overlap and placement are cleaned up, and only for real missing
behavioral coverage.

Read `.agents/policies/testing.md`, `.agents/policies/quality.md`,
`.agents/policies/code-standards.md`, and `.agents/policies/git.md` when present.
In another project, use that repo's equivalent policies and commit workflow.

## Safety Rules

- Preserve or improve behavioral coverage. Do not delete a test unless equivalent
  or stronger coverage remains.
- Prefer merge, rehome, rename, or simplify before adding new tests.
- Aim for less durable test LOC. If test consolidation is net-positive, explain
  what duplicate, brittle, or missing behavior it replaces.
- Keep production behavior unchanged unless the user explicitly asked for product
  fixes discovered by tests.
- Use the smallest test layer that proves the behavior. Move heavy browser tests
  down to unit/integration tests only when browser behavior is not what is being
  proven.
- Keep package boundaries intact: same-package tests use relative imports;
  cross-package tests use package imports.
- Stop and use `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) first when ownership, behavior coverage, or deletion
  safety is unclear. When file changes are known, that proposal must use the
  canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order phases by
  operations/dependencies. If `Total LOC` is positive, include
  `Why net-positive is still consolidation:`.
- Before proposing `+` new test files, fixtures, or helpers, search existing
  tests and helpers for similar behavior and prefer merging into, rehoming, or
  renaming existing coverage over creating a parallel test surface.

## Workflow

1. Identify the target package, app, server, feature, or approved test-audit
   proposal.
2. Recover scoped state with the repo-approved Git workflow from
   `.agents.local/project.md` when present: path-scoped status, unstaged diff,
   and staged diff.
3. Map tests and helpers:
    - `rg --files <target> | rg '(^|/)(__tests__|tests)/|\.test\.|\.spec\.'`
    - Search behavior/domain terms repo-wide only when coverage may be indirect.
4. Read the overlapping tests, their fixtures/helpers, and the public behavior
   they prove before editing.
5. Consolidate in small batches:
    - Merge duplicate assertions into one stronger behavior test.
    - Rehome tests to the owning package/app/server layer and fix imports.
    - Rename files, `describe` blocks, and cases to precise current domain terms.
    - Delete stale, shallow, brittle, or duplicate tests only after coverage is
      preserved elsewhere.
    - Consolidate copied fixtures/helpers into the clearest owning test helper
      without hiding the behavior contract.
    - Add missing tests only after overlap and placement are cleaned up.
6. Run targeted tests after each risky batch, then the owning package/app test
   command when practical. Use Playwright for browser/rendering behavior and
   `xvfb-run` for server-side WebGPU/WebGL tests. Do not run a full suite unless
   explicitly requested, required for signoff, or necessary because targeted
   coverage cannot prove the preserved behavior.
7. If verification passes, commit the scoped consolidation by default using
   `x-commit` (`.agents/skills/x-commit/SKILL.md`).

## Output

Apply the shared colorful output vocabulary directly. Keep the report compact
and scannable; do not fall back to a paragraph summary when multiple test
changes were made.

```text
▌ Test Consolidation
✓ path - merged / rehomed / renamed / simplified / deleted, and what coverage remains

▌ Coverage Kept
✓ Behavior that remains proven after consolidation.

▌ Added
□ Any new tests added after consolidation, and why they were still needed.

▌ Verified
✓ command/result
· not run: reason

▌ Remaining
□ unclear ownership, risky coverage gaps, or follow-up proposal items
```

If no safe consolidation exists, say `No safe test consolidation found` and
explain whether `x-test-audit` (`.agents/skills/x-test-audit/SKILL.md`) or `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) should run next.
