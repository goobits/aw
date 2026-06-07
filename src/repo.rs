use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AwError, Result};
use crate::paths::path_string;

const CANONICAL_AGENTS: &str = "infra/agent-workspace/agents/.agents";

pub fn run_doctor(args: &[String]) -> Result<i32> {
    if !args.is_empty() {
        return Err(AwError::usage("aw doctor repo does not accept arguments"));
    }
    doctor_repo()
}

pub fn doctor_repo() -> Result<i32> {
    let root = std::env::current_dir()?;
    let mut failed = false;
    failed |= check_file(&root, "infra/agent-workspace/Cargo.toml");
    failed |= check_file(&root, "infra/agent-workspace/agents/.agents/AGENTS.md");
    failed |= check_file(&root, "AGENTS.md");
    failed |= check_file(&root, ".agents.local/project.md");
    failed |= check_symlink(&root, ".agents", CANONICAL_AGENTS);
    failed |= check_symlink(&root, "CLAUDE.md", "AGENTS.md");
    failed |= check_symlink(&root, ".claude/skills", "../.agents/skills");
    failed |= check_dir(&root, "config/aw");
    failed |= check_git_tab(&root);

    if failed {
        println!("fail    repo adapters need attention");
        return Ok(1);
    }
    println!("ok      repo adapters ready");
    Ok(0)
}

pub fn run_migrate(args: &[String]) -> Result<i32> {
    let Some((scope, rest)) = args.split_first() else {
        return Err(AwError::usage(
            "aw migrate requires a scope, for example: aw migrate repo",
        ));
    };
    if scope != "repo" {
        return Err(AwError::usage(format!(
            "aw migrate: unknown scope {}",
            scope
        )));
    }
    let dry_run = match rest {
        [] => false,
        [flag] if flag == "--dry-run" => true,
        _ => return Err(AwError::usage("aw migrate repo accepts only --dry-run")),
    };

    let root = std::env::current_dir()?;
    ensure_agents_bundle(&root)?;
    ensure_symlink(&root, ".agents", CANONICAL_AGENTS, dry_run)?;
    ensure_symlink(&root, "CLAUDE.md", "AGENTS.md", dry_run)?;
    ensure_symlink(&root, ".claude/skills", "../.agents/skills", dry_run)?;
    if !dry_run {
        println!();
        let status = doctor_repo()?;
        if status != 0 {
            return Ok(status);
        }
    }
    println!(
        "done    repo migration{}",
        if dry_run { " dry-run" } else { "" }
    );
    Ok(0)
}

fn ensure_agents_bundle(root: &Path) -> Result<()> {
    if root.join(CANONICAL_AGENTS).join("AGENTS.md").is_file() {
        println!("ok      {} exists", CANONICAL_AGENTS);
        return Ok(());
    }
    Err(AwError::new(
        "aw migrate repo failed: missing infra/agent-workspace/agents/.agents/AGENTS.md",
        1,
    ))
}

fn check_file(root: &Path, path: &str) -> bool {
    if root.join(path).is_file() {
        println!("ok      {}", path);
        return false;
    }
    println!("missing {}", path);
    true
}

fn check_dir(root: &Path, path: &str) -> bool {
    if root.join(path).is_dir() {
        println!("ok      {}/", path);
        return false;
    }
    println!("missing {}/", path);
    true
}

fn check_symlink(root: &Path, link: &str, target: &str) -> bool {
    let path = root.join(link);
    if is_symlink_to(&path, target) {
        println!("ok      {} -> {}", link, target);
        return false;
    }
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let actual = fs::read_link(&path)
                .map(|path| path_string(&path))
                .unwrap_or_else(|_| "<unreadable>".to_string());
            println!("wrong   {} -> {} expected {}", link, actual, target);
        }
        Ok(_) => println!("wrong   {} exists but is not a symlink", link),
        Err(_) => println!("missing {} -> {}", link, target),
    }
    true
}

fn check_git_tab(root: &Path) -> bool {
    let config_dir = root.join("config/aw");
    let Ok(entries) = fs::read_dir(&config_dir) else {
        println!("missing lowercase git tab");
        return true;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("tabs") {
            continue;
        }
        if fs::read_to_string(&path)
            .is_ok_and(|contents| contents.lines().any(|line| line.trim() == "git"))
        {
            println!("ok      lowercase git tab");
            return false;
        }
    }
    println!("missing lowercase git tab");
    true
}

fn ensure_symlink(root: &Path, link: &str, target: &str, dry_run: bool) -> Result<()> {
    let path = root.join(link);
    if is_symlink_to(&path, target) {
        println!("ok      {} -> {}", link, target);
        return Ok(());
    }
    if fs::symlink_metadata(&path).is_ok() {
        return Err(AwError::new(
            format!("aw migrate repo blocked: {} already exists", link),
            1,
        ));
    }
    if dry_run {
        println!("would   {} -> {}", link, target);
        return Ok(());
    }
    create_symlink(root, link, target)?;
    println!("linked  {} -> {}", link, target);
    Ok(())
}

fn create_symlink(root: &Path, link: &str, target: &str) -> Result<()> {
    let path = root.join(link);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    symlink(target, &path)
}

fn is_symlink_to(path: &Path, target: &str) -> bool {
    fs::read_link(path).is_ok_and(|actual| actual == PathBuf::from(target))
}

#[cfg(unix)]
fn symlink(target: &str, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(not(unix))]
fn symlink(_target: &str, _link: &Path) -> Result<()> {
    Err(AwError::new(
        "aw migrate repo failed: symlink creation is only supported on Unix",
        1,
    ))
}
