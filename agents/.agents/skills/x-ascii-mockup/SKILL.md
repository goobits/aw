---
name: x-ascii-mockup
description: 'Use when the user invokes $x-ascii-mockup or /x-ascii-mockup, asks for an ASCII mockup, wireframe, terminal sketch, layout sketch, flow sketch, dashboard/table mockup, checklist shape, CLI output shape, before/after view, or wants information shown visually in plain text.'
---

# X ASCII Mockup

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to turn an idea, workflow, page, terminal output, checklist, or
information layout into a clear plain-text mockup.

This is output-only by default. Do not edit files unless the user explicitly asks
to implement the mockup.

## When To Use

- UI or page wireframes
- CLI output shapes
- terminal dashboards
- owner checklists or forms
- before/after states
- order-of-operations views
- architecture, flow, or state diagrams
- compact comparisons where a table or box layout is easier than prose

## Rules

- Keep it readable in a terminal. Prefer simple boxes, aligned labels, short
  rows, and whitespace over dense art.
- Use polished terminal text by default. Prefer Unicode box drawing and symbols
  for terminal-capable output; fall back to ASCII when the user asks for ASCII
  only or the surface cannot render Unicode.
- Use color and styling directly in the mockup when it improves scanning. Do not
  describe the colors instead of applying them.
- Make labels concrete and short. Avoid filler words and decorative noise.
- Show useful states, not just the happy path: empty, loading, error, blocked,
  ready, selected, or done when relevant.
- Preserve the user's domain terms, but keep the text layman-friendly when the
  user asks for clarity.
- If multiple variants would help, show 2-4 compact options and name the tradeoff
  for each.
- Do not claim the mockup is final UI. Call it a sketch, shape, or output
  proposal.
- If the mockup implies implementation work, list the smallest next action after
  the sketch.

## Styling

Always produce a colorful response for terminal-capable output unless the user
asks for plain Markdown or strict ASCII. Use real ANSI escape sequences for
terminal or terminal-like chat output. Apply styles directly; do not describe
them in the output. Do not wrap the final colored mockup in a code fence,
because code fences often prevent ANSI color from rendering.

| Element                         | Style                 |
| ------------------------------- | --------------------- |
| mockup title or main object     | bold blue             |
| panel/column headers            | bold purple           |
| ready/done/saved/recommended    | bold green            |
| warning/needs attention/pending | bold yellow           |
| blocked/error/destructive       | bold red              |
| hints/secondary labels          | gray/dim              |
| literal commands, paths, values | monospace if Markdown |

Use these symbols consistently:

- `▌` section label
- `□` empty item or available choice
- `✓` done, ready, saved, or recommended
- `!` warning or needs attention
- `↳` hint under the relevant row
- `·` secondary option or detail

For ANSI mockup output, indent hint/detail child lines with four normal spaces.
Do not use non-breaking spaces unless the user explicitly asks for fixed
alignment that collapses with normal spaces.

### ANSI Palette

Emit the actual escape characters in the final answer, not the literal text
`\033`, not bash variables, and not a `printf` snippet. The shell snippet below
is only a precise palette reference.

```bash
BLUE='\033[1;38;5;75m'    # title/main object
PURPLE='\033[1;38;5;141m' # panel/column headers
GREEN='\033[1;38;5;114m'  # ready/done/saved/recommended
YELLOW='\033[1;38;5;221m' # warning/pending/needs attention
RED='\033[1;38;5;203m'    # blocked/error/destructive
DIM='\033[2;3m'           # hints/secondary labels
RESET='\033[0m'
```

For terminal-like output, try ANSI first even if the renderer might strip it.
Fallback to Markdown only when the user asks for plain Markdown or reports that
ANSI rendering failed. Use a fenced `text` block only when preserving exact
alignment is more important than rendered color.

## Output Shapes

Pick the smallest shape that communicates the idea. The examples below are
structure templates only; do not copy them as final output without applying the
styling rules above.

```text
Panel
╭─────────────────────────────╮
│ Status  ✓ Ready             │
│ Next    Run final smoke     │
╰─────────────────────────────╯
```

```text
Checklist
□ Value needed: ______
□ Decision: option A / option B / unsure
✓ Already handled
```

```text
Two-column comparison
╭─────────────┬────────────────────────╮
│ Option      │ Best when              │
├─────────────┼────────────────────────┤
│ Compact     │ user needs quick scan  │
│ Detailed    │ user must act on every │
╰─────────────┴────────────────────────╯
```

```text
Flow
Input → Check → Fix → Verify → Commit
          |
          ╰─ ! Blocked: ask owner
```

## Response Pattern

**Mockup**

<colored terminal sketch, unfenced unless strict fixed-width alignment matters more than rendered color>

**Why this shape**

- One or two bullets explaining the readability choice.

**Next**

- Optional smallest next action.

Skip `Why this shape` when the user only asks to see the mockup.
