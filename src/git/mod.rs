use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{AwError, Result};
use crate::help;
use crate::paths::path_string;
use crate::queue_lock::{self, QueueLock};

mod health;
mod owned_commit;
mod raw_policy;

use raw_policy::{assert_allowed_raw_git, is_lockless_read_allowed, parse_git_invocation};

const QUEUE_NAME: &str = "gitq";

pub fn run(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        print_usage();
        return Ok(0);
    };
    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(0)
        }
        "lock-info" => print_lock_info(),
        "status" => {
            let mut git_args = vec!["status".to_string()];
            git_args.extend(rest.iter().cloned());
            with_git_queue(
                &git_args,
                GitOptions {
                    isolated_index: false,
                    lockless_read: true,
                },
            )
        }
        "status-fast" => {
            let mut git_args = vec![
                "status".to_string(),
                "--short".to_string(),
                "--untracked-files=no".to_string(),
                "--ignore-submodules=dirty".to_string(),
            ];
            git_args.extend(rest.iter().cloned());
            with_git_queue(
                &git_args,
                GitOptions {
                    isolated_index: false,
                    lockless_read: true,
                },
            )
        }
        "shell" => run_shell(),
        "commit-owned" => owned_commit::commit_owned(rest),
        "health" => health::health(rest),
        "repair-index" => health::repair_index(rest),
        "chmod" => chmod(rest),
        "fetch" | "push" | "worktree" | "clone" => {
            let mut git_args = vec![command.clone()];
            git_args.extend(rest.iter().cloned());
            run_queued_mutation(&git_args)
        }
        "lfs-push" => {
            let mut git_args = vec!["lfs".to_string(), "push".to_string()];
            git_args.extend(rest.iter().cloned());
            run_queued_mutation(&git_args)
        }
        "submodule-sync" => {
            let mut git_args = vec!["submodule".to_string(), "sync".to_string()];
            git_args.extend(rest.iter().cloned());
            run_queued_mutation(&git_args)
        }
        "submodule-update" => {
            let mut git_args = vec!["submodule".to_string(), "update".to_string()];
            git_args.extend(rest.iter().cloned());
            run_queued_mutation(&git_args)
        }
        "maintenance" => maintenance(),
        "submodule-status" => with_git_queue(
            &[
                "submodule".to_string(),
                "status".to_string(),
                "--recursive".to_string(),
            ],
            GitOptions {
                isolated_index: true,
                lockless_read: false,
            },
        ),
        "--" => {
            if rest.is_empty() {
                return Err(AwError::new("gitq -- requires git arguments", 1));
            }
            assert_allowed_raw_git(rest)?;
            with_git_queue(
                rest,
                GitOptions {
                    isolated_index: false,
                    lockless_read: true,
                },
            )
        }
        other => Err(AwError::new(format!("Unknown gitq command: {other}"), 1)),
    }
}

#[derive(Clone, Copy)]
struct GitOptions {
    isolated_index: bool,
    lockless_read: bool,
}

fn print_lock_info() -> Result<i32> {
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let info = queue_lock::read_lock_info(QUEUE_NAME, &repo_root)?;
    if info.is_null() {
        println!("gitq: no active queue lock");
    } else {
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    }
    Ok(0)
}

fn with_git_queue(git_args: &[String], options: GitOptions) -> Result<i32> {
    let invocation = parse_git_invocation(git_args)?;
    let repo_root = queue_lock::find_repo_root(&invocation.cwd)?;
    let isolated_index =
        options.isolated_index || raw_policy::uses_isolated_index(&invocation.command);
    if options.lockless_read && is_lockless_read_allowed(git_args, &invocation) {
        return if isolated_index {
            run_with_isolated_index(&repo_root, git_args, env::current_dir()?, true)
        } else {
            run_git_status(git_args, &env::current_dir()?, true)
        };
    }

    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        &format!("git {}", git_args.join(" ")),
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    if isolated_index {
        run_with_isolated_index(&repo_root, git_args, env::current_dir()?, true)
    } else {
        run_git_status(git_args, &env::current_dir()?, false)
    }
}

fn run_queued_mutation(git_args: &[String]) -> Result<i32> {
    let invocation = parse_git_invocation(git_args)?;
    let repo_root = queue_lock::find_repo_root(&invocation.cwd)?;
    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        &format!("git {}", git_args.join(" ")),
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    run_git_status(git_args, &env::current_dir()?, false)
}

fn run_shell() -> Result<i32> {
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let _lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        "gitq shell",
        queue_lock::timeout_from_env("GITQ_TIMEOUT_MS")?,
    )?;
    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    eprintln!("gitq: lock acquired. Exit the shell to release it.");
    queue_lock::run_status(&shell, &[], &env::current_dir()?, &[("GITQ_ACTIVE", "1")])
}

fn chmod(args: &[String]) -> Result<i32> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(AwError::new("gitq chmod requires +x or -x", 1));
    };
    if mode != "+x" && mode != "-x" {
        return Err(AwError::new("gitq chmod requires +x or -x", 1));
    }
    if rest.first().map(String::as_str) != Some("--") {
        return Err(AwError::new("gitq chmod requires -- before paths", 1));
    }
    let paths = rest[1..].to_vec();
    if paths.is_empty() {
        return Err(AwError::new("gitq chmod requires at least one path", 1));
    }
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let owned_paths = owned_commit::normalize_owned_paths(&paths, &repo_root)?;
    let mut git_args = vec![
        "update-index".to_string(),
        if mode == "+x" {
            "--chmod=+x"
        } else {
            "--chmod=-x"
        }
        .to_string(),
        "--".to_string(),
    ];
    git_args.extend(owned_paths);
    run_queued_mutation(&git_args)
}

fn maintenance() -> Result<i32> {
    let first = run_queued_mutation(&[
        "maintenance".to_string(),
        "run".to_string(),
        "--task=commit-graph".to_string(),
        "--task=loose-objects".to_string(),
        "--task=incremental-repack".to_string(),
    ])?;
    if first != 0 {
        return Ok(first);
    }
    run_queued_mutation(&["prune-packed".to_string()])
}

pub(super) fn run_with_isolated_index(
    repo_root: &Path,
    git_args: &[String],
    cwd: PathBuf,
    inherit: bool,
) -> Result<i32> {
    let temp_dir =
        env::temp_dir().join(format!("gitq-index-{}-{}", std::process::id(), timestamp()));
    fs::create_dir_all(&temp_dir)?;
    let temp_index = temp_dir.join("index");
    let result = (|| {
        let git_dir = run_capture_git(
            &["rev-parse".to_string(), "--git-dir".to_string()],
            repo_root,
            true,
        )?;
        let real_index = repo_root.join(git_dir).join("index");
        if real_index.exists() {
            fs::copy(&real_index, &temp_index)?;
        } else {
            let seed_args = if has_head(repo_root)? {
                vec!["read-tree".to_string(), "HEAD".to_string()]
            } else {
                vec!["read-tree".to_string(), "--empty".to_string()]
            };
            let status = run_git_with_env(
                &seed_args,
                repo_root,
                &[("GIT_INDEX_FILE", path_string(&temp_index).as_str())],
                false,
            )?;
            if status != 0 {
                return Ok(status);
            }
        }
        run_git_with_env(
            git_args,
            &cwd,
            &[
                ("GIT_INDEX_FILE", path_string(&temp_index).as_str()),
                ("GIT_OPTIONAL_LOCKS", "0"),
            ],
            inherit,
        )
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

pub(super) fn run_with_isolated_index_capture(
    repo_root: &Path,
    git_args: &[String],
) -> Result<String> {
    let temp_dir =
        env::temp_dir().join(format!("gitq-index-{}-{}", std::process::id(), timestamp()));
    fs::create_dir_all(&temp_dir)?;
    let temp_index = temp_dir.join("index");
    let result = (|| {
        let git_dir = run_capture_git(
            &["rev-parse".to_string(), "--git-dir".to_string()],
            repo_root,
            true,
        )?;
        let real_index = repo_root.join(git_dir).join("index");
        if real_index.exists() {
            fs::copy(&real_index, &temp_index)?;
        } else if has_head(repo_root)? {
            let _ = run_git_with_env(
                &["read-tree".to_string(), "HEAD".to_string()],
                repo_root,
                &[("GIT_INDEX_FILE", path_string(&temp_index).as_str())],
                false,
            )?;
        }
        run_capture_git_with_env(
            git_args,
            repo_root,
            &[
                ("GIT_INDEX_FILE", path_string(&temp_index).as_str()),
                ("GIT_OPTIONAL_LOCKS", "0"),
            ],
        )
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

pub(super) fn run_git_status(git_args: &[String], cwd: &Path, read_only: bool) -> Result<i32> {
    run_git_with_env(
        git_args,
        cwd,
        if read_only {
            &[("GIT_OPTIONAL_LOCKS", "0")]
        } else {
            &[]
        },
        true,
    )
}

fn run_git_with_env(
    git_args: &[String],
    cwd: &Path,
    envs: &[(&str, &str)],
    inherit: bool,
) -> Result<i32> {
    let mut command = Command::new("git");
    command
        .args(git_args)
        .current_dir(cwd)
        .env("GITQ_ACTIVE", "1");
    for (key, value) in envs {
        command.env(key, value);
    }
    if !inherit {
        command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit());
    }
    let status = command.status()?;
    Ok(status.code().unwrap_or(1))
}

pub(super) fn run_capture_git(git_args: &[String], cwd: &Path, read_only: bool) -> Result<String> {
    run_capture_git_with_env(
        git_args,
        cwd,
        if read_only {
            &[("GIT_OPTIONAL_LOCKS", "0")]
        } else {
            &[]
        },
    )
}

fn run_capture_git_with_env(
    git_args: &[String],
    cwd: &Path,
    envs: &[(&str, &str)],
) -> Result<String> {
    let mut command = Command::new("git");
    command
        .args(git_args)
        .current_dir(cwd)
        .env("GITQ_ACTIVE", "1");
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(AwError::new(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            output.status.code().unwrap_or(1),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(super) fn has_head(repo_root: &Path) -> Result<bool> {
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(repo_root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .status()?;
    Ok(status.success())
}

pub(super) fn count_lines(text: &str) -> usize {
    text.lines().filter(|line| !line.trim().is_empty()).count()
}

pub(super) fn print_list(items: &[String]) {
    for item in items {
        eprintln!("  {item}");
    }
}

pub(super) fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis()
        .to_string()
}

fn print_usage() {
    help::println(
        "Usage:
  aw owner git status [git status args]
  aw owner git status-fast [git status args]
  aw owner git lock-info
  aw owner git health [--deep] [--recursive]
  aw owner git repair-index [--recursive]
  aw owner git chmod +x| -x -- <paths...>
  aw owner git fetch <git fetch args...>
  aw owner git push <git push args...>
  aw owner git lfs-push <git lfs push args...>
  aw owner git worktree <git worktree args...>
  aw owner git clone <git clone args...>
  aw owner git submodule-sync <git submodule sync args...>
  aw owner git submodule-update <git submodule update args...>
  aw owner git maintenance
  aw owner git submodule-status
  aw owner git shell
  aw owner git commit-owned -m \"message\" -- <owned paths...>
  aw owner git -- <read-only raw git args...>",
    );
}
