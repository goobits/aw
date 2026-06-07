pub(crate) mod brush_worktree;

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use crate::error::{AwError, Result};

const DEFAULT_GIT_MEASURE_PATH: &str = "infra/agent-workspace";

pub fn run(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        print_usage();
        return Ok(0);
    };
    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(0)
        }
        "cleanup-generated" => cleanup_generated(rest),
        "measure-git" => measure_git(rest),
        "probe-git-config" => probe_git_config(rest),
        other => Err(AwError::new(
            format!("aw workspace: unknown command {other}"),
            1,
        )),
    }
}

fn cleanup_generated(args: &[String]) -> Result<i32> {
    let args = args
        .iter()
        .filter(|arg| arg.as_str() != "--")
        .cloned()
        .collect::<BTreeSet<_>>();
    let allowed = [
        "--all-safe",
        "--build-outputs",
        "--delete",
        "--generated",
        "--nested-node-modules",
        "--preprocessed",
        "--rust-targets",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    for arg in &args {
        if !allowed.contains(arg.as_str()) {
            return Err(AwError::new(format!("Unknown option: {arg}"), 1));
        }
    }
    let options = CleanupOptions {
        delete_matches: args.contains("--delete"),
        include_generated_caches: args.contains("--generated")
            || args.contains("--all-safe")
            || has_no_category(&args),
        include_rust_targets: args.contains("--rust-targets")
            || args.contains("--all-safe")
            || has_no_category(&args),
        include_nested_node_modules: args.contains("--nested-node-modules")
            || args.contains("--all-safe")
            || has_no_category(&args),
        include_build_outputs: args.contains("--build-outputs"),
        include_preprocessed_caches: args.contains("--preprocessed"),
    };
    let repo_root = env::current_dir()?;
    let mut seen = BTreeSet::new();
    let mut matches = Vec::new();
    walk_cleanup(&repo_root, &repo_root, &options, &mut seen, &mut matches)?;
    matches.sort_by(|left, right| left.path.cmp(&right.path));
    if matches.is_empty() {
        println!("No generated cleanup candidates found.");
        return Ok(0);
    }
    println!(
        "{}",
        if options.delete_matches {
            "Deleting generated cleanup candidates:"
        } else {
            "Generated cleanup candidates:"
        }
    );
    for cleanup_match in &matches {
        println!(
            "{}\t{}\t{}",
            if options.delete_matches {
                "delete"
            } else {
                "keep"
            },
            cleanup_match.category,
            relative_path(&repo_root, &cleanup_match.path)
        );
        if options.delete_matches {
            fs::remove_dir_all(&cleanup_match.path)?;
        }
    }
    if !options.delete_matches {
        println!();
        println!("Dry run only. Re-run with --delete and one or more category flags to remove candidates.");
    }
    Ok(0)
}

fn measure_git(args: &[String]) -> Result<i32> {
    let target_path = args
        .iter()
        .find(|arg| arg.as_str() != "--")
        .cloned()
        .unwrap_or_else(|| DEFAULT_GIT_MEASURE_PATH.to_string());
    let cases = vec![
        ("status-fast full", vec!["gitq", "status-fast"]),
        (
            "status-fast scoped",
            vec!["gitq", "status-fast", "--", &target_path],
        ),
        (
            "diff scoped",
            vec!["gitq", "--", "diff", "--name-status", "--", &target_path],
        ),
        (
            "diff cached scoped",
            vec![
                "gitq",
                "--",
                "diff",
                "--cached",
                "--name-status",
                "--",
                &target_path,
            ],
        ),
        (
            "ls-files modified",
            vec!["gitq", "--", "ls-files", "-m", "--", &target_path],
        ),
        ("full status audit", vec!["gitq", "status", "--short"]),
        ("health", vec!["gitq", "health"]),
    ];
    let exe = env::current_exe()?;
    let mut exit_code = 0;
    for (label, command) in cases {
        let started = Instant::now();
        let output = Command::new(&exe)
            .args(command)
            .current_dir(env::current_dir()?)
            .output()?;
        let ms = started.elapsed().as_millis();
        let status = output.status.code().unwrap_or(1);
        println!(
            "{label}: {ms}ms\texit={status}\tstdoutLines={}",
            line_count(&String::from_utf8_lossy(&output.stdout))
        );
        if status != 0 {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                eprintln!("{}", stderr.trim());
            }
            exit_code = status;
        }
    }
    Ok(exit_code)
}

fn probe_git_config(args: &[String]) -> Result<i32> {
    let mut apply = false;
    let mut target_path = DEFAULT_GIT_MEASURE_PATH.to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--apply" => {
                apply = true;
                index += 1;
            }
            "--path" => {
                target_path = args
                    .get(index + 1)
                    .ok_or_else(|| AwError::new("--path requires a value", 1))?
                    .clone();
                index += 2;
            }
            "--" => index += 1,
            other => return Err(AwError::new(format!("Unknown option: {other}"), 1)),
        }
    }
    let probes = vec![
        Probe {
            name: "preloadIndex",
            config: "core.preloadIndex=true",
            apply_key: "core.preloadIndex",
            apply_value: "true",
        },
        Probe {
            name: "untrackedCache",
            config: "core.untrackedCache=true",
            apply_key: "core.untrackedCache",
            apply_value: "true",
        },
        Probe {
            name: "splitIndex",
            config: "core.splitIndex=true",
            apply_key: "core.splitIndex",
            apply_value: "true",
        },
    ];
    let baseline = measure_probe("baseline", None, &target_path)?;
    println!("{}", baseline.format());
    let mut results = Vec::new();
    for probe in &probes {
        let result = measure_probe(probe.name, Some(probe.config), &target_path)?;
        println!("{}", result.format());
        results.push((probe, result));
    }
    let winners = results
        .into_iter()
        .filter(|(_, result)| result.status == 0 && result.ms < baseline.ms)
        .collect::<Vec<_>>();
    if !apply {
        println!("Dry run only. Re-run with --apply to write winning config values.");
        return Ok(0);
    }
    let exe = env::current_exe()?;
    let mut exit_code = 0;
    for (probe, _) in winners {
        let output = Command::new(&exe)
            .args(["gitq", "--", "config", probe.apply_key, probe.apply_value])
            .current_dir(env::current_dir()?)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                eprintln!("{}", stderr.trim());
            }
            exit_code = output.status.code().unwrap_or(1);
            continue;
        }
        println!("applied {}={}", probe.apply_key, probe.apply_value);
    }
    Ok(exit_code)
}

struct CleanupOptions {
    delete_matches: bool,
    include_generated_caches: bool,
    include_rust_targets: bool,
    include_nested_node_modules: bool,
    include_build_outputs: bool,
    include_preprocessed_caches: bool,
}

struct CleanupMatch {
    category: &'static str,
    path: PathBuf,
}

fn walk_cleanup(
    repo_root: &Path,
    directory: &Path,
    options: &CleanupOptions,
    seen: &mut BTreeSet<String>,
    matches: &mut Vec<CleanupMatch>,
) -> Result<()> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Ok(());
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let full_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(cleanup_match) = classify_cleanup(repo_root, &full_path, &name, options) {
            let key = fs::canonicalize(&cleanup_match.path)
                .unwrap_or_else(|_| cleanup_match.path.clone())
                .to_string_lossy()
                .to_string();
            if seen.insert(key) {
                matches.push(cleanup_match);
            }
            continue;
        }
        if name == ".git" || name == "node_modules" {
            continue;
        }
        walk_cleanup(repo_root, &full_path, options, seen, matches)?;
    }
    Ok(())
}

fn classify_cleanup(
    repo_root: &Path,
    full_path: &Path,
    name: &str,
    options: &CleanupOptions,
) -> Option<CleanupMatch> {
    if options.include_nested_node_modules
        && name == "node_modules"
        && full_path != repo_root.join("node_modules")
    {
        return Some(CleanupMatch {
            category: "nested-node_modules",
            path: full_path.to_path_buf(),
        });
    }
    if options.include_generated_caches && (name == ".turbo" || name == ".svelte-kit") {
        return Some(CleanupMatch {
            category: "generated-cache",
            path: full_path.to_path_buf(),
        });
    }
    if options.include_rust_targets && name == "target" && looks_like_rust_target(full_path) {
        return Some(CleanupMatch {
            category: "rust-target",
            path: full_path.to_path_buf(),
        });
    }
    if options.include_build_outputs && (name == "dist" || name == "build") {
        return Some(CleanupMatch {
            category: "build-output",
            path: full_path.to_path_buf(),
        });
    }
    if options.include_preprocessed_caches && is_preprocessed_cache(repo_root, full_path, name) {
        return Some(CleanupMatch {
            category: "preprocessed-cache",
            path: full_path.to_path_buf(),
        });
    }
    None
}

fn has_no_category(args: &BTreeSet<String>) -> bool {
    ![
        "--all-safe",
        "--build-outputs",
        "--generated",
        "--nested-node-modules",
        "--preprocessed",
        "--rust-targets",
    ]
    .iter()
    .any(|arg| args.contains(*arg))
}

fn is_preprocessed_cache(repo_root: &Path, full_path: &Path, name: &str) -> bool {
    name == "_preprocessed"
        || name == "cache" && full_path.parent() == Some(&repo_root.join("tasks/code-watcher"))
}

fn looks_like_rust_target(full_path: &Path) -> bool {
    let Some(parent) = full_path.parent() else {
        return false;
    };
    parent.join("Cargo.toml").exists() || parent.ends_with("rust") || parent.ends_with("src-tauri")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

struct Probe {
    name: &'static str,
    config: &'static str,
    apply_key: &'static str,
    apply_value: &'static str,
}

struct ProbeResult {
    name: String,
    ms: u128,
    status: i32,
    stdout_lines: usize,
    stderr: String,
}

impl ProbeResult {
    fn format(&self) -> String {
        let suffix = if self.status == 0 {
            String::new()
        } else {
            format!("\tstderr={:?}", self.stderr.trim())
        };
        format!(
            "{}: {}ms\texit={}\tstdoutLines={}{}",
            self.name, self.ms, self.status, self.stdout_lines, suffix
        )
    }
}

fn measure_probe(name: &str, config: Option<&str>, target_path: &str) -> Result<ProbeResult> {
    let exe = env::current_exe()?;
    let mut args = vec!["gitq".to_string(), "--".to_string()];
    if let Some(config) = config {
        args.push("-c".to_string());
        args.push(config.to_string());
    }
    args.extend([
        "status".to_string(),
        "--short".to_string(),
        "--untracked-files=no".to_string(),
        "--ignore-submodules=dirty".to_string(),
        "--".to_string(),
        target_path.to_string(),
    ]);
    let started = Instant::now();
    let output = Command::new(exe)
        .args(args)
        .current_dir(env::current_dir()?)
        .output()?;
    Ok(ProbeResult {
        name: name.to_string(),
        ms: started.elapsed().as_millis(),
        status: output.status.code().unwrap_or(1),
        stdout_lines: line_count(&String::from_utf8_lossy(&output.stdout)),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn line_count(text: &str) -> usize {
    if text.trim().is_empty() {
        0
    } else {
        text.trim().lines().count()
    }
}

fn print_usage() {
    println!(
        "Usage:
  aw workspace cleanup-generated [--delete] [--generated|--rust-targets|--nested-node-modules|--all-safe|--build-outputs|--preprocessed]
  aw workspace measure-git [path]
  aw workspace probe-git-config [--path path] [--apply]"
    );
}
