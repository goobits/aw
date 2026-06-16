---
name: x-lint-cleanup
description: "Use when the user invokes $x-lint-cleanup or /x-lint-cleanup, asks to clean up lint, type, warning, or check issues after a phase, or says phrases like 'check if we can cleanup any types with this phase done'."
---

# X Lint Cleanup

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use after a phase when the user asks to clean up lint, TypeScript types,
warnings, check failures, or type duplication. Fix the real issue; do not hide
diagnostics with suppressions, casts, wider types, or ignored warnings.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/testing.md`, and `.agents/policies/git.md` when present.

Workflow:

1. Establish the current lint/type state with the smallest relevant command:
   file/package check first, repo check only when the affected surface is broad
   or no narrower command exists.
2. Identify lint and type cleanup made possible by the completed phase: stale `.d.ts` files, duplicate exported types, trivial aliases, exposed `any`/`unknown`, references to removed APIs, and lint warnings tied to the touched slice. Do not treat the whole repo as the cleanup target unless the user explicitly asks for broad cleanup.
3. Remove or tighten only when ownership and call sites are clear.
4. When cleanup creates, moves, or renames code files, apply local file naming policy
   instead of preserving stale or public-looking names.
5. Fix lint/type errors directly; never widen types, add casts, suppress diagnostics, ignore warnings, or hide failures to make checks pass.
6. Do not delete package-boundary types only because same-package references are absent; verify package exports and cross-package consumers first.
7. Use targeted package checks after small cleanup batches when the affected package has a clear command.
   When the repo provides a test selector and lint/type cleanup touches test or
   runner surfaces, use the selector in dry-run mode to choose any needed
   behavior checks before falling back to broad suites.
8. Before calling the cleanup complete, rerun the narrowest check that covers the
   edited slice. Run the repo's full check command only when the user asks, the
   cleanup touches broad/shared contracts, targeted checks cannot cover the
   risk, or local policy requires full-suite signoff.
9. If this skill edits files and verification passes, commit the scoped cleanup by default using `x-commit` (`.agents/skills/x-commit/SKILL.md`).

If cleanup is blocked by pre-existing broad lint/type debt, report the exact blocker and the smallest safe next slice. Separate pre-existing failures from failures introduced by the current cleanup.

Do not commit if verification fails, ownership is unclear, target paths overlap unrelated work, the cleanup cannot be cleanly scoped, or the user explicitly says not to commit.

Output:

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

```text
Cleaned
- path - diagnostic fixed and why it belongs to this slice

Verified
- command/result

Blocked / Remaining
- pre-existing or out-of-scope diagnostics with the smallest safe next slice
```
