mod support;

use std::process::Command;

use support::command::{assert_failure, assert_success, path_with, stderr};
use support::fake_zellij;
use support::temp::{self, read, TempDir};

fn copy_aw_as(path: impl AsRef<std::path::Path>) {
    std::fs::copy(support::command::aw(), path.as_ref()).expect("copy aw helper");
    temp::make_executable(path);
}

#[test]
fn zwork_orders_tabs_refuses_cross_session_and_repairs_shell_paths() {
    let tmp = TempDir::new("launch-contract");
    let bin = tmp.join("bin");
    let profile = tmp.join("profiles/test-profile");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&profile).unwrap();
    for helper in [
        "zwork",
        "zellij-render-layout",
        "zellij-open-session",
        "zellij-launch-session",
    ] {
        copy_aw_as(bin.join(helper));
    }
    fake_zellij::install(&bin);

    temp::write(profile.join("profile.conf"), "root=/tmp/test-root\n");
    temp::write(
        profile.join("backend.tabs"),
        "editor\nserver\ndatabase\nscratch\n",
    );
    temp::write(profile.join("frontend.tabs"), "preview\ndocs\nscratch\n");
    let tabs = tmp.join("tabs.tsv");
    temp::write(&tabs, "");
    temp::write(tabs.with_extension("tsv.panes"), "");

    let base_path = path_with(&bin);
    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "backend", "backend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ", "1")
        .env("ZELLIJ_SESSION_NAME", "backend-test")
        .env(
            "FAKE_ZELLIJ_SWITCH_ARGS",
            tmp.join("backend-switch-session.txt"),
        )
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", tmp.join("backend-order.txt"))
        .output()
        .expect("run backend zwork");
    assert_success("backend zwork", &output);
    assert_eq!(
        read(tmp.join("backend-order.txt")).trim_end(),
        "backend-test\neditor\nserver\ndatabase\nscratch"
    );
    assert_eq!(
        read(tmp.join("backend-switch-session.txt")).trim_end(),
        "action switch-session backend-test"
    );

    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "backend", "backend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env_remove("ZELLIJ")
        .env("ZELLIJ_SESSION_NAME", "backend-test")
        .env("FAKE_ZELLIJ_FAIL_ON_ATTACH", "1")
        .env(
            "FAKE_ZELLIJ_SWITCH_ARGS",
            tmp.join("session-name-only-switch-session.txt"),
        )
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            tmp.join("session-name-only-order.txt"),
        )
        .output()
        .expect("run zwork with session name only");
    assert_success("session-name-only zwork", &output);
    assert_eq!(
        read(tmp.join("session-name-only-order.txt")).trim_end(),
        "backend-test\neditor\nserver\ndatabase\nscratch"
    );
    assert_eq!(
        read(tmp.join("session-name-only-switch-session.txt")).trim_end(),
        "action switch-session backend-test"
    );

    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "backend", "backend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env_remove("ZELLIJ")
        .env_remove("ZELLIJ_SESSION_NAME")
        .env("FAKE_ZELLIJ_SESSIONS", "backend-test")
        .env("FAKE_ZELLIJ_FAIL_ON_LAUNCH_ENV_LEAK", "1")
        .env(
            "FAKE_ZELLIJ_LAUNCH_ARGS",
            tmp.join("stripped-env-attach.txt"),
        )
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", tmp.join("stripped-env-order.txt"))
        .output()
        .expect("run zwork with stripped zellij env");
    assert_success("stripped-env zwork", &output);
    assert_eq!(
        read(tmp.join("stripped-env-order.txt")).trim_end(),
        "backend-test\neditor\nserver\ndatabase\nscratch"
    );
    assert_eq!(
        read(tmp.join("stripped-env-attach.txt")).trim_end(),
        "attach --force-run-commands backend-test options --mirror-session true"
    );

    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "frontend", "frontend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ", "1")
        .env("ZELLIJ_SESSION_NAME", "frontend-test")
        .env(
            "FAKE_ZELLIJ_SWITCH_ARGS",
            tmp.join("frontend-switch-session.txt"),
        )
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", tmp.join("frontend-order.txt"))
        .output()
        .expect("run frontend zwork");
    assert_success("frontend zwork", &output);
    assert_eq!(
        read(tmp.join("frontend-order.txt")).trim_end(),
        "frontend-test\npreview\ndocs\nscratch"
    );
    assert_eq!(
        read(tmp.join("frontend-switch-session.txt")).trim_end(),
        "action switch-session frontend-test"
    );

    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "backend", "backend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ", "1")
        .env("ZELLIJ_SESSION_NAME", "frontend-test")
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("run refused zwork");
    assert_failure("cross-session zwork", &output);
    assert!(stderr(&output).contains(
        "cannot open session backend-test from inside active Zellij session frontend-test"
    ));

    let output = Command::new(bin.join("zwork"))
        .args(["test-profile", "backend", "backend-test", "/workspace"])
        .env("PATH", &base_path)
        .env("HOME", tmp.join("home"))
        .env("ZELLIJ", "1")
        .env("ZELLIJ_SESSION_NAME", "frontend-test")
        .env("AW_SWITCH_SESSION", "1")
        .env("ZELLIJ_PROFILE_DIR", tmp.join("profiles"))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_SWITCH_ARGS",
            tmp.join("switch-session-args.txt"),
        )
        .output()
        .expect("run switch zwork");
    assert_success("switch zwork", &output);
    assert!(read(tmp.join("switch-session-args.txt")).starts_with("action switch-session "));
    assert!(read(tmp.join("switch-session-args.txt")).contains("backend-test"));

    let repair_bin = tmp.join("repair-bin");
    let repair_cache = tmp.join("repair-cache");
    let repair_session = repair_cache.join("zellij/contract_version_1/session_info/fresh-session");
    std::fs::create_dir_all(&repair_bin).unwrap();
    std::fs::create_dir_all(&repair_session).unwrap();
    copy_aw_as(repair_bin.join("zellij-launch-session"));
    temp::write(
        repair_bin.join("zellij-session-tab-order"),
        "#!/usr/bin/env bash\nset -euo pipefail\nexit 0\n",
    );
    temp::make_executable(repair_bin.join("zellij-session-tab-order"));
    temp::write(
        repair_bin.join("zellij"),
        "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"${1:-}\" == \"list-sessions\" ]]; then exit 0; fi\nprintf '%s\\n' \"$*\" > \"${FAKE_ZELLIJ_LAUNCH_ARGS:?}\"\n",
    );
    temp::make_executable(repair_bin.join("zellij"));
    let repair_shell = tmp.join("repair-shell");
    temp::write(&repair_shell, "#!/usr/bin/env sh\nexit 0\n");
    temp::make_executable(&repair_shell);
    temp::write(
        repair_session.join("session-layout.kdl"),
        "pane command=\"/missing/zsh\"\n",
    );
    temp::write(
        repair_session.join("session-metadata.kdl"),
        "command \"/missing/zsh\"\n",
    );
    temp::write(tmp.join("layout.kdl"), "layout {}\n");

    let output = Command::new(repair_bin.join("zellij-launch-session"))
        .arg(tmp.join("layout.kdl"))
        .args(["fresh-session", "/workspace", "editor", "scratch"])
        .env("PATH", path_with(&repair_bin))
        .env("HOME", tmp.join("home"))
        .env("XDG_CACHE_HOME", &repair_cache)
        .env("SHELL", &repair_shell)
        .env("ZELLIJ", "")
        .env("ZELLIJ_REPAIR_BROKEN_SHELL", "/missing/zsh")
        .env(
            "FAKE_ZELLIJ_LAUNCH_ARGS",
            tmp.join("repair-launch-args.txt"),
        )
        .output()
        .expect("run repair launch");
    assert_success("repair launch", &output);
    let repaired_layout = read(repair_session.join("session-layout.kdl"));
    let repaired_metadata = read(repair_session.join("session-metadata.kdl"));
    assert!(!repaired_layout.contains("/missing/zsh"));
    assert!(!repaired_metadata.contains("/missing/zsh"));
    assert!(repaired_layout.contains(&repair_shell.display().to_string()));
    assert!(repaired_metadata.contains(&repair_shell.display().to_string()));
    assert_eq!(
        read(tmp.join("repair-launch-args.txt")).trim_end(),
        format!(
            "--layout {} attach --force-run-commands fresh-session --create options --mirror-session true",
            tmp.join("layout.kdl").display()
        )
    );
}
