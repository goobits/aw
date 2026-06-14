mod check;
mod paths;
mod store;
mod types;

use std::time::Duration;

use crate::commit_queue::store::{MoveInput, RequestInput};
use crate::commit_queue::types::{CommitRequest, QueueReport};
use crate::error::{AwError, Result};

pub fn run_status(action: &str, args: &[String]) -> Result<i32> {
    match action {
        "request" => {
            print!("{}", store::request(parse_request_args(args)?)?);
            Ok(0)
        }
        "list" => {
            let (root, state) = parse_list_args(args)?;
            print!("{}", store::list(root.as_deref(), state.as_deref())?);
            Ok(0)
        }
        "check" => {
            let (root, json) = parse_check_args(args)?;
            let report = store::check(root.as_deref())?;
            print!("{}", render_check(&report, json));
            Ok(if report.blockers.is_empty() { 0 } else { 1 })
        }
        "next" => {
            let (root, json) = parse_next_args(args)?;
            let (candidate, report) = store::next(root.as_deref())?;
            print!("{}", render_next(candidate.as_ref(), json));
            Ok(if candidate.is_some() || report.blockers.is_empty() {
                0
            } else {
                1
            })
        }
        "done" | "block" => {
            print!(
                "{}",
                store::move_request(parse_move_args(args, action)?, move_state(action))?
            );
            Ok(0)
        }
        "wait" => {
            let (root, id, timeout, poll, _json) = parse_wait_args(args)?;
            store::wait(root.as_deref(), &id, timeout, poll)
        }
        _ => Err(AwError::usage(format!(
            "aw: unknown commit queue action {}",
            action
        ))),
    }
}

pub fn capture(action: &str, args: &[String]) -> Result<String> {
    match action {
        "request" => store::request(parse_request_args(args)?),
        "list" => {
            let (root, state) = parse_list_args(args)?;
            store::list(root.as_deref(), state.as_deref())
        }
        "next" => {
            let (root, json) = parse_next_args(args)?;
            let (candidate, _report) = store::next(root.as_deref())?;
            Ok(render_next(candidate.as_ref(), json))
        }
        "check" => {
            let (root, json) = parse_check_args(args)?;
            let report = store::check(root.as_deref())?;
            Ok(render_check(&report, json))
        }
        "done" | "block" => store::move_request(parse_move_args(args, action)?, move_state(action)),
        _ => Err(AwError::new(
            format!("commitq: unknown command: {action}"),
            1,
        )),
    }
}

fn render_check(report: &QueueReport, json: bool) -> String {
    if json {
        return format!("{}\n", serde_json::to_string_pretty(report).unwrap());
    }

    let mut text = format!(
        "Queue: {}\nPending: {}\n",
        report.queue_root, report.pending
    );
    if report.blockers.is_empty() {
        text.push_str("No blockers.\n");
    } else {
        text.push_str("Blockers:\n");
        for blocker in &report.blockers {
            text.push_str(&format!("- {}: {}\n", blocker.kind, blocker.message));
        }
    }
    text
}

fn render_next(candidate: Option<&CommitRequest>, json: bool) -> String {
    if json {
        return format!("{}\n", serde_json::to_string_pretty(&candidate).unwrap());
    }

    candidate
        .map(|request| format!("{}\t{}\n{}\n", request.id, request.title, request.file))
        .unwrap_or_else(|| "No safe pending commit request.\n".to_string())
}

fn parse_request_args(args: &[String]) -> Result<RequestInput> {
    let mut input = RequestInput::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--" => {
                index += 1;
            }
            "--root" | "--queue-root" => {
                input.root = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            "--title" => {
                input.title = require_value(args, index)?.to_string();
                index += 2;
            }
            "--summary" => {
                input.summary = require_value(args, index)?.to_string();
                index += 2;
            }
            "--owner" => {
                input.owner = require_value(args, index)?.to_string();
                index += 2;
            }
            "--path" => {
                input.paths.push(require_value(args, index)?.to_string());
                index += 2;
            }
            "--verify" | "--check" => {
                input
                    .verification
                    .push(require_value(args, index)?.to_string());
                index += 2;
            }
            "--must-contain" => {
                input
                    .must_contain
                    .push(require_value(args, index)?.to_string());
                index += 2;
            }
            "--must-not-contain" => {
                input
                    .must_not_contain
                    .push(require_value(args, index)?.to_string());
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(AwError::new(format!("commitq: unknown option: {other}"), 1));
            }
            other => {
                input.paths.push(other.to_string());
                index += 1;
            }
        }
    }
    Ok(input)
}

fn parse_list_args(args: &[String]) -> Result<(Option<String>, Option<String>)> {
    let mut root = None;
    let mut state = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                root = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            "--state" => {
                state = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            other => {
                return Err(AwError::new(format!("commitq: unknown option: {other}"), 1));
            }
        }
    }
    Ok((root, state))
}

fn parse_check_args(args: &[String]) -> Result<(Option<String>, bool)> {
    let mut root = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                root = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            other => {
                return Err(AwError::new(format!("commitq: unknown option: {other}"), 1));
            }
        }
    }
    Ok((root, json))
}

fn parse_next_args(args: &[String]) -> Result<(Option<String>, bool)> {
    parse_check_args(args)
}

fn move_state(action: &str) -> &str {
    if action == "done" {
        "done"
    } else {
        "blocked"
    }
}

fn parse_move_args(args: &[String], action: &str) -> Result<MoveInput> {
    let mut input = MoveInput::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                input.root = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            "--reason" => {
                input.reason = require_value(args, index)?.to_string();
                index += 2;
            }
            "--commit" => {
                input.commit = require_value(args, index)?.to_string();
                index += 2;
            }
            "--message" => {
                input.message = require_value(args, index)?.to_string();
                index += 2;
            }
            "--verify-result" => {
                input.verify_result = require_value(args, index)?.to_string();
                index += 2;
            }
            "--note" => {
                input.note = require_value(args, index)?.to_string();
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(AwError::new(
                    format!("aw: unknown commit {action} option {other}"),
                    2,
                ));
            }
            other => {
                if input.id.is_empty() {
                    input.id = other.to_string();
                    index += 1;
                } else {
                    return Err(commit_action_usage(
                        format!("aw: commit {action} got an extra argument: {other}"),
                        action,
                    ));
                }
            }
        }
    }
    if input.id.is_empty() {
        return Err(commit_action_usage(
            format!("aw: commit {action} requires a request id"),
            action,
        ));
    }
    if action == "block" && input.reason.is_empty() {
        return Err(commit_action_usage(
            "aw: commit block requires --reason <reason>",
            action,
        ));
    }
    Ok(input)
}

fn parse_wait_args(args: &[String]) -> Result<(Option<String>, String, Duration, Duration, bool)> {
    let mut root = None;
    let mut id = String::new();
    let mut timeout = store::parse_duration("10m")?;
    let mut poll = store::parse_duration("1s")?;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" | "--queue-root" => {
                root = Some(require_value(args, index)?.to_string());
                index += 2;
            }
            "--timeout" => {
                timeout = store::parse_duration(require_value(args, index)?)?;
                index += 2;
            }
            "--poll" => {
                poll = store::parse_duration(require_value(args, index)?)?;
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            other if other.starts_with("--") => {
                return Err(commit_action_usage(
                    format!("aw: unknown commit wait option {other}"),
                    "wait",
                ));
            }
            other => {
                if id.is_empty() {
                    id = other.to_string();
                    index += 1;
                } else {
                    return Err(commit_action_usage(
                        format!("aw: commit wait got an extra argument: {other}"),
                        "wait",
                    ));
                }
            }
        }
    }
    if id.is_empty() {
        return Err(commit_action_usage(
            "aw: commit wait requires a request id",
            "wait",
        ));
    }
    Ok((root, id, timeout, poll, json))
}

fn require_value(args: &[String], index: usize) -> Result<&str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty() || value.starts_with("--") {
        return Err(AwError::new(format!("aw: {option} requires a value"), 2));
    }
    Ok(value)
}

fn commit_action_usage(message: impl Into<String>, action: &str) -> AwError {
    let usage = match action {
        "wait" => "aw commit wait <id> [--queue-root <path>] [--timeout 10m]",
        "done" => "aw commit done <id> [--queue-root <path>]",
        "block" => "aw commit block <id> --reason <reason> [--queue-root <path>]",
        _ => "aw commit <action>",
    };
    AwError::new(format!("{}\n\nusage:\n  {usage}", message.into()), 2)
}
