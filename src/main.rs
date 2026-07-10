mod cli;
mod commit;
mod core;
mod git;
mod install;
mod pkg;
mod profile;
mod repo;
mod runtime;
mod workspace;

pub(crate) use commit::queue as commit_queue;
pub(crate) use core::{error, help, paths};
pub(crate) use git as git_queue;
pub(crate) use install as installer;
pub(crate) use pkg as package_queue;
pub(crate) use repo as repo_tasks;
pub(crate) use runtime::queue_lock;
pub(crate) use workspace as workspace_tasks;
pub(crate) use workspace::brush_worktree;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let rest = args.split_off(1);
    let result = cli::run(rest);

    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            if !error.message.is_empty() {
                eprintln!("{}", error.message);
            }
            if error.show_usage {
                eprint!("{}", help::format_block(cli::USAGE.trim_end()));
            }
            std::process::exit(error.code);
        }
    }
}
