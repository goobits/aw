mod support;

use std::process::Command;

use support::command::{assert_failure, assert_success, stderr, stdout};
use support::temp::{self, TempDir};

fn git(repo: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git command");
    assert_success(&format!("git {}", args.join(" ")), &output);
}

fn repo(name: &str) -> TempDir {
    let repo = TempDir::new(name);
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["config", "user.email", "aw@example.test"]);
    git(
        repo.path(),
        &["config", "user.name", "Agent Workspace Test"],
    );
    temp::write(repo.join("owned.txt"), "initial\n");
    git(repo.path(), &["add", "owned.txt"]);
    git(repo.path(), &["commit", "-q", "-m", "initial"]);
    repo
}

#[test]
fn owner_git_rejects_raw_mutations_and_allows_lockless_reads() {
    let repo = repo("owner-git-raw");
    let status = Command::new(support::command::aw())
        .args(["owner", "git", "status-fast"])
        .current_dir(repo.path())
        .output()
        .expect("status-fast");
    assert_success("status-fast", &status);

    let raw_add = Command::new(support::command::aw())
        .args(["owner", "git", "--", "add", "owned.txt"])
        .current_dir(repo.path())
        .output()
        .expect("raw add");
    assert_failure("raw add", &raw_add);
    assert!(stderr(&raw_add).contains("Refusing raw \"git add\""));
}

#[test]
fn owner_git_status_reports_full_worktree_awareness() {
    let project = repo("owner-git-status-aware");
    let submodule_source = repo("owner-git-status-aware-submodule");
    let submodule_source_path = submodule_source.path().to_string_lossy().to_string();
    git(
        project.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &submodule_source_path,
            "nested",
        ],
    );
    git(project.path(), &["add", ".gitmodules", "nested"]);
    git(
        project.path(),
        &["commit", "-q", "-m", "add nested submodule"],
    );

    temp::write(project.join("owned.txt"), "updated\n");
    temp::write(project.join("untracked.txt"), "new\n");
    temp::write(project.join("nested/owned.txt"), "nested update\n");

    let status = Command::new(support::command::aw())
        .args(["owner", "git", "status", "--short"])
        .current_dir(project.path())
        .output()
        .expect("status");
    assert_success("status", &status);
    let output = stdout(&status);
    assert!(output.contains(" M owned.txt"), "{output}");
    assert!(output.contains("?? untracked.txt"), "{output}");
    assert!(output.contains("nested"), "{output}");
}

#[test]
fn owner_git_commit_owned_commits_only_requested_paths() {
    let repo = repo("owner-git-commit-owned");
    temp::write(repo.join("owned.txt"), "updated\n");
    temp::write(repo.join("other.txt"), "untracked\n");

    let commit = Command::new(support::command::aw())
        .args([
            "owner",
            "git",
            "commit-owned",
            "-m",
            "update owned",
            "--",
            "owned.txt",
        ])
        .current_dir(repo.path())
        .output()
        .expect("commit-owned");
    assert_success("commit-owned", &commit);

    let log = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo.path())
        .output()
        .expect("git log");
    assert_success("git log", &log);
    assert_eq!(stdout(&log), "update owned");

    let glob = Command::new(support::command::aw())
        .args(["owner", "git", "commit-owned", "-m", "bad", "--", "*.txt"])
        .current_dir(repo.path())
        .output()
        .expect("glob");
    assert_failure("glob", &glob);
    assert!(stderr(&glob).contains("not globs"));
}

#[test]
fn owner_pkg_lock_info_uses_native_queue_surface() {
    let repo = repo("owner-pkg-lock-info");
    let output = Command::new(support::command::aw())
        .args(["owner", "pkg", "lock-info"])
        .current_dir(repo.path())
        .output()
        .expect("owner-pkg lock-info");
    assert_success("owner-pkg lock-info", &output);
    assert_eq!(stdout(&output), "pkgq: no active queue lock");
}
