---
name: x-proposal
description: "Use when the user invokes $x-proposal or /x-proposal, asks for a TLDR tree diff proposal, compact file-change proposal, edit/create/delete proposal, phased plan with LOC +/- estimates, total LOC +/- summary, a final layman's wins summary, or a strategy/tradeoff proposal before implementation."
---

# X Proposal

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user asks for "tree diff", "TLDR tree diff", "tree diff proposal", "edit/create/delete", or a compact phase proposal showing what files would be created, edited, or deleted.

This is a planning/reporting skill. Do not edit files unless the user explicitly asks to proceed after the proposal.

When the user asks about coding strategy, implementation approach, tradeoffs, or
"best way to build this", compare 2-4 viable approaches before the phase plan:

- `Option`: short name.
- `Fit`: when it is the right choice.
- `Tradeoff`: performance, maintenance, boundary, test, or migration cost.
- `Verdict`: keep one recommended approach and explain why it is the cleanest
  long-term fit.

Each phase must be independently stoppable: coherent repo state, no known
breakage, its own verification, and no dependency on uncommitted follow-up work.

Verification defaults to the lightest sufficient check: typecheck, existing
test, build, or manual step. Propose a new test only for behavior no existing
check covers; treat it as a `+` surface under the Existing-First Guard.

When sequencing matters, phases must be ordered by operations/dependencies:
earlier phases should unblock later phases, and verification should happen as
soon as the relevant slice can be proven.

Use this compact format:

Style proposals with shared colors when useful. The fenced blocks are templates,
not literal output.

```text
+ path/to/new-file.ts
~ path/to/edited-file.ts
- path/to/deleted-file.ts
```

Meaning:

- `+` create
- `~` edit
- `-` delete

When phases are useful, group by numbered phase:

```text
Phase 1: Short goal
LOC: +N / -N
Verify: lightest sufficient check (typecheck, existing test, build, or manual step)
~ path/to/file.ts
+ path/to/new-file.ts

Phase 2: Short goal
LOC: +N / -N
Verify: lightest sufficient check (typecheck, existing test, build, or manual step)
- path/to/old-file.ts
~ path/to/caller.ts

Total LOC: +N / -N

Layman's wins
- ...
```

Each phase must include a `LOC: +N / -N` line with the best current estimate.
Use a tight range when exact counts are not knowable before implementation, such
as `LOC: +20-40 / -10-25`.

Every phased proposal must put `Total LOC: +N / -N` immediately before
`Layman's wins`. Sum all phase estimates and preserve ranges.

## Existing-First Guard

Before proposing work, search for an existing behavior, helper, doc, test, tool,
queue command, workflow, or owner. The best proposal is often reuse,
consolidation, or editing the current owner.

- Search the codebase, docs, and tests for similar behavior, names, domain
  terms, helpers, fixtures, scripts, and owners.
- Prefer `~` editing, extending, rehoming, or consolidating an existing owner
  over creating a parallel surface.
- Treat related skills, policies, scripts, tasks, docs, package exports, tests,
  fixtures, and existing CLI commands as candidate owners before proposing a new
  surface.
- Use `+` only when no suitable owner exists or the new file is clearly the
  cleanest long-term boundary.
- If a proposal includes any `+` new code file, helper, abstraction, test helper,
  tool, CLI command, durable doc, or new workflow, include a concise
  `Existing check:` note before the phase list naming what was checked and why
  reuse/editing was not enough.
- If the existing-first check reveals duplicate or overlapping owners, propose
  consolidation or rehoming first. Do not propose parallel work beside a similar
  owner unless the proposal explicitly explains why separation is cleaner long
  term.

## File Naming Guard

When a proposal creates, moves, renames, or deletes code files, apply local file
naming policy from `.agents/policies/code-standards.md` and
`.agents.local/project.md`. In this repo: `PascalCase.ts` public/normal classes,
`_PascalCase.ts` private/internal classes, `camelCase.ts` helpers/factories/features,
and `_camelCase.ts` private/internal helpers.

## Migration Finish Guard

When a proposal introduces a replacement model, staged migration, compatibility
path, new owner, or transitional abstraction, include a final deletion/hardening
phase unless the user explicitly asks for a spike only.

That phase removes obsolete legacy paths, compatibility wrappers, flags,
aliases, temporary branches, stale old-behavior tests, stale comments/docs, and
duplicate helpers or owners created during migration.

For migrations intended to simplify architecture, state the final LOC goal:
net lower, flat, or intentionally higher with a reason. Do not call the plan
done if both the old and new models remain permanent.

Keep entries specific enough to guide implementation. Avoid listing speculative files unless clearly marked as candidates. If the proposal depends on codebase discovery, inspect the relevant files first and separate confirmed changes from likely follow-ups.

Prefer concise notes after the tree diff only when needed for risk, sequencing,
order-of-ops, phase boundaries, or open questions. Do not turn the response into
a prose design doc.

Use ownership markers only when they clarify responsibility: `🫵` for user-owned
input, approval, secrets, business decisions, or external evidence; `🤖` for
agent-owned implementation, verification, cleanup, docs, commits, or follow-up
checks. If one phase needs both, split A/B subphases or use `Blocked input:`.
Do not put `🫵` on a phase title that includes agent work, and do not put
ownership emojis on `+`, `~`, or `-` file-change lines:

```text
Phase 2A: 🫵 User decision
LOC: +0 / -0
Verify: decision recorded
Blocked input: final production data source

Phase 2B: 🤖 Agent edit
LOC: +20-40 / -0
Verify: targeted check
~ path/to/file.md

Total LOC: +20-40 / -0

Layman's wins
- ...
```

End with `Total LOC` and a short `Layman's wins` section that states the
practical upside in layman's terms. Keep wins to 2-4 bullets, focused on what
gets cleaner, safer, faster, easier to maintain, or easier to understand.
