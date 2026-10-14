//! Shared spawn fixtures, also used by the hosted world's program tests.
use skein_io::kernel::{Fd, Pipe, Spawn, Way};
use skein_lib::Secret;
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    Identity,
    invocation::{self, Command},
};
/// Only the header may hold this token's bytes.
pub const TOKEN: &[u8] = b"fake-git-identity-token";
/// The scenario's git identity.
#[must_use]
pub fn identity() -> Identity {
    Identity {
        token: Secret::new(TOKEN.into()),
        login: b"worker".as_slice().into(),
        name: b"Worker".as_slice().into(),
        email: b"worker@example.invalid".as_slice().into(),
    }
}
/// The spawn the component makes for the command's operation.
#[must_use]
pub fn spawn_of(command: &Command) -> Spawn {
    let kind = match command {
        Command::Clone { .. } => Some(git::Kind::Clone),
        Command::Fetch { .. } => Some(git::Kind::Fetch),
        Command::Create { .. } => Some(git::Kind::Create),
        Command::Push { .. } | Command::IsAncestor { .. } => Some(git::Kind::Push),
        Command::Unmerged
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::WriteTree
        | Command::CommitTree { .. } => Some(git::Kind::Commit),
        Command::CheckOut { .. } | Command::Clean => Some(git::Kind::CheckOut),
        Command::Merge { .. } => Some(git::Kind::Merge),
        Command::Version => None,
    };
    let mut pipes =
        vec![Pipe { child: 1, way: Way::Out, parent: None }, Pipe { child: 2, way: Way::Out, parent: None }];
    if matches!(command, Command::CommitTree { .. }) {
        pipes.insert(0, Pipe { child: 0, way: Way::In, parent: None });
    }
    Spawn {
        program: b"/usr/bin/git".as_slice().into(),
        args: invocation::arguments(command),
        env: invocation::environment(b"/worker/home", kind, Some(&identity())),
        root: Fd::new(1),
        dir: b"workspaces/1/app".as_slice().into(),
        pipes: pipes.into_boxed_slice(),
    }
}
