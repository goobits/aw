mod support;

use std::process::Command;

fn aw() -> Command {
    Command::new(support::command::aw())
}

#[test]
fn help_describes_shelly_coordination_surface() {
    let output = aw().arg("help").output().expect("run aw help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("\n🌀 aw: Agent Workspace coordination for Shelly"));
    assert!(stdout.contains("aw commit setup [--tab git]"));
    assert!(!stdout.to_ascii_lowercase().contains("zellij"));
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
    assert!(stdout.contains("\u{1b}[38;2;255;121;198mcommit queue:"));
}

#[test]
fn namespace_help_and_errors_are_scoped() {
    let commit = aw().args(["commit", "--help"]).output().unwrap();
    assert!(commit.status.success());
    let commit_stdout = String::from_utf8_lossy(&commit.stdout);
    assert!(commit_stdout.contains("aw commit request <title> <path>..."));
    assert!(!commit_stdout.contains("repo maintenance:"));

    for (args, expected) in [
        (vec!["owner"], "aw: owner requires git or pkg"),
        (vec!["repo", "bogus"], "aw: unknown repo command bogus"),
        (vec!["commit", "bogus"], "aw: unknown commit action bogus"),
        (
            vec!["install", "--surprise"],
            "aw: unknown install argument --surprise",
        ),
        (
            vec!["paths", "extra"],
            "aw: paths does not accept arguments",
        ),
        (vec!["ps"], "aw: unknown command ps"),
    ] {
        let output = aw().args(args).output().expect("run aw error case");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn paths_reports_the_small_installed_surface() {
    let output = aw().arg("paths").output().expect("run aw paths");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AW Paths"));
    assert!(stdout.contains("Completions"));
    assert!(stdout.contains("Public bin"));
    assert!(!stdout.contains("Plugins"));
}
