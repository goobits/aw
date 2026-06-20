mod support;

use std::process::Command;

use support::command::{
    assert_failure, assert_order, assert_success, expected_session, path_with, stderr, stdout,
    TestHome,
};
use support::fake_zellij;
use support::temp::{self, read, TempDir};

fn install_test_aw(name: &str) -> TestHome {
    fake_zellij::installed_home(name)
}

#[test]
fn bare_aw_shows_help_instead_of_launching_default_workspace() {
    let home = install_test_aw("workspace-help");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        "name=my-site\nroot=/tmp/project\ndefault_workspace=frontend\ndefault_workspaces=frontend\n",
    );
    temp::write(profile.join("frontend.tabs"), "app\nui\nscratch\n");

    let output = home
        .aw_command()
        .current_dir(&project)
        .env("FAKE_ZELLIJ_FAIL_ON_ATTACH", "1")
        .output()
        .expect("run bare aw");
    assert_success("bare aw", &output);
    assert!(stdout(&output).contains("🌀 aw: Zero-friction Zellij workspaces"));
    assert!(stdout(&output).contains("aw                                show help"));
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
fn default_sessions_are_scoped_to_local_config_owner_root() {
    let home = install_test_aw("workspace-session-scope");
    let tabs = home.root.join("tabs.tsv");
    temp::write(&tabs, "");
    temp::write(tabs.with_extension("tsv.panes"), "");

    for project_name in ["checkout-a", "checkout-b"] {
        let project = home.root.join(project_name);
        let profile = project.join("config/aw");
        temp::write(
            profile.join("profile.conf"),
            "name=shared\nroot=/same/checked-in/root\ndefault_workspace=front\ndefault_workspaces=front\n",
        );
        temp::write(profile.join("front.tabs"), "app\nscratch\n");

        let output = home
            .aw_command()
            .arg("front")
            .current_dir(&project)
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .env(
                "FAKE_ZELLIJ_ORDER_ARGS",
                home.root.join(format!("{project_name}-order.txt")),
            )
            .env_remove("ZELLIJ")
            .env_remove("ZELLIJ_SESSION_NAME")
            .output()
            .expect("launch scoped workspace");
        assert_success("launch scoped workspace", &output);

        assert_order(
            home.root.join(format!("{project_name}-order.txt")),
            &expected_session("shared", "front", project.display()),
            &["app", "scratch"],
        );

        let session_name = home
            .aw_command()
            .args(["session", "name"])
            .current_dir(&project)
            .output()
            .expect("default session name");
        assert_success("default session name", &session_name);
        assert_eq!(
            stdout(&session_name),
            expected_session("shared", "front", project.display())
        );

        let named_workspace = home
            .aw_command()
            .args(["session", "name", "front"])
            .current_dir(&project)
            .output()
            .expect("workspace session name");
        assert_success("workspace session name", &named_workspace);
        assert_eq!(
            stdout(&named_workspace),
            expected_session("shared", "front", project.display())
        );
    }

    assert_ne!(
        read(home.root.join("checkout-a-order.txt"))
            .lines()
            .next()
            .unwrap(),
        read(home.root.join("checkout-b-order.txt"))
            .lines()
            .next()
            .unwrap(),
    );
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
    assert!(home.home.join(".aw/profiles/my-site/main.tabs").is_file());
    assert_order(
        home.root.join("main-order.txt"),
        &expected_session("my-site", "main", project.display()),
        &["app", "server", "infra", "scratch"],
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

    let renamed_session = home.root.join("renamed-session.txt");
    let live_tabs = home.root.join("backend-live-tabs.tsv");
    temp::write(&live_tabs, "");
    temp::write(live_tabs.with_extension("tsv.panes"), "");
    let output = home
        .aw_command()
        .args(["rename", "backend", "services"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &live_tabs)
        .env(
            "FAKE_ZELLIJ_SESSIONS",
            expected_session("my-site", "backend", project.display()),
        )
        .env("FAKE_ZELLIJ_RENAMED_SESSION", &renamed_session)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("services-order.txt"),
        )
        .output()
        .expect("rename backend");
    assert_success("rename backend", &output);
    assert!(!project.join("config/aw/backend.tabs").exists());
    assert_eq!(
        read(project.join("config/aw/services.tabs")),
        "infra\napi\ndb\n"
    );
    assert_eq!(
        read(&renamed_session).trim_end(),
        format!(
            "{}\t{}",
            expected_session("my-site", "backend", project.display()),
            expected_session("my-site", "services", project.display())
        )
    );
    assert_order(
        home.root.join("services-order.txt"),
        &expected_session("my-site", "services", project.display()),
        &["infra", "api", "db"],
    );
    assert!(read(project.join("config/aw/profile.conf"))
        .contains("default_workspaces=main frontend services"));

    let output = run_in_project(&home, &project, &["list"]);
    assert_success("list", &output);
    assert_eq!(
        stdout(&output),
        "frontend\n  app\n  ui\n  tools\nmain\n  app\n  server\n  infra\n  scratch\nservices\n  infra\n  api\n  db"
    );

    let output = run_in_project(&home, &project, &["rename", "services", "frontend"]);
    assert_failure("rename over existing", &output);

    let output = run_in_project(&home, &project, &["remove", "frontend"]);
    assert_success("remove frontend", &output);
    assert!(!project.join("config/aw/frontend.tabs").exists());
    assert!(!home
        .home
        .join(".aw/profiles/my-site/frontend.tabs")
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
        "services\n  infra\n  api\n  db"
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
fn tab_rename_preserves_live_tab_instead_of_recreating_it() {
    let home = install_test_aw("workspace-tab-rename-live");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        &format!(
            "name=project\nroot={}\ndefault_workspace=front\ndefault_workspaces=front\n",
            project.display()
        ),
    );
    temp::write(
        profile.join("front.tabs"),
        "tools\ncomponents\nkeyboard\nscratch\n",
    );

    let tabs = home.root.join("live-tabs");
    temp::write(
        &tabs,
        "0\t0\tfalse\ttools\n1\t1\tfalse\tcomponents\n2\t2\ttrue\tkeyboard\n3\t3\tfalse\tscratch\n",
    );

    let output = home
        .aw_command()
        .args(["front", "tab", "rename", "keyboard", "keys@1"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_SESSIONS",
            expected_session("project", "front", project.display()),
        )
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-rename-order.txt"),
        )
        .output()
        .expect("tab rename");
    assert_success("tab rename", &output);

    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeys\ncomponents\nscratch\n"
    );
    assert_eq!(fake_zellij::tab_name(&tabs, "2"), "keys");
    assert_eq!(
        fake_zellij::sorted_tab_names(&tabs),
        vec!["tools", "keys", "components", "scratch"]
    );
    assert!(!tabs.with_extension("cwds").exists());
}

#[test]
fn tab_commands_infer_the_only_workspace() {
    let home = install_test_aw("workspace-tab-shorthand");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        &format!(
            "name=project\nroot={}\ndefault_workspace=front\ndefault_workspaces=front\n",
            project.display()
        ),
    );
    temp::write(profile.join("front.tabs"), "tools\nkeyboard\nscratch\n");

    let tabs = home.root.join("tabs.tsv");
    temp::write(
        &tabs,
        "0\t0\tfalse\ttools\n1\t1\ttrue\tkeyboard\n2\t2\tfalse\tscratch\n",
    );

    let output = home
        .aw_command()
        .args(["tab", "list"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("tab list shorthand");
    assert_success("tab list shorthand", &output);
    assert_eq!(stdout(&output), "  0 tools\n* 1 keyboard\n  2 scratch");

    let output = home
        .aw_command()
        .args(["tab", "list", "front"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("removed workspace tab shorthand");
    assert_failure("removed workspace tab shorthand", &output);
    assert!(stderr(&output).contains("aw tab list [--session <name>]"));

    let front_session = expected_session("project", "front", project.display());
    for (args, expected_tabs, order_file) in [
        (
            vec!["tab", "add", "search@1"],
            "tools\nsearch\nkeyboard\nscratch\n",
            "shorthand-add-order.txt",
        ),
        (
            vec!["tab", "move", "keyboard@1"],
            "tools\nkeyboard\nsearch\nscratch\n",
            "shorthand-move-order.txt",
        ),
        (
            vec!["tab", "rename", "keyboard", "keys@0"],
            "keys\ntools\nsearch\nscratch\n",
            "shorthand-rename-order.txt",
        ),
        (
            vec!["tab", "remove", "keys"],
            "tools\nsearch\nscratch\n",
            "shorthand-remove-order.txt",
        ),
    ] {
        let output = home
            .aw_command()
            .args(args)
            .current_dir(&project)
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .env("FAKE_ZELLIJ_SESSIONS", &front_session)
            .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join(order_file))
            .output()
            .expect("tab edit shorthand");
        assert_success("tab edit shorthand", &output);
        assert_eq!(read(profile.join("front.tabs")), expected_tabs);
    }

    let output = home
        .aw_command()
        .args(["tab", "refresh"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_SESSIONS", &front_session)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("shorthand-refresh-order.txt"),
        )
        .output()
        .expect("tab refresh shorthand");
    assert_success("tab refresh shorthand", &output);
    assert_eq!(stdout(&output), "Converged workspace front.");
}

#[test]
fn tab_refresh_reports_missing_target_session_with_live_hint() {
    let home = install_test_aw("workspace-tab-refresh-missing-session");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        &format!(
            "name=project\nroot={}\ndefault_workspace=front\ndefault_workspaces=front\n",
            project.display()
        ),
    );
    temp::write(profile.join("front.tabs"), "tools\nkeyboard\nscratch\n");

    let tabs = home.root.join("live-tabs");
    temp::write(
        &tabs,
        "0\t0\tfalse\ttools\n1\t1\ttrue\tkeyboard\n2\t2\tfalse\tscratch\n",
    );
    temp::write(tabs.with_extension("tsv.panes"), "");

    let output = home
        .aw_command()
        .args(["refresh", "front"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_SESSIONS", "front\nother")
        .output()
        .expect("missing target session refresh");
    assert_failure("missing target session refresh", &output);
    assert!(stderr(&output).contains(&format!(
        "target Zellij session {} is not running",
        expected_session("project", "front", project.display())
    )));
    assert!(stderr(&output).contains("Live sessions: front, other"));
    assert!(stderr(&output).contains("Use --session front"));
    assert!(!stderr(&output).contains("Converged workspace front."));
}

#[test]
fn malformed_tab_and_launch_commands_report_scoped_usage() {
    let home = install_test_aw("workspace-cli-bad-input");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(
        profile.join("profile.conf"),
        &format!(
            "name=project\nroot={}\ndefault_workspace=front\ndefault_workspaces=front backend\n",
            project.display()
        ),
    );
    temp::write(profile.join("front.tabs"), "tools\nkeyboard\nscratch\n");
    temp::write(profile.join("backend.tabs"), "api\ndb\n");

    let bad_rename = run_in_project(&home, &project, &["tab", "rename", "keyboard"]);
    assert_failure("bad tab rename", &bad_rename);
    assert!(stderr(&bad_rename).contains("aw tab rename <old-tab> <new-tab[@index]>"));
    assert!(!stderr(&bad_rename).contains("Zero-friction Zellij workspaces"));

    let bad_workspace_list = run_in_project(&home, &project, &["front", "tab", "list", "extra"]);
    assert_failure("bad workspace tab list", &bad_workspace_list);
    assert!(stderr(&bad_workspace_list).contains("usage:\n  aw front tab list"));
    assert!(!stderr(&bad_workspace_list).contains("commit queue:"));

    let bad_move = run_in_project(&home, &project, &["front", "tab", "move", "keyboard"]);
    assert_failure("bad workspace tab move", &bad_move);
    assert!(stderr(&bad_move).contains("aw front tab move keyboard@1"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let bad_index = run_in_project(&home, &project, &["front", "tab", "move", "keyboard@later"]);
    assert_failure("bad tab index", &bad_index);
    assert!(stderr(&bad_index).contains("tab index must be a number"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let out_of_range_index =
        run_in_project(&home, &project, &["front", "tab", "move", "keyboard@3"]);
    assert_failure("out of range tab index", &out_of_range_index);
    assert!(stderr(&out_of_range_index).contains("tab index 3 is past the end"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let bad_rename_index = run_in_project(
        &home,
        &project,
        &["front", "tab", "rename", "keyboard", "keys@later"],
    );
    assert_failure("bad tab rename index", &bad_rename_index);
    assert!(stderr(&bad_rename_index).contains("tab index must be a number"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let out_of_range_rename_index = run_in_project(
        &home,
        &project,
        &["front", "tab", "rename", "keyboard", "keys@3"],
    );
    assert_failure("out of range rename index", &out_of_range_rename_index);
    assert!(stderr(&out_of_range_rename_index).contains("tab index 3 is past the end"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let removed_tab_add = run_in_project(&home, &project, &["tab", "add", "front", "search"]);
    assert_failure("removed tab add shorthand", &removed_tab_add);
    assert!(stderr(&removed_tab_add).contains("aw tab add <tab[@index]>"));

    let bad_refresh = run_in_project(&home, &project, &["refresh"]);
    assert_failure("bad refresh", &bad_refresh);
    assert!(stderr(&bad_refresh).contains("usage:\n  aw refresh <workspace>"));
    assert!(!stderr(&bad_refresh).contains("commit queue:"));

    let huge_index = "keyboard@999999999999999999999999999999999999999";
    let bad_huge_index = run_in_project(&home, &project, &["front", "tab", "move", huge_index]);
    assert_failure("bad huge tab index", &bad_huge_index);
    assert!(stderr(&bad_huge_index).contains("tab index is too large"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let empty_add = run_in_project(&home, &project, &["front", "tab", "add", ""]);
    assert_failure("empty tab add", &empty_add);
    assert!(stderr(&empty_add).contains("tab name cannot be empty"));
    assert_eq!(
        read(profile.join("front.tabs")),
        "tools\nkeyboard\nscratch\n"
    );

    let missing_session = run_in_project(&home, &project, &["front", "--session"]);
    assert_failure("missing launch session", &missing_session);
    assert!(stderr(&missing_session).contains("--session requires a session name"));
    assert!(!stderr(&missing_session).contains("zwork: missing profile"));

    let missing_root = run_in_project(&home, &project, &["front", "--root"]);
    assert_failure("missing launch root", &missing_root);
    assert!(stderr(&missing_root).contains("--root requires a path"));
    assert!(!stderr(&missing_root).contains("root directory does not exist:"));
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
            .nth(1)
            .unwrap(),
        "🌀 aw: Zero-friction Zellij workspaces"
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
        .env(
            "FAKE_ZELLIJ_SESSIONS",
            expected_session("strict-site", "frontend", strict_project.display()),
        )
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
        "backend\n  api\n  database\n  scratch\nextra\n  notes\n  scratch\nfrontend\n  app\n  ui\n  scratch"
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
        .arg("frontend")
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("default-order.txt"),
        )
        .env_remove("ZELLIJ")
        .env_remove("ZELLIJ_SESSION_NAME")
        .output()
        .expect("frontend launch");
    assert_success("frontend launch", &output);
    assert!(stdout(&output).is_empty());
    assert_order(
        home.root.join("default-order.txt"),
        &expected_session("my-site", "frontend", project.display()),
        &["app", "ui", "scratch"],
    );

    let alternate_root = home.root.join("alternate-root");
    std::fs::create_dir_all(&alternate_root).unwrap();
    let output = home
        .aw_command()
        .args(["extra", "-r"])
        .arg(&alternate_root)
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("extra-root-override-order.txt"),
        )
        .env_remove("ZELLIJ")
        .env_remove("ZELLIJ_SESSION_NAME")
        .output()
        .expect("extra root override launch");
    assert_success("extra root override launch", &output);
    assert_order(
        home.root.join("extra-root-override-order.txt"),
        &expected_session("my-site", "extra", project.display()),
        &["notes", "scratch"],
    );

    let output = home
        .aw_command()
        .args(["extra", "-s", "extra-session", "-r"])
        .arg(&project)
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join("extra-order.txt"))
        .env_remove("ZELLIJ")
        .env_remove("ZELLIJ_SESSION_NAME")
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
        .args(["front", "tab", "list"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("tab list");
    assert_success("tab list", &output);
    assert_eq!(
        stdout(&output),
        "  0 tools\n  1 components\n  2 keyboard\n* 3 skills\n  4 scratch"
    );

    let front_session = expected_session("my-site", "front", project.display());
    for (args, expected_tabs, order_file) in [
        (
            vec!["front", "tab", "add", "search@1"],
            "tools\nsearch\ncomponents\nkeyboard\nskills\nscratch\n",
            "front-add-order.txt",
        ),
        (
            vec!["front", "tab", "move", "keyboard@1"],
            "tools\nkeyboard\nsearch\ncomponents\nskills\nscratch\n",
            "front-move-order.txt",
        ),
        (
            vec!["front", "tab", "rename", "keyboard", "keys"],
            "tools\nkeys\nsearch\ncomponents\nskills\nscratch\n",
            "front-rename-order.txt",
        ),
        (
            vec!["front", "tab", "remove", "keys"],
            "tools\nsearch\ncomponents\nskills\nscratch\n",
            "front-remove-order.txt",
        ),
    ] {
        let output = home
            .aw_command()
            .args(args)
            .current_dir(&project)
            .env("FAKE_ZELLIJ_TABS", &tabs)
            .env("FAKE_ZELLIJ_SESSIONS", &front_session)
            .env("FAKE_ZELLIJ_ORDER_ARGS", home.root.join(order_file))
            .output()
            .expect("tab edit");
        assert_success("tab edit", &output);
        assert_eq!(read(profile.join("front.tabs")), expected_tabs);
    }
    let output = home
        .aw_command()
        .args(["front", "tab", "refresh", "--session", "front-explicit"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_SESSIONS", "front-explicit")
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-explicit-refresh-order.txt"),
        )
        .output()
        .expect("explicit session tab refresh");
    assert_success("explicit session tab refresh", &output);
    assert_order(
        home.root.join("front-explicit-refresh-order.txt"),
        "front-explicit",
        &["tools", "search", "components", "skills", "scratch"],
    );

    let output = home
        .aw_command()
        .args(["refresh", "front"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env("FAKE_ZELLIJ_SESSIONS", &front_session)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-refresh-order.txt"),
        )
        .output()
        .expect("front refresh");
    assert_success("front refresh", &output);
    assert_eq!(stdout(&output), "Converged workspace front.");

    let output = home
        .aw_command()
        .args(["tab", "rename", "keyboard", "keys"])
        .current_dir(&project)
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .output()
        .expect("ambiguous tab shorthand");
    assert_failure("ambiguous tab shorthand", &output);
    assert!(
        stderr(&output).contains("tab rename needs a workspace because multiple workspaces exist")
    );
    assert!(stderr(&output).contains("Example: aw backend tab rename"));

    assert_failure(
        "unsupported front list",
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
