---
name: x-consolidate-todos
description: 'Use when the user invokes $x-consolidate-todos or /x-consolidate-todos, asks what the remaining todos are for a domain/area/feature, asks to consolidate scattered todos/checklists/roadmaps into one rolling tracker, asks to merge or dedupe overlapping todo docs, asks to put remaining work in order of operations, or asks for the single source-of-truth todo list for a product, platform, operations, account, service, or similar area.'
---

# X Consolidate Todos

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user wants one domain's remaining work pulled from scattered docs
into a single operations-ordered rolling tracker, with overlapping docs cleaned
up.

This is an editing skill. It owns exactly one tracker per domain. It commits the
scoped consolidation by default (the workflow ends in a commit), then reports the
verdict and remaining plan. Never commit when the user says not to.

Todo consolidation should reduce durable task surface. Prefer updating or moving
an existing tracker, demoting overlaps to pointers, and deleting absorbed lists.
Create a tracker only when no canonical one exists, and make it replace
scattered live task lists.

Scope is one domain per run. If the user names several domains, confirm whether
they want one tracker each or stop after the first.

Read `.agents/policies/docs.md`, `.agents/policies/quality.md`,
`.agents/policies/git.md`, and `.agents.local/project.md` when present. Keep
project-specific doc homes and commit mechanics in those policy files.

## Workflow

1. **Fix the domain and territory.** Identify the area and possible task homes:
   `proposals/`, `proposals/INDEX.md`, `.llm/docs/` and its `INDEX.md`,
   domain docs folders, package/server `README.md` files, and any
   `*-tracker.md`, `*-roadmap.md`, `*-readiness.md`, `launch-*.md`, or
   `remaining-*.md` files.
2. **Find every task source.** Sweep for open work across that territory:
    - `rg -l "\[ \]|TODO|FIXME|remaining|roadmap|readiness|launch" <territory paths>`
    - `rg -n "<domain term>" proposals .llm/docs <domain dirs>`
      List docs with live tasks plus context docs with no live tasks.
3. **Pick or create one tracker.** Reuse the canonical tracker when present. If
   several compete, choose the best-located one and merge the rest. If none
   exists, create one (see Placement). Finish with exactly one rolling tracker.
4. **Merge tasks in.** Pull every open item into the tracker. Dedupe aggressively:
   when two docs state the same task, keep one crisp wording. Drop items already
   shipped; move them to Completed Foundations only if they give useful context.
   Preserve real distinctions; do not collapse two genuinely different tasks.
5. **Order by operations.** Sequence remaining todos into numbered phases where
   each phase depends only on earlier phases. Put inputs/credentials/decisions
   first, verification and rollout decisions last, and tasks in execution order.
6. **Clean up overlap.** Delete absorbed task-list docs, or replace their task
   sections with a pointer when the doc still has rationale or history value.
   Leave no second live todo copy. Update affected indexes and READMEs.
7. **Verify lightly.** `rg` for the old task wording and removed filenames to
   confirm no dangling references. No build unless the user explicitly approved
   one.
8. **Commit.** Hand the scoped slice (tracker, cleaned docs, touched indexes) to
   `x-commit` (`.agents/skills/x-commit/SKILL.md`) with the established message:
   `Consolidate <domain> task tracking`.
9. **Report and plan.** Give the current verdict and the ordered remaining plan
   (see Report).

## Tracker document shape

The canonical tracker is one rolling Markdown doc:

```markdown
---
Status: Rolling
Date: <today, absolute>
Depends: <comma-separated source docs this tracker subordinates>
---

# <Domain> <Rollout|Task> Tracker

<one-paragraph orientation: what work this tracker covers>

## Current Verdict

<2-4 sentences: what is done/code-ready, what gates remain before complete>

## Remaining Tasks In Order

### Phase 1: <name>

- [ ] task

### Phase 2: <name>

- [ ] task

## Completed Foundations

- [x] already-shipped item that gives context

## Verification Log

- <date> <what was run and the result>

## Related Context

- <link to a subordinated doc and what it holds now>
```

Rules for the tracker:

- One rolling tracker per domain. It is the remaining-work source of truth.
- Creating a new tracker should be net smaller after absorbed task lists are
  demoted or removed. If the final docs LOC is net-positive, explain why the new
  tracker still lowers maintenance.
- Every remaining task is a `[ ]` checkbox under a numbered, ordered phase.
  Completed work is `[x]` under Completed Foundations, not deleted.
- Keep the frontmatter `Date` absolute and current; refresh it each run.
- `Depends`/`Related Context` link absorbed or superseded docs.
- No em-dashes anywhere in the tracker prose. Restructure with periods, commas,
  colons, or parentheses.

## Placement

- Default the tracker next to the domain it tracks: a domain proposal area
  (`proposals/<domain>/<domain>-rollout-tracker.md`) or the domain's docs folder
  (`<domain>/docs/remaining-order-of-ops.md`). Follow the precedent already set
  for sibling domains.
- Human-facing roadmaps/proposals stay under `proposals/`; LLM-facing synthesis
  stays under `.llm/docs/`. Do not create a root-level tracker.
- When the tracker lives in `proposals/`, update `proposals/INDEX.md`. When it
  lives in `.llm/docs/`, update `.llm/docs/INDEX.md`.

## Boundaries with other skills

- `x-sync-docs` (`.agents/skills/x-sync-docs/SKILL.md`) handles broad doc drift;
  this skill maintains one todo tracker for a domain.
- `x-next` (`.agents/skills/x-next/SKILL.md`) executes the tracker's next phase.
  This skill orders the tracker; it does not implement tasks.
- If the user asks for executable next phases after consolidation, use
  `x-next` (`.agents/skills/x-next/SKILL.md`) suggested-phase mode or `x-proposal` (`.agents/skills/x-proposal/SKILL.md`). Do not present the
  tracker summary as an approved implementation plan.
- `x-update-changelog` (`.agents/skills/x-update-changelog/SKILL.md`) records
  shipped history. This skill records remaining work.
- When the user wants a `+ ~ -` change preview before you touch docs, use
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) first, then run this skill. The preview must use the canonical
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order phases by
  operations/dependencies.
- Before proposing `+` a new tracker, search existing todos, trackers,
  proposals, and docs for the domain and prefer updating or moving the canonical
  owner over creating a parallel tracker.
- If the preview or final slice is net-positive, include a short
  `Why net-positive is still consolidation:` line in the report.
- Use ownership markers only when they clarify responsibility: `🫵` for
  user-owned input, approval, secrets, business decisions, or external evidence;
  `🤖` for agent-owned implementation, verification, cleanup, docs, commits, or
  follow-up checks. If one phase needs both, split A/B subphases or use
  `Blocked input:`; do not put `🫵` on a phase title that includes agent edits.

## Report

After committing, report:

Use this output shape:

```text
Verdict
- <one or two lines: where the domain stands>

Tracker
- <path> - canonical rolling tracker (N remaining across M phases)

Consolidated
- <path> - merged in / demoted to pointer / deleted

Remaining work
- Phase 1A: 🤖 <agent-owned work> - <count> tasks
- Phase 1B: 🫵 <user decision/input> - <count> tasks
...

Approval needed before implementation.

Committed
- Consolidate <domain> task tracking

Open questions
- decisions or inputs only the user can unblock
```

If no scattered task sources exist for the domain, say so and do not invent a
tracker.
