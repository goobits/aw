mod support;

use std::fs;

use support::command::{assert_success, stdout, TestHome};
use support::temp;

#[test]
fn install_writes_only_the_public_binary_and_completions() {
    let home = TestHome::new("install-contract");
    temp::write(
        home.home.join(".bashrc"),
        "before\n# >>> zellij workspaces >>>\nlegacy\n# <<< zellij workspaces <<<\nafter\n",
    );
    temp::write(
        home.home.join(".aw/bin/.zellij-agent-tab-watcher"),
        "legacy\n",
    );

    let output = home
        .command(support::command::aw())
        .arg("install")
        .output()
        .unwrap();
    assert_success("aw install", &output);
    assert_eq!(
        stdout(&output),
        "Installed Agent Workspace coordination tools.\nOpen a new shell or run: export PATH=\"$HOME/.local/bin:$PATH\""
    );
    assert!(home.home.join(".local/bin/aw").is_file());
    assert!(home.home.join(".aw/completions/_aw").is_file());
    assert!(home.home.join(".aw/completions/aw.bash").is_file());
    assert!(!home.home.join(".aw/bin/.zellij-agent-tab-watcher").exists());
    assert_eq!(
        fs::read_to_string(home.home.join(".bashrc")).unwrap(),
        "before\nafter\n"
    );
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
        .unwrap();
    assert_success("aw install --repo --dry-run", &output);
    assert!(stdout(&output).contains("would   AGENTS.md"));
    assert!(stdout(&output).contains("would   .agents -> infra/aw/agents/.agents"));
    assert!(!repo.join("AGENTS.md").exists());
}

#[test]
fn install_repo_creates_adapters_and_runs_doctor() {
    let home = TestHome::new("install-repo");
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
        "name=demo\nroot=.\ncommit_owner=enabled\n",
    );

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["install", "--repo"])
        .output()
        .unwrap();
    assert_success("aw install --repo", &output);
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join(".agents.local/project.md").is_file());
    assert_eq!(
        fs::read_link(repo.join(".agents")).unwrap(),
        std::path::Path::new("infra/aw/agents/.agents")
    );
    assert!(stdout(&output).contains("ok      commit_owner=enabled"));
    assert!(stdout(&output).contains("ok      repo adapters ready"));
}

#[cfg(unix)]
#[test]
fn install_repo_repairs_a_stale_managed_symlink() {
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
        "name=demo\nroot=.\ncommit_owner=disabled\n",
    );
    std::os::unix::fs::symlink("infra/old/agents/.agents", repo.join(".agents")).unwrap();

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["install", "--repo"])
        .output()
        .unwrap();
    assert_success("aw install --repo", &output);
    assert_eq!(
        fs::read_link(repo.join(".agents")).unwrap(),
        std::path::Path::new("infra/aw/agents/.agents")
    );
}
