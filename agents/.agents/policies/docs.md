# Documentation Policy

## Homes

All proposal authority for this repository belongs under the top-level
`proposals/` tree. This includes product and technical proposals, RFCs, release
plans, decision-owning roadmaps, and trackers that determine scope or priority.
Keep every proposal discoverable through `proposals/INDEX.md`; preserve useful
implemented, rejected, parked, or superseded proposals under
`proposals/archive/` when their decision history still matters.

`.llm/docs/` may contain implementation notes, runbooks, evidence, inventories,
and diagnostics that support an approved proposal, but those documents must not
introduce independent scope or act as a second task owner. Completed LLM-facing
proposal documents move to `proposals/archive/`; `.llm/docs/archive/` is only
for completed non-proposal agent documentation.

Nested repositories keep independent Git ownership. A proposal governing this
parent repository or its product portfolio still belongs in the parent
`proposals/` tree; repository-home checks must not rewrite or claim unrelated
nested-repository policy.

All other agent-generated support files go in `.llm/`:

- scratch/debug artifacts and raw captures: `.llm/scratch/`
- reusable LLM-facing docs: `.llm/docs/`
- completed LLM-facing docs: `.llm/docs/archive/`
- automation: `.llm/scripts/`

Never write debug artifacts to the repo root or source folders. Screenshots,
PDFs, SVGs, logs, HTML reports, Lighthouse output, raw JSON snapshots, and
similar artifacts belong in `.llm/scratch/`.

Before creating LLM docs, search `.llm/docs/`; update existing docs instead of
duplicating them, and maintain `.llm/docs/INDEX.md`. Treat `.llm/docs/` as a
maintained wiki: raw evidence stays in `scratch/`, durable synthesized truth
goes in `docs/`, and every new durable page needs an index or README entry.
Use the wiki as a navigation map, not an answer engine: agents should verify
current behavioral claims against source, tests, local policy, or runbooks before
making code changes.

## Changelog

When asked to update `CHANGELOG.md`:

- read the audit cutoff comment near the top
- run `git log <cutoff>..HEAD --oneline` and audit every commit since the cutoff
- update `[Unreleased]` with grouped, user-meaningful entries
- put non-user-facing work under `Internal`
- move the cutoff comment to the current `HEAD` short hash and date
- do not add one bullet per commit unless explicitly asked
