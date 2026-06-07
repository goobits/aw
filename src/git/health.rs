use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AwError, Result};
use crate::paths::path_string;
use crate::queue_lock::{self, QueueLock};

use super::{
    count_lines, run_capture_git, run_git_status, run_with_isolated_index,
    run_with_isolated_index_capture, timestamp, QUEUE_NAME,
};

pub(super) fn health(args: &[String]) -> Result<i32> {
    let mut deep = false;
    let mut recursive = false;
    for arg in args {
        match arg.as_str() {
            "--deep" => deep = true,
            "--recursive" => recursive = true,
            other => return Err(AwError::new(format!("Unknown health option: {other}"), 1)),
        }
    }
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        "gitq health",
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    let mut failed = !print_repo_health(&repo_root, ".", deep)?;
    let locks = [".git/index.lock", ".git/pkgq.lock"]
        .iter()
        .filter(|lock| repo_root.join(lock).exists())
        .copied()
        .collect::<Vec<_>>();
    if locks.is_empty() {
        println!("locks: none");
    } else {
        println!("locks: {}", locks.join(", "));
        failed = true;
    }
    if recursive {
        for nested in find_nested_repos(&repo_root)? {
            if !print_repo_health(&nested.absolute_path, &nested.relative_path, deep)? {
                failed = true;
            }
        }
    }
    Ok(if failed { 1 } else { 0 })
}

pub(super) fn repair_index(args: &[String]) -> Result<i32> {
    let mut recursive = false;
    for arg in args {
        match arg.as_str() {
            "--recursive" => recursive = true,
            other => {
                return Err(AwError::new(
                    format!("Unknown repair-index option: {other}"),
                    1,
                ))
            }
        }
    }
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        if recursive {
            "gitq repair-index --recursive"
        } else {
            "gitq repair-index"
        },
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    let mut failed = !repair_index_for_repo(&repo_root, ".")?;
    if recursive {
        for nested in find_nested_repos(&repo_root)? {
            if !repair_index_for_repo(&nested.absolute_path, &nested.relative_path)? {
                failed = true;
            }
        }
    }
    Ok(if failed { 1 } else { 0 })
}

fn print_repo_health(repo_root: &Path, label: &str, deep: bool) -> Result<bool> {
    let index_count = count_lines(&run_capture_git(
        &["ls-files".to_string(), "--stage".to_string()],
        repo_root,
        true,
    )?);
    let head_count = count_lines(&run_capture_git(
        &[
            "ls-tree".to_string(),
            "-r".to_string(),
            "--name-only".to_string(),
            "HEAD".to_string(),
        ],
        repo_root,
        false,
    )?);
    let staged = run_with_isolated_index_capture(
        repo_root,
        &[
            "diff".to_string(),
            "--cached".to_string(),
            "--name-status".to_string(),
        ],
    )?;
    let mut failed = false;
    println!("[{label}] index entries: {index_count}");
    println!("[{label}] HEAD entries: {head_count}");
    println!(
        "[{label}] staged changes: {}",
        if staged.trim().is_empty() {
            "no"
        } else {
            "yes"
        }
    );
    if index_count != head_count {
        eprintln!(
            "[{label}] Index entry count does not match HEAD ({index_count} != {head_count})."
        );
        failed = true;
    }
    if !staged.trim().is_empty() {
        failed = true;
    }
    if deep {
        let status = run_git_status(
            &["fsck".to_string(), "--no-dangling".to_string()],
            repo_root,
            true,
        )?;
        if status != 0 {
            failed = true;
        }
    } else {
        println!("[{label}] deep fsck: skipped");
    }
    Ok(!failed)
}

fn repair_index_for_repo(repo_root: &Path, label: &str) -> Result<bool> {
    let git_dir = run_capture_git(
        &["rev-parse".to_string(), "--git-dir".to_string()],
        repo_root,
        true,
    )?;
    let index_path = repo_root.join(git_dir).join("index");
    let backup_path =
        index_path.with_extension(format!("backup-before-gitq-repair-{}", timestamp()));
    if index_path.exists() {
        fs::copy(&index_path, &backup_path)?;
        eprintln!(
            "[{label}] Backed up current index to {}",
            path_string(&backup_path)
        );
    }
    if index_path.with_extension("lock").exists() {
        eprintln!(
            "[{label}] Refusing to repair while {} exists.",
            path_string(&index_path.with_extension("lock"))
        );
        return Ok(false);
    }
    let status = run_git_status(
        &["read-tree".to_string(), "HEAD".to_string()],
        repo_root,
        false,
    )?;
    if status != 0 {
        return Ok(false);
    }
    let installed_count = count_lines(&run_capture_git(
        &["ls-files".to_string(), "--stage".to_string()],
        repo_root,
        true,
    )?);
    println!("[{label}] Installed repaired index with {installed_count} entries.");
    let diff_status = run_with_isolated_index(
        repo_root,
        &[
            "diff".to_string(),
            "--cached".to_string(),
            "--name-status".to_string(),
        ],
        repo_root.to_path_buf(),
        true,
    )?;
    Ok(diff_status == 0)
}

fn find_nested_repos(repo_root: &Path) -> Result<Vec<NestedRepo>> {
    let mut repos = BTreeSet::new();
    let mut output = Vec::new();
    add_declared_submodules(repo_root, repo_root, &mut repos, &mut output)?;
    visit_nested(repo_root, repo_root, &mut repos, &mut output)?;
    output.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(output)
}

fn add_declared_submodules(
    owner_root: &Path,
    repo_root: &Path,
    seen: &mut BTreeSet<PathBuf>,
    output: &mut Vec<NestedRepo>,
) -> Result<()> {
    for submodule in read_declared_submodule_paths(owner_root)? {
        let absolute = owner_root.join(&submodule);
        if add_nested_repo(&absolute, repo_root, seen, output) {
            add_declared_submodules(&absolute, repo_root, seen, output)?;
        }
    }
    Ok(())
}

fn visit_nested(
    dir: &Path,
    repo_root: &Path,
    seen: &mut BTreeSet<PathBuf>,
    output: &mut Vec<NestedRepo>,
) -> Result<()> {
    let skip = [
        ".git",
        ".llm",
        ".secrets",
        "node_modules",
        ".pnpm-store",
        ".turbo",
        ".svelte-kit",
        "dist",
        "build",
        "formats-for-testing",
        "target",
        "tools",
    ];
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        let absolute = entry.path();
        if absolute.join(".git").exists() && add_nested_repo(&absolute, repo_root, seen, output) {
            add_declared_submodules(&absolute, repo_root, seen, output)?;
        }
        visit_nested(&absolute, repo_root, seen, output)?;
    }
    Ok(())
}

fn add_nested_repo(
    absolute: &Path,
    repo_root: &Path,
    seen: &mut BTreeSet<PathBuf>,
    output: &mut Vec<NestedRepo>,
) -> bool {
    let resolved = absolute.to_path_buf();
    if resolved == repo_root || seen.contains(&resolved) || !resolved.join(".git").exists() {
        return false;
    }
    seen.insert(resolved.clone());
    output.push(NestedRepo {
        absolute_path: resolved.clone(),
        relative_path: resolved
            .strip_prefix(repo_root)
            .unwrap_or(&resolved)
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/"),
    });
    true
}

fn read_declared_submodule_paths(repo_root: &Path) -> Result<Vec<String>> {
    if !repo_root.join(".gitmodules").exists() {
        return Ok(Vec::new());
    }
    let output = Command::new("git")
        .args([
            "config",
            "--file",
            ".gitmodules",
            "--get-regexp",
            "^submodule\\..*\\.path$",
        ])
        .current_dir(repo_root)
        .output()?;
    if output.status.code() == Some(1) {
        return Ok(Vec::new());
    }
    if !output.status.success() {
        return Err(AwError::new(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            output.status.code().unwrap_or(1),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1).map(ToString::to_string))
        .collect())
}

struct NestedRepo {
    absolute_path: PathBuf,
    relative_path: String,
}
