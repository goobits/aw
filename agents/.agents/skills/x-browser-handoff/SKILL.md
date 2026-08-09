---
name: x-browser-handoff
description: 'Use when the user invokes $x-browser-handoff or /x-browser-handoff, asks for instructions to give a browser agent, asks to translate prior plans, checklists, provider setup, or launch tasks into a self-contained browser-only work order, or says the browser agent has no repository, filesystem, shell, or project context. Produce exact URLs, navigation, field/value mappings, duplicate prevention, safe stop points, and a structured redacted return receipt without executing the browser work.'
---

# X Browser Handoff

Turn established external-console work into standalone instructions that the
user can copy directly to a browser-only agent. Treat that agent as having no
conversation history, repository access, filesystem access, shell, or internal
project knowledge.

This is a reporting-only skill. Do not execute browser actions, mutate provider
state, or edit project files while producing the handoff.

## Workflow

1. Recover only the relevant browser-owned work from the conversation, supplied
   checklist, canonical tracker, or current configuration.
2. Remove source edits, tests, builds, commits, CLI commands, internal paths,
   implementation phases, and deferred scope that the browser agent cannot or
   should not perform.
3. Resolve every known non-secret value. Verify current console URLs and form
   terminology against official provider sources when they may have changed.
   Never guess a URL, account, environment, field value, or approval.
4. List unresolved owner inputs before the handoff. Mark affected assignments
   `NEEDS OWNER INPUT`; do not tell the browser agent to invent or infer them.
5. Split work by provider, account, environment, and authorization boundary.
   Keep test and production work separate. Preserve dependency order.
6. Emit one self-contained, copyable assignment per independent boundary using
   the required format below.

Prefer editing or reusing an exact existing provider resource over creating a
new one. Require the browser agent to inspect existing resources first and stop
on ambiguous ownership, unexpected duplicates, environment mismatch, or
material differences from the instructions.

## Information Rules

- Include exact public URLs, account or organization names, regions,
  environments, resource names, field labels, field values, buttons, expected
  results, and evidence to return when known.
- Replace internal shorthand with the minimum plain-language context needed to
  recognize the correct external resource.
- Never refer to “the project,” “the repo,” “the tracker,” “the files,” “above,”
  or “as discussed” inside the browser-agent assignment.
- Never include local paths, source filenames, terminal commands, commit IDs,
  implementation details, or instructions to edit code.
- Never include passwords, private keys, secret values, session cookies,
  recovery codes, payment details, or reversible secret hashes.
- Refer to secrets by approved secret-store label or destination only. When a
  provider reveals a new secret once, instruct the browser agent to pause before
  creation so the owner can capture it through the approved secure channel.
  The browser agent must never return the secret in chat.
- Use explicit placeholders such as `[OWNER INPUT REQUIRED: launch window]`
  only in a draft assignment. A `READY TO SEND` assignment contains no unresolved
  placeholders.

## Safety Boundaries

- Default to read-only inspection until a numbered step explicitly authorizes a
  mutation.
- Do not broaden permissions, change unrelated settings, perform cleanup, or
  create convenience duplicates.
- Confirm the provider, organization/account, region, and test/production
  environment immediately before every mutation.
- Require an existing-state snapshot and a rollback value before DNS, routing,
  deployment, authentication, billing, or production changes.
- Stop for owner interaction at login, MFA, CAPTCHA, hardware-key, wallet,
  payment-method, one-time-code, or one-time-secret screens. Never ask the owner
  to send those values through chat.
- For key rotation, create the replacement with the exact approved permissions,
  install and smoke it, then revoke the old key. Never revoke first.
- For financial work, state the exact mode, currency, amount, total cap, refund
  behavior, and permission restoration. Stop if any displayed total exceeds the
  approved cap.
- For production or customer-traffic changes, quote the exact authorization in
  the assignment. If it is absent, instruct inspection or preparation only.
- If the live UI differs materially from the verified labels or expected state,
  stop and return the visible labels and a redacted screenshot instead of
  improvising.

## Required Output

Start with one of:

- `Status: READY TO SEND`
- `Status: NEEDS OWNER INPUT`

When inputs are missing, list only the missing values or approvals before the
assignments. Then place every assignment in its own fenced `text` block so it
can be copied without surrounding commentary.

Use this exact assignment shape:

```text
BROWSER AGENT ASSIGNMENT
Task ID: <stable short id>
Objective: <one measurable browser outcome>

AUTHORIZED SCOPE
- <specific inspection or mutation that is allowed>

DO NOT
- Use a filesystem, shell, source repository, or local project documentation.
- Change anything outside the authorized scope.
- Return passwords, private keys, secret values, cookies, payment details, or recovery codes.

PREREQUISITES
- <required account, environment, owner presence, or approval>

INSTRUCTIONS
1. Open: <exact official URL>
2. Confirm before changing anything:
   - Provider: <value>
   - Account or organization: <value>
   - Environment: <test, sandbox, staging, or production>
   - Region, project, or resource: <value>
3. Inspect existing state:
   - <what must already exist or be absent>
   - Reuse the exact matching resource; do not create a duplicate.
4. Enter or update:
   - <field label> => <exact value>
   - Leave unchanged: <field labels or settings>
5. Submit:
   - <exact button or action>
6. Verify:
   - <visible result, resource id, delivery, request status, or health proof>

STOP AND RETURN BLOCKED IF
- <environment mismatch, duplicate, missing approval, unexpected value, or unsafe condition>

OWNER INTERVENTION
- <exact screen where owner must take over, or “None”>

RETURN EXACTLY
TASK_RECEIPT
task_id: <same task id>
status: complete | blocked | skipped
provider: <provider>
account_or_organization: <non-secret identifier>
environment: <environment>
started_at_utc: <ISO-8601>
completed_at_utc: <ISO-8601 or null>
existing_resource_found: yes | no | not_applicable
resource_ids: <non-secret ids or none>
changes_made: <concise list or none>
verification: <checks and visible outcomes>
redacted_screenshots_or_links: <references or none>
unexpected_differences: <details or none>
owner_action_needed: <exact action or none>
secrets_in_response: no
```

## Quality Gate

Before returning the handoff, confirm:

- Every assignment is understandable without this conversation.
- Every mutable step names the exact account and environment.
- Every form value is exact or explicitly blocked on owner input.
- Existing-resource inspection prevents accidental duplication.
- Test, production, financial, and credential boundaries are not combined.
- Owner-only interactions occur at the last responsible moment.
- The requested receipt proves what changed without exposing secrets.
- No browser assignment contains repo-only or robot implementation work.

Use `x-owner-checklist` (`.agents/skills/x-owner-checklist/SKILL.md`) first when
the source material still mixes human inputs with robot work. Use this skill
afterward to translate selected browser-owned items; do not copy
`x-owner-checklist` styling into the browser prompt.
