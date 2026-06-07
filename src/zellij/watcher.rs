use std::collections::HashSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::error::{AwError, Result};
use crate::paths::home_dir;
use crate::tab_order::saved_session_order;
use crate::zellij::{base_name, value_to_string};

pub fn watcher_command(args: &[String]) -> Result<i32> {
    let session = match env::var("ZELLIJ_SESSION_NAME") {
        Ok(session) if !session.is_empty() => session,
        _ => return Ok(0),
    };
    let state = WatcherState::new(session);
    match args.first().map(String::as_str).unwrap_or("--once") {
        "--once" => run_once(&state),
        "--reset" => {
            rename_tabs_to_base_names();
            let _ = normalize_saved_session_order(&state);
            let _ = fs::remove_dir_all(&state.dir);
            fs::create_dir_all(&state.dir)?;
            Ok(0)
        }
        "--status" => {
            if !state.status_file.is_file() {
                return Err(AwError::new(
                    format!("No watcher status yet for session {}.", state.session),
                    1,
                ));
            }
            println!("PANE\tTAB\tNAME\tTITLE\tBUSY\tNOTIFY");
            print!("{}", fs::read_to_string(&state.status_file)?);
            Ok(0)
        }
        "--log" => tail_file(
            &state.title_log_file,
            args.get(1).map(String::as_str).unwrap_or("40"),
            &state.session,
            "title",
        ),
        "--watcher-log" => tail_file(
            &state.log_file,
            args.get(1).map(String::as_str).unwrap_or("40"),
            &state.session,
            "watcher",
        ),
        "--saved-loop" => {
            let requested = args.get(1).cloned().unwrap_or_default();
            if requested != state.session {
                return Ok(0);
            }
            loop {
                let _ = normalize_saved_session_order(&state);
                let seconds = env::var("ZELLIJ_AGENT_TAB_WATCHER_SAVED_POLL_SECONDS")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.2);
                thread::sleep(Duration::from_secs_f64(seconds));
            }
        }
        "--loop" => {
            let requested = args.get(1).cloned().unwrap_or_default();
            if requested != state.session {
                return Ok(0);
            }
            loop {
                let _ = normalize_saved_session_order(&state);
                let _ = run_once(&state);
                let seconds = env::var("ZELLIJ_AGENT_TAB_WATCHER_POLL_SECONDS")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.25);
                thread::sleep(Duration::from_secs_f64(seconds));
            }
        }
        "--start" => start_watchers(&state),
        "--stop" => stop_watchers(&state),
        "--restart" => {
            let _ = stop_watchers(&state);
            start_watchers(&state)
        }
        _ => run_once(&state),
    }
}

struct WatcherState {
    session: String,
    dir: PathBuf,
    status_file: PathBuf,
    title_log_file: PathBuf,
    log_file: PathBuf,
    pid_file: PathBuf,
    saved_pid_file: PathBuf,
}

impl WatcherState {
    fn new(session: String) -> Self {
        let runtime = env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let dir = runtime.join(format!(
            "zellij-agent-tab-watcher-{}-{}",
            unsafe { uid() },
            session
        ));
        Self {
            session,
            status_file: dir.join("status.tsv"),
            title_log_file: dir.join("title.log"),
            log_file: dir.join("watcher.log"),
            pid_file: dir.join("watcher.pid"),
            saved_pid_file: dir.join("saved-watcher.pid"),
            dir,
        }
    }
}

fn start_watchers(state: &WatcherState) -> Result<i32> {
    fs::create_dir_all(&state.dir)?;
    start_loop(state, "--loop", &state.pid_file)?;
    start_loop(state, "--saved-loop", &state.saved_pid_file)?;
    Ok(0)
}

fn start_loop(state: &WatcherState, mode: &str, pid_file: &PathBuf) -> Result<()> {
    if let Some(pid) = read_live_pid(pid_file) {
        append_watcher_log(state, &format!("{} already running as pid {}", mode, pid));
        return Ok(());
    }

    let executable = env::current_exe()?;
    let executable_name = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let helper_argv0 = matches!(
        executable_name,
        ".zellij-agent-tab-watcher" | "zellij-agent-tab-watcher"
    );
    let mut command = Command::new(executable);
    if !helper_argv0 {
        command.arg(".zellij-agent-tab-watcher");
    }
    command
        .arg(mode)
        .arg(&state.session)
        .env("ZELLIJ_SESSION_NAME", &state.session)
        .stdin(Stdio::null());

    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_file)?;
    let err = log.try_clone()?;
    let child = command
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(err))
        .spawn()?;
    fs::write(pid_file, format!("{}\n", child.id()))?;
    append_watcher_log(state, &format!("started {} as pid {}", mode, child.id()));
    Ok(())
}

fn stop_watchers(state: &WatcherState) -> Result<i32> {
    stop_loop(state, "--loop", &state.pid_file);
    stop_loop(state, "--saved-loop", &state.saved_pid_file);
    Ok(0)
}

fn stop_loop(state: &WatcherState, mode: &str, pid_file: &PathBuf) {
    let pid = fs::read_to_string(pid_file)
        .ok()
        .and_then(|contents| contents.trim().parse::<u32>().ok());
    if let Some(pid) = pid {
        if process_alive(pid) {
            let _ = Command::new("kill")
                .arg(pid.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            append_watcher_log(state, &format!("stopped {} pid {}", mode, pid));
        }
    }
    let _ = fs::remove_file(pid_file);
}

fn read_live_pid(pid_file: &PathBuf) -> Option<u32> {
    let pid = fs::read_to_string(pid_file)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()?;
    if process_alive(pid) {
        Some(pid)
    } else {
        let _ = fs::remove_file(pid_file);
        None
    }
}

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn append_watcher_log(state: &WatcherState, message: &str) {
    let _ = fs::create_dir_all(&state.dir);
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_file)
        .and_then(|mut file| {
            use std::io::Write;
            writeln!(file, "{}", message)
        });
}

fn run_once(state: &WatcherState) -> Result<i32> {
    fs::create_dir_all(&state.dir)?;
    let panes = list_panes();
    if panes.is_empty() {
        return Ok(0);
    }
    let tabs = list_tabs();
    let mut busy_tabs = HashSet::<String>::new();
    for pane in &panes {
        let title = pane.get("title").and_then(Value::as_str).unwrap_or("");
        let pane_id = pane.get("id").map(value_to_string).unwrap_or_default();
        if is_busy_title(title) || is_busy_screen(&pane_id, title) {
            if let Some(tab_id) = pane.get("tab_id").map(value_to_string) {
                busy_tabs.insert(tab_id);
            }
        }
    }

    let mut notify_tabs = HashSet::<String>::new();
    for tab in &tabs {
        let tab_id = tab.get("tab_id").map(value_to_string).unwrap_or_default();
        let active = tab.get("active").and_then(Value::as_bool).unwrap_or(false);
        let current_busy = busy_tabs.contains(&tab_id);
        let previous_file = state.dir.join(format!("tab-busy-{}.txt", tab_id));
        let previous_busy = fs::read_to_string(&previous_file).unwrap_or_default() == "true";
        let notify_file = state.dir.join(format!("notify-{}.txt", tab_id));
        if current_busy || active {
            let _ = fs::remove_file(&notify_file);
        } else if previous_busy {
            fs::write(&notify_file, "true")?;
        }
        fs::write(previous_file, if current_busy { "true" } else { "false" })?;
        if notify_file.is_file() {
            notify_tabs.insert(tab_id);
        }
    }

    let mut status = String::new();
    for pane in &panes {
        let pane_id = pane.get("id").map(value_to_string).unwrap_or_default();
        let tab_id = pane.get("tab_id").map(value_to_string).unwrap_or_default();
        let tab_name = pane.get("tab_name").and_then(Value::as_str).unwrap_or("");
        let title = pane.get("title").and_then(Value::as_str).unwrap_or("");
        status.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            pane_id,
            tab_id,
            tab_name,
            title,
            busy_tabs.contains(&tab_id),
            notify_tabs.contains(&tab_id)
        ));
    }
    fs::write(&state.status_file, status)?;

    for tab in &tabs {
        let tab_id = tab.get("tab_id").map(value_to_string).unwrap_or_default();
        let tab_name = tab.get("name").and_then(Value::as_str).unwrap_or("");
        let base = base_name(tab_name);
        let next = if busy_tabs.contains(&tab_id) {
            format!("{} 🤖", base)
        } else if notify_tabs.contains(&tab_id) {
            format!("{} 🔔", base)
        } else {
            base
        };
        if tab_name != next {
            let _ = Command::new("zellij")
                .args(["action", "rename-tab-by-id", &tab_id, &next])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
    normalize_saved_session_order(state)?;
    Ok(0)
}

fn normalize_saved_session_order(state: &WatcherState) -> Result<()> {
    let mut order = configured_session_tab_order(&state.session);
    if order.is_empty() {
        order = list_tabs()
            .into_iter()
            .filter_map(|tab| tab.get("name").and_then(Value::as_str).map(base_name))
            .collect();
    }
    if !order.is_empty() {
        env::set_var("ZELLIJ_SESSION_TAB_ORDER_SAVED_ONLY", "1");
        env::set_var("ZELLIJ_SESSION_TAB_ORDER_STRICT", "1");
        let _ = saved_session_order(&state.session, &order);
    }
    Ok(())
}

fn configured_session_tab_order(session: &str) -> Vec<String> {
    let default = home_dir().join(".local/share/agent-workspace/default-profile");
    let Ok(profile_name) = fs::read_to_string(default) else {
        return Vec::new();
    };
    let tabs_file = home_dir()
        .join(".local/share/agent-workspace/profiles")
        .join(profile_name.trim())
        .join(format!("{}.tabs", session));
    fs::read_to_string(tabs_file)
        .map(|contents| {
            contents
                .lines()
                .filter_map(|line| line.split('\t').next())
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn rename_tabs_to_base_names() {
    for tab in list_tabs() {
        let id = tab.get("tab_id").map(value_to_string).unwrap_or_default();
        let name = tab.get("name").and_then(Value::as_str).unwrap_or("");
        let base = base_name(name);
        if name != base {
            let _ = Command::new("zellij")
                .args(["action", "rename-tab-by-id", &id, &base])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

fn list_tabs() -> Vec<Value> {
    Command::new("zellij")
        .args(["action", "list-tabs", "--json"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|output| serde_json::from_slice::<Value>(&output.stdout).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}

fn list_panes() -> Vec<Value> {
    Command::new("zellij")
        .args(["action", "list-panes", "--all", "--json"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|output| serde_json::from_slice::<Value>(&output.stdout).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter(|pane| {
            !pane
                .get("is_plugin")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .collect()
}

fn is_busy_title(title: &str) -> bool {
    title.chars().next().is_some_and(|ch| {
        matches!(
            ch,
            '⠂' | '⠐' | '⠋' | '⠙' | '⠹' | '⠸' | '⠼' | '⠴' | '⠦' | '⠧' | '⠇' | '⠏'
        )
    }) || title.starts_with('✦')
}

fn is_busy_screen(pane_id: &str, title: &str) -> bool {
    let lower = title.to_ascii_lowercase();
    if !(lower.contains("claude") || lower.contains("gemini") || lower.contains("codex")) {
        return false;
    }
    let output = Command::new("zellij")
        .args(["action", "dump-screen", "--pane-id", pane_id])
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else {
        return false;
    };
    let screen = String::from_utf8_lossy(&output.stdout);
    screen.contains("Deciphering…")
        || screen.contains("Thinking…")
        || screen.contains("· ")
        || screen.contains("✦ ")
}

fn tail_file(path: &PathBuf, count: &str, session: &str, kind: &str) -> Result<i32> {
    if !path.is_file() {
        return Err(AwError::new(
            format!("No {} log yet for session {}.", kind, session),
            1,
        ));
    }
    let count = count.parse::<usize>().unwrap_or(40);
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();
    for line in lines.iter().skip(lines.len().saturating_sub(count)) {
        println!("{}", line);
    }
    Ok(0)
}

unsafe fn uid() -> u32 {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    getuid()
}
