use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::{AwError, Result};
use crate::layout::render_layout;
use crate::paths::{
    aw_config_file, aw_private_bin_dir, aw_profile_dir_candidates, home_dir, is_executable,
    local_bin_dir, path_string, validate_name,
};
use crate::profile::{default_session_name_from_profile_dir, install_profile, profile_value};
use crate::tab_order::{live_tab_order, saved_session_order, session_tab_order};
use crate::tabs::read_tab_lines;
use crate::watcher::watcher_command;
use crate::zellij::{base_name, zellij_passthrough};

const HELPERS: &[&str] = &[
    "zwork",
    "zellij-open-session",
    "zellij-launch-session",
    "zellij-render-layout",
    "zellij-saved-session-order",
    "zellij-live-tab-order",
    "zellij-session-tab-order",
    "zellij-new-scratch-tab",
    ".zellij-new-scratch-tab",
    "zellij-workspace-init",
    "zellij-workspace-doctor",
    "zellij-agent-tab-watcher",
    ".zellij-agent-tab-watcher",
];

pub fn is_helper_name(name: &str) -> bool {
    HELPERS.contains(&name)
}

pub fn run(name: &str, args: Vec<String>) -> Result<i32> {
    match name {
        "zellij-render-layout" => render_layout_command(&args),
        "zwork" => zwork_command(&args),
        "zellij-open-session" => open_session_command(&args),
        "zellij-launch-session" => launch_session_command(&args),
        "zellij-session-tab-order" => session_tab_order_command(&args),
        "zellij-live-tab-order" => live_tab_order_command(&args),
        "zellij-saved-session-order" => saved_session_order_command(&args),
        "zellij-new-scratch-tab" | ".zellij-new-scratch-tab" => new_scratch_tab_command(&args),
        "zellij-workspace-init" => workspace_init_command(&args),
        "zellij-workspace-doctor" => workspace_doctor_command(&args),
        "zellij-agent-tab-watcher" | ".zellij-agent-tab-watcher" => watcher_command(&args),
        _ => Err(AwError::new(format!("unknown helper {}", name), 2)),
    }
}

fn render_layout_command(args: &[String]) -> Result<i32> {
    if args.len() != 2 {
        return Err(AwError::new(
            "usage: zellij-render-layout <tabs-file> <workdir>",
            2,
        ));
    }
    print!("{}", render_layout(Path::new(&args[0]), &args[1])?);
    Ok(0)
}

fn zwork_command(args: &[String]) -> Result<i32> {
    if args.len() < 2 {
        return Err(AwError::new(
            "usage: zwork <profile> <workspace> [session] [workdir]",
            2,
        ));
    }
    let profile = &args[0];
    let workspace = &args[1];
    if !profile.contains('/') {
        validate_name_for("zwork", "profile", profile)?;
    }
    validate_name_for("zwork", "workspace", workspace)?;

    let profile_dir = resolve_profile_dir(profile)?;
    let root = profile_value(&profile_dir.join("profile.conf"), "root", "/workspace");
    let workdir = args
        .get(3)
        .filter(|s| !s.is_empty())
        .cloned()
        .or_else(|| env::var("ZELLIJ_WORKDIR").ok())
        .unwrap_or(root);
    let session = args
        .get(2)
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_default();
    open_session(&profile_dir, workspace, &session, &workdir)
}

fn open_session_command(args: &[String]) -> Result<i32> {
    if args.len() != 4 {
        return Err(AwError::new(
            "usage: zellij-open-session <profile-dir> <workspace> <session> <workdir>",
            2,
        ));
    }
    let profile_dir = PathBuf::from(&args[0]);
    open_session(&profile_dir, &args[1], &args[2], &args[3])
}

fn open_session(profile_dir: &Path, workspace: &str, session: &str, workdir: &str) -> Result<i32> {
    validate_name_for("zellij-open-session", "workspace", workspace)?;
    if !profile_dir.is_dir() {
        return Err(AwError::new(
            format!(
                "zellij-open-session: missing profile directory {}",
                profile_dir.display()
            ),
            1,
        ));
    }
    let fallback_profile_name = profile_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("profile")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let profile_name = profile_value(
        &profile_dir.join("profile.conf"),
        "name",
        &fallback_profile_name,
    );
    let session_spec = profile_dir.join(format!("{}.tabs", workspace));
    if !session_spec.is_file() {
        return Err(AwError::new(
            format!("zwork: missing workspace spec {}", session_spec.display()),
            1,
        ));
    }
    let layout_dir = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(format!("agent-workspace-{}", unsafe { libc_uid() }));
    fs::create_dir_all(&layout_dir)?;
    let layout_file = layout_dir.join(format!("{}-{}.kdl", profile_name, workspace));
    fs::write(&layout_file, render_layout(&session_spec, workdir)?)?;
    let tab_order = read_tab_lines(&session_spec)?;
    let session = if session.is_empty() {
        default_session_name_from_profile_dir(profile_dir, workspace)
    } else {
        session.to_string()
    };
    launch_session(&layout_file, &session, workdir, &tab_order)
}

fn launch_session_command(args: &[String]) -> Result<i32> {
    if args.len() < 4 {
        return Err(AwError::new(
            "usage: zellij-launch-session <layout-file> <session> <workdir> <tab-name>...",
            2,
        ));
    }
    launch_session(Path::new(&args[0]), &args[1], &args[2], &args[3..])
}

fn launch_session(
    layout_file: &Path,
    session: &str,
    workdir: &str,
    tabs: &[String],
) -> Result<i32> {
    validate_name_for("zellij-launch-session", "session", session)?;
    if !layout_file.is_file() {
        return Err(AwError::new(
            format!(
                "zellij-launch-session: missing layout file {}",
                layout_file.display()
            ),
            1,
        ));
    }
    let original_zellij = env::var_os("ZELLIJ");
    let original_session_name = env::var_os("ZELLIJ_SESSION_NAME");
    let current_session = env::var("ZELLIJ_SESSION_NAME").unwrap_or_default();
    let inside_zellij = env::var("ZELLIJ").is_ok_and(|value| !value.is_empty() && value != "0")
        || !current_session.is_empty();
    if inside_zellij
        && !current_session.is_empty()
        && current_session != session
        && env::var("AW_SWITCH_SESSION").unwrap_or_default() != "1"
    {
        return Err(AwError::new(
            format!(
                "zellij-launch-session: cannot open session {} from inside active Zellij session {}.\nOpen a separate terminal and run this command there, or set AW_SWITCH_SESSION=1 to switch this whole Zellij client.",
                session, current_session
            ),
            1,
        ));
    }
    if Path::new(workdir).is_dir() {
        let _ = env::set_current_dir(workdir);
    }
    repair_saved_session_shells(session)?;
    if inside_zellij && env::var("AW_SWITCH_SESSION").unwrap_or_default() == "1" {
        return zellij_passthrough(&[
            "action",
            "switch-session",
            "--layout",
            &path_string(layout_file),
            "--cwd",
            workdir,
            session,
        ]);
    }
    env::set_var("ZELLIJ_SESSION_TAB_DEFAULT_CWD", workdir);
    env::set_var("ZELLIJ_SESSION_TAB_ORDER_CREATE_MISSING", "1");
    env::set_var("ZELLIJ_SESSION_TAB_ORDER_STRICT", "1");
    let _ = session_tab_order(session, tabs);

    if env::var("ZELLIJ_AGENT_TAB_WATCHER_DISABLE").unwrap_or_default() != "1" {
        env::set_var("ZELLIJ", "1");
        env::set_var("ZELLIJ_SESSION_NAME", session);
        let _ = watcher_command(&["--start".to_string()]);
        restore_env_var("ZELLIJ", original_zellij.as_deref());
        restore_env_var("ZELLIJ_SESSION_NAME", original_session_name.as_deref());
    }
    if inside_zellij && current_session == session {
        return focus_existing_session(session);
    }
    if inside_zellij {
        return zellij_passthrough(&[
            "action",
            "switch-session",
            "--layout",
            &path_string(layout_file),
            "--cwd",
            workdir,
            session,
        ]);
    }
    if session_exists(session)? {
        return attach_existing_session(session);
    }
    zellij_passthrough(&[
        "--layout",
        &path_string(layout_file),
        "attach",
        "--force-run-commands",
        session,
        "--create",
        "options",
        "--mirror-session",
        "true",
    ])
}

fn attach_existing_session(session: &str) -> Result<i32> {
    let status = Command::new("zellij")
        .args([
            "attach",
            "--force-run-commands",
            session,
            "options",
            "--mirror-session",
            "true",
        ])
        .status()?;
    Ok(status.code().unwrap_or(1))
}

fn focus_existing_session(session: &str) -> Result<i32> {
    zellij_passthrough(&["action", "switch-session", session])
}

fn restore_env_var(key: &str, value: Option<&std::ffi::OsStr>) {
    match value {
        Some(value) => env::set_var(key, value),
        None => env::remove_var(key),
    }
}

fn session_tab_order_command(args: &[String]) -> Result<i32> {
    if args.len() < 2 {
        return Err(AwError::new(
            "usage: zellij-session-tab-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    session_tab_order(&args[0], &args[1..])
}

fn live_tab_order_command(args: &[String]) -> Result<i32> {
    if args.len() < 2 {
        return Err(AwError::new(
            "usage: zellij-live-tab-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    live_tab_order(&args[0], &args[1..])
}

fn saved_session_order_command(args: &[String]) -> Result<i32> {
    if args.len() < 2 {
        return Err(AwError::new(
            "usage: zellij-saved-session-order <session> <tab-name>[<tab><cwd>]...",
            2,
        ));
    }
    saved_session_order(&args[0], &args[1..])
}

fn new_scratch_tab_command(args: &[String]) -> Result<i32> {
    let base = args.first().map(String::as_str).unwrap_or("scratch");
    validate_name("tab", base).map_err(|_| {
        AwError::new(
            format!("zellij-new-scratch-tab: tab name may only use letters, numbers, dot, underscore, and dash: {}", base),
            2,
        )
    })?;
    let output = Command::new("zellij")
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null())
        .output();
    let mut names = Vec::new();
    if let Ok(output) = output {
        if let Ok(serde_json::Value::Array(tabs)) =
            serde_json::from_slice::<serde_json::Value>(&output.stdout)
        {
            for tab in tabs {
                if let Some(name) = tab.get("name").and_then(|v| v.as_str()) {
                    names.push(base_name(name));
                }
            }
        }
    }
    let mut next = base.to_string();
    let mut index = 1;
    while names.iter().any(|name| name == &next) {
        next = format!("{}{}", base, index);
        index += 1;
    }
    let cwd = env::current_dir().unwrap_or_else(|_| home_dir());
    let _ = Command::new("zellij")
        .args([
            "action",
            "new-tab",
            "--name",
            &next,
            "--cwd",
            &path_string(&cwd),
        ])
        .stdout(Stdio::null())
        .status()?;
    let _ = Command::new("zellij")
        .args(["action", "save-session"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(0)
}

fn workspace_init_command(args: &[String]) -> Result<i32> {
    let mut config_dir = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                config_dir = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "-h" | "--help" => {
                eprintln!("usage: zellij-workspace-init --config <profile-dir>");
                return Ok(0);
            }
            other => {
                return Err(AwError::new(
                    format!("zellij-workspace-init: unknown argument {}\nusage: zellij-workspace-init --config <profile-dir>", other),
                    2,
                ));
            }
        }
    }
    if config_dir.is_empty() {
        return Err(AwError::new(
            "usage: zellij-workspace-init --config <profile-dir>",
            2,
        ));
    }
    install_profile(Path::new(&config_dir), false)?;
    Ok(0)
}

fn workspace_doctor_command(args: &[String]) -> Result<i32> {
    let mut config_dir = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                config_dir = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "-h" | "--help" => {
                eprintln!("usage: zellij-workspace-doctor [--config <profile-dir>]");
                return Ok(0);
            }
            other => {
                return Err(AwError::new(
                    format!("zellij-workspace-doctor: unknown argument {}\nusage: zellij-workspace-doctor [--config <profile-dir>]", other),
                    2,
                ));
            }
        }
    }
    doctor(&config_dir)
}

fn doctor(config_dir: &str) -> Result<i32> {
    let mut failures = 0;
    let local_bin = local_bin_dir();
    let internal_bin = env::var_os("ZELLIJ_WORKSPACES_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(aw_private_bin_dir);
    if Command::new("zellij")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
    {
        println!("ok: zellij installed");
    } else {
        eprintln!("missing: zellij command");
        failures += 1;
    }
    check_file(&aw_config_file(), &mut failures);
    check_exec(&local_bin.join("aw"), &mut failures);
    check_absent(&local_bin.join("goob"), &mut failures);
    for executable in [
        "zwork",
        "zellij-launch-session",
        "zellij-open-session",
        "zellij-render-layout",
        "zellij-saved-session-order",
        "zellij-live-tab-order",
        "zellij-session-tab-order",
        "zellij-new-scratch-tab",
        "zellij-workspace-init",
        "zellij-workspace-doctor",
    ] {
        check_exec(&internal_bin.join(executable), &mut failures);
        check_absent(&local_bin.join(executable), &mut failures);
    }
    check_exec(
        &internal_bin.join(".zellij-agent-tab-watcher"),
        &mut failures,
    );
    check_exec(&local_bin.join(".zellij-new-scratch-tab"), &mut failures);
    check_absent(&local_bin.join(".zellij-agent-tab-watcher"), &mut failures);
    check_absent(&local_bin.join(".zellij-codex-tab-watcher"), &mut failures);
    if !config_dir.is_empty() {
        let config = PathBuf::from(config_dir);
        if !config.is_dir() {
            eprintln!("missing profile config directory: {}", config.display());
            failures += 1;
        } else {
            let profile_name = profile_value(
                &config.join("profile.conf"),
                "name",
                config.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            );
            let target = aw_profile_dir_candidates(&profile_name)
                .into_iter()
                .find(|path| path.is_dir())
                .unwrap_or_else(|| aw_profile_dir_candidates(&profile_name)[0].clone());
            check_file(&config.join("profile.conf"), &mut failures);
            check_file(&target.join("profile.conf"), &mut failures);
            for tabs_file in tabs_files(&config) {
                let workspace = tabs_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_string();
                check_file(
                    &target.join(tabs_file.file_name().unwrap_or_default()),
                    &mut failures,
                );
                let session = default_session_name_from_profile_dir(&config, &workspace);
                check_runtime_workspace_order(&workspace, &session, &tabs_file, &mut failures);
            }
        }
    }
    if failures > 0 {
        return Err(AwError::new(
            format!("zellij-workspace-doctor: {} check(s) failed", failures),
            1,
        ));
    }
    println!("zellij-workspace-doctor: all checks passed");
    Ok(0)
}

fn check_runtime_workspace_order(
    workspace: &str,
    session: &str,
    tabs_file: &Path,
    failures: &mut i32,
) {
    let expected = tabs_file_order(tabs_file);
    if expected.is_empty() {
        return;
    }
    let live_session = if session_exists(session).unwrap_or(false) {
        Some(session)
    } else if session != workspace && session_exists(workspace).unwrap_or(false) {
        Some(workspace)
    } else {
        None
    };
    if let Some(live_session) = live_session {
        let live = live_session_order(live_session);
        if !live.is_empty() {
            if live == expected {
                println!("ok: live tab order {}", workspace);
            } else {
                eprintln!(
                    "mismatch: live tab order {}\nexpected: {}\nactual:   {}",
                    workspace, expected, live
                );
                *failures += 1;
            }
        }
    }
    let mut saved = saved_layout_order(session);
    if saved.is_empty() && session != workspace {
        saved = saved_layout_order(workspace);
    }
    if !saved.is_empty() {
        if saved == expected {
            println!("ok: saved tab order {}", workspace);
        } else {
            eprintln!(
                "mismatch: saved tab order {}\nexpected: {}\nactual:   {}",
                workspace, expected, saved
            );
            *failures += 1;
        }
    }
}

fn tabs_file_order(tabs_file: &Path) -> String {
    fs::read_to_string(tabs_file)
        .map(|contents| {
            contents
                .lines()
                .filter_map(|line| line.split('\t').next())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn saved_layout_order(session: &str) -> String {
    let layout_file = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("zellij/contract_version_1/session_info")
        .join(session)
        .join("session-layout.kdl");
    fs::read_to_string(layout_file)
        .map(|contents| {
            contents
                .lines()
                .filter_map(|line| {
                    let line = line.trim_start();
                    if !line.starts_with("tab name=\"") {
                        return None;
                    }
                    line.split('"').nth(1).map(base_name)
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn live_session_order(session: &str) -> String {
    Command::new("zellij")
        .env("ZELLIJ_SESSION_NAME", session)
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|output| serde_json::from_slice::<serde_json::Value>(&output.stdout).ok())
        .and_then(|value| value.as_array().cloned())
        .map(|mut tabs| {
            tabs.sort_by_key(|tab| {
                tab.get("position")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0)
            });
            tabs.into_iter()
                .filter_map(|tab| {
                    tab.get("name")
                        .and_then(serde_json::Value::as_str)
                        .map(base_name)
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn tabs_files(config: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(config) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("tabs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn check_file(path: &Path, failures: &mut i32) {
    if path.is_file() {
        println!("ok: {}", path.display());
    } else {
        eprintln!("missing: {}", path.display());
        *failures += 1;
    }
}

fn check_exec(path: &Path, failures: &mut i32) {
    if is_executable(path) {
        println!("ok: {}", path.display());
    } else {
        eprintln!("missing executable: {}", path.display());
        *failures += 1;
    }
}

fn check_absent(path: &Path, failures: &mut i32) {
    if !path.exists() {
        println!("ok: absent {}", path.display());
    } else {
        eprintln!("unexpected public helper: {}", path.display());
        *failures += 1;
    }
}

fn resolve_profile_dir(profile_name: &str) -> Result<PathBuf> {
    let direct = PathBuf::from(profile_name);
    if direct.is_dir() {
        return Ok(direct.canonicalize().unwrap_or(direct));
    }
    if let Some(root) = env::var_os("ZELLIJ_PROFILE_DIR") {
        let candidate = PathBuf::from(root).join(profile_name);
        if candidate.is_dir() {
            return Ok(candidate.canonicalize().unwrap_or(candidate));
        }
    }
    for candidate in aw_profile_dir_candidates(profile_name) {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(AwError::new(
        format!("zwork: missing profile {}", profile_name),
        1,
    ))
}

fn session_exists(session: &str) -> Result<bool> {
    let output = Command::new("zellij")
        .args(["list-sessions", "--short", "--no-formatting"])
        .stderr(Stdio::null())
        .output();
    Ok(output.is_ok_and(|output| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == session)
    }))
}

fn repair_saved_session_shells(session: &str) -> Result<()> {
    let user_shell = env::var("SHELL").unwrap_or_default();
    let broken_shell =
        env::var("ZELLIJ_REPAIR_BROKEN_SHELL").unwrap_or_else(|_| "/usr/bin/zsh".to_string());
    if user_shell.is_empty()
        || user_shell == broken_shell
        || !Path::new(&user_shell).is_file()
        || Path::new(&broken_shell).is_file()
    {
        return Ok(());
    }
    let session_dir = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("zellij/contract_version_1/session_info")
        .join(session);
    for file in ["session-layout.kdl", "session-metadata.kdl"] {
        let path = session_dir.join(file);
        if path.is_file() {
            let contents = fs::read_to_string(&path)?;
            fs::write(path, contents.replace(&broken_shell, &user_shell))?;
        }
    }
    Ok(())
}

fn validate_name_for(command: &str, kind: &str, value: &str) -> Result<()> {
    validate_name(kind, value).map_err(|_| {
        AwError::new(
            format!(
                "{}: {} names may only use letters, numbers, dot, underscore, and dash",
                command, kind
            ),
            2,
        )
    })
}

unsafe fn libc_uid() -> u32 {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    getuid()
}
