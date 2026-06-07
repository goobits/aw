use std::fs;
use std::path::Path;

use crate::error::{AwError, Result};

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
    output.push_str(
        "    default_tab_template {\n        pane size=1 borderless=true {\n            plugin location=\"zellij:tab-bar\"\n        }\n        children\n    }\n\n",
    );

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
