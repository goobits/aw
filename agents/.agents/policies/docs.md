# Documentation Policy

## Homes

Human-facing proposals belong in `proposals/`. Keep proposals current, remove
superseded proposal docs instead of leaving legacy copies, and update
`proposals/INDEX.md` when proposal files change.

All LLM-generated files go in `.llm/`:

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

## Changelog

When asked to update `CHANGELOG.md`:

- read the audit cutoff comment near the top
- run `git log <cutoff>..HEAD --oneline` and audit every commit since the cutoff
- update `[Unreleased]` with grouped, user-meaningful entries
- put non-user-facing work under `Internal`
- move the cutoff comment to the current `HEAD` short hash and date
- do not add one bullet per commit unless explicitly asked
