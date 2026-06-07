mod support;

use std::process::Command;

use support::command::{assert_failure, assert_success, path_with, stderr, stdout, TestHome};
use support::fake_zellij;
use support::temp::{self, read, TempDir};

fn install_test_aw(name: &str) -> TestHome {
    fake_zellij::installed_home(name)
}

fn run_in_project(
    home: &TestHome,
    project: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    home.aw_command()
        .args(args)
        .current_dir(project)
        .output()
        .expect("run aw in project")
}

#[test]
fn workspace_assignment_rename_remove_and_validation_are_rust_contracts() {
    let home = install_test_aw("workspace-create");
    let project = home.root.join("my-site");
    std::fs::create_dir_all(&project).unwrap();

    let output = home
        .aw_command()
        .arg("main=app,server,infra,scratch")
        .current_dir(&project)
        .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join("main-order.txt"))
        .output()
        .expect("create main workspace");
    assert_success("create main", &output);
    assert_eq!(
        read(project.join("config/aw/profile.conf")),
        format!(
            "name=my-site\nroot={}\ndefault_workspace=main\ndefault_workspaces=main\n",
            project.display()
        )
    );
    assert_eq!(
        read(project.join("config/aw/main.tabs")),
        "app\nserver\ninfra\nscratch\n"
    );
    assert!(home
        .home
        .join(".local/share/agent-workspace/profiles/my-site/main.tabs")
        .is_file());
    assert_eq!(
        read(home.root.join("main-order.txt")).trim_end(),
        "main\napp\nserver\ninfra\nscratch"
    );

    for (workspace, tabs) in [("frontend", "app,ui,tools"), ("backend", "infra,api,db")] {
        let output = home
            .aw_command()
            .arg(format!("{workspace}={tabs}"))
            .current_dir(&project)
            .env(
                "FAKE_ZELLIJ_ORDER_ARGS",
                home.root.join(format!("{workspace}-order.txt")),
            )
            .output()
            .expect("create workspace");
        assert_success("create workspace", &output);
    }
    assert_eq!(
        read(project.join("config/aw/frontend.tabs")),
        "app\nui\ntools\n"
    );
    assert_eq!(
        read(project.join("config/aw/backend.tabs")),
        "infra\napi\ndb\n"
    );

    let output = run_in_project(&home, &project, &["rename", "backend", "services"]);
    assert_success("rename backend", &output);
    assert!(!project.join("config/aw/backend.tabs").exists());
    assert_eq!(
        read(project.join("config/aw/services.tabs")),
        "infra\napi\ndb\n"
    );
    assert!(read(project.join("config/aw/profile.conf"))
        .contains("default_workspaces=main frontend services"));

    let output = run_in_project(&home, &project, &["list"]);
    assert_success("list", &output);
    assert_eq!(stdout(&output), "frontend\nmain\nservices");

    let output = run_in_project(&home, &project, &["rename", "services", "frontend"]);
    assert_failure("rename over existing", &output);

    let output = run_in_project(&home, &project, &["remove", "frontend"]);
    assert_success("remove frontend", &output);
    assert!(!project.join("config/aw/frontend.tabs").exists());
    assert!(!home
        .home
        .join(".local/share/agent-workspace/profiles/my-site/frontend.tabs")
        .exists());

    let output = run_in_project(&home, &project, &["remove", "main"]);
    assert_success("remove main", &output);
    assert_eq!(
        read(project.join("config/aw/profile.conf")),
        format!(
            "name=my-site\nroot={}\ndefault_workspace=services\ndefault_workspaces=services\n",
            project.display()
        )
    );
    assert_eq!(
        stdout(&run_in_project(&home, &project, &["list"])),
        "services"
    );
    assert_failure(
        "remove missing",
        &run_in_project(&home, &project, &["remove", "missing"]),
    );
    assert_failure(
        "remove last",
        &run_in_project(&home, &project, &["remove", "services"]),
    );

    for args in [
        vec!["bad=tab,,name"],
        vec!["bad="],
        vec!["front=end", "plain-tab"],
        vec!["bad/name=tab"],
    ] {
        assert_failure(
            "invalid assignment",
            &run_in_project(&home, &project, &args),
        );
    }
}

#[test]
fn doctor_refresh_tab_edit_scratch_and_session_commands_use_aw_surface() {
    let home = install_test_aw("workspace-cli");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        "name=my-site\nroot=/tmp/project\ndefault_workspace=frontend\ndefault_workspaces=frontend backend extra\n",
    );
    temp::write(profile.join("frontend.tabs"), "app\nui\nscratch\n");
    temp::write(profile.join("backend.tabs"), "api\ndatabase\nscratch\n");
    temp::write(profile.join("extra.tabs"), "notes\nscratch\n");
    let tabs = home.root.join("tabs.tsv");
    temp::write(&tabs, "");
    temp::write(tabs.with_extension("tsv.panes"), "");

    let output = home
        .aw_command()
        .args(["setup", "--config"])
        .arg(&profile)
        .output()
        .expect("setup");
    assert_success("setup", &output);
    assert_eq!(
        stdout(&output),
        "Installed Zellij profile my-site.\nRun: aw frontend"
    );
    let output = home
        .aw_command()
        .args(["doctor", "--config"])
        .arg(&profile)
        .env("FAKE_ZELLIJ_SESSIONS", "frontend\nbackend")
        .env("FAKE_ZELLIJ_GENERATE_PANES_FROM_TABS", "1")
        .output()
        .expect("doctor");
    assert_success("doctor", &output);

    let runtime = TempDir::new("runtime-doctor");
    let runtime_profile = runtime.join("config/aw");
    let runtime_cache = runtime.join("cache");
    let runtime_tabs = runtime.join("tabs.tsv");
    std::fs::create_dir_all(runtime_cache.join("zellij/contract_version_1/session_info/front"))
        .unwrap();
    temp::write(
        runtime_profile.join("profile.conf"),
        "name=runtime-site\nroot=/tmp/runtime-project\ndefault_workspace=front\ndefault_workspaces=front\n",
    );
    temp::write(runtime_profile.join("front.tabs"), "path\noutline\ntools\n");
    temp::write(
        &runtime_tabs,
        "1\t0\ttrue\tpath\n2\t1\tfalse\toutline\n3\t2\tfalse\ttools\n",
    );
    let session_layout =
        runtime_cache.join("zellij/contract_version_1/session_info/front/session-layout.kdl");
    temp::write(
        &session_layout,
        "layout {\n    tab name=\"outline\" {\n        pane\n    }\n    tab name=\"tools\" {\n        pane\n    }\n    tab name=\"path\" {\n        pane\n    }\n}\n",
    );
    assert_success(
        "runtime setup",
        &home
            .aw_command()
            .args(["setup", "--config"])
            .arg(&runtime_profile)
            .env("FAKE_ZELLIJ_SESSIONS", "front")
            .output()
            .unwrap(),
    );
    let output = home
        .aw_command()
        .args(["doctor", "--config"])
        .arg(&runtime_profile)
        .env("XDG_CACHE_HOME", &runtime_cache)
        .env("FAKE_ZELLIJ_SESSIONS", "front")
        .env("FAKE_ZELLIJ_TABS", &runtime_tabs)
        .output()
        .expect("runtime doctor drift");
    assert_failure("runtime doctor drift", &output);
    assert!(stderr(&output).contains("mismatch: saved tab order front"));
    temp::write(
        &session_layout,
        "layout {\n    tab name=\"path\" {\n        pane\n    }\n    tab name=\"outline\" {\n        pane\n    }\n    tab name=\"tools\" {\n        pane\n    }\n}\n",
    );
    assert_success(
        "runtime doctor fixed",
        &home
            .aw_command()
            .args(["doctor", "--config"])
            .arg(&runtime_profile)
            .env("XDG_CACHE_HOME", &runtime_cache)
            .env("FAKE_ZELLIJ_SESSIONS", "front")
            .env("FAKE_ZELLIJ_TABS", &runtime_tabs)
            .output()
            .unwrap(),
    );

    assert_eq!(
        stdout(&home.aw_command().arg("help").output().unwrap())
            .lines()
            .next()
            .unwrap(),
        "aw: Zero-friction Zellij workspaces"
    );

    let strict_project = home.root.join("strict-project");
    let strict_profile = strict_project.join("config/aw");
    let strict_tabs = home.root.join("strict-tabs.tsv");
    temp::write(
        strict_profile.join("profile.conf"),
        "name=strict-site\nroot=/tmp/strict-project\ndefault_workspace=frontend\ndefault_workspaces=frontend\n",
    );
    temp::write(strict_profile.join("frontend.tabs"), "app\nui\nscratch\n");
    temp::write(
        &strict_tabs,
        "0\t0\tfalse\tui\n1\t1\ttrue\tapp 🤖\n2\t2\tfalse\textra\n3\t3\tfalse\tapp\n",
    );
    temp::write(strict_tabs.with_extension("tsv.panes"), "");
    let output = home
        .aw_command()
        .args(["refresh", "frontend"])
        .current_dir(&strict_project)
        .env("FAKE_ZELLIJ_TABS", &strict_tabs)
        .env("FAKE_ZELLIJ_SESSIONS", "frontend")
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("strict-refresh-order.txt"),
        )
        .output()
        .expect("strict refresh");
    assert_success("strict refresh", &output);
    assert_eq!(stdout(&output), "Converged workspace frontend.");
    assert_eq!(
        fake_zellij::sorted_tab_names(&strict_tabs),
        vec!["app 🤖", "ui", "scratch"]
    );

    assert_eq!(
        stdout(
            &home
                .aw_command()
                .args(["list", "--config"])
                .arg(&profile)
                .output()
                .unwrap()
        ),
        "backend\nextra\nfrontend"
    );
    let output = home
        .aw_command()
        .arg("frtonend")
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("missing workspace");
    assert_failure("missing workspace", &output);
    assert!(stderr(&output).contains("workspace not found: frtonend"));

    let output = home
        .aw_command()
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("default-order.txt"),
        )
        .output()
        .expect("default launch");
    assert_success("default launch", &output);
    assert!(stdout(&output).is_empty());
    assert_eq!(
        read(home.root.join("default-order.txt")).trim_end(),
        "frontend\napp\nui\nscratch"
    );

    let output = home
        .aw_command()
        .args(["extra", "-s", "extra-session", "-r"])
        .arg(&project)
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join("extra-order.txt"))
        .output()
        .expect("extra launch");
    assert_success("extra launch", &output);
    assert_eq!(
        read(home.root.join("extra-order.txt")).trim_end(),
        "extra-session\nnotes\nscratch"
    );

    let output = home
        .aw_command()
        .arg("now=tools,components,scratch")
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join("now-order.txt"))
        .output()
        .expect("upsert now");
    assert_success("upsert now", &output);
    assert_eq!(
        read(profile.join("now.tabs")),
        "tools\ncomponents\nscratch\n"
    );

    let output = home
        .aw_command()
        .args(["create", "docs", "guide", "api", "scratch"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join("docs-order.txt"))
        .output()
        .expect("create docs");
    assert_success("create docs", &output);
    assert_eq!(read(profile.join("docs.tabs")), "guide\napi\nscratch\n");

    temp::write(
        profile.join("front.tabs"),
        "tools\ncomponents\nkeyboard\nskills\nscratch\n",
    );
    temp::write(
        &tabs,
        "0\t0\tfalse\ttools 🤖\n1\t1\tfalse\tcomponents 🤖\n2\t2\tfalse\tkeyboard 🔔\n3\t3\ttrue\tskills 🤖\n4\t4\tfalse\tscratch 🤖\n",
    );
    let output = home
        .aw_command()
        .args(["tab", "list", "front"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("tab list");
    assert_success("tab list", &output);
    assert_eq!(
        stdout(&output),
        "  0 tools\n  1 components\n  2 keyboard\n* 3 skills\n  4 scratch"
    );

    for (args, expected_tabs, order_file) in [
        (
            vec!["tab", "add", "front", "search@1"],
            "tools\nsearch\ncomponents\nkeyboard\nskills\nscratch\n",
            "front-add-order.txt",
        ),
        (
            vec!["tab", "move", "front", "keyboard@1"],
            "tools\nkeyboard\nsearch\ncomponents\nskills\nscratch\n",
            "front-move-order.txt",
        ),
        (
            vec!["tab", "rename", "front", "keyboard", "keys"],
            "tools\nkeys\nsearch\ncomponents\nskills\nscratch\n",
            "front-rename-order.txt",
        ),
        (
            vec!["tab", "remove", "front", "keys"],
            "tools\nsearch\ncomponents\nskills\nscratch\n",
            "front-remove-order.txt",
        ),
    ] {
        let output = home
            .aw_command()
            .args(args)
            .current_dir(&project)
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join(order_file))
            .output()
            .expect("tab edit");
        assert_success("tab edit", &output);
        assert_eq!(read(profile.join("front.tabs")), expected_tabs);
    }
    let output = home
        .aw_command()
        .args(["refresh", "front"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-refresh-order.txt"),
        )
        .output()
        .expect("front refresh");
    assert_success("front refresh", &output);
    assert_eq!(stdout(&output), "Converged workspace front.");

    assert_failure(
        "legacy front list",
        &home
            .aw_command()
            .args(["front", "list"])
            .current_dir(&project)
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .output()
            .unwrap(),
    );
    let output = home
        .aw_command()
        .arg("check")
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("top-level check", &output);
    assert!(stderr(&output).contains("workspace not found: check"));

    temp::write(&tabs, "0\t0\tfalse\ttools\n1\t1\ttrue\tscratch 🤖\n");
    let output = Command::new(home.home.join(".local/bin/.zellij-new-scratch-tab"))
        .env("HOME", &home.home)
        .env("PATH", path_with(&home.bin))
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("scratch helper");
    assert_success("scratch helper", &output);
    assert!(read(&tabs).contains("2\t2\tfalse\tscratch1"));
    assert!(tabs.with_extension("tsv.saved").exists());

    assert_eq!(
        stdout(
            &home
                .aw_command()
                .arg("ps")
                .env("FAKE_ZELLIJ_SESSIONS", "frontend\nbackend")
                .output()
                .unwrap()
        ),
        "frontend\nbackend"
    );
    let deleted = home.root.join("deleted-session.txt");
    assert_success(
        "kill session",
        &home
            .aw_command()
            .args(["kill", "extra-session"])
            .env("FAKE_ZELLIJ_DELETED_SESSION", &deleted)
            .output()
            .unwrap(),
    );
    assert_eq!(read(deleted).trim_end(), "extra-session");
}
