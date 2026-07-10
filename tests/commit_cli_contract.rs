mod support;

use support::command::{assert_failure, assert_success, stderr, stdout, TestHome};
use support::temp::{self, read};

fn installed_home(name: &str) -> TestHome {
    let home = TestHome::new(name);
    home.install_aw();
    home
}

#[test]
fn commit_queue_creates_reports_and_completes_requests() {
    let home = installed_home("commit-cli");
    let project = home.root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    temp::write(project.join("README.md"), "# Commit queue test\n");
    let queue = home.root.join("commit-queue");

    let output = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Commit queue docs",
            "README.md",
            "--owner",
            "Maple",
            "--check",
            "echo ok",
            "--queue-root",
        ])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("commit request", &output);
    let request_id = stdout(&output)
        .split_whitespace()
        .nth(3)
        .unwrap()
        .trim_end_matches('.')
        .to_string();
    assert!(stdout(&output).contains("Run `aw commit poke --queue-root "));

    let status = home
        .aw_command()
        .args(["commit", "status", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("commit status", &status);
    assert!(stdout(&status).contains("Pending  1"));
    assert!(stdout(&status).contains("Next     ready"));

    let done = home
        .aw_command()
        .args(["commit", "done", &request_id, "--root"])
        .arg(&queue)
        .args([
            "--commit",
            "abc123",
            "--message",
            "Commit queue docs committed",
            "--verify-result",
            "passed",
        ])
        .output()
        .unwrap();
    assert_success("commit done", &done);

    let waited = home
        .aw_command()
        .args(["commit", "wait", &request_id, "--root"])
        .arg(&queue)
        .args(["--timeout", "1s"])
        .output()
        .unwrap();
    assert_success("commit wait", &waited);
    assert!(stdout(&waited).contains("Commit   abc123"));

    let missing = home
        .aw_command()
        .args(["commit", "request", "Missing paths"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("missing paths", &missing);
    assert!(stderr(&missing).contains("requires a title and at least one path"));
}

#[test]
fn commit_poke_uses_one_configured_shelly_hook() {
    let home = installed_home("commit-poke");
    let project = home.root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    temp::write(project.join("README.md"), "# Commit queue test\n");
    let calls = home.root.join("hook-calls.txt");
    let hook = home.bin.join("commit-owner-hook");
    temp::write(
        &hook,
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >> \"${HOOK_CALLS:?}\"\n",
    );
    temp::make_executable(&hook);

    let poke = home
        .aw_command()
        .args(["commit", "poke"])
        .env("AW_COMMIT_POKE_PROGRAM", &hook)
        .env("AW_COMMIT_POKE_ARG", "adapter")
        .env("HOOK_CALLS", &calls)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("commit poke", &poke);
    assert_eq!(stdout(&poke), "Poked git with $x-commit next.");
    assert_eq!(read(&calls).trim(), "adapter $x-commit next");

    let queue = home.root.join("poke-queue");
    let request = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Poked docs",
            "README.md",
            "--queue-root",
        ])
        .arg(&queue)
        .args(["--poke"])
        .env("AW_COMMIT_POKE_PROGRAM", &hook)
        .env("HOOK_CALLS", &calls)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("request and poke", &request);
    assert!(stdout(&request).contains("Poked git with $x-commit next --root "));
    assert!(read(&calls).contains("$x-commit next --root"));

    let no_hook = home
        .aw_command()
        .args(["commit", "setup"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("setup without hook", &no_hook);
    assert!(stderr(&no_hook).contains("Shelly commit owner was not reached"));

    let custom_tab = home
        .aw_command()
        .args(["commit", "poke", "other"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("custom tab", &custom_tab);
    assert!(stderr(&custom_tab).contains("stable git launch id"));
}

#[test]
fn disabled_commit_owner_is_explicit_and_does_not_create_tickets() {
    let home = installed_home("commit-disabled");
    let project = home.root.join("project");
    temp::write(
        project.join("config/aw/profile.conf"),
        "name=test\nroot=.\ncommit_owner=disabled\n",
    );
    temp::write(project.join("README.md"), "# Disabled\n");
    let queue = home.root.join("disabled-queue");

    let status = home
        .aw_command()
        .args(["commit", "status", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("disabled status", &status);
    assert!(stdout(&status).contains("Status   disabled"));
    assert!(stdout(&status).contains("Next     direct git workflow"));

    let request = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Disabled request",
            "README.md",
            "--root",
        ])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("disabled request", &request);
    assert!(stderr(&request).contains("use the direct git workflow"));
    assert!(!queue.exists());
}
