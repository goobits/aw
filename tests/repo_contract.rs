mod support;

use std::fs;
use std::path::Path;

use support::command::{assert_success, stdout, TestHome};
use support::temp;

#[test]
fn doctor_repo_reports_ready_adapters_and_git_tab() {
    let home = TestHome::new("repo-doctor");
    let repo = home.root.join("repo");
    write_ready_repo(&repo);

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["repo", "doctor"])
        .output()
        .expect("run aw repo doctor");

    assert_success("aw repo doctor", &output);
    let stdout = stdout(&output);
    assert!(stdout.contains("ok      .agents -> infra/aw/agents/.agents"));
    assert!(stdout.contains("ok      lowercase git tab"));
    assert!(stdout.contains("ok      repo adapters ready"));
}

#[test]
fn migrate_repo_creates_current_adapters() {
    let home = TestHome::new("repo-migrate");
    let repo = home.root.join("repo");
    temp::write(
        repo.join("infra/aw/Cargo.toml"),
        "[package]\nname = \"aw\"\n",
    );
    temp::write(
        repo.join("infra/aw/agents/.agents/AGENTS.md"),
        "# shared agents\n",
    );
    temp::write(repo.join("AGENTS.md"), "# root\n");
    temp::write(repo.join(".agents.local/project.md"), "# project\n");
    temp::write(
        repo.join("config/aw/profile.conf"),
        "name=demo\nroot=.\ndefault_workspace=front\n",
    );
    temp::write(repo.join("config/aw/front.tabs"), "app\ngit\nscratch\n");

    let output = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["repo", "migrate"])
        .output()
        .expect("run aw repo migrate");

    assert_success("aw repo migrate", &output);
    assert!(repo
        .join("infra/aw/agents/.agents/AGENTS.md")
        .is_file());
    assert_eq!(
        fs::read_link(repo.join(".agents")).unwrap(),
        Path::new("infra/aw/agents/.agents")
    );
    assert_eq!(
        fs::read_link(repo.join("CLAUDE.md")).unwrap(),
        Path::new("AGENTS.md")
    );
    assert_eq!(
        fs::read_link(repo.join(".claude/skills")).unwrap(),
        Path::new("../.agents/skills")
    );

    let doctor = home
        .command(support::command::aw())
        .current_dir(&repo)
        .args(["repo", "doctor"])
        .output()
        .expect("run repo doctor after migrate");
    assert_success("aw repo doctor after migrate", &doctor);
}

fn write_ready_repo(repo: &Path) {
    temp::write(
        repo.join("infra/aw/Cargo.toml"),
        "[package]\nname = \"aw\"\n",
    );
    temp::write(
        repo.join("infra/aw/agents/.agents/AGENTS.md"),
        "# shared\n",
    );
    temp::write(repo.join("AGENTS.md"), "# root\n");
    temp::write(repo.join(".agents.local/project.md"), "# project\n");
    temp::write(repo.join("config/aw/profile.conf"), "name=demo\n");
    temp::write(repo.join("config/aw/front.tabs"), "app\ngit\nscratch\n");
    symlink(
        "infra/aw/agents/.agents",
        &repo.join(".agents"),
    );
    symlink("AGENTS.md", &repo.join("CLAUDE.md"));
    symlink("../.agents/skills", &repo.join(".claude/skills"));
}

#[cfg(unix)]
fn symlink(target: &str, link: &Path) {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(not(unix))]
fn symlink(_target: &str, _link: &Path) {}
