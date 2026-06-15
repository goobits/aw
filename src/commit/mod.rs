pub(crate) mod queue;

use std::path::Path;

use serde_json::Value;

use crate::commit_queue;
use crate::error::{AwError, Result};
use crate::help;
use crate::paths::{shell_quote, validate_name};
use crate::profile::{default_workspace_from_config, find_config_dir, install_profile};
use crate::tabs::upsert_workspace_tab_line;
use crate::zellij::{
    default_workspace_session_name, ensure_workspace_tabs_file, send_to_commit_tab,
    sync_workspace_session,
};

const COMMIT_USAGE: &str = r#"usage:
  aw commit setup [workspace] [--tab git] [--session <name>] [--agent <cmd>|--no-agent]
  aw commit request <title> <path>... [--check <cmd>] [--summary <text>] [--queue-root <path>] [--poke [tab]] [--workspace <workspace>] [--session <name>] [--wait] [--timeout 10m]
  aw commit status [--queue-root <path>]
  aw commit doctor [--queue-root <path>]
  aw commit wait <id> [--queue-root <path>] [--timeout 10m]
  aw commit poke [tab] [--queue-root <path>] [--workspace <workspace>] [--session <name>]"#;

pub fn run_commit_command(args: &[String]) -> Result<i32> {
    let Some((action, rest)) = args.split_first() else {
        return Err(commit_usage("aw: commit requires an action"));
    };

    match action.as_str() {
        "-h" | "--help" | "help" => {
            help::println(COMMIT_USAGE);
            Ok(0)
        }
        "setup" => {
            setup_commit_tab(rest)?;
            Ok(0)
        }
        "request" => {
            commit_request(rest)?;
            Ok(0)
        }
        "raw-request" => {
            commit_raw_request(rest)?;
            Ok(0)
        }
        "status" => {
            let root = parse_root_only(rest, "status")?;
            print_commit_status(root.as_deref())?;
            Ok(0)
        }
        "doctor" => {
            let root = parse_root_only(rest, "doctor")?;
            print_commit_doctor(root.as_deref())?;
            Ok(0)
        }
        "list" | "check" | "next" | "done" | "block" | "wait" => {
            let normalized = normalize_queue_root_args(rest)?;
            commit_queue::run_status(action, &normalized)
        }
        "poke" => {
            commit_poke(rest)?;
            Ok(0)
        }
        other => Err(commit_usage(format!("aw: unknown commit action {}", other))),
    }
}

fn setup_commit_tab(args: &[String]) -> Result<()> {
    let mut index = 0;
    let mut workspace = "";
    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            workspace = first;
            index = 1;
        }
    }

    let mut tab_name = "git".to_string();
    let mut agent_command = Some("codex".to_string());
    let mut session_name = String::new();

    while index < args.len() {
        match args[index].as_str() {
            "--tab" => {
                tab_name = require_option_value(args, index)?.to_string();
                index += 2;
            }
            "--agent" => {
                agent_command = Some(require_option_value(args, index)?.to_string());
                index += 2;
            }
            "--session" => {
                session_name = require_option_value(args, index)?.to_string();
                index += 2;
            }
            "--no-agent" => {
                agent_command = None;
                index += 1;
            }
            other => {
                return Err(commit_usage(format!(
                    "aw: unknown commit setup argument {}",
                    other
                )))
            }
        }
    }

    validate_name("tab", &tab_name)?;
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };

    let workspace = if workspace.is_empty() {
        default_workspace_from_config(&config_dir)
    } else {
        workspace.to_string()
    };
    if workspace.is_empty() {
        return Err(AwError::new(
            "aw: no default workspace configured; pass a workspace name",
            1,
        ));
    }
    validate_name("workspace", &workspace)?;
    if session_name.is_empty() {
        session_name = default_workspace_session_name(&config_dir, &workspace);
    }

    let tabs_file = ensure_workspace_tabs_file(&config_dir, &workspace)?;
    upsert_workspace_tab_line(&tabs_file, &tab_name)?;
    install_profile(&config_dir, true)?;
    sync_workspace_session(&config_dir, &workspace, Some(&session_name))?;

    if let Some(agent_command) = agent_command {
        if send_to_commit_tab(&tab_name, &agent_command, Some(&session_name), None)? {
            println!(
                "Commit tab {} is ready in {} and received `{}`.",
                tab_name, session_name, agent_command
            );
        } else {
            println!(
                "Commit tab {} is ready in {}, but no live tab was found to start `{}`.",
                tab_name, session_name, agent_command
            );
            if tab_name == "git" {
                println!("Open the workspace, then run `aw commit poke`.");
            } else {
                println!(
                    "Open the workspace, then run `aw commit poke {}`.",
                    tab_name
                );
            }
        }
    } else {
        println!("Commit tab {} is ready in {}.", tab_name, session_name);
    }
    Ok(())
}

fn commit_request_from_title(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(commit_usage(
            "aw: commit request requires a title and at least one path",
        ));
    }

    let mut filtered = vec!["--title".to_string(), args[0].clone()];
    let mut index = 1;
    let mut path_count = 0;
    let mut root_value = String::new();
    let mut poke = false;
    let mut poke_tab = "git".to_string();
    let mut session_name = String::new();
    let mut workspace_name = String::new();
    let mut wait = false;
    let mut wait_timeout = String::new();
    let mut wait_poll = String::new();

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--check" | "--verify" => {
                filtered.push("--verify".to_string());
                filtered.push(require_option_value(args, index)?.to_string());
                index += 2;
            }
            "--root" | "--queue-root" => {
                root_value = require_option_value(args, index)?.to_string();
                filtered.push("--root".to_string());
                filtered.push(root_value.clone());
                index += 2;
            }
            "--summary" | "--owner" | "--must-contain" | "--must-not-contain" => {
                filtered.push(arg.clone());
                filtered.push(require_option_value(args, index)?.to_string());
                index += 2;
            }
            "--poke" => {
                poke = true;
                index += 1;
                if let Some(next) = args.get(index) {
                    if !next.starts_with("--") {
                        poke_tab = next.clone();
                        index += 1;
                    }
                }
            }
            "--session" => {
                session_name = require_option_value(args, index)?.to_string();
                validate_name("session", &session_name)?;
                index += 2;
            }
            "--workspace" => {
                workspace_name = require_option_value(args, index)?.to_string();
                validate_name("workspace", &workspace_name)?;
                index += 2;
            }
            "--wait" => {
                wait = true;
                index += 1;
            }
            "--timeout" => {
                wait_timeout = require_option_value(args, index)?.to_string();
                index += 2;
            }
            "--poll" => {
                wait_poll = require_option_value(args, index)?.to_string();
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(commit_usage(format!(
                    "aw: unknown commit request argument {other}"
                )));
            }
            path => {
                filtered.push("--path".to_string());
                filtered.push(path.to_string());
                path_count += 1;
                index += 1;
            }
        }
    }

    if path_count == 0 {
        return Err(commit_usage(
            "aw: commit request requires at least one path",
        ));
    }

    let request_file = commit_queue::capture("request", &filtered)?;
    let request_id = Path::new(request_file.trim())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_string();
    println!("Created commit request {}.", request_id);

    if poke {
        let missing = format!(
            "Created commit request {}. No live Zellij tab named {} was found to poke.",
            request_id, poke_tab
        );
        poke_commit_tab(
            &poke_tab,
            resolved_poke_session(&session_name, &workspace_name)?.as_deref(),
            Some(&missing),
            root_arg(&root_value).as_deref(),
        )?;
    } else if !root_value.is_empty() {
        println!(
            "Run `{}` to wake the git tab.",
            poke_command(&poke_tab, &root_value, &session_name, &workspace_name)
        );
    } else if !session_name.is_empty() {
        println!(
            "Run `{}` to wake the git tab.",
            poke_command(&poke_tab, &root_value, &session_name, &workspace_name)
        );
    } else if !workspace_name.is_empty() {
        println!(
            "Run `{}` to wake the git tab.",
            poke_command(&poke_tab, &root_value, &session_name, &workspace_name)
        );
    } else if poke_tab == "git" {
        println!("Run `aw commit poke` to wake the git tab.");
    } else {
        println!("Run `aw commit poke {}` to wake the git tab.", poke_tab);
    }

    if wait {
        let mut wait_args = vec![request_id];
        if !root_value.is_empty() {
            wait_args.push("--root".to_string());
            wait_args.push(root_value);
        }
        if !wait_timeout.is_empty() {
            wait_args.push("--timeout".to_string());
            wait_args.push(wait_timeout);
        }
        if !wait_poll.is_empty() {
            wait_args.push("--poll".to_string());
            wait_args.push(wait_poll);
        }
        let status = commit_queue::run_status("wait", &wait_args)?;
        if status != 0 {
            return Err(AwError::new("", status));
        }
    }

    Ok(())
}

fn commit_request(args: &[String]) -> Result<()> {
    if args.first().is_some_and(|arg| arg.starts_with("--")) {
        return commit_raw_request(args);
    }
    commit_request_from_title(args)
}

fn commit_raw_request(args: &[String]) -> Result<()> {
    let mut filtered = Vec::new();
    let mut root_value = String::new();
    let mut poke = false;
    let mut poke_tab = "git".to_string();
    let mut session_name = String::new();
    let mut workspace_name = String::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--check" => {
                filtered.push("--verify".to_string());
                filtered.push(require_option_value(args, index)?.to_string());
                index += 2;
            }
            "--root" | "--queue-root" => {
                root_value = require_option_value(args, index)?.to_string();
                filtered.push("--root".to_string());
                filtered.push(root_value.clone());
                index += 2;
            }
            "--poke" => {
                poke = true;
                index += 1;
                if let Some(next) = args.get(index) {
                    if !next.starts_with("--") {
                        poke_tab = next.clone();
                        index += 1;
                    }
                }
            }
            "--session" => {
                session_name = require_option_value(args, index)?.to_string();
                validate_name("session", &session_name)?;
                index += 2;
            }
            "--workspace" => {
                workspace_name = require_option_value(args, index)?.to_string();
                validate_name("workspace", &workspace_name)?;
                index += 2;
            }
            other => {
                filtered.push(other.to_string());
                index += 1;
            }
        }
    }

    let output = commit_queue::capture("request", &filtered)?;
    print!("{}", output);
    if poke {
        poke_commit_tab(
            &poke_tab,
            resolved_poke_session(&session_name, &workspace_name)?.as_deref(),
            None,
            root_arg(&root_value).as_deref(),
        )?;
    }
    Ok(())
}

fn commit_poke(args: &[String]) -> Result<()> {
    let mut poke_tab = "git".to_string();
    let mut root_value = String::new();
    let mut session_name = String::new();
    let mut workspace_name = String::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                root_value = require_option_value(args, index)?.to_string();
                index += 2;
            }
            "--session" => {
                session_name = require_option_value(args, index)?.to_string();
                validate_name("session", &session_name)?;
                index += 2;
            }
            "--workspace" => {
                workspace_name = require_option_value(args, index)?.to_string();
                validate_name("workspace", &workspace_name)?;
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(commit_usage(format!(
                    "aw: unknown commit poke argument {}",
                    other
                )));
            }
            tab => {
                if poke_tab != "git" {
                    return Err(commit_usage("aw: commit poke accepts at most one tab name"));
                }
                poke_tab = tab.to_string();
                index += 1;
            }
        }
    }

    poke_commit_tab(
        &poke_tab,
        resolved_poke_session(&session_name, &workspace_name)?.as_deref(),
        None,
        root_arg(&root_value).as_deref(),
    )
}

fn poke_commit_tab(
    requested_name: &str,
    session: Option<&str>,
    missing_message: Option<&str>,
    root_value: Option<&str>,
) -> Result<()> {
    let mut message = "$x-commit next".to_string();
    if let Some(root_value) = root_value {
        if !root_value.is_empty() {
            message.push_str(" --root ");
            message.push_str(&shell_quote(root_value));
        }
    }

    if send_to_commit_tab(requested_name, &message, session, missing_message)? {
        println!("Poked {} with {}.", requested_name, message);
    }
    Ok(())
}

fn resolved_poke_session(
    explicit_session: &str,
    explicit_workspace: &str,
) -> Result<Option<String>> {
    if !explicit_session.is_empty() {
        return Ok(Some(explicit_session.to_string()));
    }

    let Some(config_dir) = find_config_dir() else {
        return Ok(None);
    };
    let workspace = if explicit_workspace.is_empty() {
        default_workspace_from_config(&config_dir)
    } else {
        explicit_workspace.to_string()
    };
    if workspace.is_empty() {
        return Ok(None);
    }
    ensure_workspace_tabs_file(&config_dir, &workspace)?;
    Ok(Some(default_workspace_session_name(
        &config_dir,
        &workspace,
    )))
}

fn poke_command(tab: &str, root: &str, session: &str, workspace: &str) -> String {
    let mut command = "aw commit poke".to_string();
    if tab != "git" {
        command.push(' ');
        command.push_str(tab);
    }
    if !root.is_empty() {
        command.push_str(" --queue-root ");
        command.push_str(&shell_quote(root));
    }
    if !workspace.is_empty() {
        command.push_str(" --workspace ");
        command.push_str(&shell_quote(workspace));
    }
    if !session.is_empty() {
        command.push_str(" --session ");
        command.push_str(&shell_quote(session));
    }
    command
}

fn print_commit_status(root: Option<&str>) -> Result<()> {
    let root_args = root_args(root);
    let pending_lines = commit_list_lines(None, &root_args)?;
    let pending = pending_lines.len();
    let done = commit_list_count(Some("done"), &root_args)?;
    let blocked = commit_list_count(Some("blocked"), &root_args)?;
    let check = commit_queue_json("check", &append_args(&root_args, &["--json"]))?;
    let unsafe_count = check
        .get("blockers")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let status = if unsafe_count > 0 {
        "blocked"
    } else if pending == 0 {
        "empty"
    } else {
        "ready"
    };
    let next_line = if unsafe_count == 0 {
        next_request_line(&root_args)?
    } else {
        None
    };

    println!("Commit Queue\n");
    println!("Status   {}", status);
    println!("Pending  {}", pending);
    println!("Unsafe   {}", unsafe_count);
    println!("Blocked  {}", blocked);
    println!("Done     {}", done);
    println!(
        "Next     {}",
        if unsafe_count > 0 {
            "blocked"
        } else if next_line.is_some() {
            "ready"
        } else {
            "none"
        }
    );

    println!("\nNext");
    if unsafe_count == 0 {
        if let Some(next_line) = &next_line {
            println!("[ready]  {next_line}");
            if let Some(root) = root {
                println!("Command  aw commit poke --queue-root {}", shell_quote(root));
            } else {
                println!("Command  aw commit poke");
            }
        } else {
            println!("[none]   no pending safe request");
        }
    } else {
        println!("[blocked] queue has unsafe overlaps or invalid tickets");
        if let Some(root) = root {
            println!(
                "Command  aw commit doctor --queue-root {}",
                shell_quote(root)
            );
        } else {
            println!("Command  aw commit doctor");
        }
    }

    if !pending_lines.is_empty() {
        println!("\nPending Requests");
        for line in pending_lines.iter().take(5) {
            println!("- {}", summarize_commit_list_line(line));
        }
        if pending_lines.len() > 5 {
            println!("- ... {} more", pending_lines.len() - 5);
        }
    }
    if unsafe_count > 0 {
        println!("\nBlocked By");
        let blockers = check
            .get("blockers")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for blocker in blockers.iter().take(3) {
            let message = blocker
                .get("message")
                .or_else(|| blocker.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("queue blocker");
            println!("- {}", message);
        }
        if unsafe_count > 3 {
            println!("- ... {} more", unsafe_count - 3);
        }
    } else if blocked > 0 {
        println!("\nBlocked Archive");
        println!("- {} blocked ticket(s) still need reconciliation", blocked);
    }
    Ok(())
}

fn commit_list_count(state: Option<&str>, root_args: &[String]) -> Result<usize> {
    Ok(commit_list_lines(state, root_args)?.len())
}

fn commit_list_lines(state: Option<&str>, root_args: &[String]) -> Result<Vec<String>> {
    let mut args = Vec::new();
    if let Some(state) = state {
        args.push("--state".to_string());
        args.push(state.to_string());
    }
    args.extend(root_args.iter().cloned());
    let output = commit_queue::capture("list", &args)?;
    Ok(output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn summarize_commit_list_line(line: &str) -> String {
    let mut parts = line.split('\t');
    let id = parts.next().unwrap_or("");
    let title = parts.next().unwrap_or("");
    let paths = parts.next().unwrap_or("");
    if paths.is_empty() {
        format!("{} {}", id, title).trim().to_string()
    } else {
        format!("{} {} ({})", id, title, paths)
    }
}

fn print_commit_doctor(root: Option<&str>) -> Result<()> {
    let root_args = root_args(root);
    let pending = commit_list_count(None, &root_args)?;
    let done = commit_list_count(Some("done"), &root_args)?;
    let blocked = commit_list_count(Some("blocked"), &root_args)?;
    let check = commit_queue_json("check", &append_args(&root_args, &["--json"]))?;
    let blockers = check
        .get("blockers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let unsafe_count = blockers.len();

    let status = if unsafe_count > 0 {
        "blocked"
    } else if pending == 0 {
        "empty"
    } else {
        "ready"
    };

    println!("Commit Queue\n");
    println!("Status    {}", status);
    println!("Pending   {}", pending);
    println!("Unsafe    {}", unsafe_count);
    println!("Blocked   {}", blocked);
    println!("Done      {}", done);
    println!("\nNext");
    if unsafe_count == 0 {
        if let Some(next_line) = next_request_line(&root_args)? {
            println!("[ready]   {}", next_line);
        } else {
            println!("[none]    no pending safe request");
        }
    } else {
        println!("[blocked] queue has unsafe overlaps or invalid tickets");
    }

    if unsafe_count > 0 {
        println!("\nWhy");
        for blocker in blockers.iter().take(5) {
            let message = blocker
                .get("message")
                .or_else(|| blocker.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("queue blocker");
            println!("- {}", message);
        }
        if blockers.is_empty() {
            println!("- queue check reported blockers");
        }
        println!("\nTry");
        println!("- ask the git tab to resolve blocked or overlapping tickets");
        println!("- run `aw commit check` for raw blocker details");
    } else {
        if blocked > 0 {
            println!("\nBlocked");
            println!(
                "[triage] {} blocked ticket(s) still need reconciliation.",
                blocked
            );
        }
        println!("\nTry");
        if next_request_line(&root_args)?.is_some() {
            println!("- aw commit poke");
            if blocked > 0 {
                println!("- reconcile blocked tickets before calling the queue fully clean");
            }
        } else {
            println!("- no commit wakeup needed");
        }
    }
    Ok(())
}

fn parse_root_only(args: &[String], action: &str) -> Result<Option<String>> {
    let mut root_value = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                root_value = Some(require_option_value(args, index)?.to_string());
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(commit_usage(format!(
                    "aw: unknown commit {} argument {}",
                    action, other
                )));
            }
            other => {
                return Err(commit_usage(format!(
                    "aw: commit {} does not accept positional arguments: {}",
                    action, other
                )));
            }
        }
    }
    Ok(root_value)
}

fn normalize_queue_root_args(args: &[String]) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--queue-root" => {
                normalized.push("--root".to_string());
                normalized.push(require_option_value(args, index)?.to_string());
                index += 2;
            }
            value => {
                normalized.push(value.to_string());
                index += 1;
            }
        }
    }
    Ok(normalized)
}

fn next_request_line(root_args: &[String]) -> Result<Option<String>> {
    let output = commit_queue::capture("next", root_args).unwrap_or_default();
    let line = output.lines().next().unwrap_or("");
    if line.is_empty() || line == "No safe pending commit request." {
        Ok(None)
    } else {
        Ok(Some(line.to_string()))
    }
}

fn commit_queue_json(action: &str, args: &[String]) -> Result<Value> {
    serde_json::from_str(&commit_queue::capture(action, args)?)
        .map_err(|error| AwError::new(error.to_string(), 1))
}

fn root_arg(root: &str) -> Option<String> {
    if root.is_empty() {
        None
    } else {
        Some(root.to_string())
    }
}

fn root_args(root: Option<&str>) -> Vec<String> {
    root.map(|root| vec!["--root".to_string(), root.to_string()])
        .unwrap_or_default()
}

fn append_args(base: &[String], extras: &[&str]) -> Vec<String> {
    let mut next = base.to_vec();
    next.extend(extras.iter().map(|item| item.to_string()));
    next
}

fn require_option_value(args: &[String], index: usize) -> Result<&str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty() || value.starts_with("--") {
        return Err(AwError::new(format!("aw: {} requires a value", option), 2));
    }
    Ok(value)
}

fn commit_usage(message: impl Into<String>) -> AwError {
    AwError::new(format!("{}\n\n{}", message.into(), COMMIT_USAGE), 2)
}
