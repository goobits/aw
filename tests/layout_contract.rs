mod support;

use support::command::{assert_success, stdout};
use support::temp::{self, TempDir};

fn rendered_tab_names(rendered: &str) -> Vec<String> {
    rendered
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("tab name=\""))
        .filter_map(|line| line.split('"').next())
        .map(str::to_string)
        .collect()
}

#[test]
fn render_layout_preserves_tab_order_and_tab_specific_cwd() {
    let output = std::process::Command::new(support::command::aw())
        .args([
            "zellij-render-layout",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/basic-website/default.tabs"
            ),
            "/workspace",
        ])
        .output()
        .expect("render example layout");
    assert_success("render example layout", &output);
    assert_eq!(
        rendered_tab_names(&stdout(&output)),
        vec!["editor", "server", "database", "logs", "scratch", "git"]
    );

    let tmp = TempDir::new("layout-contract");
    let tabs = tmp.join("cwd.tabs");
    temp::write(
        &tabs,
        "app\nelectron\t/workspace/apps/sketchpad/distributions/electronApp\n",
    );
    let output = std::process::Command::new(support::command::aw())
        .arg("zellij-render-layout")
        .arg(&tabs)
        .arg("/workspace")
        .output()
        .expect("render cwd layout");
    assert_success("render cwd layout", &output);
    assert!(stdout(&output)
        .contains("pane cwd=\"/workspace/apps/sketchpad/distributions/electronApp\""));
}
