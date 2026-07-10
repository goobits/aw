use std::path::Path;

use crate::brush_worktree;
use crate::commit::run_commit_command;
use crate::error::{AwError, Result};
use crate::git_queue;
use crate::help;
use crate::installer::{install_repo_adapters, install_workspace_setup};
use crate::package_queue;
use crate::paths::{aw_completions_dir, aw_home, local_bin_dir, path_string};
use crate::repo_tasks;
use crate::workspace_tasks;

pub const USAGE: &str = r#"🌀 aw: Agent Workspace coordination for Shelly

usage:
  aw help

commit queue:
  aw commit setup [--tab git]
  aw commit request <title> <path>... [--owner <name>] [--check <cmd>] [--summary <text>] [--queue-root <path>] [--poke [git]] [--wait] [--timeout 10m]
  aw commit status [--queue-root <path>]
  aw commit doctor [--queue-root <path>]
  aw commit wait <id> [--queue-root <path>] [--timeout 10m]
  aw commit poke [git] [--queue-root <path>]

repo maintenance:
  aw repo doctor
  aw repo migrate [--dry-run]
  aw repo clean [--delete] [--generated|--rust-targets|--nested-node-modules|--all-safe|--build-outputs|--preprocessed]
  aw repo measure-git [path]
  aw repo probe-git-config [--path path] [--apply]
  aw repo routes [doctor] [--config path]
  aw repo worktree <path> [--branch name] [--base ref]

owner internals:
  aw owner git <owner-git-command>
  aw owner pkg <pnpm-args...>

system:
  aw install [--repo] [--dry-run]
  aw doctor
  aw paths
"#;

pub fn run(args: Vec<String>) -> Result<i32> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "" | "-h" | "--help" | "help" => {
            help::print(USAGE);
            Ok(0)
        }
        "install" => run_install(&args[1..]),
        "doctor" => run_doctor(&args[1..]),
        "paths" => run_paths(&args[1..]),
        "commit" => run_commit_command(&args[1..]),
        "owner" => run_owner_command(&args[1..]),
        "repo" => run_repo_command(&args[1..]),
        "gitq" => git_queue::run(&args[1..]),
        "pkgq" => package_queue::run(&args[1..]),
        other => Err(AwError::usage(format!("aw: unknown command {other}"))),
    }
}

fn run_owner_command(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        return Err(scoped_usage(
            "aw: owner requires git or pkg",
            "aw owner <git|pkg> ...",
        ));
    };
    match command.as_str() {
        "git" => git_queue::run(rest),
        "pkg" => package_queue::run(rest),
        other => Err(scoped_usage(
            format!("aw: unknown owner command {other}"),
            "aw owner <git|pkg> ...",
        )),
    }
}

fn run_repo_command(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        workspace_tasks::print_usage();
        return Ok(0);
    };
    match command.as_str() {
        "-h" | "--help" | "help" => {
            workspace_tasks::print_usage();
            Ok(0)
        }
        "doctor" => repo_tasks::run_doctor(rest),
        "migrate" => repo_tasks::run_migrate(rest),
        "clean" => workspace_tasks::run_named("cleanup-generated", rest),
        "measure-git" => workspace_tasks::run_named("measure-git", rest),
        "probe-git-config" => workspace_tasks::run_named("probe-git-config", rest),
        "routes" => workspace_tasks::run_named("routes", rest),
        "worktree" => brush_worktree::run(rest),
        other => Err(AwError::new(
            format!(
                "aw: unknown repo command {other}\n\n{}",
                workspace_tasks::REPO_USAGE
            ),
            2,
        )),
    }
}

fn run_paths(args: &[String]) -> Result<i32> {
    if !args.is_empty() {
        return Err(scoped_usage(
            "aw: paths does not accept arguments",
            "aw paths",
        ));
    }
    println!("AW Paths");
    println!("Home         {}", path_string(&aw_home()));
    println!("Completions  {}", path_string(&aw_completions_dir()));
    println!("Public bin   {}", path_string(&local_bin_dir()));
    Ok(0)
}

fn run_install(args: &[String]) -> Result<i32> {
    let options = parse_install_args(args)?;
    if options.dry_run && !options.repo {
        return Err(AwError::usage("aw: install --dry-run requires --repo"));
    }
    if options.dry_run {
        install_repo_adapters(true)?;
        return Ok(0);
    }
    install_workspace_setup()?;
    if options.repo {
        install_repo_adapters(false)?;
        println!();
        return repo_tasks::doctor_repo();
    }
    Ok(0)
}

fn run_doctor(args: &[String]) -> Result<i32> {
    if !args.is_empty() {
        return Err(scoped_usage(
            "aw: doctor does not accept arguments",
            "aw doctor",
        ));
    }
    println!("ok      aw command available");
    if Path::new("config/aw/profile.conf").is_file() {
        println!("ok      config/aw/profile.conf");
    } else {
        println!("note    no repo-local config/aw/profile.conf");
    }
    Ok(0)
}

#[derive(Default)]
struct InstallArgs {
    repo: bool,
    dry_run: bool,
}

fn parse_install_args(args: &[String]) -> Result<InstallArgs> {
    let mut options = InstallArgs::default();
    for arg in args {
        match arg.as_str() {
            "--repo" => options.repo = true,
            "--dry-run" => options.dry_run = true,
            other => {
                return Err(scoped_usage(
                    format!("aw: unknown install argument {other}"),
                    "aw install [--repo] [--dry-run]",
                ))
            }
        }
    }
    Ok(options)
}

fn scoped_usage(message: impl Into<String>, usage: &str) -> AwError {
    AwError::new(format!("{}\n\nusage:\n  {usage}", message.into()), 2)
}
