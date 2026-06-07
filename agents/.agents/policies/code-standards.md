# Code Standards Policy

Use this as the reusable code-quality baseline. Local language, framework,
package, import, and documentation overrides belong in `.agents.local/project.md`.

## General

- Prefer the dominant language, module system, and framework already used by the
  target package/app.
- Prefer simple, explicit modules over clever compatibility layers.
- No defensive programming except for external input, security boundaries, or
  documented unreliable integrations.
- Do not claim things are pre-existing; verify from local context.
- Ask before writing automatic migration scripts; prefer careful one-by-one
  edits when correctness depends on judgment.

## Imports And Boundaries

- Keep private internals private.
- Use local relative imports for same-package internals unless the project policy
  says otherwise.
- Cross package/app boundaries through documented public entrypoints.
- Do not import another package's private source paths unless the project policy
  explicitly allows it.
- Tests follow the same boundary rules as source.

## Files And Names

- Prefer responsibility names over history names.
- Avoid compatibility names like `Legacy`, `Wrapper`, `Shim`, and stale domain
  terms after a concept is renamed.
- Keep file naming consistent with the target package/app.

## Types And Public APIs

- Colocate types by default; centralize only when genuinely shared.
- Public APIs should be documented when the project expects generated API docs or
  external consumers.
- Avoid widening types, adding casts, suppressing diagnostics, or hiding errors
  to make checks pass.
- Internal helpers do not need public-style docs unless exported through a
  public surface.
