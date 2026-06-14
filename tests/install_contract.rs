mod support;

use std::fs;

use support::command::{assert_failure, assert_success, stderr, stdout, TestHome};
use support::temp;

#[test]
fn install_writes_public_binary_private_helpers_config_and_completions() {
    let home = TestHome::new("install-contract");
    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .arg("install")
        .output()
        .expect("run aw install");
    assert_success("aw install", &output);

    assert_eq!(
        stdout(&output),
        "Installed Agent Workspace setup.\nOpen a new shell or run: export PATH=\"$HOME/.local/bin:$PATH\"\nIn a project directory, create a profile with: aw main=app,server,infra,scratch\nThen open a workspace with: aw main"
    );

    for executable in ["aw", ".zellij-new-scratch-tab"] {
        assert!(home.home.join(".local/bin").join(executable).is_file());
    }
    assert!(!home.home.join(".local/bin/goob").exists());

    for executable in [
        "zellij-launch-session",
        "zellij-session-tab-order",
        "zellij-saved-session-order",
        "zellij-live-tab-order",
        "zellij-new-scratch-tab",
        "zellij-open-session",
        "zellij-render-layout",
        "zwork",
        "zellij-workspace-init",
        "zellij-workspace-doctor",
        ".zellij-agent-tab-watcher",
    ] {
        assert!(
            home.home.join(".aw/bin").join(executable).is_file(),
            "missing private helper {executable}"
        );
    }

    for executable in [
        "zwork",
        "zellij-launch-session",
        "zellij-session-tab-order",
        "zellij-saved-session-order",
        "zellij-live-tab-order",
        "zellij-new-scratch-tab",
        "zellij-open-session",
        "zellij-render-layout",
        "zellij-workspace-init",
        "zellij-workspace-doctor",
        ".zellij-agent-tab-watcher",
    ] {
        assert!(
            !home.home.join(".local/bin").join(executable).exists(),
            "private helper leaked to public bin: {executable}"
        );
    }

    for file in [
        ".aw/config.kdl",
        ".codex/config.toml",
        ".claude/settings.json",
        ".aw/completions/_aw",
        ".aw/completions/aw.bash",
    ] {
        assert!(home.home.join(file).is_file(), "missing installed {file}");
    }
    let codex_config = fs::read_to_string(home.home.join(".codex/config.toml")).unwrap();
    assert!(codex_config.contains("[tui]"));
    assert!(codex_config.contains("status_line = ["));
    assert!(codex_config.contains("\"context-used\""));

    let claude_statusline = home.home.join(".aw/bin/claude-statusline");
    assert!(claude_statusline.is_file());
    let claude_statusline_script = fs::read_to_string(&claude_statusline).unwrap();
    assert!(claude_statusline_script.contains("used_percentage"));
    let claude_settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(home.home.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(claude_settings["statusLine"]["type"], "command");
    assert!(claude_settings["statusLine"]["command"]
        .as_str()
        .unwrap()
        .contains("claude-statusline"));

    let bash_completion = fs::read_to_string(home.home.join(".aw/completions/aw.bash")).unwrap();
    assert!(bash_completion.contains(
        "--check --verify --queue-root --root --summary --owner --must-contain --must-not-contain --poke --workspace --session --wait --timeout --poll"
    ));
    assert!(bash_completion.contains("git --queue-root --root --workspace --session"));
    assert!(bash_completion.contains("status|doctor)"));
    assert!(bash_completion.contains("doctor)"));
    assert!(bash_completion.contains("migrate)"));
    assert!(bash_completion.contains("routes)"));
    assert!(bash_completion.contains("doctor paths repo"));
    assert!(bash_completion.contains("tab session commit"));
    assert!(bash_completion.contains("wait)"));

    let zsh_completion = fs::read_to_string(home.home.join(".aw/completions/_aw")).unwrap();
    assert!(zsh_completion.contains(
        "--check --verify --queue-root --root --summary --owner --must-contain --must-not-contain --poke --workspace --session --wait --timeout --poll"
    ));
    assert!(zsh_completion.contains("git --queue-root --root --workspace --session"));
    assert!(zsh_completion
        .contains("doctor migrate clean measure-git probe-git-config routes worktree"));
    assert!(zsh_completion.contains("doctor paths repo"));
    assert!(zsh_completion.contains("tab session commit"));
    assert!(!zsh_completion.contains("Git --root"));

    let config = fs::read_to_string(home.home.join(".aw/config.kdl")).unwrap();
    assert!(!config.contains("/usr/bin/zsh"));
    assert!(config.contains("post_command_discovery_hook \"printf"));
    assert!(config.contains("${SHELL:-sh}"));
    assert!(config.contains("Run \".zellij-new-scratch-tab\""));
    assert!(!home.home.join(".zshrc").exists());
    assert!(!home.home.join(".bashrc").exists());
}

#[test]
fn install_copies_aw_tab_bar_plugin_when_wasm_source_is_available() {
    let home = TestHome::new("install-tab-bar-plugin");
    let plugin = home.root.join("aw-tab-bar.wasm");
    temp::write(&plugin, "wasm\n");

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .env("AW_TAB_BAR_WASM_SOURCE", &plugin)
        .arg("install")
        .output()
        .expect("run aw install");
    assert_success("aw install", &output);

    let installed = home.home.join(".aw/plugins/aw-tab-bar.wasm");
    assert_eq!(fs::read_to_string(&installed).unwrap(), "wasm\n");

    let permissions = fs::read_to_string(home.home.join(".cache/zellij/permissions.kdl")).unwrap();
    assert!(permissions.contains(&format!("\"file:{}\"", installed.display())));
    assert!(permissions.contains(&format!("\"{}\"", installed.display())));
    assert!(permissions.contains("\"aw-tab-bar\""));
    assert!(permissions.contains("\"aw-tab-bar.wasm\""));
    assert!(permissions.contains("ReadApplicationState"));
    assert!(permissions.contains("ChangeApplicationState"));
    assert!(permissions.contains("RunCommands"));
}

#[test]
fn install_preserves_existing_codex_status_line() {
    let home = TestHome::new("install-codex-config");
    temp::write(
        home.home.join(".codex/config.toml"),
        "model = \"gpt-5.5\"\n\n[tui]\nstatus_line = [\"model\", \"current-dir\"]\n",
    );

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .arg("install")
        .output()
        .expect("run aw install");
    assert_success("aw install", &output);

    let codex_config = fs::read_to_string(home.home.join(".codex/config.toml")).unwrap();
    assert!(codex_config.contains("model = \"gpt-5.5\""));
    assert!(codex_config.contains("status_line = [\"model\", \"current-dir\"]"));
    assert!(!codex_config.contains("\"context-used\""));
}

#[test]
fn install_preserves_existing_claude_status_line() {
    let home = TestHome::new("install-claude-settings");
    temp::write(
        home.home.join(".claude/settings.json"),
        r#"{"cleanupPeriodDays":30,"statusLine":{"type":"command","command":"custom status"}}"#,
    );

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .arg("install")
        .output()
        .expect("run aw install");
    assert_success("aw install", &output);

    let claude_settings = fs::read_to_string(home.home.join(".claude/settings.json")).unwrap();
    assert!(claude_settings.contains("\"custom status\""));
    assert!(claude_settings.contains("\"cleanupPeriodDays\":30"));
    assert!(!claude_settings.contains("claude-statusline"));
    assert!(home.home.join(".aw/bin/claude-statusline").is_file());
}

#[test]
fn install_migrates_legacy_aw_state_when_new_home_is_missing() {
    let home = TestHome::new("install-legacy-aw-migration");
    temp::write(
        home.home
            .join(".local/share/agent-workspace/profiles/legacy/front.tabs"),
        "app\nscratch\n",
    );
    temp::write(
        home.home
            .join(".local/share/agent-workspace/profiles/legacy/profile.conf"),
        "name=legacy\n",
    );
    temp::write(
        home.home
            .join(".local/share/agent-workspace/default-profile"),
        "legacy\n",
    );
    temp::write(home.home.join(".config/aw/config.kdl"), "legacy-config\n");

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .arg("install")
        .output()
        .expect("run aw install");
    assert_success("aw install", &output);

    assert!(home.home.join(".aw/profiles/legacy/front.tabs").is_file());
    assert_eq!(
        fs::read_to_string(home.home.join(".aw/default-profile")).unwrap(),
        "legacy\n"
    );
    assert!(home.home.join(".aw/config.kdl").is_file());
}

#[test]
fn install_repo_dry_run_reports_aw_adapters() {
    let home = TestHome::new("install-repo-dry-run");
    let repo = home.root.join("repo");
    temp::write(
        repo.join("infra/aw/agents/.agents/AGENTS.md"),
        "# shared agents\n",
    );
    temp::write(
        repo.join("infra/aw/Cargo.toml"),
        "[package]\nname = \"aw\"\n",
    );

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["install", "--repo", "--dry-run"])
        .output()
        .expect("run aw install dry-run");
    assert_success("aw install --repo --dry-run", &output);

    assert_eq!(
        stdout(&output),
        "would   AGENTS.md from infra/aw/agents/.agents/templates/root-AGENTS.md\nwould   .agents.local/project.md from infra/aw/agents/.agents/templates/project.md\nwould   .agents -> infra/aw/agents/.agents\nwould   CLAUDE.md -> AGENTS.md\nwould   .claude/skills -> ../.agents/skills\ndone    infra/aw"
    );
    assert!(!repo.join("AGENTS.md").exists());
    assert!(!repo.join(".agents").exists());
}

#[test]
fn install_repo_rejects_external_config_before_partial_setup() {
    let home = TestHome::new("install-repo-external-config");
    let repo = home.root.join("repo");
    temp::write(repo.join("profiles/demo/profile.conf"), "name=demo\n");

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["install", "--repo", "--config", "profiles/demo"])
        .output()
        .expect("run external-config repo install");

    assert_failure("aw install --repo --config profiles/demo", &output);
    assert!(stderr(&output).contains("install --repo uses config/aw"));
    assert!(!repo.join("AGENTS.md").exists());
}

#[test]
fn install_config_rejects_missing_value_before_setup() {
    let home = TestHome::new("install-missing-config-value");
    let repo = home.root.join("repo");
    std::fs::create_dir_all(&repo).unwrap();

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["install", "--config", "--repo"])
        .output()
        .expect("run install missing config value");

    assert_failure("aw install --config --repo", &output);
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("install --config requires a path"));
    assert!(!repo.join("AGENTS.md").exists());
}

#[test]
fn install_repo_auto_config_creates_adapters_and_profile() {
    let home = TestHome::new("install-repo-config");
    let repo = home.root.join("repo");
    temp::write(
        repo.join("infra/aw/agents/.agents/AGENTS.md"),
        "# shared agents\n",
    );
    temp::write(
        repo.join("infra/aw/Cargo.toml"),
        "[package]\nname = \"aw\"\n",
    );
    temp::write(
        repo.join("config/aw/profile.conf"),
        "name=demo\nroot=.\ndefault_workspace=front\n",
    );
    temp::write(repo.join("config/aw/front.tabs"), "app\ngit\nscratch\n");

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .current_dir(&repo)
        .args(["install", "--repo"])
        .output()
        .expect("run one-stop aw install");
    assert_success("aw install --repo", &output);

    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join(".agents.local/project.md").is_file());
    assert_eq!(
        fs::read_link(repo.join(".agents")).unwrap(),
        std::path::Path::new("infra/aw/agents/.agents")
    );
    assert_eq!(
        fs::read_link(repo.join("CLAUDE.md")).unwrap(),
        std::path::Path::new("AGENTS.md")
    );
    assert_eq!(
        fs::read_link(repo.join(".claude/skills")).unwrap(),
        std::path::Path::new("../.agents/skills")
    );
    assert!(home.home.join(".aw/profiles/demo/front.tabs").is_file());
    assert!(stdout(&output).contains("Installed Zellij profile demo."));
}

#[cfg(unix)]
#[test]
fn install_repo_repairs_stale_managed_symlink() {
    let home = TestHome::new("install-repo-stale-link");
    let repo = home.root.join("repo");
    temp::write(
        repo.join("infra/aw/agents/.agents/AGENTS.md"),
        "# shared agents\n",
    );
    temp::write(
        repo.join("infra/aw/Cargo.toml"),
        "[package]\nname = \"aw\"\n",
    );
    temp::write(
        repo.join("config/aw/profile.conf"),
        "name=demo\nroot=.\ndefault_workspace=front\n",
    );
    temp::write(repo.join("config/aw/front.tabs"), "app\ngit\nscratch\n");
    std::os::unix::fs::symlink("infra/agent-workspace/agents/.agents", repo.join(".agents"))
        .unwrap();

    let output = home
        .command(support::command::aw())
        .env("ZELLIJ_INSTALL_BINARY", "0")
        .env("ZELLIJ_INSTALL_SHELL_RC", "0")
        .current_dir(&repo)
        .args(["install", "--repo"])
        .output()
        .expect("run aw install with stale symlink");
    assert_success("aw install --repo", &output);

    assert_eq!(
        fs::read_link(repo.join(".agents")).unwrap(),
        std::path::Path::new("infra/aw/agents/.agents")
    );
}
