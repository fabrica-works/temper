//! Cross-check the written forge members, without reading their files.
use crate::{
    given, limits,
    problem::{Problems, What},
    setup::{Mode, Setup},
};
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::{Path, Segment};
use skein_forge_setup::limits::Limits;
use skein_json::Document;
use skein_lib::Writer;
/// How the forge's user is written.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum User {
    Id(u64),
    File(Box<[u8]>),
}
/// The forge and its joined identity, supplied by the binary.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Forge {
    pub number: u16,
    pub api: Option<SocketAddr>,
    pub plaintext: bool,
    pub git: Box<[u8]>,
    pub user: User,
    pub api_token: Box<[u8]>,
    pub webhook_secret: Option<Box<[u8]>>,
    pub identity_token: Box<[u8]>,
    pub identity_login: Box<[u8]>,
}
/// Forge members and the deployment's other listeners.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Members {
    pub forges: Box<[Forge]>,
    pub listeners: Box<[SocketAddr]>,
}
/// Setup and limits on which a command may act.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Checked {
    pub setup: Setup,
    pub limits: Limits,
}
fn require(ok: bool, path: Path, ps: &mut Problems) {
    if !ok {
        ps.push(path, What::Reason(b"invalid form"));
    }
}
pub(crate) fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for p in parts {
        len = len.checked_add(p.len()).expect("bounded text");
    }
    let mut w = Writer::new(len);
    for p in parts {
        w.put(p).expect("measured text");
    }
    w.finish()
}
/// Checks each entry against its forge, and listener and entry uniqueness.
pub fn members(setup: &Setup, members: &Members, webhooks: bool, ps: &mut Problems) {
    for (i, e) in setup.entries.iter().enumerate() {
        let p = Path::root()
            .member(b"application")
            .member(b"setup")
            .child(Segment::Index(u32::try_from(i).expect("bounded entries")));
        for prev in setup.entries.iter().take(i) {
            require(e.forge != prev.forge, p.member(b"forge"), ps);
            require(e.directory != prev.directory, p.member(b"directory"), ps);
        }
        let mut found = None;
        for f in &members.forges {
            if f.number == e.forge {
                found = Some(f);
            }
        }
        let Some(f) = found else {
            ps.push(p.member(b"forge"), What::Reason(b"unknown reference"));
            continue;
        };
        let fp = Path::root().member(b"application").member(b"forges");
        let mut fi = 0;
        for (n, candidate) in members.forges.iter().enumerate() {
            if candidate.number == f.number {
                fi = u32::try_from(n).expect("bounded forges");
            }
        }
        let fp = fp.child(Segment::Index(fi));
        if webhooks {
            require(
                f.webhook_secret == Some(joined(&[&e.directory, b"/webhook_secret"])),
                fp.member(b"webhook_secret"),
                ps,
            );
        }
        require(f.api_token == joined(&[&e.directory, b"/api_token"]), fp.member(b"api_token"), ps);
        require(f.user == User::File(joined(&[&e.directory, b"/user"])), fp.member(b"user"), ps);
        require(f.identity_token == joined(&[&e.directory, b"/git_token"]), p.member(b"forge"), ps);
        require(f.identity_login == e.bot.login, p.member(b"bot").member(b"login"), ps);
        match &e.mode {
            Mode::Managed { .. } => {}
            Mode::Owned(o) => {
                let op = p.member(b"mode").member(b"owned");
                require(
                    o.person.login != e.bot.login && o.person.login.as_ref() != b"temper-admin",
                    op.member(b"person").member(b"login"),
                    ps,
                );
                require(f.api == Some(o.listen), fp.member(b"api"), ps);
                require(f.plaintext, fp.member(b"transport"), ps);
                require(git_listen(&f.git, o.listen), fp.member(b"git"), ps);
                require(!members.listeners.contains(&o.listen), op.member(b"listen"), ps);
                for prev in setup.entries.iter().take(i) {
                    match &prev.mode {
                        Mode::Owned(other) => require(o.listen != other.listen, op.member(b"listen"), ps),
                        Mode::Managed { .. } => {}
                    }
                }
            }
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "configuration protocol parses the written HTTP socket address")]
fn git_listen(git: &[u8], listen: SocketAddr) -> bool {
    let Some(bytes) = git.strip_prefix(b"http://") else {
        return false;
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return false;
    };
    text.parse::<SocketAddr>() == Ok(listen)
}
/// Applies setup overrides, checks limits and decodes and cross-checks the list.
pub fn checked(
    document: &Document,
    at: u32,
    members_value: &Members,
    webhooks: bool,
    overrides: &[(Box<[u8]>, u64)],
) -> Result<Checked, Problems> {
    let mut ps = Problems::empty();
    let mut limits = limits::small();
    let lp = Path::root().member(b"application").member(b"limits").member(b"overrides");
    for (name, value) in overrides {
        if let Err(w) = limits::apply(&mut limits, name, *value) {
            ps.push(
                lp.member(name),
                What::Reason(match w {
                    limits::Refused::Unknown => b"no limit of temper's has this name",
                    limits::Refused::Range => b"outside its type",
                }),
            );
        }
    }
    if let Err(why) = skein_forge_setup::limits::check(&limits) {
        ps.extend(given::refusal(0, why));
    }
    let setup = Setup::read(document, at, &limits, &mut ps);
    if let Some(setup) = &setup {
        members(setup, members_value, webhooks, &mut ps);
    }
    if ps.is_empty() { Ok(Checked { setup: setup.expect("clean parse"), limits }) } else { Err(ps) }
}
