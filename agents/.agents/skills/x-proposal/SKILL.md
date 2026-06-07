---
name: x-proposal
description: "Use when the user invokes $x-proposal or /x-proposal, asks for a TLDR tree diff proposal, compact file-change proposal, edit/create/delete proposal, phased plan with LOC +/- estimates, total LOC +/- summary, a final layman's wins summary, or a strategy/tradeoff proposal before implementation."
---

# X Proposal

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use when the user asks for "tree diff", "TLDR tree diff", "tree diff proposal", "edit/create/delete", or a compact phase proposal showing what files would be created, edited, or deleted.

This is a planning/reporting skill. Do not edit files unless the user explicitly asks to proceed after the proposal.

When the user asks about coding strategy, implementation approach, tradeoffs, or
"best way to build this", compare 2-4 viable approaches before the phase plan:

- `Option`: short name.
- `Fit`: when it is the right choice.
- `Tradeoff`: performance, maintenance, boundary, test, or migration cost.
- `Verdict`: keep one recommended approach and explain why it is the cleanest
  long-term fit.

Each proposed phase must be independently stoppable: it should leave the repo in a coherent state, avoid known breakage, include its own verification step, and not depend on uncommitted follow-up work to be safe.

When sequencing matters, phases must be ordered by operations/dependencies:
earlier phases should unblock later phases, and verification should happen as
soon as the relevant slice can be proven.

Use this compact format:

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
Verify: targeted check or test
~ path/to/file.ts
+ path/to/new-file.ts

Phase 2: Short goal
LOC: +N / -N
Verify: targeted check or test
- path/to/old-file.ts
~ path/to/caller.ts

Total LOC: +N / -N

Layman's wins
- ...
```

Each phase must include a `LOC: +N / -N` line with the best current estimate.
Use a tight range when exact counts are not knowable before implementation, such
as `LOC: +20-40 / -10-25`.

Every proposal with one or more phases must end with a `Total LOC: +N / -N`
line immediately before `Layman's wins`. Sum the best current estimate across
all phases. Preserve ranges when any phase uses ranges, such as
`Total LOC: +100-180 / -20-45`.

## Existing-First Guard

Before proposing work, check whether the requested behavior, helper, doc, test,
tool, queue command, workflow, or owner already exists elsewhere. Do this even
when the likely answer is to edit existing files, because the best proposal is
often "reuse or consolidate this existing thing" rather than "build a new one".

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

Keep entries specific enough to guide implementation. Avoid listing speculative files unless clearly marked as candidates. If the proposal depends on codebase discovery, inspect the relevant files first and separate confirmed changes from likely follow-ups.

Prefer concise notes after the tree diff only when needed for risk, sequencing,
order-of-ops, phase boundaries, or open questions. Do not turn the response into
a prose design doc.

When reporting next steps, blockers, or order of operations, mark ownership only
where it clarifies who acts next. Use at most one marker per actionable item;
neutral/context lines can stay unmarked:

- `🫵` only for user input, approval, secrets, credentials, business decisions, or
  external evidence.
- `🤖` for agent-owned implementation, verification, cleanup, docs, commits, or
  follow-up checks.

When one phase has both user-required input and agent-owned file edits, split it
into A/B subphases or label the exact input line. Do not put `🫵` on a phase
title that also contains agent work, and do not put ownership emojis on `+`,
`~`, or `-` file-change lines:

```text
Phase 2A: 🫵 User decision
LOC: +0 / -0
Verify: decision recorded
Blocked input: final import/archive decision and production data source

Phase 2B: 🤖 Agent evidence template
LOC: +80-140 / -0
Verify: links to import, compatibility, and activation commands are current
+ docs/release/data-cutover-evidence.md
~ docs/release/launch-tracker.md

Total LOC: +80-140 / -0

Layman's wins
- ...
```

End with `Total LOC` and a short `Layman's wins` section that states the
practical upside in layman's terms. Keep wins to 2-4 bullets, focused on what
gets cleaner, safer, faster, easier to maintain, or easier to understand.
