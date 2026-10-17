//! The one exception for the lock and exec (setup.md, 6.4).
use std::{
    fs, io,
    os::unix::{fs::OpenOptionsExt, process::CommandExt},
    path::Path,
    process::{Command, Stdio},
};
use temper_forge_setup::spec::Spec;
/// Held until dropped or handed across exec as standard input.
#[derive(Debug)]
pub struct Lock {
    file: fs::File,
}
/// A lock is held, or its file could not be opened.
#[derive(Debug)]
pub enum Refused {
    Held,
    Io(io::ErrorKind),
}
/// Takes an exclusive lock without waiting, making its file private.
pub fn lock(path: &Path) -> Result<Lock, Refused> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)
        .map_err(|why| Refused::Io(why.kind()))?;
    file.try_lock().map_err(|why| match why {
        fs::TryLockError::WouldBlock => Refused::Held,
        fs::TryLockError::Error(why) => Refused::Io(why.kind()),
    })?;
    Ok(Lock { file })
}
/// Builds the exec's isolated command; its lock is standard input.
#[must_use]
pub fn command(spec: &Spec, lock: Lock) -> Command {
    let mut command = Command::new(crate::path(&spec.program));
    command
        .args(spec.arguments.iter().map(|arg| crate::path(arg)))
        .env_clear()
        .current_dir(crate::path(&spec.directory))
        .stdin(Stdio::from(lock.file));
    for entry in &spec.environment {
        let split = entry.iter().position(|c| *c == b'=').expect("environment assignment");
        command.env(crate::path(&entry[..split]), crate::path(&entry[split + 1..]));
    }
    command
}
/// Replaces this process; only an unsuccessful exec returns.
#[must_use]
pub fn exec(spec: &Spec, lock: Lock) -> io::Error {
    command(spec, lock).exec()
}
