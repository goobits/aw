mod support;

use std::process::Command;

fn aw() -> Command {
    Command::new(support::command::aw())
}

#[test]
fn help_prints_public_cli_header_on_stdout() {
    let output = aw().arg("help").output().expect("run aw help");
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with("aw: Zero-friction Zellij workspaces")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn help_supports_forced_dracula_color() {
    let output = aw()
        .arg("help")
        .env("AW_COLOR", "always")
        .output()
        .expect("run aw help with color");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\u{1b}[38;2;189;147;249m"));
    assert!(stdout.contains("\u{1b}[38;2;255;121;198mworkspaces:"));
    assert!(stdout.contains("aw tab rename <old-tab> <new-tab[@index]>"));
}

#[test]
fn commit_request_rejects_missing_paths_before_queue_lookup() {
    let output = aw()
        .args(["commit", "request", "Missing paths"])
        .output()
        .expect("run aw commit request");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("commit request requires a title and at least one path"));
    assert!(stderr.contains("aw commit request <title> <path>..."));
    assert!(!stderr.contains("aw: Zero-friction Zellij workspaces"));
}

#[test]
fn namespace_help_is_scoped() {
    let commit = aw()
        .args(["commit", "--help"])
        .output()
        .expect("run aw commit --help");
    assert!(commit.status.success());
    let commit_stdout = String::from_utf8_lossy(&commit.stdout);
    assert!(commit_stdout.contains("aw commit request <title> <path>..."));
    assert!(!commit_stdout.contains("workspaces:"));

    let repo = aw()
        .args(["repo", "--help"])
        .output()
        .expect("run aw repo --help");
    assert!(repo.status.success());
    let repo_stdout = String::from_utf8_lossy(&repo.stdout);
    assert!(repo_stdout.contains("aw repo routes [doctor]"));
    assert!(repo_stdout.contains("aw repo worktree <path>"));
    assert!(!repo_stdout.contains("commit queue:"));
}

#[test]
fn namespace_errors_are_scoped() {
    for (args, expected, unexpected) in [
        (
            vec!["owner"],
            "aw: owner requires git or pkg",
            "aw: Zero-friction Zellij workspaces",
        ),
        (
            vec!["repo", "bogus"],
            "aw: unknown repo command bogus",
            "commit queue:",
        ),
        (
            vec!["commit", "bogus"],
            "aw: unknown commit action bogus",
            "workspaces:",
        ),
        (
            vec!["install", "--surprise"],
            "aw: unknown install argument --surprise",
            "commit queue:",
        ),
        (
            vec!["paths", "extra"],
            "aw: paths does not accept arguments",
            "workspaces:",
        ),
        (
            vec!["ps", "extra"],
            "aw: ps does not accept arguments",
            "commit queue:",
        ),
    ] {
        let output = aw().args(args).output().expect("run aw error case");
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "{stderr}");
        assert!(!stderr.contains(unexpected), "{stderr}");
    }
}

#[test]
fn paths_reports_aw_home_layout() {
    let output = aw().arg("paths").output().expect("run aw paths");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AW Paths"));
    assert!(stdout.contains(".aw"));
    assert!(stdout.contains("Plugins"));
}
