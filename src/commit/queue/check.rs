use std::path::Path;

use crate::commit_queue::types::{CommitRequest, QueueBlocker, QueueReport};
use crate::error::{AwError, Result};
use crate::paths::path_string;

use super::paths::{
    fingerprint_mismatches, missing_paths, normalize_repo_path, overlapping_path_owners,
};
use super::store::read_queue;

pub(super) fn check_queue(queue_root: &Path, repo_root: &Path) -> Result<QueueReport> {
    let queue = read_queue(queue_root, "pending");
    let requests = queue.requests;
    let mut blockers = queue.blockers;
    let mut path_owners = Vec::<(String, CommitRequest)>::new();

    for request in &requests {
        let file_id = Path::new(&request.file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if !file_id.is_empty() && !request.id.is_empty() && request.id != file_id {
            blockers.push(blocker(
                "invalid",
                vec![request.id.clone()],
                format!(
                    "request id mismatch: file {file_id} contains {}",
                    request.id
                ),
            ));
            continue;
        }
        let validation = validate_request(request);
        for message in &validation {
            blockers.push(blocker(
                "invalid",
                vec![request.id.clone()],
                message.clone(),
            ));
        }
        if !validation.is_empty() {
            continue;
        }
        for state in ["done", "blocked"] {
            let terminal_file = queue_root.join(state).join(format!("{}.json", request.id));
            if terminal_file.exists() {
                blockers.push(blocker(
                    "terminal-duplicate",
                    vec![request.id.clone()],
                    format!("{} already exists in {state}", request.id),
                ));
            }
        }
        let mut normalized_paths = Vec::new();
        for requested_path in &request.paths {
            match normalize_repo_path(requested_path, repo_root) {
                Ok(normalized) => {
                    normalized_paths.push(normalized.clone());
                    path_owners.push((normalized, request.clone()));
                }
                Err(error) => blockers.push(blocker(
                    "invalid",
                    vec![request.id.clone()],
                    format!(
                        "{} has invalid path {}: {}",
                        request.id, requested_path, error.message
                    ),
                )),
            }
        }
        for missing in missing_paths(&normalized_paths, repo_root) {
            blockers.push(blocker(
                "missing-path",
                vec![request.id.clone()],
                format!("{} references missing path {missing}", request.id),
            ));
        }
        for mismatch in fingerprint_mismatches(request, repo_root) {
            blockers.push(blocker("fingerprint", vec![request.id.clone()], mismatch));
        }
    }

    for (requested_path, owners) in overlapping_path_owners(&path_owners) {
        if owners.len() <= 1 {
            continue;
        }
        blockers.push(blocker(
            "overlap",
            owners.clone(),
            format!("{} is claimed by {}", requested_path, owners.join(", ")),
        ));
    }

    Ok(QueueReport {
        queue_root: path_string(queue_root),
        pending: requests.len(),
        blockers,
    })
}

fn validate_request(request: &CommitRequest) -> Vec<String> {
    let mut messages = Vec::new();
    if request.id.is_empty() {
        messages.push("request is missing id".to_string());
    }
    if request.title.is_empty() {
        messages.push(format!("{} is missing title", request_name(request)));
    }
    if request.paths.is_empty() {
        messages.push(format!("{} is missing paths", request_name(request)));
    }
    if let Some(fingerprints) = &request.fingerprints {
        if fingerprints
            .must_contain
            .iter()
            .any(|value| value.is_empty())
        {
            messages.push(format!(
                "{} fingerprints.must_contain entries must be non-empty strings",
                request_name(request)
            ));
        }
        if fingerprints
            .must_not_contain
            .iter()
            .any(|value| value.is_empty())
        {
            messages.push(format!(
                "{} fingerprints.must_not_contain entries must be non-empty strings",
                request_name(request)
            ));
        }
    }
    messages
}

pub(super) fn validate_move_request(
    queue_root: &Path,
    source: &Path,
    request: &CommitRequest,
    state: &str,
    repo_root: &Path,
) -> Result<()> {
    let id = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    if request.id != id {
        return Err(AwError::new(
            format!(
                "commitq: request id mismatch: file {id} contains {}",
                if request.id.is_empty() {
                    "missing id"
                } else {
                    &request.id
                }
            ),
            1,
        ));
    }
    if request.done_at.is_some() || request.blocked_at.is_some() {
        return Err(AwError::new(
            format!(
                "commitq: {} already has terminal metadata in pending queue",
                request.id
            ),
            1,
        ));
    }
    let validation = validate_request(request);
    if !validation.is_empty() {
        return Err(AwError::new(
            format!("commitq: {}", validation.join("; ")),
            1,
        ));
    }
    if state == "done" {
        let missing = missing_paths(&request.paths, repo_root);
        if !missing.is_empty() {
            return Err(AwError::new(
                format!(
                    "commitq: {} references missing path {}",
                    request.id,
                    missing.join(", ")
                ),
                1,
            ));
        }
        let fingerprints = fingerprint_mismatches(request, repo_root);
        if !fingerprints.is_empty() {
            return Err(AwError::new(
                format!("commitq: {}", fingerprints.join("; ")),
                1,
            ));
        }
    }
    for terminal_state in ["done", "blocked"] {
        let terminal_file = queue_root
            .join(terminal_state)
            .join(source.file_name().unwrap_or_default());
        if terminal_file.exists() {
            return Err(AwError::new(
                format!("commitq: {} already exists in {terminal_state}", request.id),
                1,
            ));
        }
    }
    Ok(())
}

fn request_name(request: &CommitRequest) -> &str {
    if request.id.is_empty() {
        "request"
    } else {
        &request.id
    }
}

fn blocker(kind: &str, ids: Vec<String>, message: String) -> QueueBlocker {
    QueueBlocker {
        kind: kind.to_string(),
        ids,
        message,
    }
}
