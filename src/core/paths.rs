use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AwError, Result};

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn current_dir() -> Result<PathBuf> {
    Ok(env::current_dir()?)
}

pub fn helper_path(executable: &str) -> PathBuf {
    if let Some(bin) = env::var_os("ZELLIJ_WORKSPACES_BIN") {
        return PathBuf::from(bin).join(executable);
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(executable);
            if sibling.is_file() {
                return sibling;
            }
        }
    }

    home_dir()
        .join(".local/share/agent-workspace/bin")
        .join(executable)
}

pub fn resolve_root(path: &str) -> Result<PathBuf> {
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(AwError::new(
            format!("aw: root directory does not exist: {}", path),
            1,
        ));
    }
    Ok(root.canonicalize()?)
}

pub fn validate_name(kind: &str, value: &str) -> Result<()> {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        return Ok(());
    }

    Err(AwError::new(
        format!(
            "aw: {} may only use letters, numbers, dot, underscore, and dash: {}",
            kind, value
        ),
        2,
    ))
}

pub fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':'))
    {
        return value.to_string();
    }

    let mut quoted = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':') {
            quoted.push(ch);
        } else {
            quoted.push('\\');
            quoted.push(ch);
        }
    }
    quoted
}

pub fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(unix)]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_file() && fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
pub fn is_executable(path: &Path) -> bool {
    path.is_file()
}
