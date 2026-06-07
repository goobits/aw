mod support;

use std::process::Command;
use std::thread;
use std::time::Duration;

use support::command::{assert_success, path_with};
use support::fake_zellij;
use support::temp::{self, read, TempDir};

fn run_watcher(tmp: &TempDir, bin: &std::path::Path, tabs: &std::path::Path, args: &[&str]) {
    let output = Command::new(support::command::aw())
        .arg(".zellij-agent-tab-watcher")
        .args(args)
        .env("PATH", path_with(bin))
        .env("XDG_RUNTIME_DIR", tmp.join("runtime"))
        .env("XDG_CACHE_HOME", tmp.path())
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ_SESSION_NAME", "watcher-test")
        .env("FAKE_ZELLIJ_TABS", tabs)
        .output()
        .expect("run watcher");
    assert_success("watcher", &output);
}

fn layout_order(path: impl AsRef<std::path::Path>) -> Vec<String> {
    read(path)
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("tab name=\""))
        .filter_map(|line| line.split('"').next())
        .map(str::to_string)
        .collect()
}

#[test]
fn watcher_marks_tabs_and_repairs_saved_order() {
    let tmp = TempDir::new("watcher-contract");
    let bin = tmp.join("bin");
    let screen_dir = tmp.join("screens");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&screen_dir).unwrap();
    fake_zellij::install(&bin);
    let tabs = tmp.join("tabs.tsv");
    let panes = tabs.with_extension("tsv.panes");

    for (title, expected) in [
        ("⠋ codex", "infra 🤖"),
        ("⠐ Claude Code", "infra 🤖"),
        ("✦ Working", "infra 🤖"),
    ] {
        temp::write(&tabs, "1\t0\ttrue\tinfra\n");
        temp::write(&panes, &format!("1\t1\tinfra\tfalse\t{title}\n"));
        run_watcher(&tmp, &bin, &tabs, &["--once"]);
        assert_eq!(fake_zellij::tab_name(&tabs, "1"), expected);
    }

    temp::write(&tabs, "1\t0\ttrue\tinfra\n");
    temp::write(&panes, "1\t1\tinfra\tfalse\t✳ Claude Code\n");
    temp::write(
        screen_dir.join("1.txt"),
        "Claude Code v2.1.156\n\n· Deciphering…\n",
    );
    let output = Command::new(support::command::aw())
        .arg(".zellij-agent-tab-watcher")
        .arg("--once")
        .env("PATH", path_with(&bin))
        .env("XDG_RUNTIME_DIR", tmp.join("runtime"))
        .env("XDG_CACHE_HOME", tmp.path())
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ_SESSION_NAME", "watcher-test")
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_SCREEN_DIR", &screen_dir)
        .output()
        .expect("run screen watcher");
    assert_success("screen watcher", &output);
    assert_eq!(fake_zellij::tab_name(&tabs, "1"), "infra 🤖");

    for title in [
        "claude --permission-mode bypassPermissions",
        "◇ Ready",
        "✋ Action Required",
    ] {
        temp::write(&tabs, "1\t0\ttrue\tinfra 🤖\n");
        temp::write(&panes, &format!("1\t1\tinfra 🤖\tfalse\t{title}\n"));
        if title.starts_with("claude") {
            temp::write(screen_dir.join("1.txt"), "Claude Code v2.1.156\n\n❯\n");
        }
        let output = Command::new(support::command::aw())
            .arg(".zellij-agent-tab-watcher")
            .arg("--once")
            .env("PATH", path_with(&bin))
            .env("XDG_RUNTIME_DIR", tmp.join("runtime"))
            .env("XDG_CACHE_HOME", tmp.path())
            .env("HOME", tmp.join("home"))
            .env("ZELLIJ_SESSION_NAME", "watcher-test")
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .env("FAKE_ZELLIJ_SCREEN_DIR", &screen_dir)
            .output()
            .expect("run idle watcher");
        assert_success("idle watcher", &output);
        assert_eq!(fake_zellij::tab_name(&tabs, "1"), "infra");
    }

    temp::write(&tabs, "1\t0\ttrue\tinfra\n");
    temp::write(&panes, "1\t1\tinfra\tfalse\t⠋ codex\n");
    run_watcher(&tmp, &bin, &tabs, &["--once"]);
    temp::write(&tabs, "1\t0\tfalse\tinfra 🤖\n");
    temp::write(&panes, "1\t1\tinfra 🤖\tfalse\tworkspace\n");
    run_watcher(&tmp, &bin, &tabs, &["--once"]);
    assert_eq!(fake_zellij::tab_name(&tabs, "1"), "infra 🔔");
    run_watcher(&tmp, &bin, &tabs, &["--reset"]);
    assert_eq!(fake_zellij::tab_name(&tabs, "1"), "infra");

    let session_dir = tmp.join("zellij/contract_version_1/session_info/watcher-test");
    std::fs::create_dir_all(&session_dir).unwrap();
    temp::write(
        &tabs,
        "1\t0\ttrue\tpath 🤖\n2\t1\tfalse\toutline\n3\t2\tfalse\ttools\n",
    );
    temp::write(
        &panes,
        "1\t1\tpath 🤖\tfalse\t⠋ codex\n2\t2\toutline\tfalse\tworkspace\n3\t3\ttools\tfalse\tworkspace\n",
    );
    temp::write(
        session_dir.join("session-layout.kdl"),
        "layout {\n    tab name=\"outline\" {\n        pane\n    }\n    tab name=\"tools\" {\n        pane\n    }\n    tab name=\"path\" {\n        pane\n    }\n}\n",
    );
    run_watcher(&tmp, &bin, &tabs, &["--once"]);
    assert_eq!(
        layout_order(session_dir.join("session-layout.kdl")),
        vec!["path", "outline", "tools"]
    );

    std::fs::create_dir_all(tmp.join("home/.local/share/agent-workspace/profiles/test-profile"))
        .unwrap();
    temp::write(
        tmp.join("home/.local/share/agent-workspace/default-profile"),
        "test-profile\n",
    );
    temp::write(
        tmp.join("home/.local/share/agent-workspace/profiles/test-profile/watcher-test.tabs"),
        "path\noutline\ntools\n",
    );
    temp::write(
        &tabs,
        "1\t0\ttrue\toutline\n2\t1\tfalse\ttools\n3\t2\tfalse\tpath\n",
    );
    temp::write(
        &panes,
        "1\t1\toutline\tfalse\tworkspace\n2\t2\ttools\tfalse\tworkspace\n3\t3\tpath\tfalse\tworkspace\n",
    );
    temp::write(
        session_dir.join("session-layout.kdl"),
        "layout {\n    tab name=\"outline\" {\n        pane\n    }\n    tab name=\"tools\" {\n        pane\n    }\n    tab name=\"path\" {\n        pane\n    }\n}\n",
    );
    run_watcher(&tmp, &bin, &tabs, &["--once"]);
    assert_eq!(
        layout_order(session_dir.join("session-layout.kdl")),
        vec!["path", "outline", "tools"]
    );

    temp::write(
        session_dir.join("session-layout.kdl"),
        "layout {\n    tab name=\"outline\" {\n        pane\n    }\n    tab name=\"tools\" {\n        pane\n    }\n    tab name=\"path\" {\n        pane\n    }\n}\n",
    );
    let mut child = Command::new(support::command::aw())
        .arg(".zellij-agent-tab-watcher")
        .args(["--saved-loop", "watcher-test"])
        .env("PATH", path_with(&bin))
        .env("HOME", tmp.join("home"))
        .env("XDG_CACHE_HOME", tmp.path())
        .env("XDG_RUNTIME_DIR", tmp.join("runtime"))
        .env("ZELLIJ_SESSION_NAME", "watcher-test")
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("ZELLIJ_AGENT_TAB_WATCHER_SAVED_POLL_SECONDS", "0.05")
        .spawn()
        .expect("start saved loop");
    thread::sleep(Duration::from_millis(250));
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(
        layout_order(session_dir.join("session-layout.kdl")),
        vec!["path", "outline", "tools"]
    );
}
