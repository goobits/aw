use std::env;

use crate::error::{AwError, Result};
use crate::help;
use crate::queue_lock::{self, QueueLock};

const QUEUE_NAME: &str = "pkgq";

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
        "lock-info" => print_lock_info(),
        "--" => run_pnpm(rest),
        _ => run_pnpm(args),
    }
}

fn run_pnpm(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        return Err(AwError::new("pkgq requires pnpm arguments", 1));
    }
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let timeout = queue_lock::timeout_from_env("PKGQ_TIMEOUT_MS")?;
    let git_lock = if env::var("GITQ_ACTIVE").ok().as_deref() == Some("1") {
        None
    } else {
        Some(QueueLock::acquire(
            "gitq",
            &repo_root,
            &format!("pkgq guarding pnpm {}", args.join(" ")),
            timeout,
        )?)
    };
    let pkg_lock = QueueLock::acquire(
        QUEUE_NAME,
        &repo_root,
        &format!("pnpm {}", args.join(" ")),
        timeout,
    )?;
    let status = queue_lock::run_status(
        "pnpm",
        args,
        &env::current_dir()?,
        &[("GITQ_ACTIVE", "1"), ("PKGQ_ACTIVE", "1")],
    )?;
    drop(pkg_lock);
    drop(git_lock);
    Ok(status)
}

fn print_lock_info() -> Result<i32> {
    let repo_root = queue_lock::find_repo_root(&env::current_dir()?)?;
    let info = queue_lock::read_lock_info(QUEUE_NAME, &repo_root)?;
    if info.is_null() {
        println!("pkgq: no active queue lock");
    } else {
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    }
    Ok(0)
}

fn print_usage() {
    help::println(
        "Usage:
  aw owner pkg lock-info
  aw owner pkg -- install --lockfile-only
  aw owner pkg -- add <package> --filter <workspace>",
    );
}
