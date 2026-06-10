# TODO

## AW tab bar double-click rename and drag reorder

Status: incomplete. It is safe to stop here from a repo-state perspective, but
the currently running Zellij session may still be using the built-in
`zellij:tab-bar`, so double-click rename and drag reorder will not work there
until the session is relaunched with the AW tab-bar layout.

What is already in progress:

- `tab_bar=aw` profile support renders the AW tab-bar plugin when the WASM file
  is installed.
- `aw install` copies `aw-tab-bar.wasm` into the Agent Workspace plugin path and
  writes Zellij permission grants.
- The plugin renders status text while waiting for permissions/loading tabs.
- Unit tests cover basic double-click rename state and drag command generation.

Remaining work:

- Make normal AW tab sync non-destructive. `aw tab move` and `aw tab rename`
  should never close live tabs just because a strict order sync is running.
- Add a separate explicit close-extra-tabs mode if strict destructive cleanup is
  still needed by launcher or maintenance workflows.
- Prefer native Zellij plugin APIs for live rename where possible
  (`rename_tab_with_id`) and keep AW file/profile sync as the persistence path.
- Detect stale existing sessions that still use `zellij:tab-bar` when
  `tab_bar=aw` is configured, and warn instead of silently attaching as if the
  AW tab bar is active.
- Verify in a fresh smoke session that:
  - the first row is the AW tab bar, not `zellij:tab-bar`;
  - double-click enters inline rename;
  - Enter renames the live tab and the workspace tab file;
  - drag reorder moves the tab without closing/recreating terminal panes.

Useful checks:

```sh
cargo test --manifest-path /workspace/infra/aw/Cargo.toml --test tab_order_contract --test workspace_cli_contract
cargo test --manifest-path /workspace/infra/aw/Cargo.toml --test launch_contract --test layout_contract --test install_contract
CARGO_TARGET_DIR=/tmp/aw-tab-bar-host-test cargo test --manifest-path /workspace/infra/aw/plugins/aw-tab-bar/Cargo.toml
```
