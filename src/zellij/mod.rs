pub(crate) mod helpers;
pub(crate) mod layout;
pub(crate) mod profile;
pub(crate) mod tab_order;
pub(crate) mod tabs;
pub(crate) mod watcher;

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::error::{AwError, Result};
use crate::paths::{helper_path, path_string, validate_name};
use crate::profile::{default_session_name_from_profile_dir, profile_value};
use crate::tab_order::session_tab_order;
use crate::tabs::{read_tab_lines, tab_name_from_line};

#[derive(Debug)]
struct LiveTab {
    pub position: i64,
    pub active: bool,
    pub base: String,
}

pub fn base_name(name: &str) -> String {
    name.strip_suffix(" 🤖")
        .or_else(|| name.strip_suffix(" 🔔"))
        .unwrap_or(name)
        .to_string()
}

pub fn sync_workspace_session(
    config_dir: &Path,
    workspace: &str,
    session_name: Option<&str>,
) -> Result<()> {
    sync_workspace_session_with_missing(config_dir, workspace, session_name, false)
}

pub fn sync_live_workspace_session(
    config_dir: &Path,
    workspace: &str,
    session_name: Option<&str>,
) -> Result<()> {
    sync_workspace_session_with_missing(config_dir, workspace, session_name, true)
}

fn sync_workspace_session_with_missing(
    config_dir: &Path,
    workspace: &str,
    session_name: Option<&str>,
    require_live_session: bool,
) -> Result<()> {
    let tabs_file = config_dir.join(format!("{}.tabs", workspace));
    let tab_order: Vec<String> = read_tab_lines(&tabs_file)?
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect();
    if tab_order.is_empty() {
        return Ok(());
    }

    let default_cwd = workspace_root(config_dir);

    let session = session_name
        .map(str::to_string)
        .unwrap_or_else(|| default_workspace_session_name(config_dir, workspace));

    if require_live_session {
        let live_sessions = live_session_names()?;
        if !live_sessions.iter().any(|name| name == &session) {
            return Err(missing_live_session_error(
                &session,
                workspace,
                &live_sessions,
            ));
        }
    }

    env::set_var("ZELLIJ_SESSION_TAB_DEFAULT_CWD", default_cwd);
    env::set_var("ZELLIJ_SESSION_TAB_ORDER_CREATE_MISSING", "1");
    env::set_var("ZELLIJ_SESSION_TAB_ORDER_STRICT", "1");
    session_tab_order(&session, &tab_order)?;
    Ok(())
}

pub fn default_workspace_session_name(config_dir: &Path, workspace: &str) -> String {
    default_session_name_from_profile_dir(config_dir, workspace)
}

pub fn rename_live_workspace_session(old_session: &str, new_session: &str) -> Result<()> {
    validate_name("session", old_session)?;
    validate_name("session", new_session)?;
    if old_session == new_session || !session_exists(old_session)? {
        return Ok(());
    }
    zellij_action(
        Some(old_session),
        &["action", "rename-session", new_session],
    )
}

pub(crate) fn session_exists(session: &str) -> Result<bool> {
    Ok(live_session_names()?.iter().any(|name| name == session))
}

pub(crate) fn live_session_names() -> Result<Vec<String>> {
    let output = match Command::new("zellij")
        .args(["list-sessions", "--short", "--no-formatting"])
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(_) => return Ok(Vec::new()),
    };
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect())
}

fn missing_live_session_error(session: &str, workspace: &str, live_sessions: &[String]) -> AwError {
    let mut message = format!(
        "aw: target Zellij session {} is not running; live tabs were not changed.",
        session
    );
    if !live_sessions.is_empty() {
        message.push_str("\nLive sessions: ");
        message.push_str(&live_sessions.join(", "));
    }
    if session != workspace && live_sessions.iter().any(|name| name == workspace) {
        message.push_str(&format!(
            "\nUse --session {} to sync that live session.",
            workspace
        ));
    } else {
        message.push_str(&format!(
            "\nStart the workspace with aw {}, or pass --session <name>.",
            workspace
        ));
    }
    AwError::new(message, 1)
}

fn workspace_root(config_dir: &Path) -> String {
    profile_value(
        &config_dir.join("profile.conf"),
        "root",
        &env::current_dir()
            .ok()
            .map(|path| path_string(&path))
            .unwrap_or_else(|| "/workspace".to_string()),
    )
}

fn live_tab_rows(session: &str, match_name: Option<&str>) -> Result<Vec<LiveTab>> {
    let mut command = Command::new("zellij");
    command
        .env("ZELLIJ_SESSION_NAME", session)
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(_) => return Ok(Vec::new()),
    };
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let Ok(Value::Array(items)) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Ok(Vec::new());
    };
    let mut rows = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let base = base_name(&name);
        if match_name.is_some_and(|wanted| wanted != base) {
            continue;
        }
        rows.push(LiveTab {
            position: item.get("position").and_then(Value::as_i64).unwrap_or(0),
            active: item.get("active").and_then(Value::as_bool).unwrap_or(false),
            base,
        });
    }
    rows.sort_by_key(|row| row.position);
    Ok(rows)
}

pub fn list_workspace_tabs(session: &str, tabs_file: &Path) -> Result<()> {
    let rows = live_tab_rows(session, None)?;
    if !rows.is_empty() {
        for row in rows {
            let marker = if row.active { "*" } else { " " };
            println!("{} {} {}", marker, row.position, row.base);
        }
        return Ok(());
    }

    for (index, line) in read_tab_lines(tabs_file)?.iter().enumerate() {
        println!("  {} {}", index, tab_name_from_line(line));
    }
    Ok(())
}

pub fn rename_live_workspace_tab(session: &str, old_name: &str, new_name: &str) -> Result<()> {
    validate_name("tab", old_name)?;
    validate_name("tab", new_name)?;
    let Some(tab_id) = tab_id_by_base_name(Some(session), old_name)? else {
        return Ok(());
    };
    zellij_action(
        Some(session),
        &["action", "rename-tab-by-id", &tab_id, new_name],
    )
}

fn tab_id_by_base_name(session: Option<&str>, requested_name: &str) -> Result<Option<String>> {
    validate_name("tab", requested_name)?;
    let mut command = Command::new("zellij");
    if let Some(session) = session {
        command.env("ZELLIJ_SESSION_NAME", session);
    }
    command
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }
    let Ok(Value::Array(items)) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Ok(None);
    };
    for item in items {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        if base_name(name) == requested_name {
            return Ok(item
                .get("tab_id")
                .or_else(|| item.get("id"))
                .map(value_to_string));
        }
    }
    Ok(None)
}

fn commit_tab_id(requested_name: &str, session: Option<&str>) -> Result<Option<String>> {
    tab_id_by_base_name(session, requested_name)
}

fn commit_tab_pane_id(requested_name: &str, session: Option<&str>) -> Result<Option<String>> {
    let mut command = Command::new("zellij");
    if let Some(session) = session {
        command.env("ZELLIJ_SESSION_NAME", session);
    }
    command
        .args(["action", "list-panes", "--json"])
        .stderr(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }
    let Ok(Value::Array(items)) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Ok(None);
    };
    for item in items {
        let tab_name = item.get("tab_name").and_then(Value::as_str).unwrap_or("");
        let is_plugin = item
            .get("is_plugin")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !is_plugin && base_name(tab_name) == requested_name {
            return Ok(item.get("id").map(value_to_string));
        }
    }
    Ok(None)
}

pub fn send_to_commit_tab(
    requested_name: &str,
    message: &str,
    session: Option<&str>,
    missing_message: Option<&str>,
) -> Result<bool> {
    validate_name("tab", requested_name)?;
    let Some(tab_id) = commit_tab_id(requested_name, session)? else {
        if let Some(message) = missing_message {
            println!("{}", message);
        } else {
            println!(
                "No live Zellij tab named {} was found to poke.",
                requested_name
            );
        }
        return Ok(false);
    };

    if let Some(pane_id) = commit_tab_pane_id(requested_name, session)? {
        zellij_action(
            session,
            &["action", "write-chars", "--pane-id", &pane_id, message],
        )?;
        submit_enter_to_commit_pane(Some(&pane_id), session)?;
        return Ok(true);
    }

    zellij_action(session, &["action", "go-to-tab-by-id", &tab_id])?;
    zellij_action(session, &["action", "write-chars", message])?;
    submit_enter_to_commit_pane(None, session)?;
    Ok(true)
}

pub fn zellij_passthrough(args: &[&str]) -> Result<i32> {
    let status = Command::new("zellij").args(args).status()?;
    Ok(status.code().unwrap_or(1))
}

pub fn run_helper(helper: &str, args: &[String]) -> Result<i32> {
    let status = Command::new(helper_path(helper)).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}

fn zellij_action(session: Option<&str>, args: &[&str]) -> Result<()> {
    let mut command = Command::new("zellij");
    if let Some(session) = session {
        command.env("ZELLIJ_SESSION_NAME", session);
    }
    let status = command.args(args).stdout(Stdio::null()).status()?;
    if !status.success() {
        return Err(AwError::new("aw: zellij action failed", 1));
    }
    Ok(())
}

fn submit_enter_to_commit_pane(pane_id: Option<&str>, session: Option<&str>) -> Result<()> {
    let delay = env::var("AW_SUBMIT_DELAY").unwrap_or_else(|_| "0.4".to_string());
    let _ = Command::new("sleep").arg(&delay).status();

    let mut args = vec!["action", "send-keys"];
    if let Some(pane_id) = pane_id {
        args.push("--pane-id");
        args.push(pane_id);
    }
    args.push("Enter");
    zellij_action(session, &args)
}

pub fn ensure_workspace_tabs_file(
    config_dir: &Path,
    workspace: &str,
) -> Result<std::path::PathBuf> {
    let tabs_file = config_dir.join(format!("{}.tabs", workspace));
    if !tabs_file.is_file() {
        return Err(AwError::new(
            format!(
                "aw: missing workspace {} in {}",
                workspace,
                path_string(config_dir)
            ),
            1,
        ));
    }
    Ok(tabs_file)
}

pub fn count_tabs_files(config_dir: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(config_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("tabs") {
            count += 1;
        }
    }
    Ok(count)
}

pub fn value_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}
