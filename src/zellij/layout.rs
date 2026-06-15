use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AwError, Result};
use crate::paths::aw_plugins_dir;
use crate::profile::profile_value;

pub fn kdl_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn render_layout(tabs_file: &Path, workdir: &str) -> Result<String> {
    if !tabs_file.is_file() {
        return Err(AwError::new(
            format!(
                "zellij-render-layout: missing tabs file {}",
                tabs_file.display()
            ),
            1,
        ));
    }

    let mut output = String::new();
    output.push_str("layout {\n");
    output.push_str(&format!("    cwd \"{}\"\n\n", kdl_escape(workdir)));
    output.push_str(&tab_bar_template(tabs_file));

    let mut first = true;
    for line in fs::read_to_string(tabs_file)?.lines() {
        if line.is_empty() {
            continue;
        }
        let (tab_name, tab_cwd) = line.split_once('\t').unwrap_or((line, ""));
        if tab_name.is_empty() {
            continue;
        }
        if first {
            output.push_str(&format!(
                "    tab name=\"{}\" focus=true {{\n",
                kdl_escape(tab_name)
            ));
            first = false;
        } else {
            output.push_str(&format!("    tab name=\"{}\" {{\n", kdl_escape(tab_name)));
        }
        if tab_cwd.is_empty() {
            output.push_str("        pane\n");
        } else {
            output.push_str(&format!("        pane cwd=\"{}\"\n", kdl_escape(tab_cwd)));
        }
        output.push_str("    }\n\n");
    }
    output.push_str("}\n");
    Ok(output)
}

fn tab_bar_template(tabs_file: &Path) -> String {
    let profile_file = tabs_file
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join("profile.conf");
    let profile_tab_bar = profile_value(&profile_file, "tab_bar", "zellij");
    if matches!(profile_tab_bar.as_str(), "aw" | "aw-tab-bar") {
        if let Some(plugin_path) = aw_tab_bar_plugin_path() {
            let workspace = tabs_file
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("workspace");
            return format!(
                "    default_tab_template {{\n        pane size=1 borderless=true {{\n            plugin location=\"file:{}\" _allow_exec_host_cmd=true {{\n                workspace \"{}\"\n                aw \"aw\"\n            }}\n        }}\n        children\n    }}\n\n",
                kdl_escape(&plugin_path.to_string_lossy()),
                kdl_escape(workspace)
            );
        }
    }

    "    default_tab_template {\n        pane size=1 borderless=true {\n            plugin location=\"zellij:tab-bar\"\n        }\n        children\n    }\n\n".to_string()
}

fn aw_tab_bar_plugin_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("AW_TAB_BAR_PLUGIN_PATH")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        if path.is_file() {
            return Some(path);
        }
    }

    let installed = aw_plugins_dir().join("aw-tab-bar.wasm");
    if installed.is_file() {
        return Some(installed);
    }
    None
}
