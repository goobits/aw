use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::commit_queue::types::CommitRequest;
use crate::error::{AwError, Result};

pub(super) fn missing_paths(paths: &[String], repo_root: &Path) -> Vec<String> {
    paths
        .iter()
        .filter_map(|requested_path| normalize_repo_path(requested_path, repo_root).ok())
        .filter(|requested_path| {
            !repo_root.join(requested_path).exists() && !is_tracked_path(requested_path, repo_root)
        })
        .collect()
}

fn is_tracked_path(requested_path: &str, repo_root: &Path) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", "--", requested_path])
        .current_dir(repo_root)
        .output()
        .is_ok_and(|output| output.status.success())
}

pub(super) fn fingerprint_mismatches(request: &CommitRequest, repo_root: &Path) -> Vec<String> {
    let Some(fingerprints) = &request.fingerprints else {
        return Vec::new();
    };
    if fingerprints.must_contain.is_empty() && fingerprints.must_not_contain.is_empty() {
        return Vec::new();
    }
    let contents = files_for_fingerprint(&request.paths, repo_root)
        .into_iter()
        .filter_map(|path| fs::read_to_string(repo_root.join(path)).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let mut mismatches = Vec::new();
    for expected in &fingerprints.must_contain {
        if !contents.contains(expected) {
            mismatches.push(format!("{} missing fingerprint: {expected}", request.id));
        }
    }
    for forbidden in &fingerprints.must_not_contain {
        if contents.contains(forbidden) {
            mismatches.push(format!(
                "{} still contains forbidden fingerprint: {forbidden}",
                request.id
            ));
        }
    }
    mismatches
}

fn files_for_fingerprint(paths: &[String], repo_root: &Path) -> Vec<String> {
    let mut files = BTreeSet::new();
    for requested_path in paths {
        let Ok(normalized) = normalize_repo_path(requested_path, repo_root) else {
            continue;
        };
        let absolute = repo_root.join(&normalized);
        if absolute.is_file() {
            files.insert(normalized);
            continue;
        }
        let Ok(output) = Command::new("git")
            .args(["ls-files", "--", &normalized])
            .current_dir(repo_root)
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if repo_root.join(line).is_file() {
                files.insert(line.to_string());
            }
        }
    }
    files.into_iter().collect()
}

pub(super) fn overlapping_path_owners(
    path_owners: &[(String, CommitRequest)],
) -> Vec<(String, Vec<String>)> {
    let mut groups = BTreeMap::<String, BTreeSet<String>>::new();
    for (index, (left_path, left_request)) in path_owners.iter().enumerate() {
        for (right_path, right_request) in path_owners.iter().skip(index + 1) {
            if !paths_overlap(left_path, right_path) {
                continue;
            }
            let mut ordered = [left_path.clone(), right_path.clone()];
            ordered.sort_by(|left, right| left.len().cmp(&right.len()).then(left.cmp(right)));
            let requested_path = if ordered[0] == ordered[1] {
                ordered[0].clone()
            } else {
                format!("{} <-> {}", ordered[0], ordered[1])
            };
            groups
                .entry(requested_path)
                .or_default()
                .extend([left_request.id.clone(), right_request.id.clone()]);
        }
    }
    groups
        .into_iter()
        .map(|(path, owners)| (path, owners.into_iter().collect()))
        .collect()
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left == "."
        || right == "."
        || left.starts_with(&format!("{right}/"))
        || right.starts_with(&format!("{left}/"))
}

pub(super) fn normalize_repo_path(value: &str, repo_root: &Path) -> Result<String> {
    if value.is_empty() || value.starts_with('-') {
        return Err(AwError::new(format!("invalid path: {value}"), 1));
    }
    let path = PathBuf::from(value);
    let absolute = if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    };
    let absolute = normalize_path(&absolute);
    let repo_root = normalize_path(repo_root);
    let relative = absolute
        .strip_prefix(&repo_root)
        .map_err(|_| AwError::new(format!("path is outside repository: {value}"), 1))?;
    let normalized = relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    Ok(if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
