# AGENTS.md

Portable instructions for LLMs working in a repository that uses this shared
`.agents` system.

## Core Rules

- Be direct, calm, practical, and concise.
- On first startup in a repo/session, choose a short stable agent name for
  yourself. Keep that name for the life of the session and use it in handoffs
  where identity matters, especially commit requests.
- Ship long-term, A++ solid work. Avoid jank, cruft, duplication, compatibility
  wrappers, legacy leftovers, and temporary bridges unless the user explicitly
  asks for a staged migration.
- Prefer small, simple, focused changes that fit the existing project shape.
- Before creating or proposing new reusable code, helpers, tests, docs, tools,
  or trackers, search for similar existing owners and prefer editing, rehoming,
  or consolidating them over creating parallel surfaces.
- Keep public/private boundaries crisp and update callers to the clean API
  instead of preserving stale surfaces.
- When a policy file below is relevant, read it before acting. If a referenced
  file is missing in another project, infer conservatively from this file and
  the local repo.
- When local commands, layout, ports, or repo-specific overrides matter, read
  `.agents.local/project.md` if present.

## Safety Rules

Never run without explicit approval:

- destructive or history-changing Git commands such as `git reset`,
  `git stash`, `git pop`, `git checkout`, `git restore`, or
  `git commit --amend`
- repo builds, unless the local project policy says they are safe and the user
  has approved them
- commands that suppress, hide, or ignore errors and warnings

Always:

- read the repo Git policy before Git/package-manager operations
- use the repo testing policy for browser, rendering, WebGL, WebGPU, or
  regression tests
- update references when changing APIs, excluding minified/generated files
- fix errors and warnings instead of hiding them; they are the audit trail

## Policy Map

Shared guidance lives in `.agents/`. Repo-specific details live outside the
shared system in `.agents.local/project.md`.

- `.agents.local/project.md`: local project layout, direction, commands, dev
  server details, and overrides.
- `.agents/policies/quality.md`: SOLID, SDP, DRY, DDD, KISS/FIST, IDEALS, and
  the no-cruft quality bar.
- `.agents/policies/git.md`: Git, package-manager, queue, submodule, and commit
  rules.
- `.agents/policies/testing.md`: test frameworks, regression suites, browser
  rules, and verification expectations.
- `.agents/policies/docs.md`: docs homes, proposal rules, `.llm` rules, and
  changelog workflow.
- `.agents/policies/code-standards.md`: language, import, file, type, and API
  standards.
- `.agents/policies/frontend.md`: UI framework, CSS, icon, and frontend
  interaction standards.

## Agent Style

Use concise, scannable Markdown. Prefer bullets, checklists, tables, tree diffs,
and ASCII mockups when they clarify. Use `monospace` for paths, commands, env
vars, and literal values. Reviews lead with findings; proposals use `+`, `~`,
and `-`; implementation results report files, verification, and remaining risk.
See `.agents/souls.md` for the expanded style reference, including the shared
colorful output vocabulary for terminal-capable reports.

## Agent Skills

Shared agent skills live in `.agents/skills/`. A repo may symlink them into
other harness-specific locations, such as `.claude/skills`. Codex invokes them
as `$x-<name>`; Claude Code invokes them as `/x-<name>`. The `SKILL.md` files
are a single source of truth for both harnesses.

Keep skills portable: put repeated workflow instructions in skills, and put
repo-specific facts in `.agents.local/project.md`. Skills should reference
policy files and the local project adapter instead of copying long policy text.

Use skills for repeated workflows. Load the skill body for exact workflow rules.
When one skill references another skill, include the direct `SKILL.md` path in
the body, such as `x-proposal` (`.agents/skills/x-proposal/SKILL.md`), so any
agent can open the referenced workflow without guessing.

Audit:

- `x-boundary-audit` (`.agents/skills/x-boundary-audit/SKILL.md`): package/API ownership, imports, exports, boundaries.
- `x-feedback-audit` (`.agents/skills/x-feedback-audit/SKILL.md`): outside advice, review notes, TODOs, pasted audits.
- `x-hardening-audit` (`.agents/skills/x-hardening-audit/SKILL.md`): final cleanup, LOC reduction, cruft, scope creep.
- `x-push-audit` (`.agents/skills/x-push-audit/SKILL.md`): committed secrets, private files, generated artifacts, push blockers.
- `x-test-audit` (`.agents/skills/x-test-audit/SKILL.md`): test coverage, placement, quality, duplication, missing behavior.
- `x-trim-code` (`.agents/skills/x-trim-code/SKILL.md`): target-scoped LOC reduction, clean boundaries, no duplicate code, visual parity.

Review:

- `x-code-review` (`.agents/skills/x-code-review/SKILL.md`): scoped code review against standards and organization.
- `x-proposal-review` (`.agents/skills/x-proposal-review/SKILL.md`): A++ long-term plan review before implementation.
- `x-security-review` (`.agents/skills/x-security-review/SKILL.md`): auth, billing, sessions, secrets, PII, permissions.

Sync:

- `x-docs-honesty` (`.agents/skills/x-docs-honesty/SKILL.md`): verify docs against code, clean stale markdown, keep READMEs truthful and right-sized.
- `x-docs-style` (`.agents/skills/x-docs-style/SKILL.md`): restyle READMEs/docs for clean, scannable, professional presentation without changing facts.
- `x-update-changelog` (`.agents/skills/x-update-changelog/SKILL.md`): update `[Unreleased]` and cutoff marker.
- `x-sync-docs` (`.agents/skills/x-sync-docs/SKILL.md`): sync AGENTS, skills, READMEs, proposals, runbooks, stale refs.
- `x-version-source` (`.agents/skills/x-version-source/SKILL.md`): remove hardcoded version drift and read versions from owning manifests.

Consolidate:

- `x-consolidate` (`.agents/skills/x-consolidate/SKILL.md`): broad code/tests/docs/todos consolidation router.
- `x-consolidate-code` (`.agents/skills/x-consolidate-code/SKILL.md`): duplicate, scattered, stale, misplaced, low-value code.
- `x-consolidate-docs` (`.agents/skills/x-consolidate-docs/SKILL.md`): duplicate, stale, conflicting, misplaced docs.
- `x-consolidate-todos` (`.agents/skills/x-consolidate-todos/SKILL.md`): scattered trackers/checklists into one ordered source.
- `x-consolidate-tests` (`.agents/skills/x-consolidate-tests/SKILL.md`): duplicate, stale, misplaced, slow, low-value tests.

Actions:

- `x-ascii-mockup` (`.agents/skills/x-ascii-mockup/SKILL.md`): plain-text UI/CLI/checklist/table/flow mockups.
- `x-commit` (`.agents/skills/x-commit/SKILL.md`): scoped commits or consume next `aw commit` queue request.
- `x-next` (`.agents/skills/x-next/SKILL.md`): next/remaining phases or approved next-phase execution.
- `x-do` (`.agents/skills/x-do/SKILL.md`): run all approved phases until done or genuinely blocked.
- `x-investigate` (`.agents/skills/x-investigate/SKILL.md`): choose the best evidence path before coding.
- `x-lint-cleanup` (`.agents/skills/x-lint-cleanup/SKILL.md`): lint/type/check cleanup without hiding errors.
- `x-logging` (`.agents/skills/x-logging/SKILL.md`): design, audit, or refactor structured contextual logging.
- `x-optimize-code` (`.agents/skills/x-optimize-code/SKILL.md`): evidence-first performance optimization.
- `x-owner-checklist` (`.agents/skills/x-owner-checklist/SKILL.md`): concise human-only values, approvals, and decisions.
- `x-layman` (`.agents/skills/x-layman/SKILL.md`): layman's explanation of prior technical output.
- `x-proposal` (`.agents/skills/x-proposal/SKILL.md`): compact file-change proposal with LOC and wins.
- `x-server-ports` (`.agents/skills/x-server-ports/SKILL.md`): fixed-port server lifecycle, PID ownership, and port-drift cleanup.
- `x-system-health` (`.agents/skills/x-system-health/SKILL.md`): CPU/memory/load/stale process report and requested cleanup.

## Debug Inputs

When the user pastes a browser or runtime stack trace, treat it as a debug
request. Parse the top first-party frame, read that file and line, trace upward
to the originating call site, and fix the root cause. Do not silence errors or
edit minified/chunk output directly; if the top frame is generated, walk back to
the nearest maintained source frame.

## Shared-Folder Git

- Shared macOS/Linux checkouts should use `core.filemode=false`; chmod-only changes will not be noticed reliably.
- In repos with a commit-owner queue, worker tabs should not run Git repair,
  chmod, staging, or commit commands directly. Hand verified slices to the
  local `git` owner instead.
