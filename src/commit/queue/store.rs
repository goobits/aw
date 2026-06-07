use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::commit_queue::types::{
    CommitRequest, Fingerprints, QueueBlocker, QueueRead, QueueReport,
};
use crate::error::{AwError, Result};
use crate::paths::path_string;

use super::check::{check_queue, validate_move_request};
use super::paths::normalize_repo_path;

const DEFAULT_QUEUE_ROOT: &str = ".llm/commit-queue";
const STATES: [&str; 3] = ["pending", "done", "blocked"];

#[derive(Default)]
pub struct RequestInput {
    pub root: Option<String>,
    pub title: String,
    pub owner: String,
    pub summary: String,
    pub paths: Vec<String>,
    pub verification: Vec<String>,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
}

#[derive(Default)]
pub struct MoveInput {
    pub root: Option<String>,
    pub id: String,
    pub reason: String,
    pub commit: String,
    pub message: String,
    pub verify_result: String,
    pub note: String,
}

pub fn request(input: RequestInput) -> Result<String> {
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(input.root.as_deref(), &repo_root);
    if input.title.is_empty() {
        return Err(AwError::new("commitq: request requires --title", 1));
    }
    if input.paths.is_empty() {
        return Err(AwError::new(
            "commitq: request requires at least one --path",
            1,
        ));
    }

    ensure_queue_dirs(&queue_root)?;
    let created_at = now_label();
    let id = format!(
        "{}-{}-{}",
        timestamp_id(&created_at),
        slug(&input.title),
        random_suffix()
    );
    let paths = input
        .paths
        .iter()
        .map(|path| normalize_repo_path(path, &repo_root))
        .collect::<Result<Vec<_>>>()?;
    let owner = if input.owner.is_empty() {
        env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    } else {
        input.owner
    };
    let fingerprints = if input.must_contain.is_empty() && input.must_not_contain.is_empty() {
        None
    } else {
        Some(Fingerprints {
            must_contain: input.must_contain,
            must_not_contain: input.must_not_contain,
        })
    };
    let payload = CommitRequest {
        id: id.clone(),
        title: input.title,
        owner,
        summary: input.summary,
        paths,
        verification: input.verification,
        fingerprints,
        created_at,
        blocked_reason: None,
        blocked_at: None,
        done_at: None,
        result: None,
        file: String::new(),
        state: String::new(),
    };
    let request_file = queue_root.join("pending").join(format!("{id}.json"));
    write_json_atomic(&request_file, &payload)?;
    Ok(format!("{}\n", path_string(&request_file)))
}

pub fn list(root: Option<&str>, state: Option<&str>) -> Result<String> {
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(root, &repo_root);
    let state = state.unwrap_or("pending");
    if !STATES.contains(&state) {
        return Err(AwError::new(format!("commitq: unknown state: {state}"), 1));
    }
    let mut output = String::new();
    for request in read_queue(&queue_root, state).requests {
        output.push_str(&format!(
            "{}\t{}\t{}\n",
            request.id,
            request.title,
            request.paths.join(",")
        ));
    }
    Ok(output)
}

pub fn check(root: Option<&str>) -> Result<QueueReport> {
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(root, &repo_root);
    ensure_queue_dirs(&queue_root)?;
    check_queue(&queue_root, &repo_root)
}

pub fn next(root: Option<&str>) -> Result<(Option<CommitRequest>, QueueReport)> {
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(root, &repo_root);
    let report = check_queue(&queue_root, &repo_root)?;
    let blocked_ids = report
        .blockers
        .iter()
        .flat_map(|blocker| blocker.ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let candidate = read_queue(&queue_root, "pending")
        .requests
        .into_iter()
        .find(|request| !blocked_ids.contains(&request.id));
    Ok((candidate, report))
}

pub fn move_request(input: MoveInput, state: &str) -> Result<String> {
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(input.root.as_deref(), &repo_root);
    if input.id.is_empty() {
        return Err(AwError::new(
            format!("commitq: {state} requires an id or request file"),
            1,
        ));
    }
    ensure_queue_dirs(&queue_root)?;
    let source = resolve_request_file(&queue_root, &input.id)?;
    let request = read_request_file(&source, "pending")?;
    validate_move_request(&queue_root, &source, &request, state, &repo_root)?;

    let mut terminal = request.clone();
    terminal.file.clear();
    terminal.state.clear();
    if state == "blocked" {
        if input.reason.is_empty() {
            return Err(AwError::new("commitq: block requires --reason", 1));
        }
        terminal.blocked_reason = Some(input.reason.clone());
        terminal.blocked_at = Some(now_label());
    }
    if state == "done" {
        terminal.done_at = Some(now_label());
    }
    let mut result = serde_json::Map::new();
    if !input.commit.is_empty() {
        result.insert("commit".to_string(), Value::String(input.commit));
    }
    if !input.message.is_empty() {
        result.insert("message".to_string(), Value::String(input.message));
    }
    if !input.verify_result.is_empty() {
        result.insert(
            "verification".to_string(),
            Value::String(input.verify_result),
        );
    }
    if !input.note.is_empty() {
        result.insert("note".to_string(), Value::String(input.note));
    }
    if state == "blocked" && !input.reason.is_empty() {
        result.insert("reason".to_string(), Value::String(input.reason));
    }
    if !result.is_empty() {
        terminal.result = Some(Value::Object(result));
    }
    let destination = queue_root
        .join(state)
        .join(source.file_name().unwrap_or_default());
    if destination.exists() {
        return Err(AwError::new(
            format!(
                "commitq: request already exists in {state}: {}",
                source.file_name().unwrap_or_default().to_string_lossy()
            ),
            1,
        ));
    }
    write_json_atomic(&destination, &terminal)?;
    fs::remove_file(&source)?;
    Ok(format!("{}\n", path_string(&destination)))
}

pub fn wait(root: Option<&str>, id: &str, timeout: Duration, poll: Duration) -> Result<i32> {
    if id.is_empty() {
        return Err(AwError::new("commitq: wait requires an id", 1));
    }
    let repo_root = find_repo_root(&env::current_dir()?)?;
    let queue_root = resolve_queue_root(root, &repo_root);
    let started = SystemTime::now();
    loop {
        if let Some(request) = find_request_any_state(&queue_root, id)? {
            if request.state != "pending" {
                print_wait_result(&request);
                return Ok(if request.state == "blocked" { 2 } else { 0 });
            }
        }
        let elapsed = SystemTime::now()
            .duration_since(started)
            .unwrap_or_else(|_| Duration::from_millis(0));
        if elapsed >= timeout {
            println!("Timeout");
            println!("Request  {id}");
            println!("State    pending");
            return Ok(3);
        }
        thread::sleep(poll.max(Duration::from_millis(100)));
    }
}

pub fn read_json_for(action: &str, root_args: &[String]) -> Result<Value> {
    match action {
        "check" => {
            let root = parse_root(root_args)?;
            let report = check(root.as_deref())?;
            Ok(serde_json::to_value(report).unwrap_or(Value::Null))
        }
        _ => Ok(Value::Null),
    }
}

pub fn parse_duration(value: &str) -> Result<Duration> {
    let trimmed = value.trim();
    let (amount, unit) = if let Some(value) = trimmed.strip_suffix("ms") {
        (value, "ms")
    } else if let Some(value) = trimmed.strip_suffix('s') {
        (value, "s")
    } else if let Some(value) = trimmed.strip_suffix('m') {
        (value, "m")
    } else {
        (trimmed, "ms")
    };
    let amount = amount
        .parse::<u64>()
        .map_err(|_| AwError::new(format!("commitq: invalid duration: {value}"), 1))?;
    Ok(match unit {
        "ms" => Duration::from_millis(amount),
        "s" => Duration::from_secs(amount),
        "m" => Duration::from_secs(amount * 60),
        _ => Duration::from_millis(amount),
    })
}

pub fn print_report(report: &QueueReport) {
    println!("Queue: {}", report.queue_root);
    println!("Pending: {}", report.pending);
    if report.blockers.is_empty() {
        println!("No blockers.");
        return;
    }
    println!("Blockers:");
    for blocker in &report.blockers {
        println!("- {}: {}", blocker.kind, blocker.message);
    }
}

pub(super) fn read_queue(queue_root: &Path, state: &str) -> QueueRead {
    let dir = queue_root.join(state);
    let mut result = QueueRead {
        requests: Vec::new(),
        blockers: Vec::new(),
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return result;
    };
    let mut files = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    for file in files {
        match read_request_file(&file, state) {
            Ok(request) => result.requests.push(request),
            Err(error) => result.blockers.push(blocker(
                "invalid",
                vec![file
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()],
                format!(
                    "{} is not readable JSON: {}",
                    file.file_name().unwrap_or_default().to_string_lossy(),
                    error.message
                ),
            )),
        }
    }
    result
}

fn read_request_file(file: &Path, state: &str) -> Result<CommitRequest> {
    let text = fs::read_to_string(file)?;
    let mut request = serde_json::from_str::<CommitRequest>(&text)
        .map_err(|error| AwError::new(error.to_string(), 1))?;
    request.file = path_string(file);
    request.state = state.to_string();
    Ok(request)
}

fn find_request_any_state(queue_root: &Path, id_or_file: &str) -> Result<Option<CommitRequest>> {
    for state in STATES {
        let dir = queue_root.join(state);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut files = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        files.sort();
        let exact = if id_or_file.ends_with(".json") {
            id_or_file.to_string()
        } else {
            format!("{id_or_file}.json")
        };
        let matches = files
            .into_iter()
            .filter(|file| {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                name == exact || name.starts_with(id_or_file)
            })
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            return read_request_file(&matches[0], state).map(Some);
        }
        if matches.len() > 1 {
            return Err(AwError::new(
                format!("commitq: ambiguous request id: {id_or_file}"),
                1,
            ));
        }
    }
    Ok(None)
}

fn resolve_request_file(queue_root: &Path, id_or_file: &str) -> Result<PathBuf> {
    let pending_dir = queue_root.join("pending");
    let candidate_path = PathBuf::from(id_or_file);
    if candidate_path.exists() {
        let resolved = candidate_path.canonicalize()?;
        assert_pending_queue_file(&pending_dir, &resolved, id_or_file)?;
        return Ok(resolved);
    }
    if id_or_file.contains('/')
        || id_or_file.contains('\\')
        || id_or_file == "."
        || id_or_file == ".."
    {
        return Err(AwError::new(
            format!("commitq: invalid request id: {id_or_file}"),
            1,
        ));
    }
    let candidate = pending_dir.join(if id_or_file.ends_with(".json") {
        id_or_file.to_string()
    } else {
        format!("{id_or_file}.json")
    });
    assert_pending_queue_file(&pending_dir, &candidate, id_or_file)?;
    if candidate.exists() {
        return Ok(candidate);
    }
    let mut matches = fs::read_dir(&pending_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|file| {
            file.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with(id_or_file)
        })
        .collect::<Vec<_>>();
    matches.sort();
    if matches.len() == 1 {
        assert_pending_queue_file(&pending_dir, &matches[0], id_or_file)?;
        return Ok(matches.remove(0));
    }
    if matches.len() > 1 {
        return Err(AwError::new(
            format!("commitq: ambiguous request id: {id_or_file}"),
            1,
        ));
    }
    Err(AwError::new(
        format!("commitq: request not found: {id_or_file}"),
        1,
    ))
}

fn assert_pending_queue_file(pending_dir: &Path, file: &Path, id_or_file: &str) -> Result<()> {
    let resolved = if file.exists() {
        file.canonicalize()?
    } else {
        file.to_path_buf()
    };
    let relative = resolved.strip_prefix(pending_dir).ok();
    if relative.is_none() {
        return Err(AwError::new(
            format!("commitq: request file is outside pending queue: {id_or_file}"),
            1,
        ));
    }
    if resolved.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err(AwError::new(
            format!("commitq: request file must be JSON: {id_or_file}"),
            1,
        ));
    }
    Ok(())
}

fn ensure_queue_dirs(queue_root: &Path) -> Result<()> {
    for state in STATES {
        fs::create_dir_all(queue_root.join(state))?;
    }
    fs::create_dir_all(queue_root.join("examples"))?;
    Ok(())
}

fn write_json_atomic<T: serde::Serialize>(file: &Path, payload: &T) -> Result<()> {
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;
    let temp = dir.join(format!(
        ".{}.{}.{}.tmp",
        file.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        random_suffix()
    ));
    let text = format!("{}\n", serde_json::to_string_pretty(payload).unwrap());
    fs::write(&temp, text)?;
    fs::rename(temp, file)?;
    Ok(())
}

fn print_wait_result(request: &CommitRequest) {
    if request.state == "done" {
        println!("Done");
        print_result_line("Request", &request.id);
        print_result_line(
            "Commit",
            result_string(request, "commit").as_deref().unwrap_or(""),
        );
        let message = result_string(request, "message").unwrap_or_else(|| request.title.clone());
        print_result_line("Message", &message);
        print_result_line(
            "Verify",
            result_string(request, "verification")
                .as_deref()
                .unwrap_or(""),
        );
        print_result_line(
            "Notes",
            result_string(request, "note").as_deref().unwrap_or(""),
        );
        return;
    }
    println!("Blocked");
    print_result_line("Request", &request.id);
    let reason = result_string(request, "reason")
        .or_else(|| request.blocked_reason.clone())
        .unwrap_or_default();
    print_result_line("Reason", &reason);
    print_result_line(
        "Notes",
        result_string(request, "note").as_deref().unwrap_or(""),
    );
}

fn result_string(request: &CommitRequest, key: &str) -> Option<String> {
    request
        .result
        .as_ref()
        .and_then(|result| result.get(key))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn print_result_line(label: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    println!("{label:<8} {value}");
}

fn parse_root(args: &[String]) -> Result<Option<String>> {
    let mut root = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--root" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AwError::new("commitq: missing value for --root", 1))?;
            root = Some(value.clone());
            index += 2;
        } else {
            index += 1;
        }
    }
    Ok(root)
}

fn resolve_queue_root(value: Option<&str>, repo_root: &Path) -> PathBuf {
    match value {
        Some(root) if Path::new(root).is_absolute() => PathBuf::from(root),
        Some(root) => repo_root.join(root),
        None => repo_root.join(DEFAULT_QUEUE_ROOT),
    }
}

fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(".git").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Ok(start.to_path_buf());
        }
    }
}

fn timestamp_id(label: &str) -> String {
    label
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .take(14)
        .collect()
}

fn now_label() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis();
    format!("{millis}")
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            if slug.len() >= 50 {
                break;
            }
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "request".to_string()
    } else {
        slug
    }
}

fn random_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_nanos(0))
        .as_nanos();
    format!("{:06x}", (nanos ^ std::process::id() as u128) & 0x00ff_ffff)
}

fn blocker(kind: &str, ids: Vec<String>, message: String) -> QueueBlocker {
    QueueBlocker {
        kind: kind.to_string(),
        ids,
        message,
    }
}

pub fn candidate_to_json(candidate: Option<CommitRequest>) -> Value {
    candidate
        .map(|request| json!(request))
        .unwrap_or(Value::Null)
}
