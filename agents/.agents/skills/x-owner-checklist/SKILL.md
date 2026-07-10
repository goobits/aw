---
name: x-owner-checklist
description: 'Use when the user invokes $x-owner-checklist or /x-owner-checklist, asks what they personally need to provide, asks for owner inputs, human-only tasks, decisions, approvals, credentials, external setup, or wants a concise form-like checklist instead of a robot implementation proposal.'
---

# X Owner Checklist

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Turn mixed project status, next steps, launch gates, or proposal output into a
concise list of what the human owner must fill in, approve, decide, create, or
provide. Reporting-only: do not edit files. For robot implementation phases, use
`x-proposal` (`.agents/skills/x-proposal/SKILL.md`) instead.

## Include

Include only human-owned items:

- credentials, accounts, secrets, tokens, ids, links, files, screenshots
- vendor access, external console work, owner verification
- approvals, business decisions, legal/compliance input, payment approvals
- production go/no-go calls

Exclude robot-owned implementation, docs, tests, commits, deployments, cleanup,
and follow-up checks unless a human item blocks that work.

Respect active scope. If a tracker or release explicitly defers an area, omit
that area's owner tasks unless the user asks to pull it forward. Mention it only
in `◇ Not needed:` or as one timing decision.

Keep the list short: prefer the top 5-12 owner items. If there are many, collapse
an area to one line. Do not repeat the same context in the section title, item
label, and hint.

## Item Rules

- Use one short `□` checkbox line per owner action.
- Use `______` only where the owner must paste or provide a value.
- Do not ask for values already recorded in the active tracker/docs. Omit known
  values, or ask for confirmation only when the owner must verify them
  externally.
- Add a hint only when it speeds the decision. Put it directly under the item as
  `    ↳ _Tiny hint._`; never as a separate paragraph.
- For decisions, keep the checkbox line as the label only. Put choices on
  child lines that start with four U+00A0 non-breaking spaces and a visible `↳`
  marker:
    - `    ↳ ✓ Recommended: <answer>`
    - `    ↳ · Other: <meaningful alternative>`
- Include exactly one `✓ Recommended:` line when the best answer is reasonably
  inferable. Do not add filler alternatives like `no`.
- If no recommendation is supported, omit `↳ ✓ Recommended:` and use
  `↳ · Other:` lines, or `↳ · Other: need more info`.
- Use a blank line between full items, but never between a checkbox and its
  child lines.

## Grouping

Use one compact list unless grouping clearly improves scanning. When grouping,
include only non-empty sections:

- `▌ Values`: concrete values, emails, ids, links, tokens, credentials, files,
  screenshots, or account names.
- `▌ Decisions`: approvals, readiness checks, created/not-created,
  available/not-available, proceed/not-yet, go/no-go calls, and choices with
  tradeoffs.

Add `◇ Not needed:` only when it prevents wasted work or resolves confusion.
Keep it one line by default. Use bullets only if there are too many items for one
readable line.

Do not use an `Evidence` bucket by default. If proof is needed, phrase it as a
value or yes/no item, such as `Verification video link: ______` or
`Webhook checked: yes / no`.

If no owner action is needed, say `Nothing needed from you right now` and list
the robot-owned next step separately in one line.

## Styling

Always produce a colorful response. Use real ANSI escape sequences for terminal
or terminal-like chat output unless the user asks for plain Markdown. Apply
styles directly; do not describe them in the output. Do not wrap the checklist
in a code fence, because code fences often prevent ANSI color from rendering.

| Element                           | Style                                   |
| --------------------------------- | --------------------------------------- |
| `Need From You`                   | bold blue                               |
| `▌ Values`                        | bold yellow                             |
| `▌ Decisions`                     | bold purple                             |
| `↳ ✓ Recommended:` and the answer | bold green                              |
| `↳ · Other:`                      | gray/dim                                |
| `↳ _Hint._`                       | dim italic, directly under its checkbox |
| `◇ Not needed:`                   | gray/dim                                |
| `□` checkbox lines                | plain/default                           |

Child lines are every `↳ ✓ Recommended:`, `↳ · Other:`, and `↳ _Hint._` line.
Use exactly four U+00A0 non-breaking spaces before each child line so chat
renderers preserve the indent. Keep the visible `↳` marker too, so hierarchy is
still clear if a surface strips spacing.

Do not output `Quick Answers`; use `Need From You`. Do not include `LOC`,
`Verify`, `+`/`~`/`-`, commit ids, or implementation phases.

### Shape Template

This is the structure only. Do not copy it as the final output without applying
the ANSI styles below.

```text
Need From You

▌ Values
□ Google OAuth credentials: ______
    ↳ Accounts production sign-in.

▌ Decisions
□ Launch mode
    ↳ ✓ Recommended: not ready yet
    ↳ · Other: ready
    ↳ Missing live sign-in proof.

◇ Not needed: deferred areas, completed smokes, already-recorded values.
```

### ANSI Palette

Emit the actual escape characters in the final answer, not the literal text
`\033`, not bash variables, and not a `printf` snippet. The shell snippet below
is only a precise palette reference.

```bash
BLUE='\033[1;38;5;75m'    # Need From You
YELLOW='\033[1;38;5;221m' # ▌ Values
PURPLE='\033[1;38;5;141m' # ▌ Decisions
GREEN='\033[1;38;5;114m'  # ↳ ✓ Recommended + answer
DIM='\033[2;3m'           # ↳ · Other, ↳ hints, ◇ Not needed
RESET='\033[0m'
NBSP='    '               # four U+00A0 non-breaking spaces

printf "${BLUE}Need From You${RESET}\n\n"
printf "${YELLOW}▌ Values${RESET}\n"
printf "□ Google OAuth credentials: ______\n"
printf "${NBSP}${DIM}↳ Accounts production sign-in.${RESET}\n"
```

For terminal-like output, try ANSI first even if the renderer might strip it.
Fallback to Markdown only when the user asks for plain Markdown or reports that
ANSI rendering failed. If indentation collapses, the visible `↳` markers should
still show hierarchy; do not drop them.
