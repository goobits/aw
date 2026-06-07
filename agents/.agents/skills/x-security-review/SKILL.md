---
name: x-security-review
description: 'Use when the user invokes $x-security-review or /x-security-review, asks to review auth, billing, sessions, CSRF, secrets, webhooks, PII, privacy, data deletion, permissions, access control, payment boundaries, or production security posture.'
---

# X Security Review

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill for high-scrutiny review of security, privacy, and abuse-risk surfaces. This is review-first. Do not edit files unless the user explicitly asks for fixes after the review.

## Context To Load

Always follow `AGENTS.md`. Read `.agents/policies/quality.md`,
`.agents/policies/code-standards.md`, `.agents/policies/testing.md`, and
`.agents/policies/git.md` when present. Load only relevant security docs:

- `.llm/docs/security/README.md` first when the scope is unclear.
- `.llm/docs/security/payment-card-boundary.md` for billing/payment scope.
- `.llm/docs/security/privacy-data-map.md`, `data-retention-policy.md`, or `privacy-request-runbook.md` for PII, export, deletion, and retention.
- `.llm/docs/security/secrets-and-server-access.md` for secrets and access.
- `.llm/docs/security/current-hashing-contract.md` for token/password/hash flows.
- `.llm/docs/security/access-review-runbook.md` for permissions and access reviews.

## Scope Recovery

1. Identify the target security surface and data involved.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present.
3. Read entrypoints, middleware, config, env templates, tests, migrations, logging, and caller/callee boundaries.
4. Trace trust boundaries: browser, server, webhook provider, database, third-party API, background script, admin, and internal service.

## What To Audit

- Auth/session correctness: identity source, session validation, token audience, expiry, revocation, cookie flags, app-session boundaries.
- Authorization: admin checks, ownership checks, team/member roles, service-to-service access, confused-deputy risks.
- CSRF/CORS/origin handling: first-party origins, unsafe methods, token validation, redirect safety.
- Billing/payment: no card data storage, webhook signature verification before processing, idempotency, provider failure handling, refund/cancel/lapse behavior.
- Secrets: no committed secrets, no logging secrets, env validation, rotation support where required.
- PII/privacy: least data retained, export/delete flows, audit logs, redaction, retention rules.
- Data integrity: transactional updates, race/idempotency, migration compatibility, irreversible operations.
- Logging: enough for audit/debugging without leaking secrets or PII.
- Tests: security behavior has targeted regression coverage.
- Existing-first fixes: before recommending new middleware, guards, policies,
  tests, docs, or tools, check for similar existing owners and prefer editing,
  rehoming, or consolidating them over creating a parallel security surface.

## Output

Lead with findings ordered by risk:

**Findings**

- Critical/High/Medium/Low: file:line - issue. Exploit/risk. Fix direction.

**? Open questions**

- ...

**✓ Verified / Not run**

- Commands or docs reviewed.

**Summary**

- **Block** / **Revise** / **Healthy with risk** / **Healthy**.

When reporting next steps, blockers, or order of operations, mark ownership only
where it clarifies who acts next. Use at most one marker per actionable item;
neutral/context lines can stay unmarked:

- `🫵` only for user input, approval, secrets, credentials, business decisions,
  legal or privacy calls, or external evidence.
- `🤖` for agent-owned implementation, verification, cleanup, docs, commits, or
  follow-up checks.

When one phase has both user-required input and agent-owned work, split it into
A/B subphases such as `Phase 2A: 🫵 User decision` and `Phase 2B: 🤖 Agent
work`, or label the exact `Blocked input:` line. Do not put `🫵` on a phase title
that also contains agent file edits.

If no issues are found, say `No findings` directly, then name residual security
risk or unverified surfaces.
