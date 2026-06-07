mod support;

use std::process::Command;

use support::command::{assert_failure, assert_success, stdout};
use support::temp::{self, TempDir};

#[test]
fn cleanup_generated_reports_and_deletes_explicit_categories() {
    let repo = TempDir::new("workspace-cleanup");
    std::fs::create_dir_all(repo.join(".turbo")).unwrap();
    std::fs::create_dir_all(repo.join("packages/app/.svelte-kit")).unwrap();
    std::fs::create_dir_all(repo.join("packages/app/node_modules")).unwrap();
    temp::write(
        repo.join("crate/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n",
    );
    std::fs::create_dir_all(repo.join("crate/target")).unwrap();

    let dry = Command::new(support::command::aw())
        .args(["workspace", "cleanup-generated"])
        .current_dir(repo.path())
        .output()
        .expect("cleanup dry");
    assert_success("cleanup dry", &dry);
    let text = stdout(&dry);
    assert!(text.contains("keep\tgenerated-cache\t.turbo"));
    assert!(text.contains("keep\tnested-node_modules\tpackages/app/node_modules"));
    assert!(repo.join(".turbo").exists());

    let delete = Command::new(support::command::aw())
        .args(["workspace", "cleanup-generated", "--generated", "--delete"])
        .current_dir(repo.path())
        .output()
        .expect("cleanup delete");
    assert_success("cleanup delete", &delete);
    assert!(!repo.join(".turbo").exists());
    assert!(!repo.join("packages/app/.svelte-kit").exists());
    assert!(repo.join("packages/app/node_modules").exists());
}

#[test]
fn cleanup_generated_rejects_unknown_options() {
    let repo = TempDir::new("workspace-cleanup-bad");
    let output = Command::new(support::command::aw())
        .args(["workspace", "cleanup-generated", "--surprise"])
        .current_dir(repo.path())
        .output()
        .expect("cleanup bad");
    assert_failure("cleanup bad", &output);
}

#[test]
fn brush_api_worktree_help_is_native() {
    let output = Command::new(support::command::aw())
        .args(["brush-api", "worktree", "--help"])
        .output()
        .expect("brush help");
    assert_success("brush help", &output);
    assert!(stdout(&output).contains("Usage: aw brush-api worktree"));
}
