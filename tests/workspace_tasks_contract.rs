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
        .args(["repo", "clean"])
        .current_dir(repo.path())
        .output()
        .expect("cleanup dry");
    assert_success("cleanup dry", &dry);
    let text = stdout(&dry);
    assert!(text.contains("keep\tgenerated-cache\t.turbo"));
    assert!(text.contains("keep\tnested-node_modules\tpackages/app/node_modules"));
    assert!(repo.join(".turbo").exists());

    let delete = Command::new(support::command::aw())
        .args(["repo", "clean", "--generated", "--delete"])
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
        .args(["repo", "clean", "--surprise"])
        .current_dir(repo.path())
        .output()
        .expect("cleanup bad");
    assert_failure("cleanup bad", &output);
}

#[test]
fn repo_routes_lists_configured_local_hosts() {
    let repo = TempDir::new("workspace-routes");
    temp::write(
        repo.join("config/aw/routes.conf"),
        "\
# role = public URLs served by the repo
main=http://localhost:3240
dev=http://dev.localhost:3240 http://dev.localtest.me:3240
prod=http://prod.localhost:3240 http://prod.localtest.me:3240
",
    );

    let output = Command::new(support::command::aw())
        .args(["repo", "routes"])
        .current_dir(repo.path())
        .output()
        .expect("routes list");

    assert_success("routes list", &output);
    let text = stdout(&output);
    assert!(text.contains("main\thttp://localhost:3240"));
    assert!(text.contains("dev\thttp://dev.localhost:3240 http://dev.localtest.me:3240"));
    assert!(text.contains("prod\thttp://prod.localhost:3240 http://prod.localtest.me:3240"));
}

#[test]
fn repo_routes_doctor_validates_configured_local_hosts() {
    let repo = TempDir::new("workspace-routes-doctor");
    temp::write(
        repo.join("config/aw/routes.conf"),
        "\
main=http://localhost:3240
dev=http://dev.localhost:3240
prod=http://prod.localhost:3240
",
    );

    let output = Command::new(support::command::aw())
        .args(["repo", "routes", "doctor"])
        .current_dir(repo.path())
        .output()
        .expect("routes doctor");

    assert_success("routes doctor", &output);
    let text = stdout(&output);
    assert!(text.contains("ok      config/aw/routes.conf"));
    assert!(text.contains("ok      route dev http://dev.localhost:3240"));
    assert!(text.contains("ok      repo routes ready"));
}

#[test]
fn repo_routes_rejects_bad_endpoints() {
    let repo = TempDir::new("workspace-routes-bad");
    temp::write(
        repo.join("config/aw/routes.conf"),
        "dev=dev.localhost:3240\n",
    );

    let output = Command::new(support::command::aw())
        .args(["repo", "routes"])
        .current_dir(repo.path())
        .output()
        .expect("routes bad");

    assert_failure("routes bad", &output);
}

#[test]
fn brush_api_worktree_help_is_native() {
    let output = Command::new(support::command::aw())
        .args(["repo", "worktree", "--help"])
        .output()
        .expect("brush help");
    assert_success("brush help", &output);
    assert!(stdout(&output).contains("Usage: aw repo worktree"));
}
