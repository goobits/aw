---
name: x-docs-style
description: 'Use when the user invokes $x-docs-style or /x-docs-style, asks to restyle README.md or markdown docs, make docs clean/scannable/polished, apply documentation style guidelines, improve README structure, or rewrite docs for concise professional presentation without changing factual meaning.'
---

# X Docs Style

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill to make documentation cleaner, more scannable, and better
structured while preserving factual meaning. It is based on the prompt palette
entry `.llm/scratch/prompt-palette/docs-style.md`.

Prefer `x-docs-honesty` (`.agents/skills/x-docs-honesty/SKILL.md`) first when
the facts may be stale or unverified. Use `x-sync-docs`
(`.agents/skills/x-sync-docs/SKILL.md`) after code changes when docs only need
alignment. Use `x-consolidate-docs`
(`.agents/skills/x-consolidate-docs/SKILL.md`) when multiple docs need merging,
deletion, or rehoming.

## Objective

Transform README or markdown documentation into clean, factual, reader-friendly
docs that respect the reader's time.

## Style Principles

- Professional, clear, understated tone.
- Concise, precise, factual descriptions.
- No hype, sales language, or unsupported claims.
- Code examples over explanation when examples are clearer.
- Real, runnable examples with language-tagged code fences.
- Group related content logically.
- Progress from simple to complex.
- Use bold only for navigation and key terms.

## README Shape

Use this structure when it fits the project. Include only relevant sections and
preserve existing project conventions when they are stronger.

- `# Project Name`
- One-line factual description of what it does.
- `## Key Features`
  - `**Feature** - Specific capability.`
- `## Quick Start`
  - Install, configure, and first-run commands in a `bash` code fence.
- `## Library Or API`
  - Real usage examples in the relevant language fence.
- `## Configuration`
  - Env vars, CLI flags, config files, or settings commands.
- `## Documentation`
  - Descriptive links and what each page contains.
- `## Development`
  - Setup, test, lint, and quality commands.
- `## License`
  - License name and `LICENSE` link when present.

## Section Guidance

- Put project name and one-line description first.
- Put features early, but make them factual rather than promotional.
- Keep setup minimal before first successful use.
- Show CLI flags, env vars, or config files when those are core workflows.
- Use optional sections only when useful: related projects, contributing,
  changelog, badges, screenshots, benchmarks, migration guides, FAQ, support.
- Prefer descriptive doc links over labels like "docs" or "more info".

## Emoji Policy

The source prompt allows functional emoji in headings. In this repo, default to
the surrounding documentation style. Add emoji only when the existing doc already
uses them or the user explicitly asks for styled/visual README headings.

## Rules

- Do not change factual meaning without evidence.
- Do not invent commands, APIs, screenshots, badges, support channels, or
  license details.
- Do not add decorative structure that makes the doc longer without helping
  scanability.
- Do not reformat proposals unless explicitly included in scope.
- If facts are questionable, stop styling that section and verify or report the
  uncertainty.

## Verification

Run lightweight checks appropriate to the edit:

- Markdown/path sanity with `rg` for renamed headings, links, commands, and old
  wording.
- Focused command checks for examples changed.
- No build unless the user explicitly approved it.

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

```text
▌ Docs Style
~ path - structure, wording, or examples improved.

▌ Preserved
· facts, commands, APIs, or local conventions intentionally left unchanged.

▌ Verified
· command/result, or not run with reason.

▌ Remaining
· sections needing fact verification or owner input.
```
