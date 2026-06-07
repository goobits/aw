---
name: x-investigate
description: 'Use when the user invokes $x-investigate or /x-investigate, asks how to figure something out, asks whether to use subagents, deeper research, local code reading, audits, experiments, probes, or wants an investigation plan before deciding what to build.'
---

# X Investigate

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to choose the smartest way to figure something out before coding.
It is discovery-first: decide what evidence is needed, gather only enough to make
the next decision, then report a clear path.

Do not edit product files from this skill unless the user explicitly asks to
proceed after the investigation. If implementation is needed, hand off to
`x-proposal` (`.agents/skills/x-proposal/SKILL.md`) for the phase plan.

## Workflow

1. Restate the unknown in one sentence.
2. Check whether the answer is likely local, external/current, behavioral, or
   judgment-based.
3. Pick the smallest useful evidence path:
    - **Local read**: inspect repo code, docs, configs, scripts, logs, or tests
      when the answer should exist in the workspace.
    - **Subagents**: use independent reviewers when the risk is judgment,
      architecture, security, consolidation, or "are we done done" confidence.
    - **Research**: use web or external docs when facts may be current, vendor
      specific, legal/security-sensitive, or outside the repo.
    - **Probe**: run a small command, smoke, benchmark, reproduction, or script
      when behavior is uncertain.
    - **Ask user**: ask only when a real owner decision, secret, external access,
      or preference is required.
4. Stop once the next action is clear. Do not keep researching after the answer
   is decision-grade.
5. If file changes are recommended, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format.

## When To Use Other Skills

- Use `x-feedback-audit` (`.agents/skills/x-feedback-audit/SKILL.md`) when the
  main input is outside advice or another agent's report.
- Use `x-code-review` (`.agents/skills/x-code-review/SKILL.md`) when the user
  wants a code quality verdict for a known scope.
- Use `x-test-audit` (`.agents/skills/x-test-audit/SKILL.md`) when the unknown is
  test coverage, placement, duplication, or strategy.
- Use `x-security-review` (`.agents/skills/x-security-review/SKILL.md`) when the
  unknown involves auth, secrets, billing, PII, access, or production security.
- Use `x-owner-checklist` (`.agents/skills/x-owner-checklist/SKILL.md`) when the
  output should be a human-only checklist of decisions, values, or approvals.

## Output

Use the smallest readable shape:
Apply the shared colorful output vocabulary directly when producing the full
shape. Keep the answer shorter when a quick check already resolves the unknown.

```text
▌ Investigation
Unknown
· what we need to figure out

Best path
✓ local read / subagents / research / probe / ask user

Why
· one or two bullets

Plan
1. first evidence step
2. second evidence step
3. stop condition or handoff
```

If the answer is obvious after a quick check, skip the full plan and give the
answer plus the evidence checked.
