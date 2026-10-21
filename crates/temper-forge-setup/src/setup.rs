//! Strict setup members and bounds (protocol/setup.md, 8).
use crate::problem::Problems;
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::{self as config, Path, read};
use skein_forge_setup::{
    given::{email, login},
    limits::Limits,
};
use skein_json::{Document, document::Kind};
use skein_lib::List;
/// A Forgejo account temper names.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Account {
    pub login: Box<[u8]>,
    pub email: Box<[u8]>,
}
/// An owned forge's runner, including the PATH of its jobs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runner {
    pub binary: Box<[u8]>,
    pub labels: Box<[Box<[u8]>]>,
    pub capacity: u32,
    pub path: Box<[u8]>,
}
/// What temper runs for one forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Owned {
    pub forgejo: Box<[u8]>,
    pub runner: Option<Runner>,
    pub listen: SocketAddr,
    pub person: Account,
}
/// One of a fake forge's repositories.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Repository {
    pub name: Box<[u8]>,
    pub landing: Box<[u8]>,
}
/// The demo's fake forge, which temper runs (protocol/setup.md, 8.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fake {
    pub listen: SocketAddr,
    pub git: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub repositories: Box<[Repository]>,
}
/// A person's Forgejo or one temper runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Managed { admin_token: Box<[u8]> },
    Owned(Owned),
    Fake(Fake),
}
/// One forge's directory, bot, orgs and mode.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    pub forge: u16,
    pub directory: Box<[u8]>,
    pub bot: Account,
    pub orgs: Box<[Box<[u8]>]>,
    pub mode: Mode,
    pub asked: bool,
}
/// The setup list, with at most one entry per forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Setup {
    pub entries: Box<[Entry]>,
}
impl Setup {
    /// Reads a list at `at`, recording paths under application.setup.
    pub fn read(
        d: &Document,
        at: u32,
        limits: &Limits,
        directories: &config::Directories,
        problems: &mut Problems,
    ) -> Option<Self> {
        let mut ps = config::Problems::empty();
        let p = Path::root().member(b"application").member(b"setup");
        let mut result = list(d, at, limits, &p, &mut ps);
        if ps.is_empty()
            && let Some(setup) = &mut result
        {
            for (index, entry) in setup.entries.iter_mut().enumerate() {
                let p = p.child(config::Segment::Index(u32::try_from(index).expect("bounded entries")));
                resolve_path(
                    directories,
                    config::Directory::Data,
                    &mut entry.directory,
                    p.member(b"directory"),
                    &mut ps,
                );
                match &mut entry.mode {
                    Mode::Managed { admin_token } => resolve_path(
                        directories,
                        config::Directory::Data,
                        admin_token,
                        p.member(b"mode").member(b"managed").member(b"admin_token"),
                        &mut ps,
                    ),
                    Mode::Fake(fake) => resolve_path(
                        directories,
                        config::Directory::Data,
                        &mut fake.git,
                        p.member(b"mode").member(b"fake").member(b"git"),
                        &mut ps,
                    ),
                    Mode::Owned(owned) => {
                        let p = p.member(b"mode").member(b"owned");
                        resolve_path(
                            directories,
                            config::Directory::Cache,
                            &mut owned.forgejo,
                            p.member(b"forgejo"),
                            &mut ps,
                        );
                        if let Some(runner) = &mut owned.runner {
                            resolve_path(
                                directories,
                                config::Directory::Cache,
                                &mut runner.binary,
                                p.member(b"runner").member(b"binary"),
                                &mut ps,
                            );
                        }
                    }
                }
            }
        }
        let clean = ps.is_empty();
        problems.append(ps);
        if clean { result } else { None }
    }
    /// Finds the entry for a forge number.
    #[must_use]
    pub fn entry(&self, number: u16) -> Option<&Entry> {
        let mut found = None;
        for e in &self.entries {
            if e.forge == number {
                found = Some(e);
            }
        }
        found
    }
}
fn object(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<read::Object> {
    match read::Object::new(d, at) {
        Ok(o) => Some(o),
        Err(w) => {
            ps.push(p.clone(), w);
            None
        }
    }
}
fn text(d: &Document, o: &mut read::Object, name: &[u8], p: &Path, ps: &mut config::Problems) -> Option<Box<[u8]>> {
    bounded_text(d, o, name, 4096, p, ps)
}
fn bounded_text(
    d: &Document,
    o: &mut read::Object,
    name: &[u8],
    max: u32,
    p: &Path,
    ps: &mut config::Problems,
) -> Option<Box<[u8]>> {
    let at = o.required(d, name, p, ps)?;
    match read::text(d, at, max) {
        Ok(v) => Some(v),
        Err(w) => {
            ps.push(p.member(name), w);
            None
        }
    }
}
fn valid(ok: bool, p: Path, ps: &mut config::Problems) {
    if !ok {
        ps.push(p, config::What::Form);
    }
}
fn path(b: &[u8]) -> bool {
    !b.is_empty() && !b.contains(&0)
}
fn account(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Account> {
    let mut o = object(d, at, p, ps)?;
    let login_value = text(d, &mut o, b"login", p, ps);
    let email_value = text(d, &mut o, b"email", p, ps);
    o.finish(d, p, ps);
    let login_value = login_value?;
    let email_value = email_value?;
    valid(login(&login_value), p.member(b"login"), ps);
    valid(email(&email_value), p.member(b"email"), ps);
    Some(Account { login: login_value, email: email_value })
}
fn strings(
    d: &Document,
    at: u32,
    max: u32,
    names: bool,
    p: &Path,
    ps: &mut config::Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let items = match read::list(d, at, max) {
        Ok(v) => v,
        Err(w) => {
            ps.push(p.clone(), w);
            return None;
        }
    };
    valid(!items.is_empty(), p.clone(), ps);
    let mut values = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let here = p.child(config::Segment::Index(u32::try_from(i).expect("bounded list")));
        match read::text(d, *at, if names { 40 } else { 128 }) {
            Ok(v) => {
                valid(if names { login(&v) } else { !v.is_empty() && !v.contains(&0) }, here, ps);
                values.push(v).expect("list capacity");
            }
            Err(w) => ps.push(here, w),
        }
    }
    Some(values.into_boxed())
}
fn runner(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Runner> {
    let mut o = object(d, at, p, ps)?;
    let binary = text(d, &mut o, b"binary", p, ps);
    let labels_at = o.required(d, b"labels", p, ps);
    let cap_at = o.required(d, b"capacity", p, ps);
    let path_value = text(d, &mut o, b"path", p, ps);
    o.finish(d, p, ps);
    let capacity = match read::integer(d, cap_at?, 64) {
        Ok(n) => n,
        Err(w) => {
            ps.push(p.member(b"capacity"), w);
            return None;
        }
    };
    valid(capacity > 0, p.member(b"capacity"), ps);
    let binary = binary?;
    let path_value = path_value?;
    valid(path(&binary), p.member(b"binary"), ps);
    valid(path(&path_value), p.member(b"path"), ps);
    Some(Runner {
        binary,
        labels: strings(d, labels_at?, 8, false, &p.member(b"labels"), ps)?,
        capacity: u32::try_from(capacity).expect("capacity bounded"),
        path: path_value,
    })
}
fn owned(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Owned> {
    let mut o = object(d, at, p, ps)?;
    let forgejo = text(d, &mut o, b"forgejo", p, ps);
    let runner_at = o.required(d, b"runner", p, ps);
    let listen_at = o.required(d, b"listen", p, ps);
    let person_at = o.required(d, b"person", p, ps);
    o.finish(d, p, ps);
    let runner_at = runner_at?;
    let runner = if d.token(runner_at)?.kind == Kind::Null {
        None
    } else {
        Some(runner(d, runner_at, &p.member(b"runner"), ps)?)
    };
    let listen = listen(d, listen_at?, &p.member(b"listen"), ps)?;
    let forgejo = forgejo?;
    valid(path(&forgejo), p.member(b"forgejo"), ps);
    Some(Owned { forgejo, runner, listen, person: account(d, person_at?, &p.member(b"person"), ps)? })
}
fn listen(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<SocketAddr> {
    let address = config::address::read(d, at, p, ps)?;
    let listen = match address {
        config::Address::Addr(a) => a.socket(),
        config::Address::Host { .. } => None,
    };
    let listen = match listen {
        Some(a) if a.ip().is_loopback() => a,
        Some(_) | None => {
            ps.push(p.clone(), config::What::Form);
            return None;
        }
    };
    Some(listen)
}
fn fake(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Fake> {
    let mut o = object(d, at, p, ps)?;
    let listen_at = o.required(d, b"listen", p, ps);
    let git = text(d, &mut o, b"git", p, ps);
    let owner = text(d, &mut o, b"owner", p, ps);
    let repositories_at = o.required(d, b"repositories", p, ps);
    o.finish(d, p, ps);
    let listen = listen(d, listen_at?, &p.member(b"listen"), ps)?;
    let git = git?;
    let owner = owner?;
    valid(path(&git), p.member(b"git"), ps);
    valid(login(&owner), p.member(b"owner"), ps);
    let rp = p.member(b"repositories");
    let items = match read::list(d, repositories_at?, 8) {
        Ok(items) => items,
        Err(what) => {
            ps.push(rp, what);
            return None;
        }
    };
    valid(!items.is_empty(), rp.clone(), ps);
    let mut repositories = List::<Repository>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let p = rp.child(config::Segment::Index(u32::try_from(index).expect("eight repositories")));
        let mut o = object(d, *at, &p, ps)?;
        let name = bounded_text(d, &mut o, b"name", 100, &p, ps);
        let landing = bounded_text(d, &mut o, b"landing", 256, &p, ps);
        o.finish(d, &p, ps);
        let name = name?;
        let landing = landing?;
        valid(crate::name::repository(&name), p.member(b"name"), ps);
        valid(crate::name::branch(&landing), p.member(b"landing"), ps);
        for prev in &repositories {
            if prev.name == name {
                ps.push(p.member(b"name"), config::What::Form);
                break;
            }
        }
        repositories.push(Repository { name, landing }).expect("eight repositories");
    }
    Some(Fake { listen, git, owner, repositories: repositories.into_boxed() })
}
fn mode(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Mode> {
    let mut o = object(d, at, p, ps)?;
    let managed = o.member(d, b"managed");
    let owned_at = o.member(d, b"owned");
    let fake_at = o.member(d, b"fake");
    o.finish(d, p, ps);
    let count = u8::from(managed.is_some())
        .saturating_add(u8::from(owned_at.is_some()))
        .saturating_add(u8::from(fake_at.is_some()));
    if count != 1 {
        ps.push(p.clone(), config::What::Form);
        return None;
    }
    if let Some(at) = managed {
        let p = p.member(b"managed");
        let mut o = object(d, at, &p, ps)?;
        let admin_token = text(d, &mut o, b"admin_token", &p, ps);
        o.finish(d, &p, ps);
        let admin_token = admin_token?;
        valid(path(&admin_token), p.member(b"admin_token"), ps);
        Some(Mode::Managed { admin_token })
    } else if let Some(at) = owned_at {
        Some(Mode::Owned(owned(d, at, &p.member(b"owned"), ps)?))
    } else {
        Some(Mode::Fake(fake(d, fake_at?, &p.member(b"fake"), ps)?))
    }
}
fn entry(d: &Document, at: u32, limits: &Limits, p: &Path, ps: &mut config::Problems) -> Option<Entry> {
    let mut o = object(d, at, p, ps)?;
    let forge_at = o.required(d, b"forge", p, ps);
    let directory = text(d, &mut o, b"directory", p, ps);
    let bot_at = o.required(d, b"bot", p, ps);
    let mode_at = o.required(d, b"mode", p, ps);
    let start_at = o.member(d, b"start");
    let mode = match mode_at {
        Some(at) => mode(d, at, &p.member(b"mode"), ps),
        None => None,
    };
    let orgs: Option<Box<[Box<[u8]>]>> = match &mode {
        Some(Mode::Fake(_)) => {
            valid(o.member(d, b"orgs").is_none(), p.member(b"orgs"), ps);
            Some(Box::new([]))
        }
        Some(Mode::Managed { .. } | Mode::Owned(_)) => {
            let at = o.required(d, b"orgs", p, ps);
            match at {
                Some(at) => strings(d, at, limits.orgs, true, &p.member(b"orgs"), ps),
                None => None,
            }
        }
        None => match o.member(d, b"orgs") {
            Some(at) => strings(d, at, limits.orgs, true, &p.member(b"orgs"), ps),
            None => None,
        },
    };
    o.finish(d, p, ps);
    let mode = mode?;
    let asked = if let Some(at) = start_at {
        let value = match read::text(d, at, 4096) {
            Ok(value) => value,
            Err(what) => {
                ps.push(p.member(b"start"), what);
                return None;
            }
        };
        valid(
            value.as_ref() == b"asked"
                && match mode {
                    Mode::Owned(_) => true,
                    Mode::Managed { .. } | Mode::Fake(_) => false,
                },
            p.member(b"start"),
            ps,
        );
        true
    } else {
        false
    };
    let forge = match read::integer(d, forge_at?, u64::from(u16::MAX)) {
        Ok(n) => u16::try_from(n).expect("forge bounded"),
        Err(w) => {
            ps.push(p.member(b"forge"), w);
            return None;
        }
    };
    let directory = directory?;
    valid(path(&directory) && !directory.ends_with(b"/"), p.member(b"directory"), ps);
    Some(Entry { forge, directory, bot: account(d, bot_at?, &p.member(b"bot"), ps)?, orgs: orgs?, mode, asked })
}
fn list(d: &Document, at: u32, limits: &Limits, p: &Path, ps: &mut config::Problems) -> Option<Setup> {
    let items = match read::list(d, at, 65_536) {
        Ok(v) => v,
        Err(w) => {
            ps.push(p.clone(), w);
            return None;
        }
    };
    let mut entries = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        if let Some(e) =
            entry(d, *at, limits, &p.child(config::Segment::Index(u32::try_from(i).expect("bounded list"))), ps)
        {
            entries.push(e).expect("entry capacity");
        }
    }
    Some(Setup { entries: entries.into_boxed() })
}

fn resolve_path(
    directories: &config::Directories,
    kind: config::Directory,
    value: &mut Box<[u8]>,
    path: Path,
    problems: &mut config::Problems,
) {
    match directories.join(kind, value) {
        Ok(resolved) => *value = resolved,
        Err(what) => problems.push(path, what),
    }
}
