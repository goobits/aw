use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{AwError, Result};
use crate::paths::{path_string, shell_quote};
use crate::queue_lock;

const DEFAULT_LOCAL_SUBMODULES: &[&str] = &[
    "apps/sketchpad",
    "apps/sketchpad-guide",
    "apps/sketchapi-com",
    "apps/sketchpad-com",
    "servers/asset",
    "packages/@goobits/auth",
    "packages/@goobits/logger",
    "packages/@goobits/goo",
    "packages/@sketchapi/imagefx",
    "packages/@sketchapi/svg/resvg/upstream",
];

const GENERATED_ARTIFACT_PATHS: &[&str] = &[
    "packages/@sketchapi/brush/fluid/rust/pkg",
    "packages/@sketchapi/brush/stamp/wasm",
];

const PACKAGE_NODE_MODULE_PATHS: &[&str] = &[
    "packages/@sketchapi/brush/api/node_modules",
    "packages/@sketchapi/brush/utils/node_modules",
];

pub fn run(args: &[String]) -> Result<i32> {
    let options = parse_args(args)?;
    if options.help || options.target.is_none() {
        print_usage();
        return Ok(if options.help { 0 } else { 1 });
    }
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let target_root = PathBuf::from(options.target.unwrap());
    let target_root = if target_root.is_absolute() {
        target_root
    } else {
        env::current_dir()?.join(target_root)
    };
    let branch = options.branch.unwrap_or_else(default_branch);
    let base = options.base.unwrap_or_else(|| "HEAD".to_string());

    if !target_root.exists() {
        let status = run_logged(
            "aw",
            &[
                "gitq".to_string(),
                "worktree".to_string(),
                "add".to_string(),
                "-b".to_string(),
                branch,
                path_string(&target_root),
                base,
            ],
            &repo_root,
        )?;
        if status != 0 {
            return Ok(status);
        }
    } else {
        println!(
            "[brush-api-worktree] Reusing existing path: {}",
            path_string(&target_root)
        );
    }

    hydrate_submodules(&repo_root, &target_root)?;
    hydrate_generated_artifacts(&repo_root, &target_root)?;
    if !options.skip_deps {
        hydrate_node_modules(&repo_root, &target_root, options.copy_deps)?;
    }

    println!("[brush-api-worktree] Ready: {}", path_string(&target_root));
    println!("[brush-api-worktree] Suggested check:");
    println!(
        "  cd {} && pnpm --filter @sketchapi/brush-api run check:types",
        shell_quote(&path_string(&target_root))
    );
    Ok(0)
}

struct Options {
    base: Option<String>,
    branch: Option<String>,
    copy_deps: bool,
    help: bool,
    skip_deps: bool,
    target: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Options> {
    let mut parsed = Options {
        base: Some("HEAD".to_string()),
        branch: None,
        copy_deps: false,
        help: false,
        skip_deps: false,
        target: None,
    };
    let args = args
        .iter()
        .filter(|arg| arg.as_str() != "--")
        .collect::<Vec<_>>();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                parsed.help = true;
                index += 1;
            }
            "--base" => {
                parsed.base = Some(require_value(&args, index)?.to_string());
                index += 2;
            }
            "--branch" => {
                parsed.branch = Some(require_value(&args, index)?.to_string());
                index += 2;
            }
            "--copy-deps" => {
                parsed.copy_deps = true;
                index += 1;
            }
            "--skip-deps" => {
                parsed.skip_deps = true;
                index += 1;
            }
            other if other.starts_with("--") => {
                return Err(AwError::new(format!("Unexpected argument: {other}"), 1));
            }
            other => {
                if parsed.target.is_none() {
                    parsed.target = Some(other.to_string());
                    index += 1;
                } else {
                    return Err(AwError::new(format!("Unexpected argument: {other}"), 1));
                }
            }
        }
    }
    Ok(parsed)
}

fn hydrate_submodules(repo_root: &Path, target_root: &Path) -> Result<()> {
    for rel_path in read_submodule_paths(repo_root)? {
        if !DEFAULT_LOCAL_SUBMODULES.contains(&rel_path.as_str()) {
            continue;
        }
        let source = repo_root.join(&rel_path);
        let target = target_root.join(&rel_path);
        if !is_usable_git_repo(&source) || is_usable_git_repo(&target) {
            continue;
        }
        if target.exists()
            && fs::read_dir(&target).is_ok_and(|mut entries| entries.next().is_some())
        {
            println!("[brush-api-worktree] Leaving non-empty submodule path alone: {rel_path}");
            continue;
        }
        let _ = fs::remove_dir_all(&target);
        fs::create_dir_all(target.parent().unwrap_or(target_root))?;
        let status = run_logged(
            "git",
            &[
                "clone".to_string(),
                "--shared".to_string(),
                path_string(&source),
                path_string(&target),
            ],
            target_root,
        )?;
        if status != 0 {
            return Ok(());
        }
        if let Some(expected) = get_gitlink_commit(target_root, &rel_path)? {
            let checkout = run_logged(
                "git",
                &["checkout".to_string(), "--detach".to_string(), expected],
                &target,
            )?;
            if checkout != 0 {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn hydrate_generated_artifacts(repo_root: &Path, target_root: &Path) -> Result<()> {
    for rel_path in GENERATED_ARTIFACT_PATHS {
        let source = repo_root.join(rel_path);
        let target = target_root.join(rel_path);
        if !source.exists() || target.exists() {
            continue;
        }
        fs::create_dir_all(target.parent().unwrap_or(target_root))?;
        let status = run_logged(
            "cp",
            &["-a".to_string(), path_string(&source), path_string(&target)],
            target_root,
        )?;
        if status != 0 {
            return Ok(());
        }
    }
    Ok(())
}

fn hydrate_node_modules(repo_root: &Path, target_root: &Path, copy_deps: bool) -> Result<()> {
    hydrate_dependency_dir(
        &repo_root.join("node_modules"),
        &target_root.join("node_modules"),
        copy_deps,
    )?;
    for rel_path in PACKAGE_NODE_MODULE_PATHS {
        hydrate_dependency_dir(
            &repo_root.join(rel_path),
            &target_root.join(rel_path),
            copy_deps,
        )?;
    }
    Ok(())
}

fn hydrate_dependency_dir(source: &Path, target: &Path, copy_deps: bool) -> Result<()> {
    if !source.exists() || target.exists() {
        return Ok(());
    }
    fs::create_dir_all(target.parent().unwrap_or_else(|| Path::new(".")))?;
    if copy_deps {
        let status = run_logged(
            "cp",
            &["-a".to_string(), path_string(source), path_string(target)],
            &env::current_dir()?,
        )?;
        if status != 0 {
            return Ok(());
        }
    } else {
        println!(
            "[brush-api-worktree] ln -s {} {}",
            shell_quote(&path_string(source)),
            shell_quote(&path_string(target))
        );
        symlink(source, target)?;
    }
    Ok(())
}

fn read_submodule_paths(repo_root: &Path) -> Result<Vec<String>> {
    let gitmodules = repo_root.join(".gitmodules");
    if !gitmodules.exists() {
        return Ok(Vec::new());
    }
    Ok(fs::read_to_string(gitmodules)?
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("path = ").map(str::trim))
        .map(ToString::to_string)
        .collect())
}

fn get_gitlink_commit(target_root: &Path, rel_path: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["ls-tree", "HEAD", "--", rel_path])
        .current_dir(target_root)
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|pair| {
            if pair[0] == "commit" {
                Some(pair[1].to_string())
            } else {
                None
            }
        }))
}

fn is_usable_git_repo(repo_path: &Path) -> bool {
    if !repo_path.exists() {
        return false;
    }
    Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(repo_path)
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()) == repo_path
        })
}

fn run_logged(command: &str, args: &[String], cwd: &Path) -> Result<i32> {
    println!(
        "[brush-api-worktree] {}",
        std::iter::once(command.to_string())
            .chain(args.iter().cloned())
            .map(|value| shell_quote(&value))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let status = Command::new(command).args(args).current_dir(cwd).status()?;
    Ok(status.code().unwrap_or(1))
}

fn require_value<'a>(args: &'a [&String], index: usize) -> Result<&'a str> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| AwError::new(format!("{} requires a value", args[index]), 1))?;
    Ok(value)
}

fn default_branch() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis();
    format!("agent/brush-api-{millis}")
}

fn print_usage() {
    println!(
        "Usage: aw brush-api worktree <path> [--branch name] [--base ref] [--skip-deps] [--copy-deps]

Creates a branch-backed Brush API worktree, hydrates local submodules from the
current checkout when possible, copies generated brush WASM artifacts, and
symlinks the dependency link structure needed for Brush API typechecks.

Options:
  --skip-deps  Leave node_modules paths untouched.
  --copy-deps  Copy node_modules paths instead of symlinking them."
    );
}
