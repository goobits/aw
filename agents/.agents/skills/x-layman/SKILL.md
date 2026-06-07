---
name: x-layman
description: 'Use when the user invokes $x-layman or /x-layman, says layman, laymans, tldr layman, plain English, asks for a layman-friendly explanation of the prior technical result, or names a topic/list/checklist that should be summarized in plain language.'
---

# X Layman

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Put the prior answer, result, plan, error, review, audit, or decision into
layman's terms with a clean, skimmable shape. Do not default to a paragraph when
bullets, a small table, a checklist, or a simple visual block would be easier to
read.

If the user invokes this skill with a named topic, list, tracker, launch plan, or
doc name, summarize that target directly. Do not require a prior technical
result.

## Topic Scope Check

Use this section only when the user names a topic/doc/list/checklist instead of
asking to restate the prior answer.

1. Identify whether the request is narrow or broad. A single file/path is narrow;
   a launch list, roadmap, release, product, subsystem family, or "all" request
   is broad.
2. For broad topics, scan for the likely canonical doc plus linked docs, sibling
   trackers, and active checklists before producing the layman summary.
3. Include a compact `Covered` line when it helps show scope, such as
   `Covered    setup, migration, auth, final checks`.
4. If you only checked part of the likely scope, say `Partial` and name what was
   not checked. Do not present a partial summary as the whole picture.

## Output Shape

Pick the clearest shape for the prior result:

- Quick answer: 1 short sentence plus the bullets needed to preserve the useful
  signal.
- Decision/tradeoff: a 2-column table such as `Choice` / `Meaning`.
- Status/result: compact sections such as `Done`, `Still open`, `Risk`, and
  `Next`.
- Launch/checklist/task list: a status block with `Status`, `Ready`, `Needs`,
  `Covered`, `Risk`, and `Next` when useful.
- Human tasks: a form-like checklist, or route to `x-owner-checklist` (`.agents/skills/x-owner-checklist/SKILL.md`) when the
  user only needs their own inputs.
- Flow/structure: a tiny ASCII diagram when it makes the idea easier to see.

Use simple visual hierarchy, not decoration. Prefer short labels, aligned
tables, checkboxes, and concise bullets over long prose.
Use lightweight Markdown styling when it helps: **bold** labels/verdicts,
_italic_ caveats, and `monospace` paths/commands/literal values. Do not wrap the
whole answer in a code fence unless fixed-width alignment is the point.

## Rules

- Keep it concise, but do not omit important items just to hit a tiny line
  count. A simple topic can be 3-8 lines; a real launch list, checklist, or
  multi-part status can be longer when that is what makes it useful.
- Never return one dense paragraph when the answer contains 3 or more distinct
  facts, tasks, blockers, or risks.
- Avoid code, jargon, acronyms, and implementation detail unless essential.
- Preserve the important conclusion and risk.
- Do not add new recommendations, phases, or claims that were not in the prior
  technical result.
- For launch/status/task topics, include at least one explicit `Needs` or
  `Still open` line when unfinished work exists.
- When summarizing a broad named topic, do not collapse it to one subsystem just
  because that subsystem was easiest to find.
- Split combined `Needs` lines into separate short items when they describe
  different tasks.
- Add `Next` when the prior result makes the next action obvious.
- Do not make every answer a checklist. Use the format that makes the prior
  result easiest to scan.
- If the user also asks for `tree diff`, use `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) first, then add a
  short `Layman's wins` section only if `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) has not already included
  one.

## Examples

```text
Short version
- We cleaned up the changelog.
- The cutoff moved forward.
- No product code changed.
```

```text
| Item | Meaning |
|---|---|
| `x-do` (`.agents/skills/x-do/SKILL.md`) | Do all approved phases |
| `x-next` (`.agents/skills/x-next/SKILL.md`) | Show what remains or the next suggested phases |
```

```text
Done       [x] Changelog updated
Open       [ ] Commit if you want it saved
Risk       Low. Docs-only change.
```

```text
Status     Close, but not ready yet.
Covered    main setup, external checks, final smoke

Ready      [x] Main setup is done
Needs      [ ] Final real-world smoke
Needs      [ ] Fresh input or data
Needs      [ ] External service proof
Needs      [ ] Final account check

Risk       Shipping early could break the parts users depend on.
Next       Finish the final checks, then decide go/no-go.
```
