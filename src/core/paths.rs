use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AwError, Result};

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn aw_home() -> PathBuf {
    env::var_os("AW_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".aw"))
}

pub fn local_bin_dir() -> PathBuf {
    home_dir().join(".local/bin")
}

pub fn aw_config_file() -> PathBuf {
    aw_home().join("config.kdl")
}

pub fn aw_default_profile_file() -> PathBuf {
    aw_home().join("default-profile")
}

pub fn aw_profiles_dir() -> PathBuf {
    aw_home().join("profiles")
}

pub fn aw_private_bin_dir() -> PathBuf {
    aw_home().join("bin")
}

pub fn aw_completions_dir() -> PathBuf {
    aw_home().join("completions")
}

pub fn aw_plugins_dir() -> PathBuf {
    aw_home().join("plugins")
}

pub fn aw_default_profile_candidates() -> [PathBuf; 1] {
    [aw_default_profile_file()]
}

pub fn aw_profile_dir_candidates(profile_name: &str) -> [PathBuf; 1] {
    [aw_profiles_dir().join(profile_name)]
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

    let primary = aw_private_bin_dir().join(executable);
    if primary.is_file() {
        return primary;
    }

    primary
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
    if value.is_empty() {
        return Err(AwError::new(format!("aw: {kind} name cannot be empty"), 2));
    }

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
