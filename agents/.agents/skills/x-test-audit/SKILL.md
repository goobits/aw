---
name: x-test-audit
description: 'Use when the user invokes $x-test-audit or /x-test-audit, asks to audit tests for a directory, module, package, app, server, or feature, asks whether coverage is enough, asks what tests are missing, asks whether tests are misplaced, duplicated, stale, poorly named, too broad, too slow, or asks to improve test strategy without immediately changing product code.'
---

# X Test Audit

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to review test coverage, placement, naming, maintainability, and
test-suite quality for a specific target area. Optimize for confidence per line
of durable test code: the audit should find missing coverage and bad excess
coverage, then recommend the smallest clear long-term test shape. This is
review/proposal-first. Do not edit files unless the user explicitly asks to
implement the recommended tests.

Use `x-consolidate-tests` (`.agents/skills/x-consolidate-tests/SKILL.md`) when the user asks to implement the merge, rehome,
rename, simplify, or delete work identified by this audit.

Read `.agents/policies/testing.md`, `.agents/policies/quality.md`, and
`.agents/policies/code-standards.md` when present. In another project, use that
repo's equivalent test, quality, and code-standard policies.

## Scope Recovery

1. Identify the target directory, module, package, app, server, feature, or recent slice.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present: path-scoped status, unstaged diff, and staged diff.
3. Map nearby tests with `rg --files <target> | rg '(^|/)(__tests__|tests)/|\.test\.|\.spec\.'` and search repo-wide only for tests that exercise the target indirectly.
4. Search for duplicate or scattered coverage by behavior/domain term, not only
   by filename. Include nearby fixtures and test helpers in the sweep.
5. Read the public entrypoints, risky call paths, and existing tests before judging coverage.
6. When the repo provides a test selector, use it as a dry-run first pass before
   recommending broad commands. In this repo, use
   `pnpm run test:select -- --path <target>` when the right focused check is
   unclear.

## What To Audit

- Behavior contracts: tests assert observable behavior users or callers depend on,
  not private call order or incidental implementation shape.
- Risk coverage: auth, billing, data deletion, migrations, storage, rendering,
  import/export, and package APIs have tests proportional to blast radius.
- Edge cases: external inputs, malformed payloads, failed providers, missing
  config, concurrency/idempotency, permission boundaries, and migration
  compatibility.
- Coverage gaps: name the missing test and the behavior it should prove.
- Coverage overlap: identify tests that prove the same behavior in multiple
  places and recommend the merge, deletion, or narrower smoke test that should
  remain.
- Test placement: tests live in the package, app, or server layer that owns the
  behavior. Same-package tests use relative imports; cross-package tests use
  package imports.
- Test layering: Vitest, integration tests, and Playwright/browser tests are used
  at the smallest layer that can prove the behavior. Heavy browser coverage must
  justify its runtime and only cover browser-specific behavior.
- Test naming: filenames, `describe` blocks, and test cases name the behavior and
  current domain terms precisely. Flag vague names like `works`, stale terms, and
  file names that hide ownership.
- Test maintainability: setup is concise, fixtures/helpers are owned in one clear
  place, repeated assertions are collapsed when one stronger behavior test would
  prove more with less maintenance, and helpers do not hide the contract being
  tested.
- Superfluous tests: flag stale, shallow, compatibility-era, duplicate, or
  brittle tests that add maintenance cost without meaningful confidence.
- Test reliability: no sleeps, broad snapshots, hidden network dependency, order
  coupling, leaked state, or fixture mutation leaks.
- Verification commands: use targeted commands first; use Playwright for
  browser/rendering and `xvfb-run` for server-side WebGPU/WebGL tests.
- Do not recommend a full suite merely for confidence. Reserve full regression
  for explicit user request, release/signoff requirements, or cases where
  targeted checks would be misleading.

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

Lead with findings:

```text
Findings
- Severity: file:line - missing, weak, misplaced, duplicated, stale, poorly
  named, slow, brittle, or poorly layered test coverage. Risk. Recommended fix.

Recommended test shape
- When implementation paths are known, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase
  format. When sequencing matters, order phases by operations/dependencies.
- Before proposing `+` new test files, fixtures, or helpers, search existing
  tests and helpers for similar behavior and prefer editing, rehoming, or
  consolidating existing coverage over creating a parallel test surface.
- Add concise intent notes when useful: missing behavior test, merge duplicate,
  rename vague test, rehome browser coverage, consolidate fixture, or delete
  stale coverage.

Existing coverage
- What is already solid.

Open questions
- ...
```

When reporting next steps, blockers, or order of operations, mark ownership only
where it clarifies who acts next. Use at most one marker per actionable item;
neutral/context lines can stay unmarked:

- `🫵` only for user input, approval, secrets, credentials, business decisions, or
  external evidence.
- `🤖` for agent-owned implementation, verification, cleanup, docs, commits, or
  follow-up checks.

When one phase has both user-required input and agent-owned work, split it into
A/B subphases such as `Phase 2A: 🫵 User decision` and `Phase 2B: 🤖 Agent
work`, or label the exact `Blocked input:` line. Do not put `🫵` on a phase title
that also contains agent file edits.

Do not replace the phase proposal with a loose path list when implementation
paths are known.

If coverage is sufficient, say `No material test gaps` and list any residual
risk, cleanup candidates, or checks not run.
