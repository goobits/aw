use std::env;
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

pub fn aw_completions_dir() -> PathBuf {
    aw_home().join("completions")
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
