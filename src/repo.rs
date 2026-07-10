use std::fs;
use std::path::Path;

use crate::error::{AwError, Result};
use crate::paths::path_string;

const CANONICAL_AGENTS: &str = "infra/aw/agents/.agents";

pub fn run_doctor(args: &[String]) -> Result<i32> {
    if !args.is_empty() {
        return Err(AwError::usage("aw repo doctor does not accept arguments"));
    }
    doctor_repo()
}

pub fn doctor_repo() -> Result<i32> {
    let root = std::env::current_dir()?;
    let mut failed = false;
    failed |= check_file(&root, "infra/aw/Cargo.toml");
    failed |= check_file(&root, "infra/aw/agents/.agents/AGENTS.md");
    failed |= check_file(&root, "AGENTS.md");
    failed |= check_file(&root, ".agents.local/project.md");
    failed |= check_symlink(&root, ".agents", CANONICAL_AGENTS);
    failed |= check_symlink(&root, "CLAUDE.md", "AGENTS.md");
    failed |= check_symlink(&root, ".claude/skills", "../.agents/skills");
    failed |= check_dir(&root, "config/aw");
    failed |= check_commit_owner_config(&root);

    if failed {
        println!("fail    repo adapters need attention");
        return Ok(1);
    }
    println!("ok      repo adapters ready");
    Ok(0)
}

pub fn run_migrate(args: &[String]) -> Result<i32> {
    let dry_run = match args {
        [] => false,
        [flag] if flag == "--dry-run" => true,
        _ => return Err(AwError::usage("aw repo migrate accepts only --dry-run")),
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
        "aw repo migrate failed: missing infra/aw/agents/.agents/AGENTS.md",
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

fn check_commit_owner_config(root: &Path) -> bool {
    let path = root.join("config/aw/profile.conf");
    let Ok(contents) = fs::read_to_string(&path) else {
        println!("missing config/aw/profile.conf");
        return true;
    };
    let value = contents
        .lines()
        .find_map(|line| line.strip_prefix("commit_owner="));
    match value {
        Some("enabled" | "disabled") => {
            println!("ok      commit_owner={}", value.unwrap_or_default());
            false
        }
        _ => {
            println!("wrong   commit_owner must be enabled or disabled");
            true
        }
    }
}

fn ensure_symlink(root: &Path, link: &str, target: &str, dry_run: bool) -> Result<()> {
    let path = root.join(link);
    if is_symlink_to(&path, target) {
        println!("ok      {} -> {}", link, target);
        return Ok(());
    }
    if fs::symlink_metadata(&path).is_ok() {
        return Err(AwError::new(
            format!("aw repo migrate blocked: {} already exists", link),
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
    fs::read_link(path).is_ok_and(|actual| actual == Path::new(target))
}

#[cfg(unix)]
fn symlink(target: &str, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(not(unix))]
fn symlink(_target: &str, _link: &Path) -> Result<()> {
    Err(AwError::new(
        "aw repo migrate failed: symlink creation is only supported on Unix",
        1,
    ))
}
