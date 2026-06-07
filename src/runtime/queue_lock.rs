use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::error::{AwError, Result};
use crate::paths::path_string;

const DEFAULT_TIMEOUT_MS: u64 = 5 * 60 * 1000;
const POLL_MS: u64 = 250;

pub struct QueueLock {
    lock_path: PathBuf,
    meta_path: PathBuf,
    token: String,
}

impl QueueLock {
    pub fn acquire(
        queue_name: &str,
        repo_root: &Path,
        command: &str,
        timeout: Duration,
    ) -> Result<Self> {
        let git_dir = find_git_dir(repo_root)?;
        let lock_path = git_dir.join(format!("{queue_name}.lock"));
        let meta_path = git_dir.join(format!("{queue_name}.lock.json"));
        let token = random_token();
        let started = SystemTime::now();
        let metadata = json!({
            "queue": queue_name,
            "pid": std::process::id(),
            "token": token,
            "user": env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            "agent": env::var("CODEX_SESSION_ID").ok().or_else(|| env::var("CURSOR_AGENT_ID").ok()),
            "command": command,
            "cwd": env::current_dir().ok().map(|path| path_string(&path)),
            "repoRoot": path_string(repo_root),
            "startedAt": millis_since_epoch(),
        });
        let mut warned = false;

        loop {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    writeln!(file, "{} {}", std::process::id(), token)?;
                    fs::write(
                        &meta_path,
                        format!("{}\n", serde_json::to_string_pretty(&metadata).unwrap()),
                    )?;
                    return Ok(Self {
                        lock_path,
                        meta_path,
                        token,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let owner = read_lock_owner_paths(&lock_path, &meta_path);
                    if owner
                        .get("pid")
                        .and_then(Value::as_u64)
                        .is_some_and(|pid| !process_alive(pid as u32))
                    {
                        eprintln!(
                            "{}: removing stale lock from dead pid {}",
                            queue_name,
                            owner.get("pid").and_then(Value::as_u64).unwrap_or(0)
                        );
                        remove_if_exists(&lock_path)?;
                        remove_if_exists(&meta_path)?;
                        continue;
                    }
                    let elapsed = SystemTime::now()
                        .duration_since(started)
                        .unwrap_or_else(|_| Duration::from_millis(0));
                    if elapsed >= timeout {
                        return Err(AwError::new(
                            format!(
                                "{queue_name}: timed out waiting for lock after {}ms\n{}",
                                elapsed.as_millis(),
                                describe_owner(&owner, &lock_path)
                            ),
                            1,
                        ));
                    }
                    if !warned {
                        eprintln!("{queue_name}: waiting for active queue lock");
                        eprintln!("{}", describe_owner(&owner, &lock_path));
                        warned = true;
                    }
                    thread::sleep(Duration::from_millis(POLL_MS));
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    pub fn release(&self) -> Result<()> {
        let owner = read_lock_owner_paths(&self.lock_path, &self.meta_path);
        if owner
            .get("token")
            .and_then(Value::as_str)
            .is_some_and(|token| token != self.token)
        {
            eprintln!(
                "Not releasing {}; it is owned by another queue token.",
                path_string(&self.lock_path)
            );
            return Ok(());
        }
        if owner.get("token").is_none()
            && owner
                .get("pid")
                .and_then(Value::as_u64)
                .is_some_and(|pid| pid != std::process::id() as u64)
        {
            eprintln!(
                "Not releasing {}; it is owned by another pid.",
                path_string(&self.lock_path)
            );
            return Ok(());
        }
        remove_if_exists(&self.meta_path)?;
        remove_if_exists(&self.lock_path)?;
        Ok(())
    }
}

impl Drop for QueueLock {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

pub fn timeout_from_env(env_name: &str) -> Result<Duration> {
    let Some(raw) = env::var_os(env_name) else {
        return Ok(Duration::from_millis(DEFAULT_TIMEOUT_MS));
    };
    let value = raw.to_string_lossy().parse::<u64>().map_err(|_| {
        AwError::new(
            format!("{env_name} must be a non-negative number of milliseconds"),
            1,
        )
    })?;
    Ok(Duration::from_millis(value))
}

pub fn read_lock_info(queue_name: &str, repo_root: &Path) -> Result<Value> {
    let git_dir = find_git_dir(repo_root)?;
    let lock_path = git_dir.join(format!("{queue_name}.lock"));
    let meta_path = git_dir.join(format!("{queue_name}.lock.json"));
    if !lock_path.exists() {
        return Ok(Value::Null);
    }
    let owner = read_lock_owner_paths(&lock_path, &meta_path);
    Ok(if owner.is_null() {
        json!({ "lockPath": path_string(&lock_path) })
    } else {
        owner
    })
}

pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(AwError::new(
                format!(
                    "Could not find a Git repository above {}",
                    path_string(start)
                ),
                1,
            ));
        }
    }
}

pub fn find_git_dir(repo_root: &Path) -> Result<PathBuf> {
    let dot_git = repo_root.join(".git");
    if dot_git.is_dir() {
        return Ok(dot_git);
    }
    let content = fs::read_to_string(&dot_git)?;
    let value = content
        .trim()
        .strip_prefix("gitdir:")
        .map(str::trim)
        .ok_or_else(|| {
            AwError::new(
                format!("Unsupported .git file format at {}", path_string(&dot_git)),
                1,
            )
        })?;
    Ok(repo_root.join(value))
}

pub fn run_status(
    command: &str,
    args: &[String],
    cwd: &Path,
    envs: &[(&str, &str)],
) -> Result<i32> {
    let mut cmd = Command::new(command);
    cmd.args(args).current_dir(cwd);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

fn read_lock_owner_paths(lock_path: &Path, meta_path: &Path) -> Value {
    if let Ok(text) = fs::read_to_string(meta_path) {
        if let Ok(value) = serde_json::from_str::<Value>(&text) {
            return value;
        }
    }
    if let Ok(text) = fs::read_to_string(lock_path) {
        let parts = text.split_whitespace().collect::<Vec<_>>();
        if let Some(pid) = parts.first().and_then(|value| value.parse::<u64>().ok()) {
            return json!({
                "pid": pid,
                "token": parts.get(1).copied(),
                "source": path_string(lock_path),
            });
        }
    }
    Value::Null
}

fn describe_owner(owner: &Value, lock_path: &Path) -> String {
    if owner.is_null() {
        return format!(
            "Lock: {}\nOwner metadata: unavailable",
            path_string(lock_path)
        );
    }
    [
        format!("Lock: {}", path_string(lock_path)),
        format!("Owner pid: {}", value_label(owner, "pid")),
        format!("Owner user: {}", value_label(owner, "user")),
        format!("Owner agent: {}", value_label(owner, "agent")),
        format!("Started: {}", value_label(owner, "startedAt")),
        format!("Command: {}", value_label(owner, "command")),
        format!("Cwd: {}", value_label(owner, "cwd")),
    ]
    .join("\n")
}

fn value_label(value: &Value, key: &str) -> String {
    value
        .get(key)
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .unwrap_or_else(|| value.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn millis_since_epoch() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis()
}

fn random_token() -> String {
    format!(
        "{:x}-{:x}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_nanos(0))
            .as_nanos()
    )
}
