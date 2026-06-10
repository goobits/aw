---
name: x-push-audit
description: 'Use when the user invokes $x-push-audit or /x-push-audit, asks whether a repo/branch/commit is safe to push, asks to check committed secrets, keys, tokens, credentials, private files, generated artifacts, or push blockers before publishing, or asks for a pre-push safety audit.'
---

# X Push Audit

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to answer: "Is this repo, branch, commit range, or staged slice
safe to push?"

This is audit-only by default. Do not edit files, delete files, rotate secrets,
rewrite history, or remove uncommitted local keys unless the user explicitly asks
for a separate fix. The core concern is push-bound content: committed history,
staged changes, or the explicit commit/range the user wants to publish.

## Context To Load

Read `.agents/policies/git.md` and `.agents.local/project.md` when present. Use
the repo-approved Git workflow. Prefer local secret scanners when the repo
already has them; otherwise use read-only Git and `rg` checks.

## Scope

1. Identify what may be pushed:
    - current branch against its upstream
    - a named commit or commit range
    - staged changes
    - a specific repo/path the user names
2. Take a scoped dirty-state baseline with the repo-approved status/diff
   commands.
3. Audit committed or push-bound content first. Do not treat untracked local
   secret files as a push blocker unless they are about to be added, already
   staged, or ignored incorrectly in a way that makes accidental add likely.

## What To Check

- Committed secrets: API keys, cloud keys, private keys, tokens, OAuth secrets,
  webhook secrets, database URLs, session secrets, cookies, `.env` values, and
  credential JSON.
- Private files: SSH keys, TLS keys/certs, service-account files, production
  configs, backups, database dumps, paid/private assets, customer data, PII, and
  local machine artifacts.
- Push-bound generated artifacts: build output, screenshots, debug dumps,
  Playwright traces/videos, logs, caches, lock/repair artifacts, temp files, and
  large binaries that do not belong in source.
- Git hygiene: unrelated staged files, dirty submodules/nested repos, pointer
  changes without committed subrepo work, broken rename/delete patterns,
  accidental chmod-only changes, and commits that mix unrelated domains.
- Nested repo pushability: parent commits that point at nested/submodule commits
  not reachable from any remote branch. This is a push blocker when the parent
  pointer is in the audited push scope, even though it is not a local commit
  blocker. Report the nested path and commit that must be pushed first.
- Policy blockers: failing required local pre-push checks, missing verification
  for risky production/security changes, or docs that would publish sensitive
  hostnames, credentials, customer data, or internal-only runbooks.

## Suggested Inspection

Adapt to the repo's local queue/policy. In repos with a commit-owner queue,
worker tabs should not run Git queue internals directly; use repo-approved
read-only inspection or ask the `git` owner for the push-bound status, diff,
commit range, and stat output.

Use `rg` for high-signal patterns against the push-bound paths or diff, not as a
license to inspect unrelated private local files. Prefer scanning:

- tracked files in the commit/range
- staged files
- files changed since upstream

Do not paste full secret values into the report. Redact to a short prefix/suffix
only when needed to identify the finding.

## Safety Rules

- Never remove uncommitted keys or local `.env` files just because they exist.
  Report only whether they are staged, tracked, committed, or at high risk of
  accidental add.
- Never run history rewrite, reset, restore, checkout, stash, or secret removal
  tooling unless the user explicitly asks for a fix plan and approves it.
- If a committed secret is found, treat it as a push blocker. Recommend
  immediate rotation and history cleanup, but do not perform those steps unless
  explicitly asked.
- If a file looks sensitive but may be a template, verify whether values are
  placeholders before calling it a secret.
- If scanner output is noisy, classify findings as confirmed, likely, or
  false-positive/template.

## Output

Style verdicts, blockers, warnings, checked scope, and next steps with shared
colors when useful.

Lead with the push verdict:

**Verdict**

- **Safe to push** / **Do not push** / **Probably safe with noted risk** / **Incomplete audit**

**Blockers**

- High: path or commit - committed secret/private file/push blocker. Action.

**Warnings**

- Medium/Low: path - issue. Why it matters.

**Checked**

- Scope, commit range, staged/tracked/untracked handling, and commands run.

**Not touched**

- Local uncommitted keys/env/private files intentionally left alone.

**Next**

- The smallest safe next step.

If no blockers are found, say `No push blockers found in the audited scope` and
name any residual risk, such as unscanned history or unavailable upstream.
