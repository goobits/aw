mod cli;
mod commit;
mod core;
mod git;
mod install;
mod pkg;
mod repo;
mod runtime;
mod workspace;
mod zellij;

pub(crate) use commit::queue as commit_queue;
pub(crate) use core::{error, paths};
pub(crate) use git as git_queue;
pub(crate) use install as installer;
pub(crate) use pkg as package_queue;
pub(crate) use repo as repo_tasks;
pub(crate) use runtime::queue_lock;
pub(crate) use workspace as workspace_tasks;
pub(crate) use workspace::brush_worktree;
pub(crate) use zellij::{helpers, layout, profile, tab_order, tabs, watcher};

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let argv0 = args.first().cloned().unwrap_or_else(|| "aw".to_string());
    let name = std::path::Path::new(&argv0)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("aw")
        .to_string();
    let rest = args.split_off(1);
    let result = if helpers::is_helper_name(&name) {
        helpers::run(&name, rest)
    } else if rest.first().is_some_and(|arg| helpers::is_helper_name(arg)) {
        let helper = rest[0].clone();
        helpers::run(&helper, rest[1..].to_vec())
    } else {
        cli::run(rest)
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            if !error.message.is_empty() {
                eprintln!("{}", error.message);
            }
            if error.show_usage {
                eprintln!("{}", cli::USAGE.trim_end());
            }
            std::process::exit(error.code);
        }
    }
}
