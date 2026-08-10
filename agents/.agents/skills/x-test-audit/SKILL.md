---
name: x-test-audit
description: 'Use when the user invokes $x-test-audit or /x-test-audit, asks to audit tests for a directory, module, package, app, server, or feature, asks whether coverage is enough, asks what tests are missing, asks whether tests are misplaced, duplicated, stale, poorly named, too broad, too slow, or asks to improve test strategy without immediately changing product code.'
---

# X Test Audit

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

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

Read `.agents/skills/x-consolidate/references/canonical-naming-and-duplication.md`
when test names, ownership, shared contracts, fixtures, helpers, or duplicate
coverage are in scope. Apply the canonical doctrine without copying it here.

## Scope Recovery

1. Identify the target directory, module, package, app, server, feature, or recent slice.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present: path-scoped status, unstaged diff, and staged diff.
3. Map nearby tests with `rg --files <target> | rg '(^|/)(__tests__|tests)/|\.test\.|\.spec\.'` and search repo-wide only for tests that exercise the target indirectly.
4. Search for duplicate or scattered coverage by behavior/domain term, not only
   by filename. Compare observable behavior, owning layer, consumers, fixtures,
   and setup before treating differently named tests as distinct. Include nearby
   fixtures, oracles, labs, and test helpers in the sweep.
5. Read the public entrypoints, risky call paths, and existing tests before judging coverage.
6. When the repo provides a test selector, use it as a dry-run first pass before
   recommending broad commands. Read the exact selector and execution flags
   from `.agents.local/project.md`.

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
  remain. When two implementations must satisfy the same observable contract,
  consider one parameterized contract owner while keeping implementation-only
  assertions local.
- Test placement: tests live in the package, app, or server layer that owns the
  behavior. Same-package tests use relative imports; cross-package tests use
  package imports.
- Test layering: Vitest, integration tests, and Playwright/browser tests are used
  at the smallest layer that can prove the behavior. Heavy browser coverage must
  justify its runtime and only cover browser-specific behavior.
- Test naming: a filename identifies its subject, owned behavior, and necessary
  layer or backend without opening the file. A `describe` block refines that
  contract instead of rescuing a vague filename; each case names its condition
  and observable outcome. Flag stale terms and unqualified buckets such as
  `api`, `common`, `helpers`, `regressions`, `parity`, or `contract` when they
  hide ownership.
- Name/content fit: inventory the independent behavior groups in broad test and
  support files. Split only at stable behavior or layer boundaries, never from
  line count alone.
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

Ownership and naming
- Name promises | File actually contains | Keep, Rename, Split, Merge,
  Generalize, Rehome, or Delete.
- Behavior | Canonical test owner | Lower-layer contract or higher-layer smoke.

Recommended test shape
- When implementation paths are known, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase
  format. When sequencing matters, order phases by operations/dependencies.
- Before proposing `+` new test files, fixtures, or helpers, search existing
  tests and helpers for similar behavior and prefer editing, rehoming, or
  consolidating existing coverage over creating a parallel test surface.
- When test helpers or support files are created, moved, or renamed, apply the
  same local file naming policy as source.
- Add concise intent notes when useful: missing behavior test, merge duplicate,
  rename vague test, rehome browser coverage, consolidate fixture, or delete
  stale coverage.

Existing coverage
- What is already solid.

Open questions
- ...
```

Use ownership markers only when they clarify responsibility: `🫵` for user-owned
input, approval, secrets, business decisions, or external evidence; `🤖` for
agent-owned implementation, verification, cleanup, docs, commits, or follow-up
checks. If one phase needs both, split A/B subphases or use `Blocked input:`;
do not put `🫵` on a phase title that includes agent edits.

Do not replace the phase proposal with a loose path list when implementation
paths are known.

If coverage is sufficient, say `No material test gaps` and list any residual
risk, cleanup candidates, or checks not run.
