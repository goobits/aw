---
name: x-optimize-code
description: 'Use when the user invokes $x-optimize-code or /x-optimize-code, asks to optimize code, improve performance, speed up runtime, reduce memory churn, reduce IO/query/rendering/bundle cost, or make a hot path faster without changing behavior.'
---

# X Optimize Code

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill for evidence-first code optimization. The goal is to improve
runtime, memory, IO, query, rendering, startup, or bundle performance while
preserving behavior and keeping the codebase simpler or equally maintainable.

This is audit/proposal-first unless the optimization is obvious, local,
behavior-preserving, and low-risk. Do not start broad rewrites without a
measured baseline or a concrete bottleneck.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/testing.md`, `.agents/policies/git.md`, and
`.agents.local/project.md` when present.

## Workflow

1. Identify the target path, user-visible slowdown, hot path, resource issue, or
   recent slice that needs optimization.
2. Recover scoped state with the repo-approved Git workflow from
   `.agents.local/project.md` when present.
3. Establish evidence before changing code:
    - benchmark, profile, trace, logs, test timing, bundle report, query plan,
      memory snapshot, or repeated local measurement
    - if direct measurement is not practical, state the inferred bottleneck and
      why the code path is plausibly hot
4. Form one optimization hypothesis at a time: what is slow, why, and what
   change should improve it.
    - When multiple optimization strategies are plausible, compare 2-3 options
      first: remove work, batch work, cache/index work, move work, or measure
      more. Prefer deleting or avoiding work over caching unless ownership and
      invalidation are clear.
5. Prefer the smallest behavior-preserving change that removes real work:
    - avoid repeated parsing, allocation, cloning, sorting, traversals, regexes,
      serialization, network calls, queries, layout work, or rendering work
    - batch, precompute, memoize, cache, or index only when invalidation and
      ownership are clear
    - delete or simplify slow layers when an existing stable API already proves
      the same behavior
6. Verify behavior with targeted tests/checks, then rerun the relevant
   measurement or timing when practical.
7. If edits pass verification, commit the scoped optimization by default using
   `x-commit` (`.agents/skills/x-commit/SKILL.md`).
8. When the user asks what to optimize next, asks for recommended work, or when
   implementation paths are known, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format.
   When sequencing matters, order phases by operations/dependencies. Keep
   `Target` and `Evidence` before the proposal when they clarify the
   optimization.
9. Before proposing `+` new optimized helpers, caches, scripts, route probes, or
   tooling, search for existing equivalents and prefer editing, rehoming, or
   consolidating the existing owner over creating a parallel performance path.

## Guardrails

- Do not trade correctness, readability, or boundary clarity for speculative
  speed.
- Do not add caches without explicit invalidation rules, ownership, and memory
  limits when relevant.
- Do not change public behavior unless the user explicitly approved the behavior
  change.
- Do not optimize by widening types, suppressing diagnostics, hiding warnings, or
  skipping tests.
- Do not combine unrelated cleanup with optimization unless it directly removes
  the measured bottleneck.
- Do not commit if verification fails, ownership is unclear, target paths
  overlap unrelated work, or the user explicitly says not to commit.
- Do not return a loose numbered list of implementation work when paths,
  commands, or tool names are known; convert it into `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phases.
- Use `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`) when the primary work is merging, deleting, rehoming,
  or simplifying duplicated/scattered code.
- Use `x-hardening-audit` (`.agents/skills/x-hardening-audit/SKILL.md`) when the primary work is release-readiness cleanup,
  scope control, or LOC reduction.

## Output

Apply the shared colorful output vocabulary directly. Keep evidence visually
separate from the proposal so the user can tell what was measured versus what is
inferred.

```text
▌ Optimization
Target   path/flow and why it matters

Evidence baseline, profile, timing, trace, or stated inference

! Bottleneck
  concrete slow path or inferred hot path

▌ Optimization Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Verified
✓ behavior checks and measurement after the change
· not run: reason

▌ Remaining
□ follow-up bottlenecks, risks, or cases that still need measurement
```

If no safe optimization exists, say `No safe code optimization found` and
explain whether `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`), `x-hardening-audit` (`.agents/skills/x-hardening-audit/SKILL.md`), or a better
measurement setup should run next.
