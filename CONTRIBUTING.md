# Contributing

Agent Workspace is a Rust CLI plus portable agent instruction bundle.

Before sending changes:

- Run `cargo test`.
- Keep reusable agent behavior under `agents/.agents`.
- Keep repo-specific facts in the consuming repo's `.agents.local/project.md`.
- Do not add secrets, local machine paths, build outputs, or generated caches.

For CLI behavior changes, update the relevant contract tests in `tests/` and
the examples in `README.md`.
