# aw-tab-bar

AW-aware Zellij tab bar plugin.

This plugin follows Zellij's official Rust plugin path:

- Rust crate using `zellij-tile`
- compiled to WASI WebAssembly
- loaded as a normal Zellij layout plugin

Current Rust toolchains expose the WASI preview 1 target as `wasm32-wasip1`:

```bash
rustup target add wasm32-wasip1
cargo build --release --manifest-path plugins/aw-tab-bar/Cargo.toml --target wasm32-wasip1
```

The release artifact is:

```text
plugins/aw-tab-bar/target/wasm32-wasip1/release/aw-tab-bar.wasm
```

`aw install` copies that artifact to:

```text
~/.aw/plugins/aw-tab-bar.wasm
```

Use `AW_TAB_BAR_WASM_SOURCE=/path/to/aw-tab-bar.wasm aw install` to install a
WASM artifact from a custom build location. Use `AW_TAB_BAR_PLUGIN_PATH` when
rendering a layout against an explicit plugin path.

Enable it per profile:

```text
tab_bar=aw
```

Configuration keys:

```kdl
plugin location="file:/path/to/aw-tab-bar.wasm" {
    workspace "front"
    aw "aw"
    double_click_ms "350"
}
```

Behavior:

- single click focuses a tab after the double-click window expires
- drag and release runs `aw tab move <workspace> <tab>@<index>`
- double-click enters inline rename
- Enter runs `aw tab rename <workspace> <old-tab> <new-tab>`
- Esc cancels inline rename

Tab names intentionally use AW's safe workspace name set:
letters, numbers, `.`, `_`, and `-`.

Smoke check the installed plugin with a disposable Zellij session:

```bash
cargo build --release --manifest-path plugins/aw-tab-bar/Cargo.toml --target wasm32-wasip1
AW_TAB_BAR_WASM_SOURCE="$PWD/plugins/aw-tab-bar/target/wasm32-wasip1/release/aw-tab-bar.wasm" aw install
env -u ZELLIJ -u ZELLIJ_SESSION_NAME SHELL=/bin/sh zellij --session aw-tab-bar-smoke --layout /path/to/rendered-layout.kdl
```

Then verify the first row renders tabs, double-clicking a tab enters inline
rename, and dragging a tab runs the AW move path.
