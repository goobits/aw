use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AwError, Result};
use crate::paths::{
    aw_default_profile_candidates, aw_default_profile_file, aw_profile_dir_candidates,
    aw_profiles_dir, current_dir, is_executable, local_bin_dir, path_string, resolve_root,
    validate_name,
};

pub fn profile_value(config_file: &Path, key: &str, default_value: &str) -> String {
    let Ok(contents) = fs::read_to_string(config_file) else {
        return default_value.to_string();
    };

    for line in contents.lines() {
        if let Some((line_key, value)) = line.split_once('=') {
            if line_key == key {
                return value.trim_end_matches('\r').to_string();
            }
        }
    }

    default_value.to_string()
}

pub fn find_config_dir() -> Option<PathBuf> {
    if let Some(candidate) = env::var_os("AW_CONFIG_DIR").map(PathBuf::from) {
        if candidate.join("profile.conf").is_file() {
            return Some(candidate.canonicalize().unwrap_or(candidate));
        }
    }

    let candidate = current_dir().ok()?.join("config/aw");
    if candidate.join("profile.conf").is_file() {
        return Some(candidate.canonicalize().unwrap_or(candidate));
    }

    None
}

pub fn resolve_profile_name(config_dir: Option<&Path>) -> String {
    if let Some(config_dir) = config_dir {
        return profile_name_from_config_dir(config_dir);
    }

    for default_file in aw_default_profile_candidates() {
        if let Ok(contents) = fs::read_to_string(default_file) {
            if let Some(line) = contents.lines().next() {
                return line.to_string();
            }
        }
    }

    "zellij".to_string()
}

pub fn profile_name_from_config_dir(config_dir: &Path) -> String {
    let config_file = config_dir.join("profile.conf");
    if config_file.is_file() {
        let fallback = config_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("zellij");
        return profile_value(&config_file, "name", fallback);
    }

    config_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zellij")
        .to_string()
}

pub fn default_session_name(profile_name: &str, workspace: &str, root: &str) -> String {
    format!(
        "{}-{}-{:016x}",
        profile_name,
        workspace,
        stable_root_hash(root)
    )
}

pub fn default_session_name_from_profile_dir(profile_dir: &Path, workspace: &str) -> String {
    let profile_name = profile_name_from_config_dir(profile_dir);
    let identity_root = profile_session_identity_root(profile_dir);
    default_session_name(&profile_name, workspace, &identity_root)
}

fn profile_session_identity_root(profile_dir: &Path) -> String {
    if let Some(local_root) = local_config_owner_root(profile_dir) {
        return path_string(&local_root);
    }

    profile_value(
        &profile_dir.join("profile.conf"),
        "root",
        &env::current_dir()
            .ok()
            .map(|path| path_string(&path))
            .unwrap_or_else(|| "/workspace".to_string()),
    )
}

fn local_config_owner_root(profile_dir: &Path) -> Option<PathBuf> {
    let config_dir = profile_dir.file_name().and_then(|name| name.to_str())?;
    let config_parent = profile_dir
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())?;
    if config_dir != "aw" || config_parent != "config" {
        return None;
    }
    let root = profile_dir.parent()?.parent()?;
    Some(root.canonicalize().unwrap_or_else(|_| root.to_path_buf()))
}

fn stable_root_hash(root: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in root.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn resolve_config_arg(args: &[String]) -> Result<PathBuf> {
    let mut config_dir: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                let value = args.get(index + 1).cloned().unwrap_or_default();
                config_dir = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(AwError::usage(format!("aw: unknown argument {}", other))),
        }
    }

    let config_dir = if let Some(config_dir) = config_dir {
        config_dir
    } else if let Some(config_dir) = find_config_dir() {
        config_dir
    } else {
        return Err(AwError::new(
            "aw: could not find config/aw; pass --config <profile-dir>",
            1,
        ));
    };

    if !config_dir.join("profile.conf").is_file() {
        return Err(AwError::new(
            format!("aw: missing {}/profile.conf", path_string(&config_dir)),
            1,
        ));
    }

    Ok(config_dir.canonicalize().unwrap_or(config_dir))
}

pub fn install_profile(config_dir: &Path, quiet: bool) -> Result<()> {
    if !config_dir.is_dir() {
        return Err(AwError::new(
            format!(
                "zellij-workspace-init: missing profile directory {}",
                path_string(config_dir)
            ),
            1,
        ));
    }
    let config_dir = config_dir
        .canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());
    let fallback_name = config_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zellij")
        .to_string();
    validate_name("profile", &fallback_name).map_err(|_| {
        AwError::new(
            "zellij-workspace-init: profile directory name may only use letters, numbers, dot, underscore, and dash",
            1,
        )
    })?;
    let config_file = config_dir.join("profile.conf");
    if !config_file.is_file() {
        return Err(AwError::new(
            format!(
                "zellij-workspace-init: missing {}/profile.conf",
                path_string(&config_dir)
            ),
            1,
        ));
    }
    let profile_name = profile_value(&config_file, "name", &fallback_name);
    let default_workspace = profile_value(&config_file, "default_workspace", "");
    validate_name("profile", &profile_name).map_err(|_| {
        AwError::new(
            "zellij-workspace-init: profile name may only use letters, numbers, dot, underscore, and dash",
            1,
        )
    })?;

    let tabs_files = tabs_files(&config_dir)?;
    if tabs_files.is_empty() {
        return Err(AwError::new(
            format!(
                "zellij-workspace-init: expected at least one *.tabs file in {}",
                path_string(&config_dir)
            ),
            1,
        ));
    }

    let profiles_dir = aw_profiles_dir();
    let profile_target = profiles_dir.join(&profile_name);
    let lock_dir = profiles_dir.join(format!(".profile-{}.lock", profile_name));
    fs::create_dir_all(&profiles_dir)?;
    let mut locked = false;
    for _ in 0..200 {
        match fs::create_dir(&lock_dir) {
            Ok(_) => {
                locked = true;
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    if !locked {
        return Err(AwError::new(
            format!(
                "zellij-workspace-init: could not acquire profile install lock {}",
                lock_dir.display()
            ),
            1,
        ));
    }
    let _guard = LockGuard(lock_dir);

    let _ = fs::remove_dir_all(&profile_target);
    fs::create_dir_all(&profile_target)?;
    fs::create_dir_all(local_bin_dir())?;
    fs::copy(&config_file, profile_target.join("profile.conf"))?;
    fs::write(aw_default_profile_file(), format!("{}\n", profile_name))?;
    for tabs_file in tabs_files {
        let target = profile_target.join(tabs_file.file_name().unwrap_or_default());
        fs::copy(tabs_file, target)?;
    }
    let launcher_dir = config_dir.join("bin");
    if launcher_dir.is_dir() {
        for entry in fs::read_dir(launcher_dir)? {
            let path = entry?.path();
            if path.is_file() && is_executable(&path) {
                fs::copy(
                    &path,
                    local_bin_dir().join(path.file_name().unwrap_or_default()),
                )?;
            }
        }
    }
    if !quiet {
        println!("Installed Zellij profile {}.", profile_name);
        if default_workspace.is_empty() {
            println!("Run: aw <workspace>");
        } else {
            println!("Run: aw {}", default_workspace);
        }
    }
    Ok(())
}

struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

fn tabs_files(config_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(config_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("tabs") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn auto_install_config() -> Result<Option<PathBuf>> {
    if let Some(config_dir) = find_config_dir() {
        install_profile(&config_dir, true)?;
        Ok(Some(config_dir))
    } else {
        Ok(None)
    }
}

pub fn profile_dir_from_installed_default() -> PathBuf {
    let profile_name = resolve_profile_name(None);
    for candidate in aw_profile_dir_candidates(&profile_name) {
        if candidate.is_dir() {
            return candidate;
        }
    }
    aw_profiles_dir().join(profile_name)
}

pub fn default_workspace_from_config(config_dir: &Path) -> String {
    let config_file = config_dir.join("profile.conf");
    let default_workspace = profile_value(&config_file, "default_workspace", "");
    if !default_workspace.is_empty() {
        return default_workspace;
    }

    profile_value(&config_file, "default_workspaces", "")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn list_workspaces(config_dir: &Path) -> Result<Vec<String>> {
    let mut workspaces = Vec::new();
    if config_dir.is_dir() {
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("tabs") {
                if let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) {
                    workspaces.push(name.to_string());
                }
            }
        }
    }
    workspaces.sort();
    Ok(workspaces)
}

pub fn workspace_exists_or_exit(profile_dir: &Path, workspace: &str) -> Result<()> {
    if profile_dir.join(format!("{}.tabs", workspace)).is_file() {
        return Ok(());
    }

    let mut message = format!(
        "aw: workspace not found: {}\nLooked in: {}",
        workspace,
        path_string(profile_dir)
    );
    let available = list_workspaces(profile_dir)?;
    if available.is_empty() {
        message.push_str("\nNo workspaces are installed for this profile.");
    } else {
        message.push_str("\nAvailable workspaces:\n");
        message.push_str(&available.join("\n"));
    }

    let local_tabs = current_dir()?
        .join("config/aw")
        .join(format!("{}.tabs", workspace));
    if local_tabs.is_file() {
        message
            .push_str("\nA local workspace file exists. Refresh with: aw setup --config config/aw");
    } else {
        message.push_str(&format!("\nCreate it with: aw {}=<tab>,...", workspace));
    }

    Err(AwError::new(message, 1))
}

pub fn add_profile_workspace(config_file: &Path, workspace: &str) -> Result<()> {
    let current = profile_value(config_file, "default_workspaces", "");
    let mut workspaces: Vec<String> = current.split_whitespace().map(str::to_string).collect();
    if workspaces.iter().any(|item| item == workspace) {
        return Ok(());
    }
    workspaces.push(workspace.to_string());
    rewrite_profile_values(config_file, &[("default_workspaces", workspaces.join(" "))])
}

pub fn replace_profile_workspace(
    config_file: &Path,
    old_workspace: &str,
    new_workspace: &str,
) -> Result<()> {
    let default_workspace = profile_value(config_file, "default_workspace", "");
    let next_default = if default_workspace == old_workspace {
        new_workspace.to_string()
    } else {
        default_workspace
    };

    let mut found = false;
    let mut emitted = Vec::<String>::new();
    for mut current in profile_value(config_file, "default_workspaces", "")
        .split_whitespace()
        .map(str::to_string)
    {
        if current == old_workspace {
            current = new_workspace.to_string();
            found = true;
        }
        if !emitted.iter().any(|item| item == &current) {
            emitted.push(current);
        }
    }
    if !found && !emitted.iter().any(|item| item == new_workspace) {
        emitted.push(new_workspace.to_string());
    }

    rewrite_profile_values(
        config_file,
        &[
            ("default_workspace", next_default),
            ("default_workspaces", emitted.join(" ")),
        ],
    )
}

pub fn remove_profile_workspace(config_file: &Path, remove_workspace: &str) -> Result<()> {
    let default_workspace = profile_value(config_file, "default_workspace", "");
    let mut found = false;
    let mut emitted = Vec::<String>::new();
    for current in profile_value(config_file, "default_workspaces", "").split_whitespace() {
        if current == remove_workspace {
            found = true;
            continue;
        }
        if !emitted.iter().any(|item| item == current) {
            emitted.push(current.to_string());
        }
    }
    if !found && default_workspace != remove_workspace {
        return Ok(());
    }

    let next_default = if default_workspace == remove_workspace {
        emitted.first().cloned().unwrap_or_default()
    } else {
        default_workspace
    };

    rewrite_profile_values(
        config_file,
        &[
            ("default_workspace", next_default),
            ("default_workspaces", emitted.join(" ")),
        ],
    )
}

pub fn create_initial_profile(config_dir: &Path, workspace: &str) -> Result<()> {
    fs::create_dir_all(config_dir)?;
    let config_dir = config_dir
        .canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());
    let cwd = current_dir()?;
    let profile_name = cwd
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string();
    validate_name("profile", &profile_name)?;
    let root = resolve_root(&path_string(&cwd))?;
    let contents = format!(
        "name={}\nroot={}\ndefault_workspace={}\ndefault_workspaces={}\n",
        profile_name,
        path_string(&root),
        workspace,
        workspace
    );
    fs::write(config_dir.join("profile.conf"), contents)?;
    Ok(())
}

fn rewrite_profile_values(config_file: &Path, updates: &[(&str, String)]) -> Result<()> {
    let contents = fs::read_to_string(config_file).unwrap_or_default();
    let mut lines = Vec::new();
    let mut written = vec![false; updates.len()];

    for line in contents.lines() {
        if let Some((key, _)) = line.split_once('=') {
            if let Some((index, (_, value))) = updates
                .iter()
                .enumerate()
                .find(|(_, (update_key, _))| key == *update_key)
            {
                lines.push(format!("{}={}", key, value));
                written[index] = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    for (index, (key, value)) in updates.iter().enumerate() {
        if !written[index] {
            lines.push(format!("{}={}", key, value));
        }
    }

    fs::write(config_file, format!("{}\n", lines.join("\n")))?;
    Ok(())
}
