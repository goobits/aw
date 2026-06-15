use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::brush_worktree;
use crate::commit::run_commit_command;
use crate::error::{AwError, Result};
use crate::git_queue;
use crate::help;
use crate::installer::{install_repo_adapters, install_workspace_setup};
use crate::package_queue;
use crate::paths::{
    aw_completions_dir, aw_config_file, aw_default_profile_file, aw_home, aw_plugins_dir,
    aw_private_bin_dir, aw_profiles_dir, current_dir, local_bin_dir, path_string, resolve_root,
    validate_name,
};
use crate::profile::{
    add_profile_workspace, auto_install_config, create_initial_profile,
    default_workspace_from_config, find_config_dir, install_profile, list_workspaces,
    profile_dir_from_installed_default, remove_profile_workspace, replace_profile_workspace,
    resolve_config_arg, resolve_profile_name, workspace_exists_or_exit,
};
use crate::repo_tasks;
use crate::tabs::{
    parse_indexed_tab_spec, parse_tabs_args, parse_tabs_csv, remove_workspace_tab_line,
    rename_workspace_tab_line_from_spec, upsert_workspace_tab_line, validate_workspace_tab_rename,
    write_tabs_file,
};
use crate::workspace_tasks;
use crate::zellij::{
    count_tabs_files, default_workspace_session_name, ensure_workspace_tabs_file,
    list_workspace_tabs, rename_live_workspace_session, rename_live_workspace_tab, run_helper,
    sync_workspace_session, zellij_passthrough,
};

pub const USAGE: &str = r#"🌀 aw: Zero-friction Zellij workspaces

usage:
  aw                                show help
  aw <workspace> [-s <session>] [-r <root>]

workspaces:
  aw list [--config <profile-dir>]                  list workspaces and saved tabs
  aw create <workspace> <tabs...>
  aw refresh <workspace>
  aw rename <workspace> <new-workspace>
  aw remove <workspace>
  aw <workspace>=<tab>,...          create, add, replace, and sync a workspace

tabs:
  aw tab list [--session <name>]                    shorthand when exactly one workspace exists
  aw tab add <tab[@index]> [--session <name>]       shorthand when exactly one workspace exists
  aw tab move <tab@index> [--session <name>]        shorthand when exactly one workspace exists
  aw tab rename <old-tab> <new-tab[@index]>
  aw tab remove <tab> [--session <name>]            shorthand when exactly one workspace exists
  aw tab refresh [--session <name>]                 shorthand when exactly one workspace exists
  aw <workspace> tab list [--session <name>]
  aw <workspace> tab add <tab[@index]> [--session <name>]
  aw <workspace> tab move <tab@index> [--session <name>]
  aw <workspace> tab rename <old-tab> <new-tab[@index]> [--session <name>]
  aw <workspace> tab remove <tab> [--session <name>]
  aw <workspace> tab refresh [--session <name>]

sessions:
  aw session name [workspace]
  aw ps
  aw kill <session>

commit queue:
  aw commit setup [workspace] [--tab git] [--session <name>] [--agent <cmd>|--no-agent]
  aw commit request <title> <path>... [--check <cmd>] [--summary <text>] [--queue-root <path>] [--poke [tab]] [--workspace <workspace>] [--session <name>] [--wait] [--timeout 10m]
  aw commit status [--queue-root <path>]
  aw commit doctor [--queue-root <path>]
  aw commit wait <id> [--queue-root <path>] [--timeout 10m]
  aw commit poke [tab] [--queue-root <path>] [--workspace <workspace>] [--session <name>]

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
  aw install [--repo] [--config <profile-dir>] [--dry-run]
  aw setup --config <profile-dir>
  aw doctor [--config <profile-dir>]
  aw paths
  aw help
"#;

pub fn run(args: Vec<String>) -> Result<i32> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "-h" | "--help" | "help" => {
            help::print(USAGE);
            Ok(0)
        }
        "" => {
            help::print(USAGE);
            Ok(0)
        }
        "install" => run_install(&args[1..]),
        "setup" => {
            let config_dir = resolve_config_arg(&args[1..])?;
            install_profile(&config_dir, false)?;
            Ok(0)
        }
        "doctor" => run_doctor(&args[1..]),
        "paths" => run_paths(&args[1..]),
        "list" => run_list(&args[1..]),
        "create" => {
            if args.len() < 3 {
                return Err(AwError::usage(
                    "aw: create requires a workspace and at least one tab",
                ));
            }
            create_local_workspace(&args[1], &args[2..])
        }
        "refresh" => {
            if args.len() != 2 {
                return Err(scoped_usage(
                    "aw: refresh requires exactly one workspace name",
                    "aw refresh <workspace>",
                ));
            }
            run_workspace_tab_command(
                &args[1],
                "refresh",
                &TabCommandArgs {
                    args: Vec::new(),
                    session: None,
                },
            )
        }
        "tab" => run_tab_command(&args[1..]),
        "session" => run_session_command(&args[1..]),
        "commit" => run_commit_command(&args[1..]),
        "owner" => run_owner_command(&args[1..]),
        "repo" => run_repo_command(&args[1..]),
        "gitq" => git_queue::run(&args[1..]),
        "pkgq" => package_queue::run(&args[1..]),
        "ps" => {
            if args.len() > 1 {
                return Err(scoped_usage("aw: ps does not accept arguments", "aw ps"));
            }
            zellij_passthrough(&["list-sessions"])
        }
        "kill" => {
            if args.len() != 2 {
                return Err(scoped_usage(
                    "aw: kill requires exactly one session name",
                    "aw kill <session>",
                ));
            }
            validate_name("session", &args[1])?;
            zellij_passthrough(&["delete-session", "--force", &args[1]])
        }
        "rename" => {
            if args.len() != 3 {
                return Err(AwError::usage(
                    "aw: rename requires old and new workspace names",
                ));
            }
            rename_local_workspace(&args[1], &args[2])
        }
        "remove" => {
            if args.len() != 2 {
                return Err(AwError::usage(
                    "aw: remove requires exactly one workspace name",
                ));
            }
            remove_local_workspace(&args[1])
        }
        other => {
            if other.contains('=') {
                if args.len() != 1 {
                    return Err(AwError::usage(
                        "aw: workspace=tab,tab does not accept extra launch arguments",
                    ));
                }
                upsert_local_workspace(other)
            } else if args.get(1).is_some_and(|arg| arg == "tab") {
                run_tab_command_for_workspace(other, &args[2..])
            } else {
                run_launch(other, &args[1..])
            }
        }
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
        return Err(AwError::new(
            "aw: paths does not accept arguments\n\nusage:\n  aw paths",
            2,
        ));
    }

    println!("AW Paths");
    println!("Home         {}", path_string(&aw_home()));
    println!("Config       {}", path_string(&aw_config_file()));
    println!("Profiles     {}", path_string(&aw_profiles_dir()));
    println!("Default      {}", path_string(&aw_default_profile_file()));
    println!("Bin          {}", path_string(&aw_private_bin_dir()));
    println!("Completions  {}", path_string(&aw_completions_dir()));
    println!("Plugins      {}", path_string(&aw_plugins_dir()));
    println!("Public bin   {}", path_string(&local_bin_dir()));
    Ok(0)
}

fn run_install(args: &[String]) -> Result<i32> {
    let options = parse_install_args(args)?;
    if options.repo {
        if let Some(config_dir) = &options.config_dir {
            if !is_repo_config_dir(config_dir)? {
                return Err(AwError::usage(
                    "aw: install --repo uses config/aw; omit --config or pass --config config/aw",
                ));
            }
        }
    }
    if options.dry_run {
        if !options.repo {
            return Err(AwError::usage("aw: install --dry-run requires --repo"));
        }
        if options.config_dir.is_some() {
            return Err(AwError::usage(
                "aw: install --dry-run cannot install a profile",
            ));
        }
        install_repo_adapters(true)?;
        return Ok(0);
    }

    install_workspace_setup()?;
    if options.repo {
        install_repo_adapters(false)?;
    }
    let config_dir = options.config_dir.or_else(|| {
        options
            .repo
            .then(|| PathBuf::from("config/aw"))
            .filter(|path| path.is_dir())
    });
    if let Some(config_dir) = config_dir {
        install_profile(&config_dir, false)?;
    }
    if options.repo {
        println!();
        let status = repo_tasks::doctor_repo()?;
        if status != 0 {
            return Ok(status);
        }
    }
    Ok(0)
}

fn is_repo_config_dir(path: &Path) -> Result<bool> {
    let root = current_dir()?;
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    Ok(clean_path(&candidate) == clean_path(&root.join("config/aw")))
}

fn clean_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                clean.pop();
            }
            other => clean.push(other.as_os_str()),
        }
    }
    clean
}

struct InstallArgs {
    repo: bool,
    config_dir: Option<PathBuf>,
    dry_run: bool,
}

fn parse_install_args(args: &[String]) -> Result<InstallArgs> {
    let mut repo = false;
    let mut dry_run = false;
    let mut config_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = true;
                index += 1;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--config" => {
                let value = require_install_value(args, index)?;
                config_dir = Some(PathBuf::from(value));
                index += 2;
            }
            other => {
                return Err(scoped_usage(
                    format!("aw: unknown install argument {}", other),
                    "aw install [--repo] [--config <profile-dir>] [--dry-run]",
                ))
            }
        }
    }
    Ok(InstallArgs {
        repo,
        config_dir,
        dry_run,
    })
}

fn require_install_value<'a>(args: &'a [String], index: usize) -> Result<&'a str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty() || value.starts_with("--") {
        return Err(scoped_usage(
            format!("aw: install {option} requires a path"),
            "aw install [--repo] [--config <profile-dir>] [--dry-run]",
        ));
    }
    Ok(value)
}

fn run_doctor(args: &[String]) -> Result<i32> {
    if args.first().is_some_and(|arg| arg == "repo") {
        return repo_tasks::run_doctor(&args[1..]);
    }
    let config_dir = if !args.is_empty() {
        resolve_config_arg(args)?
    } else if let Some(config_dir) = auto_install_config()? {
        config_dir
    } else {
        profile_dir_from_installed_default()
    };
    run_helper(
        "zellij-workspace-doctor",
        &["--config".to_string(), path_string(&config_dir)],
    )
}

fn run_list(args: &[String]) -> Result<i32> {
    let config_dir = if !args.is_empty() {
        resolve_config_arg(args)?
    } else if let Some(config_dir) = find_config_dir() {
        config_dir
    } else {
        profile_dir_from_installed_default()
    };
    for workspace in list_workspaces(&config_dir)? {
        println!("{}", workspace);
        let tabs_file = config_dir.join(format!("{workspace}.tabs"));
        for line in crate::tabs::read_tab_lines(&tabs_file)? {
            let tab = crate::tabs::tab_name_from_line(&line);
            if !tab.is_empty() {
                println!("  {tab}");
            }
        }
    }
    Ok(0)
}

fn run_session_command(args: &[String]) -> Result<i32> {
    match args {
        [action] if action == "name" => {
            let Some(config_dir) = find_config_dir() else {
                return Err(AwError::new(
                    "aw: could not find config/aw; create a workspace first",
                    1,
                ));
            };
            let workspace = default_workspace_from_config(&config_dir);
            if workspace.is_empty() {
                return Err(AwError::new(
                    "aw: no default workspace configured; pass a workspace name",
                    1,
                ));
            }
            println!(
                "{}",
                default_workspace_session_name(&config_dir, &workspace)
            );
            Ok(0)
        }
        [action, workspace] if action == "name" => {
            validate_name("workspace", workspace)?;
            let Some(config_dir) = find_config_dir() else {
                return Err(AwError::new(
                    "aw: could not find config/aw; create a workspace first",
                    1,
                ));
            };
            ensure_workspace_tabs_file(&config_dir, workspace)?;
            println!("{}", default_workspace_session_name(&config_dir, workspace));
            Ok(0)
        }
        [] => Err(scoped_usage(
            "aw: session requires an action",
            "aw session name [workspace]",
        )),
        [action, ..] => Err(scoped_usage(
            format!("aw: unknown session action {action}"),
            "aw session name [workspace]",
        )),
    }
}

fn run_tab_command(args: &[String]) -> Result<i32> {
    let Some((action, rest)) = args.split_first() else {
        return Err(scoped_usage(
            "aw: tab requires an action",
            "aw tab <list|add|move|rename|remove|refresh>",
        ));
    };
    let (workspace, tab_args) = resolve_tab_command(None, action, rest)?;
    run_workspace_tab_command(&workspace, action, &tab_args)
}

fn infer_single_tab_workspace(action: &str) -> Result<String> {
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };
    let workspaces = list_workspaces(&config_dir)?;
    match workspaces.as_slice() {
        [workspace] => Ok(workspace.clone()),
        [] => Err(AwError::new(
            "aw: tab shorthand needs one workspace, but no workspaces exist",
            1,
        )),
        _ => Err(AwError::new(
            format!(
                "aw: tab {} needs a workspace because multiple workspaces exist\nAvailable workspaces:\n{}\nExample: {}",
                action,
                workspaces.join("\n"),
                tab_action_example(Some(&workspaces[0]), action)
            ),
            2,
        )),
    }
}

fn run_tab_command_for_workspace(workspace: &str, args: &[String]) -> Result<i32> {
    let Some((action, rest)) = args.split_first() else {
        return Err(scoped_usage(
            format!("aw: {workspace} tab requires an action"),
            format!("aw {workspace} tab <list|add|move|rename|remove|refresh>"),
        ));
    };
    let (workspace, tab_args) = resolve_tab_command(Some(workspace), action, rest)?;
    run_workspace_tab_command(&workspace, action, &tab_args)
}

struct TabCommandArgs {
    args: Vec<String>,
    session: Option<String>,
}

fn resolve_tab_command(
    workspace: Option<&str>,
    action: &str,
    args: &[String],
) -> Result<(String, TabCommandArgs)> {
    let parsed = parse_tab_session_args(args, action, workspace)?;
    let positional = parsed.args.as_slice();
    match action {
        "list" | "refresh" => {
            if let Some(workspace) = workspace {
                if positional.is_empty() {
                    return Ok((workspace.to_string(), parsed));
                }
                return Err(scoped_usage(
                    format!("aw: {workspace} tab {action} does not accept extra arguments"),
                    tab_action_usage(Some(workspace), action),
                ));
            }
            match positional {
                [] => Ok((infer_single_tab_workspace(action)?, parsed)),
                _ => Err(scoped_usage(
                    format!("aw: tab {action} is only shorthand when exactly one workspace exists"),
                    tab_action_usage(None, action),
                )),
            }
        }
        "add" | "move" | "remove" => {
            if let Some(workspace) = workspace {
                if positional.len() == 1 {
                    return Ok((workspace.to_string(), parsed));
                }
                return Err(scoped_usage(
                    format!("aw: {workspace} tab {action} requires exactly one tab"),
                    tab_action_usage(Some(workspace), action),
                ));
            }
            match positional {
                [_tab] => Ok((infer_single_tab_workspace(action)?, parsed)),
                _ => Err(scoped_usage(
                    format!("aw: tab {action} is only shorthand when exactly one workspace exists"),
                    tab_action_usage(None, action),
                )),
            }
        }
        "rename" => {
            if let Some(workspace) = workspace {
                if positional.len() == 2 {
                    return Ok((workspace.to_string(), parsed));
                }
                return Err(scoped_usage(
                    format!("aw: {workspace} tab rename requires old and new tab names"),
                    tab_action_usage(Some(workspace), action),
                ));
            }
            match positional {
                [_old_tab, _new_tab] => Ok((infer_single_tab_workspace(action)?, parsed)),
                _ => Err(scoped_usage(
                    "aw: tab rename is only shorthand when exactly one workspace exists",
                    tab_action_usage(None, action),
                )),
            }
        }
        other => match workspace {
            Some(workspace) => Err(scoped_usage(
                format!("aw: unknown {workspace} tab action {other}"),
                format!("aw {workspace} tab <list|add|move|rename|remove|refresh>"),
            )),
            None => Err(scoped_usage(
                format!("aw: unknown tab action {other}"),
                "aw tab <list|add|move|rename|remove|refresh>",
            )),
        },
    }
}

fn parse_tab_session_args(
    args: &[String],
    action: &str,
    workspace: Option<&str>,
) -> Result<TabCommandArgs> {
    let mut positional = Vec::new();
    let mut session = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--session" => {
                let value = require_tab_option_value(args, index, action, workspace)?.to_string();
                validate_name("session", &value)?;
                session = Some(value);
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(scoped_usage(
                    format!("aw: tab {action} unknown argument {other}"),
                    tab_action_usage(workspace, action),
                ));
            }
            value => {
                positional.push(value.to_string());
                index += 1;
            }
        }
    }
    Ok(TabCommandArgs {
        args: positional,
        session,
    })
}

fn require_tab_option_value<'a>(
    args: &'a [String],
    index: usize,
    action: &str,
    workspace: Option<&str>,
) -> Result<&'a str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty() || value.starts_with("--") {
        return Err(scoped_usage(
            format!("aw: tab {action} {option} requires a session name"),
            tab_action_usage(workspace, action),
        ));
    }
    Ok(value)
}

fn tab_action_usage(workspace: Option<&str>, action: &str) -> String {
    match workspace {
        Some(workspace) => tab_action_example(Some(workspace), action),
        None => format!(
            "{}\n  {}",
            tab_action_example(None, action),
            tab_action_example(Some("<workspace>"), action)
        ),
    }
}

fn tab_action_example(workspace: Option<&str>, action: &str) -> String {
    let prefix = workspace
        .map(|workspace| format!("aw {workspace} tab"))
        .unwrap_or_else(|| "aw tab".to_string());
    match action {
        "list" | "refresh" => format!("{prefix} {action} [--session <name>]"),
        "add" => format!("{prefix} add <tab[@index]> [--session <name>]"),
        "move" => format!("{prefix} move <tab@index> [--session <name>]"),
        "remove" => format!("{prefix} remove <tab> [--session <name>]"),
        "rename" => format!("{prefix} rename <old-tab> <new-tab[@index]> [--session <name>]"),
        _ => format!("{prefix} <list|add|move|rename|remove|refresh>"),
    }
}

fn scoped_usage(message: impl Into<String>, usage: impl AsRef<str>) -> AwError {
    AwError::new(
        format!("{}\n\nusage:\n  {}", message.into(), usage.as_ref()),
        2,
    )
}

fn run_workspace_tab_command(
    workspace: &str,
    action: &str,
    command: &TabCommandArgs,
) -> Result<i32> {
    validate_name("workspace", workspace)?;
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };
    let tabs_file = ensure_workspace_tabs_file(&config_dir, workspace)?;
    let session_name = command
        .session
        .clone()
        .unwrap_or_else(|| default_workspace_session_name(&config_dir, workspace));
    let args = command.args.as_slice();

    match action {
        "list" => {
            if !args.is_empty() {
                return Err(AwError::usage(format!(
                    "aw: {} list does not accept extra arguments",
                    workspace
                )));
            }
            list_workspace_tabs(&session_name, &tabs_file)?;
        }
        "refresh" => {
            if !args.is_empty() {
                return Err(AwError::usage(format!(
                    "aw: {} refresh does not accept extra arguments",
                    workspace
                )));
            }
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, Some(&session_name))?;
            println!("Converged workspace {}.", workspace);
        }
        "add" => {
            let spec = args.first().ok_or_else(|| {
                AwError::usage(format!(
                    "aw: {} add requires exactly one tab name",
                    workspace
                ))
            })?;
            let indexed = upsert_workspace_tab_line(&tabs_file, spec)?;
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, Some(&session_name))?;
            println!("Added tab {} to {}.", indexed.name, workspace);
        }
        "remove" => {
            let tab = args.first().ok_or_else(|| {
                AwError::usage(format!(
                    "aw: {} remove requires exactly one tab name",
                    workspace
                ))
            })?;
            remove_workspace_tab_line(&tabs_file, tab)?;
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, Some(&session_name))?;
            println!("Removed tab {} from {}.", tab, workspace);
        }
        "move" => {
            let spec = args.first().ok_or_else(|| {
                scoped_usage(
                    format!("aw: {workspace} move requires exactly one tab@index spec"),
                    tab_action_usage(Some(workspace), action),
                )
            })?;
            let indexed = parse_indexed_tab_spec(spec)?;
            let Some(index) = indexed.index else {
                return Err(AwError::new(
                    format!(
                        "aw: move requires an index, for example aw {workspace} tab move {}@1",
                        indexed.name
                    ),
                    2,
                ));
            };
            upsert_workspace_tab_line(&tabs_file, spec)?;
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, Some(&session_name))?;
            println!("Moved tab {} to {}@{}.", indexed.name, workspace, index);
        }
        "rename" => {
            if args.len() != 2 {
                return Err(AwError::usage(format!(
                    "aw: {} rename requires old and new tab names",
                    workspace
                )));
            }
            let indexed = parse_indexed_tab_spec(&args[1])?;
            validate_workspace_tab_rename(&tabs_file, &args[0], &indexed.name)?;
            rename_live_workspace_tab(&session_name, &args[0], &indexed.name)?;
            rename_workspace_tab_line_from_spec(&tabs_file, &args[0], &args[1])?;
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, Some(&session_name))?;
            match indexed.index {
                Some(index) => println!(
                    "Renamed tab {} to {} and moved it to index {} in {}.",
                    args[0], indexed.name, index, workspace
                ),
                None => println!(
                    "Renamed tab {} to {} in {}.",
                    args[0], indexed.name, workspace
                ),
            }
        }
        other => {
            return Err(AwError::usage(format!(
                "aw: unknown workspace tab action {}",
                other
            )))
        }
    }

    Ok(0)
}

fn upsert_local_workspace(spec: &str) -> Result<i32> {
    let Some((workspace, tabs_csv)) = spec.split_once('=') else {
        return Err(AwError::new(
            format!("aw: expected workspace=tab,tab, got {}", spec),
            2,
        ));
    };
    if workspace.is_empty() || tabs_csv.is_empty() {
        return Err(AwError::new(
            format!("aw: expected workspace=tab,tab, got {}", spec),
            2,
        ));
    }

    validate_name("workspace", workspace)?;
    let tabs = parse_tabs_csv(tabs_csv)?;
    let (config_dir, created) = config_dir_or_create(workspace)?;
    write_tabs_file(&config_dir, workspace, &tabs)?;
    if !created {
        add_profile_workspace(&config_dir.join("profile.conf"), workspace)?;
    }
    install_profile(&config_dir, true)?;
    sync_workspace_session(&config_dir, workspace, None)?;
    if created {
        println!("Created profile and synced workspace {}.", workspace);
    } else {
        println!("Synced workspace {}.", workspace);
    }
    Ok(0)
}

fn create_local_workspace(workspace: &str, args: &[String]) -> Result<i32> {
    validate_name("workspace", workspace)?;
    let tabs = parse_tabs_args(args)?;
    let (config_dir, created) = config_dir_or_create(workspace)?;
    let tabs_file = config_dir.join(format!("{}.tabs", workspace));
    if tabs_file.exists() {
        return Err(AwError::new(
            format!(
                "aw: workspace already exists: {}\nReplace it with: aw {}=<tab>,...",
                workspace, workspace
            ),
            1,
        ));
    }
    write_tabs_file(&config_dir, workspace, &tabs)?;
    if !created {
        add_profile_workspace(&config_dir.join("profile.conf"), workspace)?;
    }
    install_profile(&config_dir, true)?;
    sync_workspace_session(&config_dir, workspace, None)?;
    if created {
        println!("Created profile and workspace {}.", workspace);
    } else {
        println!("Created workspace {}.", workspace);
    }
    Ok(0)
}

fn rename_local_workspace(old_workspace: &str, new_workspace: &str) -> Result<i32> {
    validate_name("workspace", old_workspace)?;
    validate_name("workspace", new_workspace)?;
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };

    let old_tabs = config_dir.join(format!("{}.tabs", old_workspace));
    let new_tabs = config_dir.join(format!("{}.tabs", new_workspace));
    if !old_tabs.is_file() {
        return Err(AwError::new(
            format!(
                "aw: missing workspace {} in {}",
                old_workspace,
                path_string(&config_dir)
            ),
            1,
        ));
    }
    if new_tabs.exists() {
        return Err(AwError::new(
            format!("aw: workspace already exists: {}", new_workspace),
            1,
        ));
    }
    let old_session = default_workspace_session_name(&config_dir, old_workspace);
    let new_session = default_workspace_session_name(&config_dir, new_workspace);
    fs::rename(old_tabs, new_tabs)?;
    replace_profile_workspace(
        &config_dir.join("profile.conf"),
        old_workspace,
        new_workspace,
    )?;
    install_profile(&config_dir, true)?;
    rename_live_workspace_session(&old_session, &new_session)?;
    sync_workspace_session(&config_dir, new_workspace, Some(&new_session))?;
    println!("Renamed workspace {} to {}.", old_workspace, new_workspace);
    Ok(0)
}

fn remove_local_workspace(workspace: &str) -> Result<i32> {
    validate_name("workspace", workspace)?;
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };

    let tabs_file = config_dir.join(format!("{}.tabs", workspace));
    if !tabs_file.is_file() {
        return Err(AwError::new(
            format!(
                "aw: missing workspace {} in {}",
                workspace,
                path_string(&config_dir)
            ),
            1,
        ));
    }
    if count_tabs_files(&config_dir)? <= 1 {
        return Err(AwError::new(
            format!(
                "aw: cannot remove the last workspace in {}",
                path_string(&config_dir)
            ),
            1,
        ));
    }
    fs::remove_file(tabs_file)?;
    remove_profile_workspace(&config_dir.join("profile.conf"), workspace)?;
    install_profile(&config_dir, true)?;
    println!("Removed workspace {}.", workspace);
    Ok(0)
}

fn run_launch(workspace: &str, args: &[String]) -> Result<i32> {
    let mut session = String::new();
    let mut root_override = String::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-s" | "--session" => {
                session = require_launch_value(args, index, "session name")?.to_string();
                index += 2;
            }
            "-r" | "--root" => {
                let value = require_launch_value(args, index, "path")?;
                root_override = path_string(&resolve_root(value)?);
                index += 2;
            }
            "-h" | "--help" => {
                help::print(USAGE);
                return Ok(0);
            }
            other => {
                return Err(AwError::usage(format!(
                    "aw: unknown launch argument {}",
                    other
                )))
            }
        }
    }

    validate_name("workspace", workspace)?;
    if !session.is_empty() {
        validate_name("session", &session)?;
    }

    let (profile_arg, profile_dir) = if let Some(config_dir) = auto_install_config()? {
        (path_string(&config_dir), config_dir)
    } else {
        let profile_name = resolve_profile_name(None);
        let profile_dir = profile_dir_from_installed_default();
        if !profile_dir.join("profile.conf").is_file() {
            return Err(AwError::usage(
                "aw: no local config/aw and no installed default profile",
            ));
        }
        (profile_name, profile_dir)
    };

    workspace_exists_or_exit(&profile_dir, workspace)?;
    let zwork_args = vec![profile_arg, workspace.to_string(), session, root_override];
    run_helper("zwork", &zwork_args)
}

fn require_launch_value<'a>(args: &'a [String], index: usize, value_name: &str) -> Result<&'a str> {
    let option = args[index].as_str();
    let value = args.get(index + 1).map(String::as_str).unwrap_or("");
    if value.is_empty()
        || matches!(
            value,
            "-s" | "--session" | "-r" | "--root" | "-h" | "--help"
        )
    {
        return Err(scoped_usage(
            format!("aw: {option} requires a {value_name}"),
            "aw <workspace> [-s <session>] [-r <root>]",
        ));
    }
    Ok(value)
}

fn config_dir_or_create(workspace: &str) -> Result<(PathBuf, bool)> {
    if let Some(config_dir) = find_config_dir() {
        return Ok((config_dir, false));
    }

    let config_dir = std::env::var_os("AW_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("config/aw")
        });
    create_initial_profile(&config_dir, workspace)?;
    Ok((config_dir.canonicalize().unwrap_or(config_dir), true))
}
