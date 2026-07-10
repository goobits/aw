---
name: x-sync-docs
description: 'Use when the user invokes $x-sync-docs or /x-sync-docs, asks to sync docs after code changes, update references, align AGENTS.md, README, proposals, .llm/docs indexes, changelog-adjacent docs, or remove clear stale references after a structural change.'
---

# X Sync Docs

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill to bring documentation and indexes back into alignment after code,
architecture, proposal, or workflow changes. This is an editing skill when stale
references are clear.

Read `.agents/policies/docs.md`, `.agents/policies/git.md`, and
`.agents.local/project.md` when present. Keep project-specific doc homes in those
policy files instead of repeating them here.

## Scope Recovery

1. Identify the changed area or structural decision.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present: path-scoped status and diff.
3. Search existing docs before creating new ones:
    - `rg "<term>" AGENTS.md README.md proposals .llm/docs <target docs>`
    - read `.llm/docs/00-START-HERE.md`, `.llm/docs/INDEX.md`, and `.llm/docs/doc-maintenance.md` when touching `.llm/docs`.
4. Update existing docs instead of creating parallel copies.

## What To Sync

- `AGENTS.md`: repo-local skills list, workflow rules, conventions, and safety rules.
- `.agents/skills/*/SKILL.md`: keep descriptions and workflows aligned with actual use.
- Skill proposal output: keep `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) as the canonical owner of compact
  phase/tree diff semantics. Other skills should reference the canonical
  `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format instead of rewriting it. When sequencing matters,
  proposed phases must be ordered by operations/dependencies.
- Before a skill proposes `+` new code, helpers, tests, docs, tools, or trackers,
  it should search for similar existing owners and prefer editing, rehoming, or
  consolidating them over creating parallel surfaces.
- Skills that propose, create, move, rename, or refactor code files should call
  out local file naming policy instead of relying on agents to remember it.
- `.llm/docs/`: durable LLM-facing synthesis only. Update indexes/READMEs when adding or moving docs.
- `proposals/`: human-facing proposals. Update current proposal references and
  `proposals/INDEX.md`; use `x-consolidate-docs` (`.agents/skills/x-consolidate-docs/SKILL.md`) when superseded proposal sets
  need merge/delete/archive decisions.
- Doc consolidation: use `x-consolidate-docs` (`.agents/skills/x-consolidate-docs/SKILL.md`) when the primary goal is merging,
  deleting, rehoming, renaming, or simplifying duplicate, stale, scattered,
  conflicting, or misplaced docs.
- READMEs: package/app/server usage, commands, exported surfaces, migration notes.
- Env examples and runbooks: config names, secrets, ports, service ownership.
- Changelog: use `x-update-changelog` (`.agents/skills/x-update-changelog/SKILL.md`); do not duplicate changelog rules here.
- Todo trackers: use `x-consolidate-todos` (`.agents/skills/x-consolidate-todos/SKILL.md`) when the primary goal is gathering
  remaining work into one ordered tracker.

## Placement Rules

- Follow `.agents/policies/docs.md` when present.
- Do not create root-level docs unless an existing root index requires that location.

## Verification

Run lightweight verification appropriate to the doc change:

- Link/path sanity by `rg` for old names and stale paths.
- Relevant package docs commands if present.
- No build unless the user explicitly approved it.

## Commit

If this skill edits files and verification passes, commit the scoped docs sync by
default using `x-commit` (`.agents/skills/x-commit/SKILL.md`). Do not commit if verification fails, ownership is
unclear, target paths overlap unrelated work, the docs are actively changing, or
the user explicitly says not to commit.

## Output

Style updated docs, removed stale references, verification, and remaining
questions with shared colors when useful.

**Updated**

- path - what changed

**Removed stale docs**

- path - why

**✓ Verified**

- command/result or not run

**? Remaining**

- unresolved doc owners or stale references
