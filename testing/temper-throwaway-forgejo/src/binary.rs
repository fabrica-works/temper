use crate::{Failure, Instance, Settings};
use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
pub const VERSION: &str = "16.0.5";
#[derive(Debug, Clone)]
pub struct Forgejo {
    binary: PathBuf,
    version: String,
    sha256: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Unset,
    Unreadable { path: PathBuf, error: String },
    Version { path: PathBuf, printed: String },
}
impl Forgejo {
    /// # Errors
    /// Returns a refusal when the variable, file or version is unsuitable.
    pub fn from_env() -> Result<Self, Refusal> {
        let path = std::env::var_os("TEMPER_FORGEJO").ok_or(Refusal::Unset)?;
        Self::from_path(Path::new(&path))
    }
    /// # Errors
    /// Returns the unreadable path or its unexpected version line.
    pub fn from_path(binary: &Path) -> Result<Self, Refusal> {
        let fail = |e: std::io::Error| Refusal::Unreadable { path: binary.to_owned(), error: e.to_string() };
        let binary = fs::canonicalize(binary).map_err(fail)?;
        let bytes = fs::read(&binary).map_err(fail)?;
        let mut cmd = Command::new(&binary);
        environment(&mut cmd, binary.parent().expect("absolute binary has a parent"));
        let output = cmd.arg("--version").output().map_err(fail)?;
        let version = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or_default().to_owned();
        let expected = version == "forgejo version 16.0.5" || version.starts_with("forgejo version 16.0.5+");
        if !output.status.success() || !expected {
            return Err(Refusal::Version { path: binary, printed: version });
        }
        let mut sha256 = String::new();
        for word in skein_io::digest::digest(&bytes).0 {
            write!(sha256, "{word:016x}").expect("string write");
        }
        Ok(Self { binary, version, sha256 })
    }
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    /// # Errors
    /// Returns the provisioning failure, cleaning up anything started.
    pub fn start(&self, scratch: &Path, settings: &Settings) -> Result<Instance, Failure> {
        Instance::start(self, scratch, settings)
    }
}
pub(crate) fn environment(cmd: &mut Command, scratch: &Path) {
    cmd.env_clear()
        .env("HOME", scratch.join("home"))
        .env("PATH", "/usr/bin:/bin")
        .env("TZ", "Asia/Kolkata")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", scratch.join("gitconfig"));
}
