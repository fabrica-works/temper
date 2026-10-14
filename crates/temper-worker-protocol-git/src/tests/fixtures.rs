use crate::{Identity, Limits};
use alloc::boxed::Box;
use skein_lib::{Duration, Secret};
use temper_worker_domain_checkout::{self as checkout, git};
pub const HOME: &[u8] = b"/srv/worker/home";
pub const REMOTE: &[u8] = b"https://forge.example.test/acme/app.git";
pub const COMMIT: git::Commit = git::Commit::new([
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const COMMIT_256: git::Commit = git::Commit::new([0xcd; 32]);
pub fn checkout() -> checkout::Limits {
    checkout::Limits {
        workspaces: 2,
        repositories: 2,
        name_bytes: 64,
        message_bytes: 256,
        conflicts: 4,
        path_bytes: 64,
        remote_timeout: Duration::from_secs(30),
        local_timeout: Duration::from_secs(5),
        facts: 8,
    }
}
pub fn limits() -> Limits {
    Limits { operations: 2, stdout_bytes: 512, stderr_bytes: 512, read_bytes: 64, sweep_entries: 4, sweep_bytes: 255 }
}
pub fn identity() -> Identity {
    Identity {
        token: Secret::new(Box::from(&b"tok-1"[..])),
        login: Box::from(&b"temper-bot"[..]),
        name: Box::from(&b"Temper Bot"[..]),
        email: Box::from(&b"bot@example.test"[..]),
    }
}
