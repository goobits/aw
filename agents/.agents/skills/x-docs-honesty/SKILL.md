---
name: x-docs-honesty
description: 'Use when the user invokes $x-docs-honesty or /x-docs-honesty, asks to verify documentation against the current codebase, clean stale or inaccurate markdown, make README/docs truthful and right-sized, remove docs bloat, or audit docs for accuracy before editing.'
---

# X Docs Honesty

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill to make documentation factual, concise, and aligned with the
current code. It is based on the prompt palette entry
`.llm/scratch/prompt-palette/docs-clean.md`.

Read `.agents/policies/docs.md`, `.agents/policies/git.md`, and
`.agents.local/project.md` when present. Use `x-sync-docs`
(`.agents/skills/x-sync-docs/SKILL.md`) when the primary task is syncing docs
after a known code or workflow change. Use `x-consolidate-docs`
(`.agents/skills/x-consolidate-docs/SKILL.md`) when duplicate, scattered, or
misplaced docs need merge/delete/rehome decisions.

## Objective

Ensure markdown files represent current code reality with no content bloat:

- Accuracy first: every claim must match maintained code, config, commands, or
  project policy.
- Concise and dense: maximum useful signal, minimum prose.
- Right-sized: README files give overview and usage; API docs carry specifics.
- One canonical source per topic.

## Workflow

1. Identify the documentation scope: root docs, package/app/server README, docs
   folder, runbook, or changed feature.
2. Search before editing:
   - `rg "<term|path|command>" AGENTS.md README.md proposals .llm/docs <target paths...>`
3. Map docs to code owners:
   - public APIs, CLIs, config files, env vars, commands, routes, package
     exports, architecture claims, and usage examples.
4. Verify each claim against source:
   - API names, parameters, return values, and exported surfaces.
   - Installation, setup, dependency, and command instructions.
   - Config names, env vars, ports, flags, and service ownership.
   - Feature lists and architecture descriptions.
   - Links, paths, and runnable snippets.
5. Edit only what evidence supports:
   - Fix incorrect statements.
   - Remove obsolete or deprecated content.
   - Add missing critical information.
   - Preserve the local style and tone unless the user asks for a rewrite.

## Rules

- Prefer bullets over long paragraphs.
- Do not add hype, filler, or generic explanations.
- Do not document internal/private APIs in user-facing docs.
- Do not duplicate information across files.
- Do not expand docs just because a section exists.
- Do not modify proposals unless the user explicitly includes them in scope.
- Test examples when practical; otherwise mark them as unverified.

## Essential Coverage

For README or user-facing docs, ensure these are covered when applicable:

- Installation and setup.
- Basic usage.
- Configuration.
- Public API or CLI reference.
- High-level architecture.
- Troubleshooting or common failure modes.

## Verification

Run lightweight checks appropriate to the edit:

- `rg` for old names, stale paths, removed commands, and duplicated claims.
- Focused command or snippet checks when examples changed.
- Link/path sanity for moved docs.
- No build unless the user explicitly approved it.

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

```text
▌ Docs Honesty
✓ path - accurate/current after verification.
~ path - corrected stale or bloated content.
- path - removed obsolete or duplicated content.

▌ Evidence
· claim - source path or command used to verify it.

▌ Verified
· command/result, or not run with reason.

▌ Remaining
· unresolved source of truth, untested snippet, or owner question.
```
