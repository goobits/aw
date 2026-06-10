---
name: x-proposal-review
description: 'Use when the user invokes $x-proposal-review or /x-proposal-review, asks to review a proposed plan before implementation, asks whether a proposal is A++, no cruft, no jank, no duplication, solid, clean, focused, or ideal long term.'
---

# X Proposal Review

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use when the user wants a proposed plan reviewed before implementation,
especially with language like "A++", "no cruft", "no jank", "no dupe",
"solid", "ideal long term", "clean", "is this proposal good?", "review this
proposal", or "audit this proposal".

Treat the proposal as something to pressure-test, not as approved implementation work. Do not start code changes unless the user explicitly asks to proceed after the review.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/testing.md`, and `.agents.local/project.md` when present.

Review the proposal for:

- Correctness: it solves the actual problem and does not rely on weak assumptions.
- Long-term fit: it aligns with repository direction, domain ownership, package boundaries, and stable APIs.
- Strategy quality: when multiple designs are plausible, it compares meaningful
  alternatives and chooses the smallest durable approach instead of assuming the
  first idea is best.
- Structural quality: it avoids compatibility wrappers, temporary bridges, legacy leftovers, circular dependencies, god modules, and broad catch-all abstractions.
- Existing-first proof: before accepting `+` new files, helpers, abstractions,
  tests, docs, or tools, verify the proposal checked for similar existing owners
  and chose reuse, editing, or consolidation when possible.
- Simplicity: it removes duplication and keeps the smallest clear design that will hold up.
- Completeness: it updates affected callers, tests, docs, exports, and references rather than leaving split APIs or half-migrations.
- Stoppability: each phase leaves the repo coherent, avoids known breakage, and has verification before moving on.
- Risk: it identifies behavior changes, migration costs, unclear ownership, and verification gaps.

If the proposal is solid, say so directly and mention the highest-risk assumptions to verify.

If the proposal is not solid, identify the specific parts that introduce cruft, jank, duplication, temporary structure, or long-term maintenance cost. Then provide the cleaner long-term version of the proposal instead of only criticizing it.

Prefer this outcome shape:

Style verdicts, keeps, required changes, blockers, and verification with shared
colors when useful.

- `Verdict`: **A++**, **Mostly solid**, or **Needs revision**.
- `Keep`: the parts of the proposal that are structurally sound.
- `Change`: the parts to revise, with concise reasons.
- `Ideal proposal`: the adjusted long-term proposal. When file changes are
  known, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters,
  order phases by operations/dependencies.
- `Verify`: the checks or tests needed before calling the work done.

Omit empty sections when they add no value. Keep the review direct and specific enough that the next implementation step is obvious.
