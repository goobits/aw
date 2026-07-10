# Testing Policy

Use targeted verification that matches the changed slice. Do not run a repo-wide
or full regression suite by default when a focused package, file, unit,
typecheck, lint, or browser check proves the edited behavior. Local test
framework names, commands, report viewers, and full regression suites belong in
`.agents.local/project.md`.

## Scope

Start with the narrowest meaningful check:

- local selector/recommender commands when the smallest check is unclear
- focused unit tests for small behavior changes
- package/app tests for owned module changes
- type/lint/check commands for exported contracts or broad refactors
- browser tests for browser-only, rendering, layout, or interaction behavior
- full regression suites only when the requested confidence or changed surface
  requires them

Broaden verification when the implementation touches shared behavior, exported
contracts, app-wide behavior, persistence, auth/security, migrations, or
user-facing workflows. Run the full suite only when the user asks for it,
release/signoff requires it, targeted checks cannot cover the risk, or the
change is broad enough that smaller checks would be misleading.

When a repo provides a test selector, use it as a dry-run first pass before
choosing a broad command. In this repository, prefer:

```bash
pnpm run test:select -- --path <changed-path>
```

Add `--run` only after the recommended scope is intended. Add full-suite flags
only when full regression or release signoff is explicitly required.

## Browser And Rendering Tests

- Use browser automation for browser-only, rendering, and interaction checks.
- Use the local project's headless/display wrapper when server-side rendering,
  WebGL, WebGPU, canvas, or visual tests require it.
- Keep assertions targeted.
- Avoid large snapshots or huge tool responses unless needed.

## Reporting

Always separate:

- checks run and passed
- checks not run and why
- pre-existing failures
- failures caused by the current slice
