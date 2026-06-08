use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::brush_worktree;
use crate::commit::run_commit_command;
use crate::error::{AwError, Result};
use crate::git_queue;
use crate::installer::{install_repo_adapters, install_workspace_setup};
use crate::package_queue;
use crate::paths::{current_dir, path_string, resolve_root, validate_name};
use crate::profile::{
    add_profile_workspace, auto_install_config, create_initial_profile, find_config_dir,
    install_profile, list_workspaces, profile_dir_from_installed_default, remove_profile_workspace,
    replace_profile_workspace, resolve_config_arg, resolve_profile_name, workspace_exists_or_exit,
};
use crate::repo_tasks;
use crate::tabs::{
    parse_tabs_args, parse_tabs_csv, remove_workspace_tab_line, rename_workspace_tab_line,
    upsert_workspace_tab_line, write_tabs_file,
};
use crate::workspace_tasks;
use crate::zellij::{
    count_tabs_files, ensure_workspace_tabs_file, installed_profile_dir, list_workspace_tabs,
    run_helper, sync_workspace_session, zellij_passthrough,
};

pub const USAGE: &str = r#"aw: Zero-friction Zellij workspaces

usage:
  aw                                show help
  aw <workspace> [-s <session>] [-r <root>]

workspaces:
  aw list [--config <profile-dir>]
  aw create <workspace> <tabs...>
  aw refresh <workspace>
  aw rename <workspace> <new-workspace>
  aw remove <workspace>
  aw <workspace>=<tab>,...          create, add, replace, and sync a workspace

tabs:
  aw tab list <workspace>
  aw tab add <workspace> <tab[@index]>
  aw tab move <workspace> <tab@index>
  aw tab rename <workspace> <old-tab> <new-tab>
  aw tab remove <workspace> <tab>
  aw tab refresh <workspace>

sessions:
  aw ps
  aw kill <session>

commit queue:
  aw commit setup [workspace] [--tab git] [--session <name>] [--agent <cmd>|--no-agent]
  aw commit request <title> <path>... [--check <cmd>] [--summary <text>] [--root <queue-root>] [--poke [tab]] [--wait] [--timeout 10m]
  aw commit status [--root <queue-root>]
  aw commit doctor [--root <queue-root>]
  aw commit wait <id> [--root <queue-root>] [--timeout 10m]
  aw commit poke [tab] [--root <queue-root>]

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
  aw help
"#;

pub fn run(args: Vec<String>) -> Result<i32> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "-h" | "--help" | "help" => {
            print!("{}", USAGE);
            Ok(0)
        }
        "" => {
            print!("{}", USAGE);
            Ok(0)
        }
        "install" => run_install(&args[1..]),
        "setup" => {
            let config_dir = resolve_config_arg(&args[1..])?;
            install_profile(&config_dir, false)?;
            Ok(0)
        }
        "doctor" => run_doctor(&args[1..]),
        "migrate" => repo_tasks::run_migrate(&args[1..]),
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
                eprintln!("aw: refresh requires exactly one workspace name");
                eprintln!("example: aw refresh front");
                return Err(AwError {
                    message: USAGE.trim_end().to_string(),
                    code: 2,
                    show_usage: false,
                });
            }
            run_workspace_tab_command(&args[1], "refresh", &[])
        }
        "tab" => run_tab_command(&args[1..]),
        "commit" => run_commit_command(&args[1..]),
        "owner" => run_owner_command(&args[1..]),
        "repo" => run_repo_command(&args[1..]),
        "gitq" => git_queue::run(&args[1..]),
        "pkgq" => package_queue::run(&args[1..]),
        "workspace" => workspace_tasks::run(&args[1..]),
        "brush-api" => run_brush_api_command(&args[1..]),
        "ps" => {
            if args.len() > 1 {
                return Err(AwError::usage("aw: ps does not accept arguments"));
            }
            zellij_passthrough(&["list-sessions"])
        }
        "kill" => {
            if args.len() != 2 {
                return Err(AwError::usage("aw: kill requires exactly one session name"));
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
            } else {
                run_launch(other, &args[1..])
            }
        }
    }
}

fn run_owner_command(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        return Err(AwError::usage("aw: owner requires git or pkg"));
    };
    match command.as_str() {
        "git" => git_queue::run(rest),
        "pkg" => package_queue::run(rest),
        other => Err(AwError::usage(format!("aw: unknown owner command {other}"))),
    }
}

fn run_repo_command(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        return Err(AwError::usage("aw: repo requires a command"));
    };
    match command.as_str() {
        "doctor" => repo_tasks::run_doctor(rest),
        "migrate" => {
            let mut migrate_args = vec!["repo".to_string()];
            migrate_args.extend(rest.iter().cloned());
            repo_tasks::run_migrate(&migrate_args)
        }
        "clean" => workspace_tasks::run_named("cleanup-generated", rest),
        "measure-git" => workspace_tasks::run_named("measure-git", rest),
        "probe-git-config" => workspace_tasks::run_named("probe-git-config", rest),
        "routes" => workspace_tasks::run_named("routes", rest),
        "worktree" => brush_worktree::run(rest),
        other => Err(AwError::usage(format!("aw: unknown repo command {other}"))),
    }
}

fn run_brush_api_command(args: &[String]) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        return Err(AwError::usage("aw: brush-api requires a command"));
    };
    match command.as_str() {
        "worktree" => brush_worktree::run(rest),
        other => Err(AwError::usage(format!(
            "aw: unknown brush-api command {}",
            other
        ))),
    }
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
                let Some(value) = args.get(index + 1) else {
                    return Err(AwError::usage("aw: install --config requires a path"));
                };
                config_dir = Some(PathBuf::from(value));
                index += 2;
            }
            other => {
                return Err(AwError::usage(format!(
                    "aw: unknown install argument {}",
                    other
                )))
            }
        }
    }
    Ok(InstallArgs {
        repo,
        config_dir,
        dry_run,
    })
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
    }
    Ok(0)
}

fn run_tab_command(args: &[String]) -> Result<i32> {
    let action = args.first().map(String::as_str).unwrap_or("");
    let workspace = args.get(1).map(String::as_str).unwrap_or("");
    if action.is_empty() || workspace.is_empty() {
        return Err(AwError::usage("aw: tab requires an action and workspace"));
    }

    match action {
        "list" | "refresh" => {
            if args.len() != 2 {
                return Err(AwError::usage(format!(
                    "aw: tab {} requires exactly one workspace",
                    action
                )));
            }
            run_workspace_tab_command(workspace, action, &[])
        }
        "add" | "move" | "remove" => {
            if args.len() != 3 {
                return Err(AwError::usage(format!(
                    "aw: tab {} requires a workspace and tab",
                    action
                )));
            }
            run_workspace_tab_command(workspace, action, &args[2..])
        }
        "rename" => {
            if args.len() != 4 {
                return Err(AwError::usage(
                    "aw: tab rename requires a workspace, old tab, and new tab",
                ));
            }
            run_workspace_tab_command(workspace, action, &args[2..])
        }
        other => Err(AwError::usage(format!("aw: unknown tab action {}", other))),
    }
}

fn run_workspace_tab_command(workspace: &str, action: &str, args: &[String]) -> Result<i32> {
    validate_name("workspace", workspace)?;
    let Some(config_dir) = find_config_dir() else {
        return Err(AwError::new(
            "aw: could not find config/aw; create a workspace first",
            1,
        ));
    };
    let tabs_file = ensure_workspace_tabs_file(&config_dir, workspace)?;

    match action {
        "list" => {
            if !args.is_empty() {
                return Err(AwError::usage(format!(
                    "aw: {} list does not accept extra arguments",
                    workspace
                )));
            }
            list_workspace_tabs(workspace, &tabs_file)?;
        }
        "refresh" => {
            if !args.is_empty() {
                return Err(AwError::usage(format!(
                    "aw: {} refresh does not accept extra arguments",
                    workspace
                )));
            }
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, None)?;
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
            sync_workspace_session(&config_dir, workspace, None)?;
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
            sync_workspace_session(&config_dir, workspace, None)?;
            println!("Removed tab {} from {}.", tab, workspace);
        }
        "move" => {
            let spec = args.first().ok_or_else(|| {
                AwError::usage(format!(
                    "aw: {} move requires exactly one tab@index spec",
                    workspace
                ))
            })?;
            let indexed = upsert_workspace_tab_line(&tabs_file, spec)?;
            let Some(index) = indexed.index else {
                return Err(AwError::new(
                    format!(
                        "aw: move requires an index, for example {} move {}@1",
                        workspace, indexed.name
                    ),
                    2,
                ));
            };
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, None)?;
            println!("Moved tab {} to {}@{}.", indexed.name, workspace, index);
        }
        "rename" => {
            if args.len() != 2 {
                return Err(AwError::usage(format!(
                    "aw: {} rename requires old and new tab names",
                    workspace
                )));
            }
            rename_workspace_tab_line(&tabs_file, &args[0], &args[1])?;
            install_profile(&config_dir, true)?;
            sync_workspace_session(&config_dir, workspace, None)?;
            println!("Renamed tab {} to {} in {}.", args[0], args[1], workspace);
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
    fs::rename(old_tabs, new_tabs)?;
    replace_profile_workspace(
        &config_dir.join("profile.conf"),
        old_workspace,
        new_workspace,
    )?;
    install_profile(&config_dir, true)?;
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
                session = args.get(index + 1).cloned().unwrap_or_default();
                index += 2;
            }
            "-r" | "--root" => {
                let value = args.get(index + 1).cloned().unwrap_or_default();
                root_override = path_string(&resolve_root(&value)?);
                index += 2;
            }
            "-h" | "--help" => {
                print!("{}", USAGE);
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

    let (profile_name, profile_dir) = if let Some(config_dir) = auto_install_config()? {
        let profile_name = resolve_profile_name(Some(&config_dir));
        let profile_dir = installed_profile_dir(&profile_name);
        (profile_name, profile_dir)
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
    let zwork_args = vec![profile_name, workspace.to_string(), session, root_override];
    run_helper("zwork", &zwork_args)
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
