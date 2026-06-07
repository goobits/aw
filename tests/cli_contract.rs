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
fn commit_add_rejects_missing_paths_before_queue_lookup() {
    let output = aw()
        .args(["commit", "add", "Missing paths"])
        .output()
        .expect("run aw commit add");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("commit add requires a title and at least one path"));
}
