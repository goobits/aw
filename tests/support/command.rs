use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::temp::TempDir;

pub fn aw() -> PathBuf {
    let cargo_binary = PathBuf::from(env!("CARGO_BIN_EXE_aw"));
    if cargo_binary.is_file() {
        return cargo_binary;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/aw")
}

pub struct TestHome {
    pub root: TempDir,
    pub home: std::path::PathBuf,
    pub bin: std::path::PathBuf,
}

impl TestHome {
    pub fn new(name: &str) -> Self {
        let root = TempDir::new(name);
        let home = root.join("home");
        let bin = root.join("bin");
        std::fs::create_dir_all(&home).expect("create home");
        std::fs::create_dir_all(&bin).expect("create bin");
        Self { root, home, bin }
    }

    pub fn install_aw(&self) {
        let output = self
            .command(aw())
            .env("ZELLIJ_INSTALL_BINARY", "0")
            .env("ZELLIJ_INSTALL_SHELL_RC", "0")
            .arg("install")
            .output()
            .expect("run aw install");
        assert_success("aw install", &output);
    }

    pub fn installed_aw(&self) -> PathBuf {
        self.home.join(".local/bin/aw")
    }

    pub fn aw_command(&self) -> Command {
        self.command(self.installed_aw())
    }

    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        command.env("HOME", &self.home);
        command.env("PATH", path_with(&self.bin));
        command.env_remove("AW_CONFIG_DIR");
        command.env_remove("ZELLIJ");
        command.env_remove("ZELLIJ_SESSION_NAME");
        command
    }
}

pub fn path_with(bin: impl AsRef<Path>) -> String {
    format!(
        "{}:{}",
        bin.as_ref().display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

pub fn expected_session(profile: &str, workspace: &str, root: impl std::fmt::Display) -> String {
    let root = root.to_string();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in root.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{profile}-{workspace}-{hash:016x}")
}

pub fn assert_order(path: impl AsRef<Path>, session: &str, tabs: &[&str]) {
    let expected = std::iter::once(session)
        .chain(tabs.iter().copied())
        .collect::<Vec<_>>()
        .join("\n");
    let actual = std::fs::read_to_string(path).expect("read tab order");
    assert_eq!(actual.trim_end(), expected);
}

pub fn assert_success(label: &str, output: &Output) {
    if !output.status.success() {
        panic!(
            "{} failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            label,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn assert_failure(label: &str, output: &Output) {
    if output.status.success() {
        panic!(
            "{} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
            label,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .trim_end_matches('\n')
        .to_string()
}
