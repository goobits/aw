---
name: x-code-review
description: 'Use when the user invokes $x-code-review or /x-code-review, asks for a code review of a specific directory, module, package, app, server, or subsystem, or asks whether an area is safe, A++, solid, clean, organized, standards-compliant, or ready to build on. Also supports diff, PR, and commit review when explicitly requested.'
---

# X Code Review

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this for review-only code-quality work. Do not edit or commit unless the
user asks for fixes after the review. Answer whether the area is healthy enough
to build on, what to fix first, and what blocks an A++ verdict.

## Context To Load

Always follow root `AGENTS.md` and repo safety rules. Prefer policy files before
deeper repo docs:

- `.agents/policies/quality.md` for the quality bar.
- `.agents/policies/code-standards.md` for language, import, file, type, and API
  rules.
- `.agents/policies/testing.md` for verification expectations.
- `.agents/policies/docs.md` for proposal, `.llm`, and changelog placement.
- `.agents.local/project.md` for repo layout and project direction.

For convention-heavy reviews, load only relevant references:

- `GLOSSARY.md` for domain terms, acronyms, and canonical names.
- `.llm/docs/conventions/common-patterns.md` for coding patterns.
- `.llm/docs/conventions/dependency-tiers.md` or the nearest equivalent for import and package-boundary rules.
- `.llm/docs/conventions/audit-standards.md` when the review is an audit doc or proposal.
- Relevant package-convention docs when package exports or new package structure are in scope.
- `proposals/core/19-typescript-strictness.md` when strictness, `any`, `unknown`, or typecheck gates are in scope.
- `.editorconfig` and `.vscode/settings.json` when formatting/editor consistency is relevant.
- The repo's harness-specific review command file when reviewing commits since
  the last reviewed commit or when the user asks for the repo review workflow,
  such as `.claude/commands/review.md` when present.

Do not bulk-read `.llm/docs/`; use indexes and nearest READMEs to choose.

## Recover Scope

1. Identify the review target: directory, module, package, app, server, or
   subsystem. Diff, PR, commit, and branch review are secondary modes only when
   explicitly requested.
2. Use the repo-approved Git workflow from `.agents.local/project.md` when
   present: path-scoped status, unstaged diff, staged diff, and commit/range
   inspection when commits are requested.
3. Map entrypoints, tests, exports, routes, config, migrations, and public
   surfaces with `rg --files <target>` or focused `find`.
4. Read representative files and trace important call paths; do not inspect only
   dirty files.
5. Separate unrelated dirty work from the target, while still reviewing the
   target's steady-state structure.

For commit review, use the local commit-review workflow when the user asks for
the repo's commit-review flow. Non-trivial commits require reading the diff and
citing one concrete diff detail.

## Review Standards

Review against repo standards. Prefer policy references over restating rule
sets:

- Quality: apply `.agents/policies/quality.md` when present.
- Code standards: apply `.agents/policies/code-standards.md` when present.
- File naming: apply local policy; in this repo, private TypeScript classes are
  `_PascalCase.ts` and private helpers are `_camelCase.ts`.
- Domain organization: code belongs with its domain language and ownership. Shared logic moves only when it is genuinely reusable.
- Package boundaries: same-package internals use relative imports; cross-package imports use package entrypoints or intentional subpaths; never import another package's `src/`.
- Public/private boundaries: exported package surfaces need clean names, JSDoc where required, and no accidental private helper leaks.
- Existing-first recommendations: before recommending a new helper, module,
  package, test file, doc, or tool, check for similar existing owners and prefer
  editing, rehoming, or consolidating them over creating a parallel surface.
- Strategy alternatives: when the best fix is not obvious, compare 2-3 viable
  approaches and recommend the smallest durable one. Include performance,
  boundary, testing, and migration tradeoffs only when they affect the decision.
- Type safety: narrow `unknown`, avoid `any`, do not add casts/suppressions to hide errors, and prefer discriminated unions for state.
- Tests: verify behavior contracts, not internal call sequences. Missing tests are findings when risk warrants coverage.
- Performance: route substantial runtime, memory, IO, query, rendering, or bundle
  optimization work to `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`) when a focused optimization workflow is
  needed.
- Security/privacy/data: treat auth, billing, sessions, secrets, PII, CSRF, webhooks, migrations, data deletion, and payment boundaries as high scrutiny.
- Frontend/UI: follow the existing design system and framework conventions, use the repo's standard component and icon libraries, avoid landing-page fluff for app surfaces, and flag overlapping/resizing UI issues.
- Documentation: apply `.agents/policies/docs.md` when present.

## Findings

Prioritize findings that would block or materially improve the change:

1. Correctness bugs and regressions.
2. Security, privacy, auth, billing, and data integrity risks.
3. API, package-boundary, migration, or caller-update breaks.
4. Missing tests for risky or user-facing behavior.
5. Duplication, organization drift, naming drift, or abstraction problems that create real long-term maintenance cost.

Do not report cosmetic nits unless they affect correctness, maintainability, accessibility, or repo standards.

For directory reviews, include these angles:

- Organization: files named by responsibility and local naming policy, no
  history names, no broad catch-all helpers, clear public/private split.
- Domain ownership: logic lives with the domain that owns it; shared helpers are genuinely cross-domain.
- Dependency direction: stable/lower-tier modules do not depend on app-specific or higher-tier modules.
- Duplication: repeated parsing, validation, config, SQL, route, UI, or type logic that should be centralized.
- Entry points and exports: package/app surfaces expose only intentional APIs and all callers use the clean API.
- Tests: coverage matches risk, tests assert behavior contracts, and fixtures are not hiding implementation coupling.
- Operational fit: config, secrets, migrations, logs, scripts, and docs match repo standards for that server/package/app.

## Verification

Recommend or run checks based on scope:

- Type/API: targeted typecheck, package check, or local broad check command when
  broad contracts changed.
- Unit behavior: focused Vitest/node tests first, then owning package tests when practical.
- Browser/rendering: Playwright; use `xvfb-run` for server-side WebGPU/WebGL tests.
- Full suites: only when explicitly requested, required for signoff, or smaller
  checks cannot cover the reviewed risk.
- Commit-review workflow: use the local review command; only mark reviewed after
  deep-dive triggers are resolved and the user asked for that workflow.

Separate pre-existing failures from failures caused by the reviewed change. Never suppress warnings, widen types, add casts, or hide errors to make verification look green.

## Output

Style verdicts, severity, blockers, checks, and residual risk with shared colors
when useful.

Lead with findings, ordered by severity. Use this shape:

**Findings**

- Severity: file:line - issue. Impact. Fix direction.

**? Open questions**

- ...

**✓ Verified / Not run**

- Commands run and results, or why checks were not run.

**Summary**

- **Short overall verdict.**

If there are no findings, say `No findings` directly, then list residual risk or unrun checks.

Verdict language:

- **Block**: must fix before building on or merging.
- **Revise**: usable after targeted cleanup or tests.
- **Healthy with risk**: structurally sound, with named residual risk.
- **Healthy**: no material issues found in the reviewed scope.
