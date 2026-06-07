use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use crate::error::{AwError, Result};

pub struct GitInvocation {
    pub cwd: PathBuf,
    pub command: String,
}

pub fn parse_git_invocation(git_args: &[String]) -> Result<GitInvocation> {
    let mut cwd = env::current_dir()?;
    let mut index = 0;
    while index < git_args.len() {
        let arg = &git_args[index];
        if arg == "--" {
            index += 1;
            break;
        }
        if arg == "-C" {
            let value = git_args
                .get(index + 1)
                .ok_or_else(|| AwError::new("git -C requires a path", 1))?;
            cwd = cwd.join(value);
            index += 2;
            continue;
        }
        if arg.starts_with("-C") && arg.len() > 2 {
            cwd = cwd.join(&arg[2..]);
            index += 1;
            continue;
        }
        if global_options_with_value().contains(arg.as_str()) {
            index += 2;
            continue;
        }
        if global_option_prefixes()
            .iter()
            .any(|prefix| arg.starts_with(prefix))
            || global_flags().contains(arg.as_str())
            || arg.starts_with('-')
        {
            index += 1;
            continue;
        }
        break;
    }
    Ok(GitInvocation {
        cwd,
        command: git_args.get(index).cloned().unwrap_or_default(),
    })
}

pub fn assert_allowed_raw_git(git_args: &[String]) -> Result<()> {
    let invocation = parse_git_invocation(git_args)?;
    let command = invocation.command.as_str();
    if command.is_empty() {
        return Err(AwError::new("gitq -- requires git arguments", 1));
    }
    if command == "config" && is_read_only_config(git_args, &invocation)
        || command == "branch" && is_read_only_branch(git_args, &invocation)
        || command == "remote" && is_read_only_remote(git_args, &invocation)
        || command == "tag" && is_read_only_tag(git_args, &invocation)
        || command == "submodule" && is_submodule_status(git_args, &invocation)
        || raw_read_commands().contains(command)
    {
        return Ok(());
    }
    if blocked_raw_commands().contains(command) || command == "rm" {
        return Err(AwError::new(
            format!("Refusing raw \"git {command}\" through gitq. Use a dedicated gitq command such as commit-owned, repair-index, chmod, or health."),
            1,
        ));
    }
    Err(AwError::new(
        format!("Refusing raw \"git {command}\" through gitq. Raw passthrough is read-only; use an explicit gitq command for mutations."),
        1,
    ))
}

pub fn is_lockless_read_allowed(git_args: &[String], invocation: &GitInvocation) -> bool {
    lockless_read_commands().contains(invocation.command.as_str())
        || invocation.command == "config" && is_read_only_config(git_args, invocation)
        || invocation.command == "diff" && has_pathspec_after_separator(git_args)
        || invocation.command == "status"
            && (is_fast_status(git_args) || has_pathspec_after_separator(git_args))
}

pub fn uses_isolated_index(command: &str) -> bool {
    isolated_index_commands().contains(command)
}

fn command_args<'a>(git_args: &'a [String], invocation: &GitInvocation) -> &'a [String] {
    let index = git_args
        .iter()
        .position(|arg| arg == &invocation.command)
        .map(|index| index + 1)
        .unwrap_or(git_args.len());
    &git_args[index..]
}

fn is_read_only_config(git_args: &[String], invocation: &GitInvocation) -> bool {
    let args = command_args(git_args, invocation);
    let read_modes = [
        "--get",
        "--get-all",
        "--get-color",
        "--get-colorbool",
        "--get-regexp",
        "--get-urlmatch",
        "--list",
        "--null",
        "--show-origin",
        "--show-scope",
    ];
    args.is_empty() || args.iter().any(|arg| read_modes.contains(&arg.as_str()))
}

fn is_submodule_status(git_args: &[String], invocation: &GitInvocation) -> bool {
    let command = command_args(git_args, invocation)
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str);
    command.is_none() || command == Some("status")
}

fn is_read_only_branch(git_args: &[String], invocation: &GitInvocation) -> bool {
    let args = command_args(git_args, invocation);
    let allowed = [
        "--all",
        "--contains",
        "--format",
        "--list",
        "--merged",
        "--no-color",
        "--no-contains",
        "--no-merged",
        "--points-at",
        "--remotes",
        "--show-current",
        "--verbose",
        "-a",
        "-r",
        "-v",
        "-vv",
    ];
    (args.is_empty()
        || args
            .iter()
            .any(|arg| arg == "--list" || arg == "--show-current"))
        && only_allowed_options(args, &allowed)
}

fn is_read_only_remote(git_args: &[String], invocation: &GitInvocation) -> bool {
    let subcommand = command_args(git_args, invocation)
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str);
    subcommand.is_none() || subcommand == Some("get-url") || subcommand == Some("show")
}

fn is_read_only_tag(git_args: &[String], invocation: &GitInvocation) -> bool {
    let args = command_args(git_args, invocation);
    let allowed = [
        "--contains",
        "--format",
        "--list",
        "--merged",
        "--no-contains",
        "--no-merged",
        "--points-at",
        "-l",
    ];
    args.iter().any(|arg| arg == "--list" || arg == "-l") && only_allowed_options(args, &allowed)
}

fn only_allowed_options(args: &[String], allowed: &[&str]) -> bool {
    args.iter().all(|arg| {
        !arg.starts_with('-')
            || allowed.contains(&arg.as_str())
            || allowed
                .iter()
                .any(|option| arg.starts_with(&format!("{option}=")))
    })
}

fn is_fast_status(git_args: &[String]) -> bool {
    git_args.iter().any(|arg| arg == "--untracked-files=no")
        && git_args
            .iter()
            .any(|arg| arg == "--ignore-submodules=dirty" || arg == "--ignore-submodules=all")
}

fn has_pathspec_after_separator(git_args: &[String]) -> bool {
    git_args
        .iter()
        .position(|arg| arg == "--")
        .is_some_and(|index| index < git_args.len() - 1)
}

fn blocked_raw_commands() -> BTreeSet<&'static str> {
    [
        "add",
        "checkout",
        "commit",
        "read-tree",
        "reset",
        "restore",
        "update-index",
    ]
    .into_iter()
    .collect()
}

fn lockless_read_commands() -> BTreeSet<&'static str> {
    ["log", "ls-files", "show"].into_iter().collect()
}

fn raw_read_commands() -> BTreeSet<&'static str> {
    [
        "check-ignore",
        "diff",
        "grep",
        "log",
        "ls-files",
        "ls-tree",
        "rev-parse",
        "show",
        "status",
        "version",
    ]
    .into_iter()
    .collect()
}

fn isolated_index_commands() -> BTreeSet<&'static str> {
    ["diff", "grep", "ls-files", "status"].into_iter().collect()
}

fn global_options_with_value() -> BTreeSet<&'static str> {
    [
        "--exec-path",
        "--git-dir",
        "--namespace",
        "--super-prefix",
        "--work-tree",
        "-c",
    ]
    .into_iter()
    .collect()
}

fn global_option_prefixes() -> Vec<&'static str> {
    vec![
        "--exec-path=",
        "--git-dir=",
        "--namespace=",
        "--super-prefix=",
        "--work-tree=",
        "-c",
    ]
}

fn global_flags() -> BTreeSet<&'static str> {
    [
        "--bare",
        "--literal-pathspecs",
        "--no-optional-locks",
        "--no-pager",
        "--no-replace-objects",
        "--paginate",
    ]
    .into_iter()
    .collect()
}
