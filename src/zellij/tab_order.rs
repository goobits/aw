use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::error::{AwError, Result};
use crate::paths::{home_dir, path_string, validate_name};
use crate::zellij::{base_name, value_to_string};

#[derive(Clone)]
struct DesiredTab {
    name: String,
    cwd: String,
}

pub fn session_tab_order(session: &str, specs: &[String]) -> Result<i32> {
    validate_session("zellij-session-tab-order", session)?;
    let desired = desired_tabs(specs);
    if desired.is_empty() {
        return Err(AwError::new(
            "usage: zellij-session-tab-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    write_fake_order_args(session, specs);
    saved_session_order(session, specs)?;
    if env::var("ZELLIJ_SESSION_TAB_ORDER_SAVED_ONLY").unwrap_or_default() == "1" {
        return Ok(0);
    }
    env::set_var("ZELLIJ_LIVE_TAB_ORDER_SAVE_SESSION", "0");
    live_tab_order(session, specs)?;
    saved_session_order(session, specs)
}

fn write_fake_order_args(session: &str, specs: &[String]) {
    if let Ok(path) = env::var("FAKE_ZELLIJ_ORDER_ARGS") {
        let mut lines = Vec::with_capacity(specs.len() + 1);
        lines.push(session.to_string());
        lines.extend(specs.iter().cloned());
        let _ = fs::write(path, format!("{}\n", lines.join("\n")));
    }
}

pub fn live_tab_order(session: &str, specs: &[String]) -> Result<i32> {
    validate_session("zellij-live-tab-order", session)?;
    let desired = desired_tabs(specs);
    if desired.is_empty() {
        return Err(AwError::new(
            "usage: zellij-live-tab-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    if !session_exists(session) {
        return Ok(0);
    }
    let mut tab_state = list_tabs(session)?;
    if tab_state.is_empty() {
        return Ok(0);
    }
    let active_id = tab_state
        .iter()
        .find(|tab| tab.get("active").and_then(Value::as_bool).unwrap_or(false))
        .and_then(tab_id)
        .unwrap_or_default();

    if env::var("ZELLIJ_SESSION_TAB_ORDER_STRICT").unwrap_or_default() == "1" {
        let desired_names: HashSet<String> = desired.iter().map(|tab| tab.name.clone()).collect();
        let mut seen = HashSet::new();
        for tab in &tab_state {
            let name = tab.get("name").and_then(Value::as_str).unwrap_or("");
            let base = base_name(name);
            let should_close = !desired_names.contains(&base) || !seen.insert(base);
            if should_close {
                if let Some(id) = tab_id(tab) {
                    let _ = zellij(session, &["action", "close-tab-by-id", &id]);
                }
            }
        }
        tab_state = list_tabs(session)?;
    }

    if env::var("ZELLIJ_SESSION_TAB_ORDER_CREATE_MISSING").unwrap_or_default() == "1" {
        for tab in &desired {
            if tab_state.iter().any(|item| {
                item.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| base_name(name) == tab.name)
            }) {
                continue;
            }
            let _ = zellij(
                session,
                &["action", "new-tab", "--name", &tab.name, "--cwd", &tab.cwd],
            );
            tab_state = list_tabs(session)?;
        }
    }

    close_status_bar_panes(session, &desired);

    let mut target_position = 0_i64;
    for tab in &desired {
        tab_state = list_tabs(session)?;
        let Some(row) = tab_state.iter().find(|item| {
            item.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| base_name(name) == tab.name)
        }) else {
            continue;
        };
        let id = tab_id(row).unwrap_or_default();
        let current_position = row
            .get("position")
            .and_then(Value::as_i64)
            .unwrap_or(target_position);
        if current_position > target_position {
            let _ = zellij(session, &["action", "go-to-tab-by-id", &id]);
            for _ in target_position..current_position {
                let _ = zellij(session, &["action", "move-tab", "left"]);
            }
        }
        target_position += 1;
    }
    if !active_id.is_empty() {
        let _ = zellij(session, &["action", "go-to-tab-by-id", &active_id]);
    }
    if env::var("ZELLIJ_LIVE_TAB_ORDER_SAVE_SESSION").unwrap_or_else(|_| "1".to_string()) == "1" {
        let _ = zellij(session, &["action", "save-session"]);
    }
    Ok(0)
}

pub fn saved_session_order(session: &str, specs: &[String]) -> Result<i32> {
    validate_session("zellij-saved-session-order", session)?;
    let desired = desired_tabs(specs);
    if desired.is_empty() {
        return Err(AwError::new(
            "usage: zellij-saved-session-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    let session_dir = cache_dir()
        .join("zellij/contract_version_1/session_info")
        .join(session);
    let layout_file = session_dir.join("session-layout.kdl");
    if layout_file.is_file() {
        let next = reorder_layout(&fs::read_to_string(&layout_file)?, &desired);
        fs::write(&layout_file, next)?;
    }
    let metadata_file = session_dir.join("session-metadata.kdl");
    if metadata_file.is_file() {
        let next = reorder_metadata(&fs::read_to_string(&metadata_file)?, &desired);
        fs::write(&metadata_file, next)?;
    }
    Ok(0)
}

fn desired_tabs(specs: &[String]) -> Vec<DesiredTab> {
    let default_cwd = env::var("ZELLIJ_SESSION_TAB_DEFAULT_CWD")
        .or_else(|_| {
            env::current_dir()
                .map(|path| path_string(&path))
                .map_err(|e| e.to_string())
        })
        .unwrap_or_else(|_| "/workspace".to_string());
    specs
        .iter()
        .filter_map(|spec| {
            let (name, cwd) = spec.split_once('\t').unwrap_or((spec, &default_cwd));
            if name.is_empty() {
                None
            } else {
                Some(DesiredTab {
                    name: name.to_string(),
                    cwd: cwd.to_string(),
                })
            }
        })
        .collect()
}

fn reorder_layout(contents: &str, desired: &[DesiredTab]) -> String {
    let rank: HashMap<String, usize> = desired
        .iter()
        .enumerate()
        .map(|(index, tab)| (tab.name.clone(), index))
        .collect();
    let strict = env::var("ZELLIJ_SESSION_TAB_ORDER_STRICT").unwrap_or_default() == "1";
    let lines = split_keep_newline(contents);
    let mut prefix = Vec::new();
    let mut blocks = Vec::<Block>::new();
    let mut suffix = Vec::new();
    let mut saw_tab = false;
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("    tab name=\"") {
            saw_tab = true;
            let mut block = Vec::new();
            let mut depth = 0_i32;
            loop {
                let line = lines[i].clone();
                depth += line.matches('{').count() as i32;
                depth -= line.matches('}').count() as i32;
                block.push(line);
                i += 1;
                if depth <= 0 || i >= lines.len() {
                    break;
                }
            }
            let name = block
                .first()
                .and_then(|line| quoted_value_after(line, "tab name="))
                .map(|name| base_name(&name))
                .unwrap_or_default();
            blocks.push(Block {
                name,
                text: block.join(""),
            });
        } else if saw_tab {
            suffix.push(lines[i].clone());
            i += 1;
        } else {
            prefix.push(lines[i].clone());
            i += 1;
        }
    }
    let ordered = order_blocks(blocks, desired, &rank, strict);
    format!("{}{}{}", prefix.join(""), ordered.join(""), suffix.join(""))
}

fn reorder_metadata(contents: &str, desired: &[DesiredTab]) -> String {
    let rank: HashMap<String, usize> = desired
        .iter()
        .enumerate()
        .map(|(index, tab)| (tab.name.clone(), index))
        .collect();
    let strict = env::var("ZELLIJ_SESSION_TAB_ORDER_STRICT").unwrap_or_default() == "1";
    let lines = split_keep_newline(contents);
    let mut prefix = Vec::new();
    let mut tabs_between = Vec::new();
    let mut panes_suffix = Vec::new();
    let mut tabs = Vec::<MetaTab>::new();
    let mut section = "prefix";
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("tabs {") {
            section = "tabs";
            prefix.push(lines[i].clone());
            i += 1;
        } else if lines[i].starts_with("panes {") {
            section = "panes";
            tabs_between.push(lines[i].clone());
            i += 1;
        } else if section == "tabs" && lines[i].starts_with("    tab {") {
            let mut block = Vec::new();
            let mut depth = 0_i32;
            loop {
                let line = lines[i].clone();
                depth += line.matches('{').count() as i32;
                depth -= line.matches('}').count() as i32;
                block.push(line);
                i += 1;
                if depth <= 0 || i >= lines.len() {
                    break;
                }
            }
            let text = block.join("");
            let name = line_value(&text, "        name \"")
                .map(|name| base_name(&name))
                .unwrap_or_default();
            let old_position = line_number_value(&text, "        position ");
            tabs.push(MetaTab {
                name,
                old_position,
                text,
            });
        } else if section == "prefix" {
            prefix.push(lines[i].clone());
            i += 1;
        } else if section == "tabs" {
            tabs_between.push(lines[i].clone());
            i += 1;
        } else {
            panes_suffix.push(lines[i].clone());
            i += 1;
        }
    }
    let mut seen = HashSet::new();
    let unique: Vec<MetaTab> = tabs
        .into_iter()
        .filter(|tab| seen.insert(tab.name.clone()))
        .collect();
    let mut known: Vec<MetaTab> = unique
        .iter()
        .filter(|tab| rank.contains_key(&tab.name))
        .cloned()
        .collect();
    known.sort_by_key(|tab| rank.get(&tab.name).copied().unwrap_or(usize::MAX));
    let mut ordered = known;
    if !strict {
        ordered.extend(
            unique
                .into_iter()
                .filter(|tab| !rank.contains_key(&tab.name)),
        );
    }
    let mut position_map = HashMap::<usize, usize>::new();
    let mut tab_text = String::new();
    for (new_position, mut tab) in ordered.into_iter().enumerate() {
        if let Some(old) = tab.old_position {
            position_map.insert(old, new_position);
        }
        tab.text = replace_line_number(&tab.text, "        position ", new_position);
        if let Some(rank_index) = rank.get(&tab.name) {
            tab.text = replace_line_quoted(&tab.text, "        name ", &desired[*rank_index].name);
        }
        tab_text.push_str(&tab.text);
    }
    let panes = rewrite_pane_positions(&panes_suffix.join(""), &position_map);
    format!(
        "{}{}{}{}",
        prefix.join(""),
        tab_text,
        tabs_between.join(""),
        panes
    )
}

#[derive(Clone)]
struct Block {
    name: String,
    text: String,
}

#[derive(Clone)]
struct MetaTab {
    name: String,
    old_position: Option<usize>,
    text: String,
}

fn order_blocks(
    blocks: Vec<Block>,
    desired: &[DesiredTab],
    rank: &HashMap<String, usize>,
    strict: bool,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let unique: Vec<Block> = blocks
        .into_iter()
        .filter(|block| seen.insert(block.name.clone()))
        .collect();
    let mut known: Vec<Block> = unique
        .iter()
        .filter(|block| rank.contains_key(&block.name))
        .cloned()
        .collect();
    known.sort_by_key(|block| rank.get(&block.name).copied().unwrap_or(usize::MAX));
    let mut out = Vec::new();
    for mut block in known {
        if let Some(rank_index) = rank.get(&block.name) {
            block.text =
                replace_line_quoted(&block.text, "    tab name=", &desired[*rank_index].name);
        }
        out.push(block.text);
    }
    if !strict {
        out.extend(
            unique
                .into_iter()
                .filter(|block| !rank.contains_key(&block.name))
                .map(|block| block.text),
        );
    }
    out
}

fn rewrite_pane_positions(contents: &str, position_map: &HashMap<usize, usize>) -> String {
    let mut out = String::new();
    let mut skip_block = false;
    for line in split_keep_newline(contents) {
        if line.starts_with("    pane {") {
            skip_block = false;
        }
        if let Some(pos) = line_number_value(&line, "        tab_position ") {
            if let Some(next) = position_map.get(&pos) {
                out.push_str(&format!("        tab_position {}\n", next));
            } else {
                skip_block = true;
            }
            continue;
        }
        if !skip_block {
            out.push_str(&line);
        }
        if skip_block && line.starts_with("    }") {
            skip_block = false;
        }
    }
    out
}

fn list_tabs(session: &str) -> Result<Vec<Value>> {
    let output = Command::new("zellij")
        .env("ZELLIJ_SESSION_NAME", session)
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else {
        return Ok(Vec::new());
    };
    let Ok(Value::Array(tabs)) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Ok(Vec::new());
    };
    Ok(tabs)
}

fn close_status_bar_panes(session: &str, desired: &[DesiredTab]) {
    let desired_names: HashSet<String> = desired.iter().map(|tab| tab.name.clone()).collect();
    let output = Command::new("zellij")
        .env("ZELLIJ_SESSION_NAME", session)
        .args(["action", "list-panes", "--all", "--json"])
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else {
        return;
    };
    let Ok(Value::Array(panes)) = serde_json::from_slice::<Value>(&output.stdout) else {
        return;
    };
    for pane in panes {
        let is_plugin = pane
            .get("is_plugin")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let title = pane.get("title").and_then(Value::as_str).unwrap_or("");
        let tab_name = pane.get("tab_name").and_then(Value::as_str).unwrap_or("");
        if is_plugin && title == "zellij:status-bar" && desired_names.contains(&base_name(tab_name))
        {
            if let Some(id) = pane.get("id").map(value_to_string) {
                let pane_id = format!("plugin_{}", id);
                let _ = zellij(session, &["action", "close-pane", "--pane-id", &pane_id]);
            }
        }
    }
}

fn session_exists(session: &str) -> bool {
    Command::new("zellij")
        .args(["list-sessions", "--short", "--no-formatting"])
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == session)
        })
}

fn zellij(session: &str, args: &[&str]) -> Result<()> {
    let _ = Command::new("zellij")
        .env("ZELLIJ_SESSION_NAME", session)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

fn validate_session(command: &str, session: &str) -> Result<()> {
    validate_name("session", session).map_err(|_| {
        AwError::new(
            format!(
                "{}: session names may only use letters, numbers, dot, underscore, and dash",
                command
            ),
            2,
        )
    })
}

fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
}

fn tab_id(value: &Value) -> Option<String> {
    value.get("tab_id").map(value_to_string)
}

fn split_keep_newline(contents: &str) -> Vec<String> {
    contents
        .split_inclusive('\n')
        .map(str::to_string)
        .chain(if contents.ends_with('\n') {
            None
        } else {
            Some(String::new())
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn quoted_value_after(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)? + prefix.len();
    let rest = &line[start..];
    let first = rest.find('"')? + 1;
    let rest = &rest[first..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn line_value(text: &str, prefix: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix('"'))
            .map(str::to_string)
    })
}

fn line_number_value(text: &str, prefix: &str) -> Option<usize> {
    text.lines().find_map(|line| {
        line.strip_prefix(prefix)
            .and_then(|rest| rest.trim().parse::<usize>().ok())
    })
}

fn replace_line_number(text: &str, prefix: &str, value: usize) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with(prefix) {
                format!("{}{}", prefix, value)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn replace_line_quoted(text: &str, prefix: &str, value: &str) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with(prefix) {
                format!(
                    "{}\"{}\"{}",
                    prefix,
                    crate::layout::kdl_escape(value),
                    line.split('"').skip(2).collect::<Vec<_>>().join("\"")
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
