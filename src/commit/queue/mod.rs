mod check;
mod paths;
mod store;
mod types;

use std::time::Duration;

use serde_json::Value;

use crate::commit_queue::store::{MoveInput, RequestInput};
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
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                store::print_report(&report);
            }
            Ok(if report.blockers.is_empty() { 0 } else { 1 })
        }
        "next" => {
            let (root, json) = parse_next_args(args)?;
            let (candidate, report) = store::next(root.as_deref())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&store::candidate_to_json(candidate.clone()))
                        .unwrap()
                );
            } else if let Some(request) = &candidate {
                println!("{}\t{}", request.id, request.title);
                println!("{}", request.file);
            } else {
                println!("No safe pending commit request.");
            }
            Ok(if candidate.is_some() || report.blockers.is_empty() {
                0
            } else {
                1
            })
        }
        "done" | "block" => {
            print!(
                "{}",
                store::move_request(
                    parse_move_args(args)?,
                    if action == "done" { "done" } else { "blocked" }
                )?
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
            if json {
                Ok(format!(
                    "{}\n",
                    serde_json::to_string_pretty(&store::candidate_to_json(candidate)).unwrap()
                ))
            } else if let Some(request) = candidate {
                Ok(format!(
                    "{}\t{}\n{}\n",
                    request.id, request.title, request.file
                ))
            } else {
                Ok("No safe pending commit request.\n".to_string())
            }
        }
        "check" => {
            let (root, json) = parse_check_args(args)?;
            let report = store::check(root.as_deref())?;
            if json {
                Ok(format!(
                    "{}\n",
                    serde_json::to_string_pretty(&report).unwrap()
                ))
            } else {
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
                Ok(text)
            }
        }
        "done" | "block" => store::move_request(parse_move_args(args)?, action),
        _ => Err(AwError::new(
            format!("commitq: unknown command: {action}"),
            1,
        )),
    }
}

pub fn capture_allow_failure(action: &str, args: &[String]) -> Result<String> {
    capture(action, args).or_else(|_| Ok(String::new()))
}

pub fn json(action: &str, args: &[String]) -> Result<Value> {
    store::read_json_for(action, args)
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
            "--root" => {
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
            "--root" => {
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
            "--root" => {
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

fn parse_move_args(args: &[String]) -> Result<MoveInput> {
    let mut input = MoveInput::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
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
                return Err(AwError::new(format!("commitq: unknown option: {other}"), 1));
            }
            other => {
                if input.id.is_empty() {
                    input.id = other.to_string();
                    index += 1;
                } else {
                    return Err(AwError::new(
                        format!("commitq: unexpected argument: {other}"),
                        1,
                    ));
                }
            }
        }
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
            "--root" => {
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
                return Err(AwError::new(format!("commitq: unknown option: {other}"), 1));
            }
            other => {
                if id.is_empty() {
                    id = other.to_string();
                    index += 1;
                } else {
                    return Err(AwError::new(
                        format!("commitq: unexpected argument: {other}"),
                        1,
                    ));
                }
            }
        }
    }
    Ok((root, id, timeout, poll, json))
}

fn require_value(args: &[String], index: usize) -> Result<&str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty() || value.starts_with("--") {
        return Err(AwError::new(
            format!("commitq: missing value for {option}"),
            1,
        ));
    }
    Ok(value)
}
