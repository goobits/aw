use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AwError, Result};
use crate::queue_lock::{self, QueueLock};

use super::{
    count_lines, has_head, print_list, run_capture_git, run_git_status,
    run_with_isolated_index_capture, timestamp, QUEUE_NAME,
};

pub(super) fn commit_owned(args: &[String]) -> Result<i32> {
    let parsed = parse_commit_owned_args(args)?;
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let owned_paths = normalize_owned_paths(&parsed.paths, &repo_root)?;
    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        &format!("gitq commit-owned {}", args.join(" ")),
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    if !ensure_index_looks_sane(&repo_root)? {
        return Ok(1);
    }

    let pre_staged = list_staged(&repo_root)?;
    let unrelated_pre_staged = pre_staged
        .iter()
        .filter(|file| !matches_any_spec(file, &owned_paths))
        .cloned()
        .collect::<Vec<_>>();
    if !unrelated_pre_staged.is_empty() {
        eprintln!("Refusing to commit because unrelated files are already staged:");
        print_list(&unrelated_pre_staged);
        return Ok(1);
    }

    let backup = backup_current_index(&repo_root)?;
    let result = commit_owned_inner(&repo_root, &owned_paths, &parsed);
    match &result {
        Ok(0) => remove_index_backup(&backup)?,
        _ => restore_index_backup(&backup)?,
    }
    result
}

fn commit_owned_inner(
    repo_root: &Path,
    owned_paths: &[String],
    parsed: &CommitOwnedArgs,
) -> Result<i32> {
    let add_status = run_git_status(
        &[
            vec!["add".to_string(), "--".to_string()],
            owned_paths.to_vec(),
        ]
        .concat(),
        repo_root,
        false,
    )?;
    if add_status != 0 {
        return Ok(add_status);
    }
    let staged = list_staged(repo_root)?;
    let unrelated_staged = staged
        .iter()
        .filter(|file| !matches_any_spec(file, owned_paths))
        .cloned()
        .collect::<Vec<_>>();
    if !unrelated_staged.is_empty() {
        eprintln!("Refusing to commit because the index contains files outside the owned paths:");
        print_list(&unrelated_staged);
        return Ok(1);
    }
    if !validate_staged_submodule_pointers(repo_root, &staged)? {
        return Ok(1);
    }
    if staged.is_empty() {
        eprintln!("Nothing staged for the requested owned paths.");
        return Ok(1);
    }
    let diff_check = run_diff_check(repo_root)?;
    if diff_check != 0 {
        return Ok(diff_check);
    }
    let mut commit_args = vec!["commit".to_string()];
    if parsed.no_verify {
        commit_args.push("--no-verify".to_string());
    }
    for message in &parsed.messages {
        commit_args.push("-m".to_string());
        commit_args.push(message.clone());
    }
    let status = run_git_status(&commit_args, repo_root, false)?;
    if status == 0 {
        eprintln!("Committed owned paths. Remaining worktree changes were not scanned; run `aw gitq status` if needed.");
    }
    Ok(status)
}

struct CommitOwnedArgs {
    messages: Vec<String>,
    no_verify: bool,
    paths: Vec<String>,
}

fn parse_commit_owned_args(args: &[String]) -> Result<CommitOwnedArgs> {
    let mut messages = Vec::new();
    let mut no_verify = false;
    let mut paths = Vec::new();
    let mut paths_only = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if paths_only {
            paths.push(arg.clone());
            index += 1;
            continue;
        }
        match arg.as_str() {
            "--" => {
                paths_only = true;
                index += 1;
            }
            "--no-verify" => {
                no_verify = true;
                index += 1;
            }
            "-m" | "--message" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| AwError::new(format!("{arg} requires a commit message"), 1))?;
                messages.push(value.clone());
                index += 2;
            }
            other if other.starts_with("--message=") => {
                messages.push(other["--message=".len()..].to_string());
                index += 1;
            }
            other => {
                paths.push(other.to_string());
                index += 1;
            }
        }
    }
    if messages.is_empty() {
        return Err(AwError::new("commit-owned requires -m/--message", 1));
    }
    if paths.is_empty() {
        return Err(AwError::new(
            "commit-owned requires at least one owned path",
            1,
        ));
    }
    Ok(CommitOwnedArgs {
        messages,
        no_verify,
        paths,
    })
}

pub(super) fn normalize_owned_paths(paths: &[String], repo_root: &Path) -> Result<Vec<String>> {
    paths
        .iter()
        .map(|path| normalize_owned_path(path, repo_root))
        .collect()
}

fn normalize_owned_path(spec: &str, repo_root: &Path) -> Result<String> {
    if spec.starts_with(':') {
        return Err(AwError::new(
            format!("commit-owned does not support Git pathspec magic: {spec}"),
            1,
        ));
    }
    if spec.contains('*') || spec.contains('?') || spec.contains('[') {
        return Err(AwError::new(
            format!("commit-owned requires literal file or directory paths, not globs: {spec}"),
            1,
        ));
    }
    let absolute = if Path::new(spec).is_absolute() {
        PathBuf::from(spec)
    } else {
        env::current_dir()?.join(spec)
    };
    let relative = absolute.strip_prefix(repo_root).map_err(|_| {
        AwError::new(
            format!("commit-owned path is outside the repository: {spec}"),
            1,
        )
    })?;
    let normalized = normalize_spec(
        &relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/"),
    );
    Ok(if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    })
}

fn matches_any_spec(file: &str, specs: &[String]) -> bool {
    specs
        .iter()
        .any(|spec| path_matches_spec(file, &normalize_spec(spec)))
}

fn path_matches_spec(file: &str, spec: &str) -> bool {
    spec.is_empty() || spec == "." || file == spec || file.starts_with(&format!("{spec}/"))
}

fn normalize_spec(spec: &str) -> String {
    spec.trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

fn list_staged(repo_root: &Path) -> Result<Vec<String>> {
    let output = run_with_isolated_index_capture(
        repo_root,
        &[
            "diff".to_string(),
            "--cached".to_string(),
            "--name-only".to_string(),
            "-z".to_string(),
        ],
    )?;
    Ok(output
        .split('\0')
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn backup_current_index(repo_root: &Path) -> Result<IndexBackup> {
    let git_dir = run_capture_git(
        &["rev-parse".to_string(), "--git-dir".to_string()],
        repo_root,
        true,
    )?;
    let index_path = repo_root.join(git_dir).join("index");
    let temp_dir = env::temp_dir().join(format!(
        "gitq-commit-owned-index-{}-{}",
        std::process::id(),
        timestamp()
    ));
    fs::create_dir_all(&temp_dir)?;
    let backup_path = temp_dir.join("index");
    let had_index = index_path.exists();
    if had_index {
        fs::copy(&index_path, &backup_path)?;
    }
    Ok(IndexBackup {
        index_path,
        temp_dir,
        backup_path,
        had_index,
    })
}

fn restore_index_backup(backup: &IndexBackup) -> Result<()> {
    if backup.had_index {
        fs::copy(&backup.backup_path, &backup.index_path)?;
    } else if backup.index_path.exists() {
        fs::remove_file(&backup.index_path)?;
    }
    remove_index_backup(backup)
}

fn remove_index_backup(backup: &IndexBackup) -> Result<()> {
    fs::remove_dir_all(&backup.temp_dir).or_else(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(error)
        }
    })?;
    Ok(())
}

struct IndexBackup {
    index_path: PathBuf,
    temp_dir: PathBuf,
    backup_path: PathBuf,
    had_index: bool,
}

fn validate_staged_submodule_pointers(repo_root: &Path, staged_paths: &[String]) -> Result<bool> {
    let mut ok = true;
    for staged_path in staged_paths {
        let output = run_capture_git(
            &[
                "ls-files".to_string(),
                "--stage".to_string(),
                "--".to_string(),
                staged_path.clone(),
            ],
            repo_root,
            true,
        )?;
        let Some((mode, oid)) = parse_index_entry(&output) else {
            continue;
        };
        if mode != "160000" {
            continue;
        }
        let submodule_root = repo_root.join(staged_path);
        if !submodule_root.join(".git").exists() {
            eprintln!("Refusing to commit submodule pointer {staged_path}: nested repository is not initialized.");
            ok = false;
            continue;
        }
        let nested_head = run_capture_git(
            &["rev-parse".to_string(), "HEAD".to_string()],
            &submodule_root,
            true,
        )?;
        if nested_head != oid {
            eprintln!("Refusing to commit submodule pointer {staged_path}: staged pointer {oid} does not match nested HEAD {nested_head}.");
            ok = false;
        }
        let nested_status = run_capture_git(
            &["status".to_string(), "--short".to_string()],
            &submodule_root,
            true,
        )?;
        if !nested_status.trim().is_empty() {
            eprintln!("Refusing to commit submodule pointer {staged_path}: nested repository has local changes.");
            ok = false;
        }
        if !is_commit_on_remote(&submodule_root, &oid)? {
            eprintln!("Warning: submodule pointer {staged_path} references {oid}, which is not reachable from a remote branch yet.");
            eprintln!("Local parent commits are allowed; push the nested repo commit before pushing or sharing the parent pointer commit.");
        }
    }
    if !ok {
        eprintln!(
            "Commit nested repo work first, ensure it is clean, then commit the parent pointer."
        );
    }
    Ok(ok)
}

fn is_commit_on_remote(repo_root: &Path, oid: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["branch", "-r", "--contains", oid])
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| !line.trim().is_empty() && !line.contains(" -> ")))
}

fn parse_index_entry(output: &str) -> Option<(String, String)> {
    let mut parts = output.split_whitespace();
    let mode = parts.next()?.to_string();
    let oid = parts.next()?.to_string();
    Some((mode, oid))
}

fn ensure_index_looks_sane(repo_root: &Path) -> Result<bool> {
    if !has_head(repo_root)? {
        return Ok(true);
    }
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
    if head_count > 0 && (index_count == 0 || (head_count >= 20 && index_count < head_count / 2)) {
        eprintln!("Refusing to commit because the index looks corrupt ({index_count} entries, HEAD has {head_count}).");
        eprintln!("Run `aw gitq repair-index` first. If submodules are dirty, run `aw gitq repair-index --recursive`.");
        return Ok(false);
    }
    Ok(true)
}

fn run_diff_check(repo_root: &Path) -> Result<i32> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--check", "--no-color"])
        .current_dir(repo_root)
        .output()?;
    if output.status.success() {
        return Ok(0);
    }
    let diagnostics = [
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ]
    .join("\n");
    let blocking = diagnostics
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.ends_with(": new blank line at EOF."))
        .collect::<Vec<_>>();
    if blocking.is_empty() && !diagnostics.trim().is_empty() {
        return Ok(0);
    }
    for line in blocking {
        eprintln!("{line}");
    }
    Ok(output.status.code().unwrap_or(1))
}
