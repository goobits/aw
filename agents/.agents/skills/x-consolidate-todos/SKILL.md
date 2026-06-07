---
name: x-consolidate-todos
description: 'Use when the user invokes $x-consolidate-todos or /x-consolidate-todos, asks what the remaining todos are for a domain/area/feature, asks to consolidate scattered todos/checklists/roadmaps into one rolling tracker, asks to merge or dedupe overlapping todo docs, asks to put remaining work in order of operations, or asks for the single source-of-truth todo list for a product, platform, operations, account, service, or similar area.'
---

# X Consolidate Todos

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use when the user wants the remaining work for one domain pulled out of scattered
docs and folded into a single rolling tracker, ordered by operations, with
overlapping docs cleaned up. Typical phrasing: "what are all remaining todos for
Do? consolidate into one rolling doc, make sure nothing else overlaps, order the
todos, clean up and commit, then give me plans."

This is an editing skill. It owns exactly one tracker per domain. It commits the
scoped consolidation by default (the workflow ends in a commit), then reports the
verdict and remaining plan. Never commit when the user says not to.

Todo consolidation should reduce durable task surface by default. Prefer
updating or moving an existing tracker, demoting overlapping task docs to
pointers, and deleting absorbed lists before creating a new tracker. A new
tracker is only acceptable when no canonical tracker exists, and it should
replace scattered live task lists.

Scope is one domain per run. If the user names several domains, confirm whether
they want one tracker each or stop after the first.

Read `.agents/policies/docs.md`, `.agents/policies/quality.md`,
`.agents/policies/git.md`, and `.agents.local/project.md` when present. Keep
project-specific doc homes and commit mechanics in those policy files.

## Workflow

1. **Fix the domain and its territory.** Identify the area and the
   directories/docs that could hold its tasks:
   `proposals/`, `proposals/INDEX.md`, `.llm/docs/` and its `INDEX.md`,
   domain docs folders, package/server `README.md` files, and any
   `*-tracker.md`, `*-roadmap.md`, `*-readiness.md`, `launch-*.md`, or
   `remaining-*.md` files.
2. **Find every task source.** Sweep for open work across that territory:
    - `rg -l "\[ \]|TODO|FIXME|remaining|roadmap|readiness|launch" <territory paths>`
    - `rg -n "<domain term>" proposals .llm/docs <domain dirs>`
      List every doc that carries domain tasks, plus docs that _describe_ the domain
      but hold no live tasks (these become "related context" links, not merges).
3. **Pick or create the one tracker.** If a canonical rolling tracker already
   exists, reuse it. If several compete, choose the best-located one as canonical
   and merge the rest into it. If none exists, create one (see Placement). There
   must be exactly one rolling tracker per domain when you finish.
4. **Merge tasks in.** Pull every open item into the tracker. Dedupe aggressively:
   when two docs state the same task, keep one crisp wording. Drop items already
   shipped; move them to Completed Foundations only if they give useful context.
   Preserve real distinctions; do not collapse two genuinely different tasks.
5. **Order by operations.** Sequence all remaining todos into numbered phases so
   each phase only depends on earlier phases. Inputs/credentials/decisions that
   unblock later work come first; verification and rollout decisions come last.
   Within a phase, list tasks in the order someone would actually do them.
6. **Clean up overlap.** For every doc you merged from, either delete it (if it
   was purely a task list now absorbed) or strip its task section down to a single
   pointer at the canonical tracker (if it still serves another purpose, like
   rationale or a historical checklist). Leave no second live copy of the same
   todos. Update `proposals/INDEX.md`, `.llm/docs/INDEX.md`, and any README that
   linked the removed or demoted docs.
7. **Verify lightly.** `rg` for the old task wording and removed filenames to
   confirm no dangling references. No build unless the user explicitly approved
   one.
8. **Commit.** Hand the scoped slice (the tracker plus the docs you cleaned up and
   the indexes you touched) to `x-commit` (`.agents/skills/x-commit/SKILL.md`). Use a message in the established form:
   `Consolidate <domain> task tracking`.
9. **Report and plan.** Give the current verdict and the ordered remaining plan
   (see Report).

## Tracker document shape

The canonical tracker is a single rolling Markdown doc. Match the established
shape:

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

- One rolling tracker per domain. It is the single source of truth for that
  domain's remaining work.
- Creating a new tracker should be net smaller after absorbed task lists are
  demoted or removed. If the final docs LOC is net-positive, explain why the new
  tracker still lowers maintenance.
- Every remaining task is a `[ ]` checkbox under a numbered, operationally ordered
  phase. Completed work is `[x]` under Completed Foundations, not deleted.
- Keep the frontmatter `Date` absolute and current; refresh it each run.
- `Depends`/`Related Context` link the docs this tracker absorbed or supersedes so
  nobody has to rediscover them.
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

- `x-sync-docs` (`.agents/skills/x-sync-docs/SKILL.md`) re-aligns docs broadly after code changes. This skill is narrower:
  it builds and maintains the one todo tracker for a domain. Use `x-sync-docs` (`.agents/skills/x-sync-docs/SKILL.md`) for
  general doc drift; use this skill to consolidate todos.
- `x-next` (`.agents/skills/x-next/SKILL.md`) executes the tracker's next phase. This skill produces and orders
  the tracker; it does not implement the tasks. The next phase should come from
  the tracker's ordered remaining-todos sequence.
- If the user asks for executable next phases after consolidation, use
  `x-next` (`.agents/skills/x-next/SKILL.md`) suggested-phase mode or `x-proposal` (`.agents/skills/x-proposal/SKILL.md`). Do not present the
  tracker summary as an approved implementation plan.
- `x-update-changelog` (`.agents/skills/x-update-changelog/SKILL.md`) records shipped history. This skill records remaining work. Do not
  duplicate changelog rules here.
- When the user wants a `+ ~ -` change preview before you touch docs, use
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) first, then run this skill. The preview must use the canonical
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order phases by
  operations/dependencies.
- Before proposing `+` a new tracker, search existing todos, trackers,
  proposals, and docs for the domain and prefer updating or moving the canonical
  owner over creating a parallel tracker.
- If the preview or final slice is net-positive, include a short
  `Why net-positive is still consolidation:` line in the report.
- When reporting next steps, blockers, or order of operations, mark ownership
  only where it clarifies who acts next. Use at most one marker per actionable
  item; neutral/context lines can stay unmarked. Use `🫵` only for user input,
  approval, secrets, credentials, business decisions, or external evidence; use
  `🤖` for agent-owned implementation, verification, cleanup, docs, commits, or
  follow-up checks. When one phase has both user-required input and agent-owned
  work, split it into A/B subphases such as `Phase 2A: 🫵 User decision` and
  `Phase 2B: 🤖 Agent work`, or label the exact `Blocked input:` line. Do not put
  `🫵` on a phase title that also contains agent file edits.

## Report

After committing, report:

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
