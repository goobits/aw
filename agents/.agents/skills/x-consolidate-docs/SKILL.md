---
name: x-consolidate-docs
description: 'Use when the user invokes $x-consolidate-docs or /x-consolidate-docs, asks to merge, delete, rehome, rename, or simplify duplicate, stale, scattered, conflicting, overlapping, misplaced, or low-value docs, or asks for one canonical doc owner for a topic.'
---

# X Consolidate Docs

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill to identify docs that should be merged, deleted, rehomed,
renamed, or simplified so one canonical source owns each topic. This is
proposal-first unless the cleanup is obvious, local, and low-risk.

Doc consolidation should reduce durable reading surface by default. Prefer
updating, moving, merging, demoting to pointers, or deleting existing docs before
creating a new doc. A new doc is only consolidation when it replaces a larger
set of duplicate, stale, or scattered docs.

Read `.agents/policies/docs.md`, `.agents/policies/quality.md`, and
`.agents/policies/git.md` when present. Keep doc-home rules in the docs policy
instead of copying them into this skill.

## Scope Recovery

1. Identify the topic, feature, package, app, server, proposal set, runbook set,
   or recent structural change.
2. Use repo-approved scoped state checks from `.agents.local/project.md` when
   present: path-scoped status, unstaged diff, and staged diff.
3. Search before editing or creating docs:
    - `rg "<topic|term|path>" AGENTS.md README.md proposals .llm/docs <target paths...>`
4. Read indexes before moving docs:
    - `.llm/docs/INDEX.md` and nearby README/index files for LLM docs
    - `proposals/INDEX.md` for human-facing proposals
5. Decide the canonical owner before proposing deletions or moves.

## What To Consolidate

- Duplicate docs that explain the same topic with overlapping or conflicting
  guidance.
- Stale docs, links, paths, screenshots, commands, or references after code,
  package, proposal, or workflow changes.
- Misplaced docs that belong in `.llm/docs/`, `proposals/`, a package README,
  app/server README, runbook, or root guidance.
- Parallel proposals, task notes, or design docs where one current version
  should remain and old versions should be removed or archived.
- Low-value docs that repeat obvious code facts without durable guidance.
- Index drift where canonical docs exist but discovery points to old or
  scattered locations.

## Safety Rules

- Prefer a proposal over edits when ownership, audience, history, or deletion
  safety is unclear.
- Aim for fewer docs and less repeated prose. If the proposal is net-positive,
  explicitly justify why the added docs reduce future maintenance.
- Do not delete durable docs unless a stronger canonical source remains and
  links/indexes are updated.
- Preserve useful historical context only when it still explains an active
  decision; otherwise remove or archive stale material.
- Follow `.agents/policies/docs.md` for raw evidence, durable LLM docs, and
  human-facing proposal homes.
- Use `x-consolidate-todos` (`.agents/skills/x-consolidate-todos/SKILL.md`) when the primary goal is one ordered todo tracker.
- Use `x-sync-docs` (`.agents/skills/x-sync-docs/SKILL.md`) after code or workflow changes when docs only need current
  references, indexes, or light alignment.
- When implementation paths are known, use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase
  format. When sequencing matters, order phases by operations/dependencies.
- Before proposing `+` new docs, indexes, or runbooks, search existing docs and
  proposal homes for the topic and prefer updating, moving, or consolidating the
  canonical owner over creating a parallel doc.

## Commit

If this skill edits files and verification passes, commit the scoped doc
consolidation by default using `x-commit` (`.agents/skills/x-commit/SKILL.md`). Do not commit if verification fails,
ownership is unclear, target paths overlap unrelated work, the docs are actively
changing, deletion safety is unresolved, or the user explicitly says not to
commit.

## Output

Lead with findings. When implementation paths are known, the consolidation plan
must use the canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) phase format. When sequencing matters, order
phases by operations/dependencies.
Do not propose a new doc owner until the existing docs/proposals for the topic
have been searched and ruled out as canonical homes.
If `Total LOC` is positive, add one short `Why net-positive is still
consolidation:` line before `Layman's wins`.
Apply the shared colorful output vocabulary directly. Keep the section labels
scan-friendly: `▌ Doc Consolidation`, `▌ Consolidation Proposal`, `▌ Out Of
Scope`, and `▌ Open Questions`.

```text
▌ Doc Consolidation
! Severity  path - duplicate, stale, scattered, conflicting, misplaced, or low-value doc. Risk. Canonical owner.
✓ Canonical path - current owner to keep.

▌ Consolidation Proposal
Use canonical `x-proposal` (`.agents/skills/x-proposal/SKILL.md`) format when file changes are recommended.

▌ Out Of Scope
◇ Docs intentionally not merged, deleted, moved, or renamed.

▌ Open Questions
· Audience, canonical owner, historical context, deletion safety, or verification decisions.
```

If no safe consolidation is found, say `No material doc consolidation findings`
and list any unverified surfaces.
