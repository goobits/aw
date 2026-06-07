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
fn gitq_rejects_raw_mutations_and_allows_lockless_reads() {
    let repo = repo("gitq-raw");
    let status = Command::new(support::command::aw())
        .args(["gitq", "status-fast"])
        .current_dir(repo.path())
        .output()
        .expect("status-fast");
    assert_success("status-fast", &status);

    let raw_add = Command::new(support::command::aw())
        .args(["gitq", "--", "add", "owned.txt"])
        .current_dir(repo.path())
        .output()
        .expect("raw add");
    assert_failure("raw add", &raw_add);
    assert!(stderr(&raw_add).contains("Refusing raw \"git add\""));
}

#[test]
fn gitq_commit_owned_commits_only_requested_paths() {
    let repo = repo("gitq-commit-owned");
    temp::write(repo.join("owned.txt"), "updated\n");
    temp::write(repo.join("other.txt"), "untracked\n");

    let commit = Command::new(support::command::aw())
        .args([
            "gitq",
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
        .args(["gitq", "commit-owned", "-m", "bad", "--", "*.txt"])
        .current_dir(repo.path())
        .output()
        .expect("glob");
    assert_failure("glob", &glob);
    assert!(stderr(&glob).contains("not globs"));
}

#[test]
fn pkgq_lock_info_uses_native_queue_surface() {
    let repo = repo("pkgq-lock-info");
    let output = Command::new(support::command::aw())
        .args(["pkgq", "lock-info"])
        .current_dir(repo.path())
        .output()
        .expect("pkgq lock-info");
    assert_success("pkgq lock-info", &output);
    assert_eq!(stdout(&output), "pkgq: no active queue lock");
}
