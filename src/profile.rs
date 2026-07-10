use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn profile_value(config_file: &Path, key: &str, default_value: &str) -> String {
    let Ok(contents) = fs::read_to_string(config_file) else {
        return default_value.to_string();
    };
    contents
        .lines()
        .find_map(|line| {
            line.split_once('=')
                .filter(|(line_key, _)| *line_key == key)
        })
        .map(|(_, value)| value.trim_end_matches('\r').to_string())
        .unwrap_or_else(|| default_value.to_string())
}

pub fn find_config_dir() -> Option<PathBuf> {
    if let Some(candidate) = env::var_os("AW_CONFIG_DIR").map(PathBuf::from) {
        if candidate.join("profile.conf").is_file() {
            return Some(candidate.canonicalize().unwrap_or(candidate));
        }
    }
    let candidate = env::current_dir().ok()?.join("config/aw");
    candidate
        .join("profile.conf")
        .is_file()
        .then(|| candidate.canonicalize().unwrap_or(candidate))
}
