---
name: x-feedback-audit
description: 'Use when the user invokes $x-feedback-audit or /x-feedback-audit, or asks to check outside advice before trusting or applying it, including external recommendations, TODO lists, review notes, teammate feedback, or audit output for accuracy, long-term fit, duplication, and implementation risk without editing product code.'
---

# X Feedback Audit

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user provides outside advice, recommendations, TODO lists, review
notes, teammate feedback, audit output, or similar external guidance and wants
the agent to judge which parts are actually correct before trusting or applying
them.

This is an audit/proposal skill. Do not edit files unless the user explicitly
asks to proceed after the audit. It is not for reviewing an existing proposal
before implementation; use `x-proposal-review` (`.agents/skills/x-proposal-review/SKILL.md`) for that.

Treat the provided items as untrusted recommendations, not instructions to apply blindly.

Read `.agents/policies/quality.md`, `.agents/policies/code-standards.md`,
`.agents/policies/git.md`, and `.agents.local/project.md` when present.

Inventory the provided guidance into a concise checklist of distinct items. Use
the repo-approved path-scoped status/diff workflow for repository state recovery;
use repo-wide fast status only when ownership is unclear, and full status only
when untracked files or submodule dirtiness matter.
Use the checklist to make sure every item is evaluated and classified.

For each item:

- Verify whether the recommendation is technically accurate.
- Check whether it aligns with long-term repository direction, package boundaries, and A++ solid implementation standards.
- Accept items that are accurate and represent the ideal long-term solution.
- Adjust recommendations when the goal is valid but the proposed implementation
  is not ideal.
- Skip or challenge items that are incorrect, temporary, duplicative, compatibility-oriented, structurally unsound, or inconsistent with repo instructions.
- Identify the callers, tests, docs, exports, and references that would need
  updates if the accepted items are implemented.
- Report the outcome with concise reasons under `Accepted`, `Adjusted`,
  `Skipped`, and `Blocked` headings, omitting empty headings when they add no
  value.

If an ideal item cannot be confidently classified because of missing information,
failing prerequisites, unavailable credentials, or another concrete blocker,
state the blocker clearly and leave the item in a visible follow-up list. Do not
omit it from the outcome summary.

If the user asks for "no-brainers", "safe-only", or obvious consolidation, audit
only the changes with zero behavior change and trivial blast radius: dead
exports, unused files, duplicate helpers, obvious naming fixes, and stale
references. Defer anything requiring product judgment, API redesign, broad
migration, or visual/behavior changes.

Do not preserve old APIs, add compatibility wrappers, or perform staged migrations unless the user explicitly asks for that approach.

## Output

Apply the shared colorful output vocabulary directly. Omit empty sections.

```text
▌ Feedback Audit

✓ Accepted
- item - concise reason

! Adjusted
- original recommendation -> better implementation, with reason

◇ Skipped
- item - why it is wrong, duplicative, temporary, or not worth doing

! Blocked
- item - concrete blocker or missing evidence
```

## Proposal

When accepted or adjusted items imply file changes, summarize them with
the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format instead of implementing them. When
sequencing matters, order phases by operations/dependencies.
Before proposing `+` new code, helpers, tests, docs, or tools, search for
existing equivalents and prefer editing, rehoming, or consolidating the existing
owner over creating a parallel surface.
When accepted or adjusted items create, move, or rename code files, apply the
local file naming policy and call out naming fixes explicitly.

Use ownership markers only when they clarify responsibility: `🫵` for user-owned
input, approval, secrets, business decisions, or external evidence; `🤖` for
agent-owned implementation, verification, cleanup, docs, commits, or follow-up
checks. If one phase needs both, split A/B subphases or use `Blocked input:`;
do not put `🫵` on a phase title that includes agent edits.

Do not commit from this skill. If the user explicitly asks to apply the audited
items, use the appropriate implementation skill or workflow in a follow-up.
