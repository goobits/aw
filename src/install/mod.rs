use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::{AwError, Result};
use crate::paths::home_dir;

const CONFIG_KDL: &str = include_str!("../../config.kdl");
const ZSH_COMPLETION: &str = include_str!("../../completions/_aw");
const BASH_COMPLETION: &str = include_str!("../../completions/aw.bash");
const ROOT_AGENTS_TEMPLATE: &str = include_str!("../../agents/.agents/templates/root-AGENTS.md");
const PROJECT_TEMPLATE: &str = include_str!("../../agents/.agents/templates/project.md");
const START_MARKER: &str = "# >>> zellij workspaces >>>";
const END_MARKER: &str = "# <<< zellij workspaces <<<";

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
    let workspace_root = root.join("infra/agent-workspace");
    if !workspace_root.join("agents/.agents/AGENTS.md").is_file() {
        return Err(AwError::new(
            "aw install failed: infra/agent-workspace/agents/.agents/AGENTS.md is missing",
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
        source: "infra/agent-workspace/agents/.agents/templates/root-AGENTS.md",
    },
    RepoAdapterFile {
        file: ".agents.local/project.md",
        contents: PROJECT_TEMPLATE,
        source: "infra/agent-workspace/agents/.agents/templates/project.md",
    },
];

const REPO_ADAPTER_LINKS: &[RepoAdapterLink] = &[
    RepoAdapterLink {
        link: ".agents",
        target: "infra/agent-workspace/agents/.agents",
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
    if command_exists("zellij") || home_dir().join(".local/bin/zellij").is_file() {
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

    let local_bin = home_dir().join(".local/bin");
    fs::create_dir_all(&local_bin)?;
    copy_executable(&zellij_binary, &local_bin.join("zellij"))?;
    Ok(())
}

fn install_files() -> Result<()> {
    let local_bin = home_dir().join(".local/bin");
    let state_dir = home_dir().join(".local/share/agent-workspace");
    let internal_bin = state_dir.join("bin");
    let completion_dir = state_dir.join("completions");
    fs::create_dir_all(home_dir().join(".config/aw"))?;
    fs::create_dir_all(&local_bin)?;
    fs::create_dir_all(&internal_bin)?;
    fs::create_dir_all(state_dir.join("profiles"))?;
    fs::create_dir_all(&completion_dir)?;

    fs::write(home_dir().join(".config/aw/config.kdl"), CONFIG_KDL)?;
    fs::write(completion_dir.join("_aw"), ZSH_COMPLETION)?;
    fs::write(completion_dir.join("aw.bash"), BASH_COMPLETION)?;

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
    for stale in [".zellij-agent-tab-watcher", ".zellij-codex-tab-watcher"] {
        let _ = fs::remove_file(local_bin.join(stale));
    }
    for stale in ["backend.kdl", "frontend.kdl"] {
        let _ = fs::remove_file(home_dir().join(".config/aw/layouts").join(stale));
    }
    Ok(())
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
  fpath=("$HOME/.local/share/agent-workspace/completions" "${fpath[@]}")
  autoload -Uz compinit
  compinit -i
  bindkey -M viins '^[^?' backward-kill-word 2>/dev/null
  bindkey -M viins '^[^H' backward-kill-word 2>/dev/null
  bindkey -M emacs '^[^?' backward-kill-word 2>/dev/null
  bindkey -M emacs '^[^H' backward-kill-word 2>/dev/null
fi

if [[ -n "${BASH_VERSION:-}" && -f "$HOME/.local/share/agent-workspace/completions/aw.bash" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.local/share/agent-workspace/completions/aw.bash"
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

if [[ -n "${ZELLIJ:-}" && "${ZELLIJ_AGENT_TAB_WATCHER_DISABLE:-0}" != "1" && -x "$HOME/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher" ]]; then
  "$HOME/.local/share/agent-workspace/bin/.zellij-agent-tab-watcher" --start
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
    let local_bin = home_dir().join(".local/bin");
    env::split_paths(path).any(|entry| entry == local_bin)
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
