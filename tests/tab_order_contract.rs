mod support;

use std::process::Command;

use support::command::{assert_success, path_with, stdout};
use support::fake_zellij;
use support::temp::{self, read, TempDir};

fn tab_names_from_layout(path: impl AsRef<std::path::Path>) -> Vec<String> {
    read(path)
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("tab name=\""))
        .filter_map(|line| line.split('"').next())
        .map(str::to_string)
        .collect()
}

fn metadata_tab_order(path: impl AsRef<std::path::Path>) -> Vec<String> {
    let mut rows = Vec::new();
    let mut in_tab = false;
    let mut position = String::new();
    let mut name = String::new();
    for line in read(path).lines() {
        let trimmed = line.trim();
        if trimmed == "tab {" {
            in_tab = true;
            position.clear();
            name.clear();
        } else if in_tab && trimmed.starts_with("position ") {
            position = trimmed.trim_start_matches("position ").to_string();
        } else if in_tab && trimmed.starts_with("name ") {
            name = trimmed
                .trim_start_matches("name ")
                .trim_matches('"')
                .to_string();
        } else if in_tab && trimmed == "}" {
            rows.push(format!("{position}\t{name}"));
            in_tab = false;
        }
    }
    rows
}

fn pane_positions(path: impl AsRef<std::path::Path>) -> Vec<String> {
    read(path)
        .lines()
        .filter_map(|line| line.trim().strip_prefix("tab_position "))
        .map(str::to_string)
        .collect()
}

#[test]
fn session_tab_order_rewrites_saved_and_live_state() {
    let tmp = TempDir::new("tab-order-contract");
    let session_dir = tmp.join("zellij/contract_version_1/session_info/test-workspace");
    std::fs::create_dir_all(&session_dir).unwrap();
    let layout = session_dir.join("session-layout.kdl");
    let metadata = session_dir.join("session-metadata.kdl");
    temp::write(
        &layout,
        r#"layout {
    cwd "/workspace"
    tab name="database" hide_floating_panes=true {
        pane contents_file="initial_contents_1"
    }
    tab name="scratch 🤖" focus=true hide_floating_panes=true {
        pane focus=true contents_file="initial_contents_2"
    }
    tab name="editor" hide_floating_panes=true {
        pane contents_file="initial_contents_3"
    }
    tab name="server" hide_floating_panes=true {
        pane contents_file="initial_contents_4"
    }
    tab name="database" hide_floating_panes=true {
        pane contents_file="initial_contents_5"
    }
    tab name="custom" hide_floating_panes=true {
        pane contents_file="initial_contents_6"
    }
    new_tab_template {
        pane cwd="/workspace"
    }
}
"#,
    );
    temp::write(
        &metadata,
        r#"name "test-workspace"
tabs {
    tab {
        position 0
        name "database"
        tab_id 0
    }
    tab {
        position 1
        name "scratch 🤖"
        tab_id 1
    }
    tab {
        position 2
        name "editor"
        tab_id 2
    }
    tab {
        position 3
        name "server"
        tab_id 3
    }
    tab {
        position 4
        name "database"
        tab_id 4
    }
    tab {
        position 5
        name "custom"
        tab_id 5
    }
}
panes {
    pane {
        id 0
        tab_position 0
    }
    pane {
        id 1
        tab_position 1
    }
    pane {
        id 2
        tab_position 2
    }
    pane {
        id 3
        tab_position 3
    }
    pane {
        id 4
        tab_position 4
    }
    pane {
        id 5
        tab_position 5
    }
}
"#,
    );

    let output = Command::new(support::command::aw())
        .args([
            "zellij-session-tab-order",
            "test-workspace",
            "editor",
            "server",
            "database\t/tmp/database",
            "logs",
            "scratch",
        ])
        .env("XDG_CACHE_HOME", tmp.path())
        .env("ZELLIJ_SESSION_TAB_ORDER_SAVED_ONLY", "1")
        .output()
        .expect("run saved order");
    assert_success("saved order", &output);
    assert_eq!(
        tab_names_from_layout(&layout),
        vec!["editor", "server", "database", "scratch", "custom"]
    );
    assert_eq!(
        metadata_tab_order(&metadata),
        vec![
            "0\teditor",
            "1\tserver",
            "2\tdatabase",
            "3\tscratch",
            "4\tcustom"
        ]
    );
    assert_eq!(pane_positions(&metadata), vec!["2", "3", "0", "1", "4"]);

    let output = Command::new(support::command::aw())
        .args([
            "zellij-session-tab-order",
            "test-workspace",
            "editor",
            "server",
            "database",
            "scratch",
        ])
        .env("XDG_CACHE_HOME", tmp.path())
        .env("ZELLIJ_SESSION_TAB_ORDER_SAVED_ONLY", "1")
        .env("ZELLIJ_SESSION_TAB_ORDER_STRICT", "1")
        .output()
        .expect("run strict saved order");
    assert_success("strict saved order", &output);
    assert_eq!(
        tab_names_from_layout(&layout),
        vec!["editor", "server", "database", "scratch"]
    );
    assert_eq!(
        metadata_tab_order(&metadata),
        vec!["0\teditor", "1\tserver", "2\tdatabase", "3\tscratch"]
    );

    let live_bin = tmp.join("bin");
    std::fs::create_dir_all(&live_bin).unwrap();
    fake_zellij::install(&live_bin);
    let live_state = tmp.join("live-tabs.tsv");
    temp::write(
        &live_state,
        "0\t0\ttrue\tinfra\n1\t1\tfalse\tserver\n2\t2\tfalse\tlogs\n3\t3\tfalse\tdocs\n4\t4\tfalse\tpreview\n5\t5\tfalse\tscratch\n",
    );
    temp::write(live_state.with_extension("tsv.panes"), "");
    let output = Command::new(support::command::aw())
        .args([
            "zellij-session-tab-order",
            "test-live",
            "infra",
            "server",
            "logs",
            "docs",
            "preview",
            "database\t/tmp/database",
            "scratch",
        ])
        .env("PATH", path_with(&live_bin))
        .env("FAKE_ZELLIJ_TABS", &live_state)
        .env("XDG_CACHE_HOME", tmp.join("live-cache"))
        .env("ZELLIJ_SESSION_TAB_ORDER_CREATE_MISSING", "1")
        .output()
        .expect("run live order");
    assert_success("live order", &output);
    assert_eq!(
        fake_zellij::sorted_tab_names(&live_state),
        vec!["infra", "server", "logs", "docs", "preview", "database", "scratch"]
    );
    assert!(!live_state.with_extension("tsv.saved").exists());
    assert!(read(live_state.with_extension("tsv.cwds")).contains("database\t/tmp/database"));
    assert!(!read(live_state.with_extension("tsv.panes")).contains("zellij:status-bar"));

    temp::write(
        &live_state,
        "0\t0\tfalse\ttools\n1\t1\ttrue\tpath 🤖\n2\t2\tfalse\textra\n3\t3\tfalse\ttools\n",
    );
    temp::write(live_state.with_extension("tsv.panes"), "");
    let _ = std::fs::remove_file(live_state.with_extension("tsv.saved"));
    let _ = std::fs::remove_file(live_state.with_extension("tsv.cwds"));
    let output = Command::new(support::command::aw())
        .args([
            "zellij-session-tab-order",
            "test-live",
            "path",
            "tools",
            "scratch",
        ])
        .env("PATH", path_with(&live_bin))
        .env("FAKE_ZELLIJ_TABS", &live_state)
        .env("XDG_CACHE_HOME", tmp.join("live-cache-strict"))
        .env("ZELLIJ_SESSION_TAB_ORDER_CREATE_MISSING", "1")
        .env("ZELLIJ_SESSION_TAB_ORDER_STRICT", "1")
        .output()
        .expect("run strict live order");
    assert_success("strict live order", &output);
    assert_eq!(
        fake_zellij::sorted_tab_names(&live_state),
        vec!["path 🤖", "tools", "scratch"]
    );
    assert!(read(live_state.with_extension("tsv.cwds")).contains("scratch\t"));
    assert!(stdout(&output).is_empty());
}
