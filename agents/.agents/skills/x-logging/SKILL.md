---
name: x-logging
description: 'Use when the user invokes $x-logging or /x-logging, asks to design, audit, or refactor logging, add structured logs, standardize log levels/env vars, propagate request context, remove noisy logs, or make logs useful across services/modules.'
---

# X Logging

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill to make logging structured, contextual, and useful without
adding noise. It is based on `.llm/scratch/prompt-palette/logging.md`.

Read `.agents.local/project.md` and nearby package docs before changing logging.
Prefer existing logger packages, adapters, and conventions over introducing a
new logging stack. In this repo, check `@goobits/logger` ownership before adding
or changing shared logging behavior.

## Objective

Every log should say what happened, where it happened, and the operational
context needed to debug it. Logging should be container-friendly, test-friendly,
and consistent across modules.

## Principles

- Use structured logs for machine-readable fields.
- Use human-readable logs for local development when supported.
- Log to stdout/stderr by default; file logging is optional and explicit.
- Use one module-scoped logger per component or package.
- Propagate context automatically where the runtime supports it.
- Keep context flat and stable; avoid deeply nested ad hoc payloads.
- Include structured error details and stack traces where useful.
- Avoid logging secrets, tokens, credentials, private payloads, or unnecessary
  PII.

## Standard Context

Use existing repo names when they differ. Otherwise prefer:

- Request: `request_id`, `session_id`, `user_id`, `method`, `path`.
- Operation: `operation`, `component`, `batch_id`, `duration_ms`.
- Error: `error_code`, `error_type`, `status_code`.

For async context propagation:

- Node.js: `AsyncLocalStorage`.
- Python: `contextvars`.
- Rust async: task-local context where appropriate.

## Standard Environment

Use existing repo env names when present. Otherwise prefer:

- `LOG_LEVEL`: `debug`, `info`, `warn`, `error`.
- `LOG_FORMAT`: `json`, `human`, or `auto`.
- `LOG_OUTPUT`: `console`, `file`, or `both` only when file output is supported.
- `LOG_TAGS`: optional context-key filtering for local debugging.

## Audit Workflow

1. Identify current logger APIs, env vars, and call sites.
2. Search for noisy, unstructured, or unsafe logging:
   - `rg "console\\.|logger\\.|log\\(|LOG_|debug" <scope>`
3. Classify findings:
   - missing context
   - wrong level
   - noisy success-path logs
   - secrets/PII risk
   - inconsistent logger initialization
   - duplicated local logger wrappers
4. Prefer a small wrapper or adapter only when it removes real duplication and
   fits the existing package boundary.
5. Update tests or snapshots that intentionally cover log output.

## Rules

- Do not add broad logging churn without a debugging or operations need.
- Do not hide errors by downgrading them to debug logs.
- Do not replace an established logging package without a proposal.
- Do not add audit, metrics, or tracing systems under the name of logging.
- When logging work creates, moves, or renames code files, apply local file naming policy.
- Suppress or stabilize logs in tests through existing test hooks.

## Verification

Run lightweight checks appropriate to the change:

- Focused tests for logger formatting/context behavior.
- `rg` for removed unsafe fields or old logger wrappers.
- Type/lint checks for touched packages when practical.
- No broad build unless explicitly approved.

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

```text
▌ Logging
~ path - structured/contextual logging updated.
! path - noisy, unsafe, or inconsistent logging remains.

▌ Context
· fields/env vars/logger API used.

▌ Verified
· command/result, or not run with reason.
```
