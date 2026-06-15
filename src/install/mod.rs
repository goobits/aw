use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::{AwError, Result};
use crate::paths::{
    aw_completions_dir, aw_config_file, aw_home, aw_plugins_dir, aw_private_bin_dir,
    aw_profiles_dir, home_dir, local_bin_dir,
};
use serde_json::{json, Value};

const CONFIG_KDL: &str = include_str!("../../config.kdl");
const ZSH_COMPLETION: &str = include_str!("../../completions/_aw");
const BASH_COMPLETION: &str = include_str!("../../completions/aw.bash");
const ROOT_AGENTS_TEMPLATE: &str = include_str!("../../agents/.agents/templates/root-AGENTS.md");
const PROJECT_TEMPLATE: &str = include_str!("../../agents/.agents/templates/project.md");
const START_MARKER: &str = "# >>> zellij workspaces >>>";
const END_MARKER: &str = "# <<< zellij workspaces <<<";
const CODEX_STATUS_LINE: &str = r#"status_line = [
  "model-with-reasoning",
  "run-state",
  "context-used",
  "git-branch",
  "current-dir",
]
"#;
const CLAUDE_STATUS_LINE_SCRIPT: &str = r#"#!/usr/bin/env node
const fs = require("node:fs");
const { execFileSync } = require("node:child_process");

let data = {};
try {
  const input = fs.readFileSync(0, "utf8");
  data = input.trim() ? JSON.parse(input) : {};
} catch {
  data = {};
}

const model =
  data.model?.display_name ||
  data.model?.id ||
  (typeof data.model === "string" ? data.model : "") ||
  "claude";
const cwd = data.workspace?.current_dir || data.cwd || process.cwd();
const dir = String(cwd).split(/[\\/]/).filter(Boolean).pop() || cwd || ".";
const context = data.context_window?.used_percentage;
const contextText =
  typeof context === "number" && Number.isFinite(context)
    ? `${Math.round(context)}% ctx`
    : "";

let branch = data.worktree?.branch || "";
if (!branch) {
  try {
    branch = execFileSync("git", ["-C", cwd, "branch", "--show-current"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 100,
    }).trim();
  } catch {
    branch = "";
  }
}

console.log([model, contextText, branch, dir].filter(Boolean).join(" | "));
"#;

const INTERNAL_EXECUTABLES: &[&str] = &[
    "zellij-saved-session-order",
    "zellij-live-tab-order",
    "zellij-session-tab-order",
    "zellij-launch-session",
    "zellij-open-session",
    "zellij-render-layout",
    "zellij-new-scratch-tab",
    "zwork",
    "zellij-workspace-init",
    "zellij-workspace-doctor",
    "zellij-agent-tab-watcher",
];

pub fn install_workspace_setup() -> Result<i32> {
    let original_path = env::var("PATH").unwrap_or_default();
    install_zellij_binary()?;
    stop_stale_watchers();
    install_files()?;
    if env::var("ZELLIJ_INSTALL_SHELL_RC").unwrap_or_else(|_| "1".to_string()) != "0" {
        update_shell_file(&home_dir().join(".zshrc"))?;
        update_shell_file(&home_dir().join(".bashrc"))?;
    }

    println!("Installed Agent Workspace setup.");
    if !path_has_local_bin(&original_path) {
        println!("Open a new shell or run: export PATH=\"$HOME/.local/bin:$PATH\"");
    }
    println!("In a project directory, create a profile with: aw main=app,server,infra,scratch");
    println!("Then open a workspace with: aw main");
    Ok(0)
}

pub fn install_repo_adapters(dry_run: bool) -> Result<()> {
    let root = env::current_dir()?;
    let workspace_root = root.join("infra/aw");
    if !workspace_root.join("agents/.agents/AGENTS.md").is_file() {
        return Err(AwError::new(
            "aw install failed: infra/aw/agents/.agents/AGENTS.md is missing",
            1,
        ));
    }

    for file in REPO_ADAPTER_FILES {
        ensure_repo_file(&root, file, dry_run)?;
    }
    for link in REPO_ADAPTER_LINKS {
        ensure_repo_symlink(&root, link, dry_run)?;
    }

    println!("done    {}", display_path(&root, &workspace_root));
    Ok(())
}

struct RepoAdapterFile {
    file: &'static str,
    contents: &'static str,
    source: &'static str,
}

struct RepoAdapterLink {
    link: &'static str,
    target: &'static str,
}

const REPO_ADAPTER_FILES: &[RepoAdapterFile] = &[
    RepoAdapterFile {
        file: "AGENTS.md",
        contents: ROOT_AGENTS_TEMPLATE,
        source: "infra/aw/agents/.agents/templates/root-AGENTS.md",
    },
    RepoAdapterFile {
        file: ".agents.local/project.md",
        contents: PROJECT_TEMPLATE,
        source: "infra/aw/agents/.agents/templates/project.md",
    },
];

const REPO_ADAPTER_LINKS: &[RepoAdapterLink] = &[
    RepoAdapterLink {
        link: ".agents",
        target: "infra/aw/agents/.agents",
    },
    RepoAdapterLink {
        link: "CLAUDE.md",
        target: "AGENTS.md",
    },
    RepoAdapterLink {
        link: ".claude/skills",
        target: "../.agents/skills",
    },
];

fn ensure_repo_file(root: &Path, item: &RepoAdapterFile, dry_run: bool) -> Result<()> {
    let path = root.join(item.file);
    if path.exists() || fs::symlink_metadata(&path).is_ok() {
        println!("keep    {} already exists; not overwritten", item.file);
        return Ok(());
    }

    if dry_run {
        println!("would   {} from {}", item.file, item.source);
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, item.contents)?;
    println!("created {} from {}", item.file, item.source);
    Ok(())
}

fn ensure_repo_symlink(root: &Path, item: &RepoAdapterLink, dry_run: bool) -> Result<()> {
    let link_path = root.join(item.link);
    match fs::symlink_metadata(&link_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() && link_target_matches(&link_path, item.target) {
                println!("ok      {} -> {}", item.link, item.target);
            } else if metadata.file_type().is_symlink() {
                if dry_run {
                    println!("would   {} -> {}", item.link, item.target);
                    return Ok(());
                }
                fs::remove_file(&link_path)?;
                create_symlink(item.target, &link_path)?;
                println!("linked  {} -> {}", item.link, item.target);
            } else {
                println!("keep    {} already exists; not overwritten", item.link);
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if dry_run {
                println!("would   {} -> {}", item.link, item.target);
                return Ok(());
            }
            if let Some(parent) = link_path.parent() {
                fs::create_dir_all(parent)?;
            }
            create_symlink(item.target, &link_path)?;
            println!("linked  {} -> {}", item.link, item.target);
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn link_target_matches(link_path: &Path, target: &str) -> bool {
    fs::read_link(link_path)
        .map(|actual| actual == Path::new(target))
        .unwrap_or(false)
}

#[cfg(unix)]
fn create_symlink(target: &str, link_path: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link_path)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_symlink(_target: &str, _link_path: &Path) -> Result<()> {
    Err(AwError::new(
        "aw install failed: symlink creation is only supported on Unix",
        1,
    ))
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn install_zellij_binary() -> Result<()> {
    if command_exists("zellij") || local_bin_dir().join("zellij").is_file() {
        return Ok(());
    }
    if env::var("ZELLIJ_INSTALL_BINARY").unwrap_or_else(|_| "1".to_string()) == "0" {
        eprintln!(
            "zellij is not installed; skipped binary install because ZELLIJ_INSTALL_BINARY=0"
        );
        return Ok(());
    }

    let target = match (env::consts::OS, env::consts::ARCH) {
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        _ => {
            eprintln!(
                "zellij is not installed and this installer does not know platform {}/{}",
                env::consts::OS,
                env::consts::ARCH
            );
            return Ok(());
        }
    };

    for required in ["curl", "tar"] {
        if !command_exists(required) {
            eprintln!(
                "zellij is not installed; need {} to install it automatically",
                required
            );
            return Ok(());
        }
    }
    if !command_exists("sha256sum") && !command_exists("shasum") {
        eprintln!("zellij is not installed; need sha256sum or shasum to install it automatically");
        return Ok(());
    }

    let version = env::var("ZELLIJ_VERSION").unwrap_or_else(|_| "0.44.3".to_string());
    let asset = format!("zellij-{}.tar.gz", target);
    let checksum_asset = format!("zellij-{}.sha256sum", target);
    let base_url = format!(
        "https://github.com/zellij-org/zellij/releases/download/v{}",
        version
    );
    let tmp = env::temp_dir().join(format!("aw-zellij-install-{}", std::process::id()));
    let _guard = TempDirGuard(tmp.clone());
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;

    run_status(
        Command::new("curl")
            .args(["-fsSL", &format!("{}/{}", base_url, asset), "-o"])
            .arg(tmp.join(&asset)),
    )?;
    run_status(
        Command::new("curl")
            .args(["-fsSL", &format!("{}/{}", base_url, checksum_asset), "-o"])
            .arg(tmp.join(&checksum_asset)),
    )?;

    let checksum_contents = fs::read_to_string(tmp.join(&checksum_asset))?;
    let expected = checksum_contents.split_whitespace().next().unwrap_or("");
    if expected.is_empty() {
        return Err(AwError::new(
            format!(
                "could not find checksum for {} in Zellij {}",
                checksum_asset, version
            ),
            1,
        ));
    }

    run_status(
        Command::new("tar")
            .arg("-xzf")
            .arg(tmp.join(&asset))
            .arg("-C")
            .arg(&tmp),
    )?;
    let zellij_binary = tmp.join("zellij");
    if command_exists("sha256sum") {
        let mut child = Command::new("sha256sum")
            .arg("-c")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "{}  {}", expected, zellij_binary.display())?;
        }
        let status = child.wait()?;
        if !status.success() {
            return Err(AwError::new(
                format!("checksum mismatch for {}", checksum_asset),
                1,
            ));
        }
    } else {
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(&zellij_binary)
            .output()?;
        let actual = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        if actual != expected {
            return Err(AwError::new(
                format!("checksum mismatch for {}", checksum_asset),
                1,
            ));
        }
    }

    let local_bin = local_bin_dir();
    fs::create_dir_all(&local_bin)?;
    copy_executable(&zellij_binary, &local_bin.join("zellij"))?;
    Ok(())
}

fn install_files() -> Result<()> {
    let local_bin = local_bin_dir();
    let internal_bin = aw_private_bin_dir();
    let completion_dir = aw_completions_dir();
    let plugin_dir = aw_plugins_dir();
    fs::create_dir_all(aw_home())?;
    fs::create_dir_all(&local_bin)?;
    fs::create_dir_all(&internal_bin)?;
    fs::create_dir_all(&plugin_dir)?;
    fs::create_dir_all(aw_profiles_dir())?;
    fs::create_dir_all(&completion_dir)?;

    fs::write(aw_config_file(), CONFIG_KDL)?;
    fs::write(completion_dir.join("_aw"), ZSH_COMPLETION)?;
    fs::write(completion_dir.join("aw.bash"), BASH_COMPLETION)?;
    install_aw_tab_bar_plugin(&plugin_dir)?;
    ensure_codex_status_line()?;
    ensure_claude_status_line(&internal_bin.join("claude-statusline"))?;

    let source_binary = env::current_exe()?;
    for executable in INTERNAL_EXECUTABLES {
        let target = if *executable == "zellij-agent-tab-watcher" {
            ".zellij-agent-tab-watcher"
        } else {
            executable
        };
        copy_executable(&source_binary, &internal_bin.join(target))?;
    }
    copy_executable(&source_binary, &local_bin.join("aw"))?;
    copy_executable(&source_binary, &local_bin.join(".zellij-new-scratch-tab"))?;

    for executable in INTERNAL_EXECUTABLES {
        let _ = fs::remove_file(local_bin.join(executable));
    }
    for stale in [
        "goob",
        ".zellij-agent-tab-watcher",
        ".zellij-codex-tab-watcher",
    ] {
        let _ = fs::remove_file(local_bin.join(stale));
    }
    Ok(())
}

fn install_aw_tab_bar_plugin(plugin_dir: &Path) -> Result<()> {
    let source = env::var_os("AW_TAB_BAR_WASM_SOURCE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("plugins/aw-tab-bar/target/wasm32-wasip1/release/aw-tab-bar.wasm")
        });
    let installed = plugin_dir.join("aw-tab-bar.wasm");
    if source.is_file() {
        fs::copy(source, &installed)?;
        grant_aw_tab_bar_permissions(&installed)?;
    }
    Ok(())
}

fn grant_aw_tab_bar_permissions(plugin_path: &Path) -> Result<()> {
    let cache_path = zellij_permissions_cache_path();
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let plugin_url = format!("file:{}", plugin_path.display());
    let plugin_path = plugin_path.display().to_string();
    let existing = fs::read_to_string(&cache_path).unwrap_or_default();
    let grant_names = [
        plugin_url.as_str(),
        plugin_path.as_str(),
        "aw-tab-bar",
        "aw-tab-bar.wasm",
    ];

    let mut next = existing;
    for grant_name in grant_names {
        next = remove_permission_block(&next, grant_name);
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(&format!(
            "{} {{\n    ReadApplicationState\n    ChangeApplicationState\n    RunCommands\n    InterceptInput\n}}\n",
            kdl_identifier(grant_name)
        ));
    }
    fs::write(cache_path, next)?;
    Ok(())
}

fn zellij_permissions_cache_path() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("zellij/permissions.kdl")
}

fn remove_permission_block(contents: &str, plugin_url: &str) -> String {
    let quoted = format!("{} {{", kdl_identifier(plugin_url));
    let mut output = String::new();
    let mut skipping = false;

    for line in contents.lines() {
        if !skipping && line.trim() == quoted {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == "}" {
                skipping = false;
            }
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }

    output
}

fn kdl_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn ensure_codex_status_line() -> Result<()> {
    let codex_dir = home_dir().join(".codex");
    fs::create_dir_all(&codex_dir)?;
    let config_path = codex_dir.join("config.toml");
    let contents = fs::read_to_string(&config_path).unwrap_or_default();
    if has_codex_status_line(&contents) {
        return Ok(());
    }

    let next = if contents.lines().any(|line| line.trim() == "[tui]") {
        insert_codex_status_line(contents)
    } else {
        append_codex_tui_section(contents)
    };
    fs::write(config_path, next)?;
    Ok(())
}

fn has_codex_status_line(contents: &str) -> bool {
    contents
        .lines()
        .any(|line| line.trim_start().starts_with("status_line"))
}

fn insert_codex_status_line(contents: String) -> String {
    let mut next = String::new();
    let mut inserted = false;
    for line in contents.lines() {
        next.push_str(line);
        next.push('\n');
        if !inserted && line.trim() == "[tui]" {
            next.push_str(CODEX_STATUS_LINE);
            inserted = true;
        }
    }
    next
}

fn append_codex_tui_section(mut contents: String) -> String {
    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    if !contents.is_empty() {
        contents.push('\n');
    }
    contents.push_str("[tui]\n");
    contents.push_str(CODEX_STATUS_LINE);
    contents
}

fn ensure_claude_status_line(script_path: &Path) -> Result<()> {
    fs::write(script_path, CLAUDE_STATUS_LINE_SCRIPT)?;
    make_executable(script_path)?;

    let claude_dir = home_dir().join(".claude");
    fs::create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.json");
    let contents = fs::read_to_string(&settings_path).unwrap_or_default();
    let mut settings = if contents.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&contents).map_err(|error| {
            AwError::new(
                format!(
                    "aw install failed: could not parse {}: {}",
                    settings_path.display(),
                    error
                ),
                1,
            )
        })?
    };

    let Some(object) = settings.as_object_mut() else {
        return Err(AwError::new(
            format!(
                "aw install failed: {} must contain a JSON object",
                settings_path.display()
            ),
            1,
        ));
    };
    if object.contains_key("statusLine") {
        return Ok(());
    }

    object.insert(
        "statusLine".to_string(),
        json!({
            "type": "command",
            "command": shell_quote(&script_path.to_string_lossy()),
        }),
    );
    fs::write(
        settings_path,
        format!("{}\n", serde_json::to_string_pretty(&settings).unwrap()),
    )?;
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn stop_stale_watchers() {
    let Ok(output) = Command::new("ps")
        .arg("-u")
        .arg(env::var("USER").unwrap_or_default())
        .args(["-o", "pid=,args="])
        .output()
    else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let home = home_dir();
    let old_agent = format!(
        "{}/.local/bin/.zellij-agent-tab-watcher --loop",
        home.display()
    );
    let old_codex = format!(
        "{}/.local/bin/.zellij-codex-tab-watcher --loop",
        home.display()
    );
    for line in stdout.lines() {
        if !(line.contains(&old_agent) || line.contains(&old_codex)) {
            continue;
        }
        if let Some(pid) = line.split_whitespace().next() {
            let _ = Command::new("kill")
                .arg(pid)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

fn update_shell_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = fs::read_to_string(path).unwrap_or_default();
    let mut next = String::new();
    let mut skip = false;
    for line in contents.lines() {
        if line == START_MARKER {
            skip = true;
            continue;
        }
        if line == END_MARKER {
            skip = false;
            continue;
        }
        if !skip {
            next.push_str(line);
            next.push('\n');
        }
    }
    next.push('\n');
    next.push_str(START_MARKER);
    next.push('\n');
    next.push_str(shell_block());
    next.push_str(END_MARKER);
    next.push('\n');
    fs::write(path, next)?;
    Ok(())
}

fn shell_block() -> &'static str {
    r#"export PATH="$HOME/.local/bin:$PATH"
alias zj='zellij'

if [[ -t 0 ]]; then
  stty -ixon 2>/dev/null
fi

if [[ -n "${ZSH_VERSION:-}" ]]; then
  fpath=("${AW_HOME:-$HOME/.aw}/completions" "${fpath[@]}")
  autoload -Uz compinit
  compinit -i
  bindkey -M viins '^[^?' backward-kill-word 2>/dev/null
  bindkey -M viins '^[^H' backward-kill-word 2>/dev/null
  bindkey -M emacs '^[^?' backward-kill-word 2>/dev/null
  bindkey -M emacs '^[^H' backward-kill-word 2>/dev/null
fi

if [[ -n "${BASH_VERSION:-}" && -f "${AW_HOME:-$HOME/.aw}/completions/aw.bash" ]]; then
  # shellcheck source=/dev/null
  source "${AW_HOME:-$HOME/.aw}/completions/aw.bash"
fi

if [[ -n "${ZELLIJ:-}" ]]; then
  codex() {
    local arg
    for arg in "$@"; do
      if [[ "$arg" == "--no-alt-screen" ]]; then
        command codex "$@"
        return
      fi
    done
    command codex --no-alt-screen "$@"
  }
fi

if [[ -n "${ZELLIJ:-}" && "${ZELLIJ_AGENT_TAB_WATCHER_DISABLE:-0}" != "1" && -x "${AW_HOME:-$HOME/.aw}/bin/.zellij-agent-tab-watcher" ]]; then
  "${AW_HOME:-$HOME/.aw}/bin/.zellij-agent-tab-watcher" --start
fi
"#
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", command))
        .status()
        .is_ok_and(|status| status.success())
}

fn run_status(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AwError::new("aw: installer command failed", 1))
    }
}

fn copy_executable(source: &Path, target: &Path) -> Result<()> {
    if source.canonicalize().ok() == target.canonicalize().ok() {
        make_executable(target)?;
        return Ok(());
    }
    let tmp = target.with_extension(format!("tmp-{}", std::process::id()));
    let _ = fs::remove_file(&tmp);
    fs::copy(source, &tmp)?;
    make_executable(&tmp)?;
    fs::rename(&tmp, target)?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn path_has_local_bin(path: &str) -> bool {
    let local_bin = local_bin_dir();
    env::split_paths(path).any(|entry| entry == local_bin)
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
