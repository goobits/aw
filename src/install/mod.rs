use std::env;
use std::fs;
use std::path::Path;

use crate::error::{AwError, Result};
use crate::paths::{aw_completions_dir, aw_home, home_dir, local_bin_dir};

const ZSH_COMPLETION: &str = include_str!("../../completions/_aw");
const BASH_COMPLETION: &str = include_str!("../../completions/aw.bash");
const ROOT_AGENTS_TEMPLATE: &str = include_str!("../../agents/.agents/templates/root-AGENTS.md");
const PROJECT_TEMPLATE: &str = include_str!("../../agents/.agents/templates/project.md");
const LEGACY_START_MARKER: &str = "# >>> zellij workspaces >>>";
const LEGACY_END_MARKER: &str = "# <<< zellij workspaces <<<";

pub fn install_workspace_setup() -> Result<i32> {
    let original_path = env::var("PATH").unwrap_or_default();
    let local_bin = local_bin_dir();
    let completion_dir = aw_completions_dir();
    fs::create_dir_all(aw_home())?;
    fs::create_dir_all(&local_bin)?;
    fs::create_dir_all(&completion_dir)?;
    fs::write(completion_dir.join("_aw"), ZSH_COMPLETION)?;
    fs::write(completion_dir.join("aw.bash"), BASH_COMPLETION)?;
    copy_executable(&env::current_exe()?, &local_bin.join("aw"))?;
    remove_legacy_shell_block(&home_dir().join(".zshrc"))?;
    remove_legacy_shell_block(&home_dir().join(".bashrc"))?;
    remove_legacy_helpers();

    println!("Installed Agent Workspace coordination tools.");
    if !path_has_local_bin(&original_path) {
        println!("Open a new shell or run: export PATH=\"$HOME/.local/bin:$PATH\"");
    }
    Ok(0)
}

pub fn install_repo_adapters(dry_run: bool) -> Result<()> {
    let root = env::current_dir()?;
    let workspace_root = root.join("infra/aw");
    if !workspace_root.join("agents/.agents/AGENTS.md").is_file() {
        return Err(AwError::new(
            "aw install failed: infra/aw/agents/.agents/AGENTS.md is missing",
            1,
        ));
    }
    for file in REPO_ADAPTER_FILES {
        ensure_repo_file(&root, file, dry_run)?;
    }
    for link in REPO_ADAPTER_LINKS {
        ensure_repo_symlink(&root, link, dry_run)?;
    }
    println!("done    {}", display_path(&root, &workspace_root));
    Ok(())
}

struct RepoAdapterFile {
    file: &'static str,
    contents: &'static str,
    source: &'static str,
}

struct RepoAdapterLink {
    link: &'static str,
    target: &'static str,
}

const REPO_ADAPTER_FILES: &[RepoAdapterFile] = &[
    RepoAdapterFile {
        file: "AGENTS.md",
        contents: ROOT_AGENTS_TEMPLATE,
        source: "infra/aw/agents/.agents/templates/root-AGENTS.md",
    },
    RepoAdapterFile {
        file: ".agents.local/project.md",
        contents: PROJECT_TEMPLATE,
        source: "infra/aw/agents/.agents/templates/project.md",
    },
];

const REPO_ADAPTER_LINKS: &[RepoAdapterLink] = &[
    RepoAdapterLink {
        link: ".agents",
        target: "infra/aw/agents/.agents",
    },
    RepoAdapterLink {
        link: "CLAUDE.md",
        target: "AGENTS.md",
    },
    RepoAdapterLink {
        link: ".claude/skills",
        target: "../.agents/skills",
    },
];

fn ensure_repo_file(root: &Path, item: &RepoAdapterFile, dry_run: bool) -> Result<()> {
    let path = root.join(item.file);
    if path.exists() || fs::symlink_metadata(&path).is_ok() {
        println!("keep    {} already exists; not overwritten", item.file);
        return Ok(());
    }
    if dry_run {
        println!("would   {} from {}", item.file, item.source);
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, item.contents)?;
    println!("created {} from {}", item.file, item.source);
    Ok(())
}

fn ensure_repo_symlink(root: &Path, item: &RepoAdapterLink, dry_run: bool) -> Result<()> {
    let link_path = root.join(item.link);
    match fs::symlink_metadata(&link_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if link_target_matches(&link_path, item.target) {
                println!("ok      {} -> {}", item.link, item.target);
            } else if dry_run {
                println!("would   {} -> {}", item.link, item.target);
            } else {
                fs::remove_file(&link_path)?;
                create_symlink(item.target, &link_path)?;
                println!("linked  {} -> {}", item.link, item.target);
            }
            Ok(())
        }
        Ok(_) => {
            println!("keep    {} already exists; not overwritten", item.link);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if dry_run {
                println!("would   {} -> {}", item.link, item.target);
                return Ok(());
            }
            if let Some(parent) = link_path.parent() {
                fs::create_dir_all(parent)?;
            }
            create_symlink(item.target, &link_path)?;
            println!("linked  {} -> {}", item.link, item.target);
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn link_target_matches(link_path: &Path, target: &str) -> bool {
    fs::read_link(link_path).is_ok_and(|actual| actual == Path::new(target))
}

fn remove_legacy_shell_block(path: &Path) -> Result<()> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(());
    };
    if !contents.contains(LEGACY_START_MARKER) {
        return Ok(());
    }
    let mut next = String::new();
    let mut skip = false;
    for line in contents.lines() {
        if line == LEGACY_START_MARKER {
            skip = true;
            continue;
        }
        if line == LEGACY_END_MARKER {
            skip = false;
            continue;
        }
        if !skip {
            next.push_str(line);
            next.push('\n');
        }
    }
    fs::write(path, next.trim_end_matches('\n').to_string() + "\n")?;
    Ok(())
}

fn remove_legacy_helpers() {
    let private_bin = aw_home().join("bin");
    for name in [
        ".zellij-agent-tab-watcher",
        ".zellij-codex-tab-watcher",
        "zellij-agent-tab-watcher",
        "zellij-launch-session",
        "zellij-live-tab-order",
        "zellij-new-scratch-tab",
        "zellij-open-session",
        "zellij-render-layout",
        "zellij-saved-session-order",
        "zellij-session-tab-order",
        "zellij-workspace-doctor",
        "zellij-workspace-init",
        "zwork",
    ] {
        let _ = fs::remove_file(private_bin.join(name));
    }
    let _ = fs::remove_file(local_bin_dir().join(".zellij-new-scratch-tab"));
    let _ = fs::remove_file(aw_home().join("config.kdl"));
    let _ = fs::remove_file(aw_home().join("plugins/aw-tab-bar.wasm"));
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn copy_executable(source: &Path, target: &Path) -> Result<()> {
    if source.canonicalize().ok() == target.canonicalize().ok() {
        make_executable(target)?;
        return Ok(());
    }
    let tmp = target.with_extension(format!("tmp-{}", std::process::id()));
    let _ = fs::remove_file(&tmp);
    fs::copy(source, &tmp)?;
    make_executable(&tmp)?;
    fs::rename(&tmp, target)?;
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &str, link_path: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link_path)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_symlink(_target: &str, _link_path: &Path) -> Result<()> {
    Err(AwError::new(
        "aw install failed: symlink creation is only supported on Unix",
        1,
    ))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn path_has_local_bin(path: &str) -> bool {
    let local_bin = local_bin_dir();
    env::split_paths(path).any(|entry| entry == local_bin)
}
