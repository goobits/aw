mod support;

use support::command::{assert_failure, assert_success, stdout, TestHome};
use support::fake_zellij;
use support::temp::{self, read};

fn installed_home(name: &str) -> TestHome {
    let home = fake_zellij::installed_home(name);
    temp::write(
        home.bin.join("sleep"),
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >> \"${FAKE_SLEEP_CALLS:?}\"\n",
    );
    temp::make_executable(home.bin.join("sleep"));
    home
}

fn expected_session(profile: &str, workspace: &str, root: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in root.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{profile}-{workspace}-{hash:016x}")
}

fn assert_order(path: impl AsRef<std::path::Path>, session: &str, tabs: &[&str]) {
    let expected = std::iter::once(session)
        .chain(tabs.iter().copied())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(read(path).trim_end(), expected);
}

fn assert_captured_sessions(path: impl AsRef<std::path::Path>, expected: &str) {
    let sessions = read(path);
    assert!(!sessions.trim().is_empty());
    assert!(sessions.lines().all(|line| line == expected), "{sessions}");
}

#[test]
fn commit_queue_commands_create_wait_report_and_validate_requests() {
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
            "--check",
            "echo ok",
            "--must-contain",
            "Commit queue test",
            "--queue-root",
        ])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .expect("commit request");
    assert_success("commit request", &output);
    assert!(stdout(&output).starts_with("Created commit request "));
    assert!(stdout(&output).contains("Run `aw commit poke --queue-root "));
    assert_eq!(
        std::fs::read_dir(queue.join("pending"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
            .count(),
        1
    );

    let wait_queue = home.root.join("wait-queue");
    let wait_add = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Wait docs",
            "README.md",
            "--queue-root",
        ])
        .arg(&wait_queue)
        .args(["--check", "echo ok"])
        .current_dir(&project)
        .output()
        .expect("wait request");
    assert_success("wait request", &wait_add);
    let wait_stdout = stdout(&wait_add);
    let wait_id = wait_stdout
        .split_whitespace()
        .nth(3)
        .unwrap()
        .trim_end_matches('.')
        .to_string();
    assert_success(
        "commit done",
        &home
            .aw_command()
            .args(["commit", "done", &wait_id, "--root"])
            .arg(&wait_queue)
            .args([
                "--commit",
                "abc123",
                "--message",
                "Wait docs committed",
                "--verify-result",
                "passed",
                "--note",
                "owner committed",
            ])
            .output()
            .unwrap(),
    );
    let wait_output = home
        .aw_command()
        .args(["commit", "wait", &wait_id, "--root"])
        .arg(&wait_queue)
        .args(["--timeout", "1s"])
        .output()
        .expect("commit wait");
    assert_success("commit wait", &wait_output);
    let wait_text = stdout(&wait_output);
    assert!(wait_text.contains("Done\nRequest"));
    assert!(wait_text.contains("Commit   abc123"));
    assert!(wait_text.contains("Verify   passed"));

    let timed_out = home
        .aw_command()
        .args(["commit", "request", "Wait timeout", "README.md", "--root"])
        .arg(home.root.join("wait-timeout-queue"))
        .args(["--wait", "--wait", "--timeout", "1ms", "--poll", "1ms"])
        .current_dir(&project)
        .output()
        .expect("wait timeout");
    assert_failure("wait timeout", &timed_out);
    assert!(stdout(&timed_out).contains("Timeout"));

    assert_failure(
        "missing paths",
        &home
            .aw_command()
            .args(["commit", "request", "Missing paths", "--root"])
            .arg(&queue)
            .current_dir(&project)
            .output()
            .unwrap(),
    );
    let bad_summary = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Bad flag value",
            "README.md",
            "--summary",
            "--root",
        ])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad summary", &bad_summary);
    assert!(support::command::stderr(&bad_summary).contains("--summary requires a value"));
    let bad_status = home
        .aw_command()
        .args(["commit", "status", "--bogus"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad status", &bad_status);
    assert!(
        support::command::stderr(&bad_status).contains("unknown commit status argument --bogus")
    );
    let bad_wait = home
        .aw_command()
        .args(["commit", "wait"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad wait", &bad_wait);
    let bad_wait_stderr = support::command::stderr(&bad_wait);
    assert!(bad_wait_stderr.contains("aw: commit wait requires a request id"));
    assert!(bad_wait_stderr.contains("aw commit wait <id>"));
    assert!(!bad_wait_stderr.contains("commitq:"));
    let bad_done = home
        .aw_command()
        .args(["commit", "done"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad done", &bad_done);
    assert!(support::command::stderr(&bad_done).contains("aw: commit done requires a request id"));
    let extra_done = home
        .aw_command()
        .args(["commit", "done", "one", "two"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("extra done", &extra_done);
    let extra_done_stderr = support::command::stderr(&extra_done);
    assert!(extra_done_stderr.contains("aw: commit done got an extra argument: two"));
    assert!(!extra_done_stderr.contains("commitq:"));
    let extra_wait = home
        .aw_command()
        .args(["commit", "wait", "one", "two"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("extra wait", &extra_wait);
    let extra_wait_stderr = support::command::stderr(&extra_wait);
    assert!(extra_wait_stderr.contains("aw: commit wait got an extra argument: two"));
    assert!(!extra_wait_stderr.contains("commitq:"));
    let bad_block = home
        .aw_command()
        .args(["commit", "block", "missing"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad block", &bad_block);
    assert!(support::command::stderr(&bad_block).contains("commit block requires --reason"));

    assert_success(
        "commit check",
        &home
            .aw_command()
            .args(["commit", "check", "--root"])
            .arg(&queue)
            .current_dir(&project)
            .output()
            .unwrap(),
    );
    let status = home
        .aw_command()
        .args(["commit", "status", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("commit status", &status);
    assert!(stdout(&status).contains("Pending  1"));
    assert!(stdout(&status).contains("Next     "));

    assert_success(
        "overlap request",
        &home
            .aw_command()
            .args([
                "commit",
                "request",
                "--title",
                "Overlapping docs",
                "--path",
                "README.md",
                "--root",
            ])
            .arg(&queue)
            .current_dir(&project)
            .output()
            .unwrap(),
    );
    let blocked = home
        .aw_command()
        .args(["commit", "status", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("blocked status", &blocked);
    assert!(stdout(&blocked).contains("Next     blocked"));
    let doctor = home
        .aw_command()
        .args(["commit", "doctor", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("commit doctor", &doctor);
    assert!(stdout(&doctor).contains("Status    blocked"));
    assert!(stdout(&doctor).contains("queue has unsafe overlaps or invalid tickets"));
}

#[test]
fn commit_doctor_surfaces_blocked_tickets_when_queue_is_ready() {
    let home = installed_home("commit-doctor-blocked");
    let project = home.root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    temp::write(project.join("README.md"), "# Commit queue test\n");
    temp::write(project.join("NEXT.md"), "# Next\n");
    let queue = home.root.join("commit-queue");

    let blocked_add = home
        .aw_command()
        .args(["commit", "request", "Blocked docs", "README.md", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .expect("blocked add");
    assert_success("blocked add", &blocked_add);
    let blocked_id = stdout(&blocked_add)
        .split_whitespace()
        .nth(3)
        .unwrap()
        .trim_end_matches('.')
        .to_string();

    assert_success(
        "commit block",
        &home
            .aw_command()
            .args(["commit", "block", &blocked_id, "--root"])
            .arg(&queue)
            .args(["--reason", "waiting for owner"])
            .current_dir(&project)
            .output()
            .unwrap(),
    );

    assert_success(
        "next add",
        &home
            .aw_command()
            .args(["commit", "request", "Next docs", "NEXT.md", "--root"])
            .arg(&queue)
            .current_dir(&project)
            .output()
            .unwrap(),
    );

    let doctor = home
        .aw_command()
        .args(["commit", "doctor", "--root"])
        .arg(&queue)
        .current_dir(&project)
        .output()
        .expect("doctor");
    assert_success("doctor", &doctor);
    let text = stdout(&doctor);
    assert!(text.contains("Status    ready"));
    assert!(text.contains("[triage] 1 blocked ticket(s) still need reconciliation."));
    assert!(text.contains("reconcile blocked tickets before calling the queue fully clean"));
}

#[test]
fn commit_poke_and_setup_target_git_tab_without_switching_active_tab() {
    let home = installed_home("commit-poke");
    let project = home.root.join("project");
    let profile = project.join("config/aw");
    std::fs::create_dir_all(&profile).unwrap();
    temp::write(project.join("README.md"), "# Commit queue test\n");
    temp::write(
        profile.join("profile.conf"),
        "name=my-site\nroot=/tmp/project\ndefault_workspace=front\ndefault_workspaces=front\n",
    );
    temp::write(
        profile.join("front.tabs"),
        "tools\nsearch\ncomponents\nskills\nscratch\n",
    );
    temp::write(profile.join("backend.tabs"), "api\ngit\nscratch\n");
    assert_success(
        "setup profile",
        &home
            .aw_command()
            .args(["setup", "--config"])
            .arg(&profile)
            .output()
            .unwrap(),
    );

    let tabs = home.root.join("tabs.tsv");
    temp::write(&tabs, "0\t0\ttrue\ttools\n1\t1\tfalse\tgit 🤖\n");
    temp::write(tabs.with_extension("tsv.panes"), "");
    for file in [
        "written-chars.txt",
        "written-panes.txt",
        "key-panes.txt",
        "sent-keys.txt",
        "sleep-calls.txt",
        "session-names.txt",
    ] {
        temp::write(home.root.join(file), "");
    }
    let front_session = expected_session("my-site", "front", &project.display().to_string());
    let output = home
        .aw_command()
        .args(["commit", "poke", "git"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("written-chars.txt"),
        )
        .env(
            "FAKE_ZELLIJ_WRITTEN_PANES",
            home.root.join("written-panes.txt"),
        )
        .env("FAKE_ZELLIJ_KEY_PANES", home.root.join("key-panes.txt"))
        .env("FAKE_ZELLIJ_SENT_KEYS", home.root.join("sent-keys.txt"))
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .env(
            "FAKE_ZELLIJ_SESSION_NAMES",
            home.root.join("session-names.txt"),
        )
        .current_dir(&project)
        .output()
        .expect("commit poke");
    assert_success("commit poke", &output);
    assert_eq!(stdout(&output), "Poked git with $x-commit next.");
    assert_eq!(read(home.root.join("written-chars.txt")), "$x-commit next");
    assert_eq!(read(home.root.join("written-panes.txt")).trim_end(), "1");
    assert_eq!(read(home.root.join("key-panes.txt")).trim_end(), "1");
    assert_eq!(read(home.root.join("sent-keys.txt")).trim_end(), "Enter");
    assert_eq!(read(home.root.join("sleep-calls.txt")).trim_end(), "0.4");
    assert_captured_sessions(home.root.join("session-names.txt"), &front_session);
    assert!(read(&tabs).contains("0\t0\ttrue\ttools"));

    for file in [
        "setup-written-chars.txt",
        "setup-written-panes.txt",
        "setup-key-panes.txt",
        "setup-sent-keys.txt",
    ] {
        temp::write(home.root.join(file), "");
    }
    temp::write(&tabs, "0\t0\ttrue\ttools\n1\t1\tfalse\tgit\n");
    let setup = home
        .aw_command()
        .args(["commit", "setup", "front"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-commit-setup-order.txt"),
        )
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("setup-written-chars.txt"),
        )
        .env(
            "FAKE_ZELLIJ_WRITTEN_PANES",
            home.root.join("setup-written-panes.txt"),
        )
        .env(
            "FAKE_ZELLIJ_KEY_PANES",
            home.root.join("setup-key-panes.txt"),
        )
        .env(
            "FAKE_ZELLIJ_SENT_KEYS",
            home.root.join("setup-sent-keys.txt"),
        )
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .current_dir(&project)
        .output()
        .expect("commit setup");
    assert_success("commit setup", &setup);
    assert_eq!(
        stdout(&setup),
        format!(
            "Commit tab git is ready in {} and received `codex`.",
            front_session
        )
    );
    assert_eq!(
        read(profile.join("front.tabs")).lines().last().unwrap(),
        "git"
    );
    assert_order(
        home.root.join("front-commit-setup-order.txt"),
        &front_session,
        &["tools", "search", "components", "skills", "scratch", "git"],
    );
    assert_eq!(read(home.root.join("setup-written-chars.txt")), "codex");
    assert_eq!(
        read(home.root.join("setup-written-panes.txt")).trim_end(),
        "1"
    );
    assert_eq!(read(home.root.join("setup-key-panes.txt")).trim_end(), "1");
    assert_eq!(
        read(home.root.join("setup-sent-keys.txt")).trim_end(),
        "Enter"
    );

    let custom = home
        .aw_command()
        .args(["commit", "setup", "front", "--session", "sketch-api"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-commit-setup-order.txt"),
        )
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("setup-written-chars.txt"),
        )
        .env(
            "FAKE_ZELLIJ_WRITTEN_PANES",
            home.root.join("setup-written-panes.txt"),
        )
        .env(
            "FAKE_ZELLIJ_KEY_PANES",
            home.root.join("setup-key-panes.txt"),
        )
        .env(
            "FAKE_ZELLIJ_SENT_KEYS",
            home.root.join("setup-sent-keys.txt"),
        )
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .current_dir(&project)
        .output()
        .expect("custom setup");
    assert_success("custom setup", &custom);
    assert_eq!(
        stdout(&custom),
        "Commit tab git is ready in sketch-api and received `codex`."
    );
    assert!(read(home.root.join("front-commit-setup-order.txt")).starts_with("sketch-api\n"));

    let no_agent = home
        .aw_command()
        .args([
            "commit",
            "setup",
            "front",
            "--tab",
            "git",
            "--session",
            "sketch-api",
            "--no-agent",
        ])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_ORDER_ARGS",
            home.root.join("front-commit-setup-order.txt"),
        )
        .current_dir(&project)
        .output()
        .expect("no agent setup");
    assert_success("no agent setup", &no_agent);
    assert_eq!(stdout(&no_agent), "Commit tab git is ready in sketch-api.");

    temp::write(home.root.join("session-names.txt"), "");
    let explicit_poke = home
        .aw_command()
        .args(["commit", "poke", "git", "--session", "sketch-api"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("written-chars.txt"),
        )
        .env("FAKE_ZELLIJ_SENT_KEYS", home.root.join("sent-keys.txt"))
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .env(
            "FAKE_ZELLIJ_SESSION_NAMES",
            home.root.join("session-names.txt"),
        )
        .current_dir(&project)
        .output()
        .expect("explicit session poke");
    assert_success("explicit session poke", &explicit_poke);
    assert_eq!(stdout(&explicit_poke), "Poked git with $x-commit next.");
    assert_captured_sessions(home.root.join("session-names.txt"), "sketch-api");

    temp::write(home.root.join("session-names.txt"), "");
    let backend_session = expected_session("my-site", "backend", &project.display().to_string());
    let workspace_poke = home
        .aw_command()
        .args(["commit", "poke", "git", "--workspace", "backend"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("written-chars.txt"),
        )
        .env("FAKE_ZELLIJ_SENT_KEYS", home.root.join("sent-keys.txt"))
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .env(
            "FAKE_ZELLIJ_SESSION_NAMES",
            home.root.join("session-names.txt"),
        )
        .current_dir(&project)
        .output()
        .expect("workspace session poke");
    assert_success("workspace session poke", &workspace_poke);
    assert_eq!(stdout(&workspace_poke), "Poked git with $x-commit next.");
    assert_captured_sessions(home.root.join("session-names.txt"), &backend_session);

    temp::write(home.root.join("session-names.txt"), "");
    temp::write(home.root.join("written-chars.txt"), "");
    let poke_queue = home.root.join("poke-queue");
    let request_poke = home
        .aw_command()
        .args([
            "commit",
            "request",
            "Poked docs",
            "README.md",
            "--queue-root",
        ])
        .arg(&poke_queue)
        .args(["--poke", "git"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("written-chars.txt"),
        )
        .env("FAKE_ZELLIJ_SENT_KEYS", home.root.join("sent-keys.txt"))
        .env("FAKE_SLEEP_CALLS", home.root.join("sleep-calls.txt"))
        .env(
            "FAKE_ZELLIJ_SESSION_NAMES",
            home.root.join("session-names.txt"),
        )
        .current_dir(&project)
        .output()
        .expect("request poke");
    assert_success("request poke", &request_poke);
    assert!(stdout(&request_poke).starts_with("Created commit request "));
    assert!(stdout(&request_poke).contains("Poked git with $x-commit next --root "));
    assert_captured_sessions(home.root.join("session-names.txt"), &front_session);

    assert_failure(
        "missing setup tab",
        &home
            .aw_command()
            .args(["commit", "setup", "front", "--tab"])
            .current_dir(&project)
            .output()
            .unwrap(),
    );
    let bad_agent = home
        .aw_command()
        .args(["commit", "setup", "front", "--agent", "--no-agent"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_failure("bad setup agent", &bad_agent);
    assert!(support::command::stderr(&bad_agent).contains("--agent requires a value"));

    let missing = home
        .aw_command()
        .args(["commit", "poke", "Missing"])
        .env("FAKE_ZELLIJ_TABS", &tabs)
        .env(
            "FAKE_ZELLIJ_WRITTEN_CHARS",
            home.root.join("written-chars.txt"),
        )
        .env("FAKE_ZELLIJ_SENT_KEYS", home.root.join("sent-keys.txt"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert_success("missing poke", &missing);
    assert_eq!(
        stdout(&missing),
        "No live Zellij tab named Missing was found to poke."
    );
}
