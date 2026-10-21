//! Connection sequencing and serialized installation rewrites (protocol/forge.md, 2.2–2.6).
use super::{Write, answer, ended, failure_text};
use alloc::boxed::Box;
use jig_protocol_store::{SecretOwner, Secrets};
use skein_forge_connection as connection;
use skein_forge_setup::{host, reconcile};
use skein_lib::List;
use skein_lib::{Map, Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;
use temper_protocol_records::{self as records, Item};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Kind {
    Manifest,
    Convert,
    Install,
    Accept,
    Reconcile,
    Follow,
}
#[derive(Debug)]
pub(super) struct Run {
    pub(super) previous: domain::Connection,
    pub(super) forge: u16,
    pub(super) kind: Kind,
    pub(super) internal: bool,
}
pub(super) struct Lifecycle {
    reconcile: Map<u16, ()>,
    deleted: Map<(u16, u64), ()>,
    deleting: Map<u16, ()>,
    pub(super) installations: Map<(u16, u64), domain::Installation>,
    next_run: u64,
}
impl Lifecycle {
    pub(super) fn ready(&self, runs: &Map<Token, Run>, limit: u32, domain: &root::Domain) -> bool {
        for (forge, ()) in &self.reconcile {
            if runs.len() < limit
                && connected_in_root(domain, *forge)
                && !self.deleting.contains_key(forge)
                && !running(runs, *forge)
            {
                return true;
            }
        }
        for ((forge, _), ()) in &self.deleted {
            if !self.deleting.contains_key(forge) && !running(runs, *forge) {
                return true;
            }
        }
        false
    }
    pub(super) fn new(forges: u32, installations: u32) -> Self {
        let count = forges.checked_mul(installations).expect("installation tables");
        Self {
            reconcile: Map::with_capacity(forges),
            deleted: Map::with_capacity(count),
            deleting: Map::with_capacity(forges),
            installations: Map::with_capacity(count),
            next_run: 1,
        }
    }
}
fn connected_in_root(root: &root::Domain, forge: u16) -> bool {
    match root.forge_connection(forge) {
        domain::Connection::Connected { .. } => true,
        domain::Connection::NotConnected | domain::Connection::Connecting | domain::Connection::Stopped { .. } => false,
    }
}
pub(crate) fn stopped_connection(engine: &mut crate::engine::Engine, forge: u16) {
    engine.setup.lifecycle.reconcile.remove(&forge);
}
pub(super) fn kind(plan: &host::Plan) -> Kind {
    match plan {
        host::Plan::Manifest { .. } => Kind::Manifest,
        host::Plan::Convert { .. } => Kind::Convert,
        host::Plan::Install { .. } => Kind::Install,
        host::Plan::Accept { .. } => Kind::Accept,
        host::Plan::Reconcile { .. } => Kind::Reconcile,
        host::Plan::Paste { .. } => unreachable!("temper-13 stages Paste"),
        host::Plan::Check { .. } | host::Plan::TakeOver { .. } | host::Plan::Adopt { .. } => {
            unreachable!("temper-14 stages Forgejo plans")
        }
        host::Plan::Reachable { .. } => unreachable!("temper-10 stages Reachable"),
    }
}
pub(super) fn busy(engine: &crate::engine::Engine, forge: u16) -> bool {
    if engine.setup.lifecycle.deleting.contains_key(&forge) {
        return true;
    }
    running(&engine.setup.runs, forge)
}
fn running(runs: &Map<Token, Run>, forge: u16) -> bool {
    for (_, run) in runs {
        if run.forge == forge {
            return true;
        }
    }
    false
}

pub(super) fn has_app(engine: &crate::engine::Engine, forge: u16) -> bool {
    match engine.forge_records.rows.forge(forge) {
        Some(rows) => rows.has(Item::AppId) || rows.has(Item::AppSlug) || rows.has(Item::AppKey),
        None => false,
    }
}
pub(crate) fn state(engine: &mut crate::engine::Engine, forge: u16, state: domain::Connection) {
    engine.events.push(root::Event::Forge(Box::new(domain::Event::Connection { forge, state })));
}
pub(super) fn keep(engine: &mut crate::engine::Engine, forge: u16, item: Item, bytes: &[u8], purpose: Purpose) {
    let value = Secret::new(records::encode(bytes).into_boxed_slice());
    let bound = usize::try_from(engine.setup.value_bytes).expect("record bound");
    assert!(value.expose().len() <= bound, "startup::checks::records proves every encoded setup write fits");
    assert!(engine.setup.next_write < (1_u64 << 60_u32), "setup write names fit prefix");
    let owner = Token::new(crate::tokens::SETUP | engine.setup.next_write);
    engine.setup.next_write = engine.setup.next_write.checked_add(1).expect("setup write names");
    let number = records::number(forge, item);
    engine
        .setup
        .writes
        .insert(owner, Write { purpose, number, value: value.clone() })
        .ok()
        .expect("bounded setup writes");
    if let Some(terminal) =
        engine.store.secrets(owner, Secrets::Keep { owner: SecretOwner::Application, number, value })
    {
        answer(engine, terminal);
    }
}
pub(super) enum Purpose {
    Host(Token),
    Origin(Token),
    Deleted(u16),
}
pub(super) fn done(engine: &mut crate::engine::Engine, token: Token, outcome: reconcile::Outcome) {
    let run = engine.setup.runs.get(&token).expect("ended setup run");
    let (forge, kind) = (run.forge, run.kind);
    if kind == Kind::Convert {
        let config = configured(engine, forge);
        let origin = records::origin(connection::settings::Kind::GitHub, &config.api).encode();
        keep(engine, forge, Item::Origin, &origin, Purpose::Origin(token));
        return;
    }
    if kind == Kind::Accept || kind == Kind::Reconcile {
        installations(engine, forge);
    }
    let outcome = match outcome {
        reconcile::Outcome::Manifest { state, page } => {
            engine.setup.hold(forge, state.clone(), page);
            root::SetupOutcome::Manifest { state }
        }
        reconcile::Outcome::Install { state, url } => root::SetupOutcome::Install { state, url },
        reconcile::Outcome::Lacking { lacking } => {
            if !lacking.is_empty() {
                engine
                    .notes
                    .push(crate::Note::ForgeLacking { number: forge, names: super::diagnostics::names(&lacking) });
            }
            root::SetupOutcome::Done
        }
        reconcile::Outcome::Nothing => root::SetupOutcome::Done,
        reconcile::Outcome::Identity { .. } | reconcile::Outcome::Reachable { .. } => {
            unreachable!("later plans are not staged")
        }
    };
    finish(engine, token, root::SetupEnd::Done { forge, outcome });
}
pub(super) fn stopped(engine: &mut crate::engine::Engine, token: Token, failure: reconcile::Failure) {
    let run = engine.setup.runs.get(&token).expect("stopped setup run");
    let mut reason = failure_text(&failure);
    if run.kind == Kind::Convert && failure.why == reconcile::Why::Refused(404) {
        reason = Box::from(
            b"setup conversion refused (404): the App may exist on GitHub without temper holding its key".as_slice(),
        );
    }
    finish(engine, token, root::SetupEnd::Stopped { forge: failure.forge, reason });
}
fn finish(engine: &mut crate::engine::Engine, token: Token, end: root::SetupEnd) {
    let run = engine.setup.runs.remove(&token).expect("terminal setup run");
    let follow = run.kind == Kind::Follow;
    if run.kind == Kind::Convert {
        state(engine, run.forge, run.previous);
    }
    if run.internal {
        match end {
            root::SetupEnd::Stopped { forge, reason } => {
                ended(engine, token, root::SetupEnd::Stopped { forge, reason });
            }
            root::SetupEnd::Done { .. } => {}
        }
    } else {
        ended(engine, token, end);
    }
    if follow {
        connect(engine, run.forge);
    }
}
pub(super) fn kept(engine: &mut crate::engine::Engine, purpose: Purpose) {
    match purpose {
        Purpose::Host(run) => engine.setup.down(host::Request::Answer { run, answer: host::Answer::Written }),
        Purpose::Origin(run) => {
            let active = engine.setup.runs.get_mut(&run).expect("conversion waits on Origin");
            active.kind = Kind::Follow;
            let forge = active.forge;
            engine.setup.down(host::Request::Run { run, plan: host::Plan::Install { forge } });
        }
        Purpose::Deleted(forge) => {
            engine.setup.lifecycle.deleting.remove(&forge);
            installations(engine, forge);
        }
    }
}
fn configured(engine: &crate::engine::Engine, number: u16) -> &crate::settings::Forge {
    for forge in &engine.workers.forges {
        if forge.number == number {
            return forge;
        }
    }
    unreachable!("configured setup forge")
}
fn connect(engine: &mut crate::engine::Engine, forge: u16) {
    let expected = records::origin(connection::settings::Kind::GitHub, &configured(engine, forge).api);
    let access = match engine.forge_records.rows.forge(forge) {
        Some(rows) => {
            records::access(connection::settings::Kind::GitHub, &expected, rows, &engine.forge_records.bounds, false)
        }
        None => Err(records::NotConnected::NoRecords),
    };
    match access {
        Ok(start) => {
            engine.forge.down(connection::Request::Connect { forge, access: start.access });
            engine.forge.down(connection::Request::Installations { forge, accepted: start.accepted });
        }
        Err(why) => engine.notes.push(crate::Note::ForgeNotConnected { number: forge, why }),
    }
}
pub(super) fn accepted(engine: &crate::engine::Engine, forge: u16) -> Box<[u64]> {
    if let Some(rows) = engine.forge_records.rows.forge(forge)
        && let Ok(bytes) = rows.get(Item::Accepted)
    {
        return skein_forge_setup::item::accepted(bytes, engine.forge_records.bounds.installations)
            .expect("kept accepted set");
    }
    Box::new([])
}
fn installations(engine: &mut crate::engine::Engine, forge: u16) {
    let accepted = accepted(engine, forge);
    for id in &accepted {
        if let Some(info) = engine.setup.lifecycle.installations.get_mut(&(forge, *id))
            && info.stray
        {
            info.stray = false;
            let info = info.clone();
            installation_event(engine, forge, *id, Some(info));
        }
    }
    engine.forge.down(connection::Request::Installations { forge, accepted });
}
pub(crate) fn connected(engine: &mut crate::engine::Engine, forge: u16) {
    let config = configured(engine, forge);
    let github = config.kind == crate::settings::Kind::GitHub;
    let (id, login) = if github {
        let rows = engine.forge_records.rows.forge(forge).expect("Connected has records");
        let id =
            skein_forge_setup::item::user(rows.get(Item::User).expect("connected User")).expect("valid User").get();
        let slug = rows.get(Item::AppSlug).expect("connected slug");
        let mut login =
            skein_lib::Writer::new(slug.len().checked_add(domain::BOT_LOGIN_SUFFIX.len()).expect("bot login"));
        login.put(slug).expect("slug room");
        login.put(domain::BOT_LOGIN_SUFFIX).expect("suffix room");
        (id, login.finish())
    } else {
        // Forgejo still starts from private files (temper-15); its login is the git identity's.
        let mut login = Box::from([]);
        if let Some(value) = engine.values.identities.get(&config.identity) {
            let mut reader = skein_lib::Reader::new(value.expose());
            let identity = temper_protocol_worker_wire::v1::Identity::decode(
                &temper_protocol_worker_wire::v1::CEILINGS,
                &mut reader,
            )
            .expect("configured identity");
            login = Box::from(identity.login());
        }
        (config.user, login)
    };
    state(engine, forge, domain::Connection::Connected { id, login });
    if github {
        engine.setup.lifecycle.reconcile.insert(forge, ()).expect("one reconcile per forge");
    }
}
fn installation_event(
    engine: &mut crate::engine::Engine,
    forge: u16,
    installation: u64,
    state: Option<domain::Installation>,
) {
    engine.events.push(root::Event::Forge(Box::new(domain::Event::Installation { forge, installation, state })));
}
pub(crate) fn installation(
    engine: &mut crate::engine::Engine,
    forge: u16,
    installation: u64,
    account: Option<connection::Account>,
    change: connection::Change,
) {
    let key = (forge, installation);
    if change == connection::Change::Deleted {
        let known = engine.setup.lifecycle.installations.remove(&key).is_some();
        let accepted = accepted(engine, forge).contains(&installation);
        if known || accepted && !engine.setup.lifecycle.deleted.contains_key(&key) {
            installation_event(engine, forge, installation, None);
        }
        if accepted {
            engine.setup.lifecycle.deleted.insert(key, ()).expect("bounded accepted deletions");
        }
        return;
    }
    let previous = engine.setup.lifecycle.installations.get(&key);
    let suspended = match change {
        connection::Change::Suspended => true,
        connection::Change::Unsuspended => false,
        // A changed repository selection leaves the suspension as it was.
        connection::Change::Created | connection::Change::Renamed | connection::Change::Repositories => {
            match previous {
                Some(info) => info.suspended,
                None => false,
            }
        }
        connection::Change::Deleted => unreachable!("deleted above"),
    };
    let info = domain::Installation {
        account: match account {
            Some(account) => Some((account.id, account.login)),
            None => None,
        },
        stray: !accepted(engine, forge).contains(&installation),
        suspended,
    };
    if previous != Some(&info) {
        engine.setup.lifecycle.installations.insert(key, info.clone()).expect("component's installation bound");
        installation_event(engine, forge, installation, Some(info));
    }
}
pub(super) fn advance(engine: &mut crate::engine::Engine) {
    if engine.setup.closing || engine.drain.is_some() || engine.end.is_some() || engine.setup.component.is_none() {
        return;
    }
    let mut forges = List::with_capacity(engine.setup.env.limits.forges);
    for forge in &engine.workers.forges {
        forges.push(forge.number).expect("configured forges");
    }
    for forge in &forges {
        if !engine.setup.room()
            || !engine.store.room()
            || !engine.forge.room(2)
            || engine.events.room() < 4
            || busy(engine, *forge)
        {
            continue;
        }
        let mut deleted = List::with_capacity(engine.forge_records.bounds.installations);
        for ((number, id), ()) in &engine.setup.lifecycle.deleted {
            if number == forge {
                deleted.push(*id).expect("accepted deletion bound");
            }
        }
        if !deleted.is_empty() {
            let mut remaining = List::with_capacity(engine.forge_records.bounds.installations);
            for id in &accepted(engine, *forge) {
                if !deleted.as_slice().contains(id) {
                    remaining.push(*id).expect("accepted bound");
                }
            }
            for id in &deleted {
                engine.setup.lifecycle.deleted.remove(&(*forge, *id));
            }
            if remaining.as_slice() == accepted(engine, *forge).as_ref() {
                continue;
            }
            engine.setup.lifecycle.deleting.insert(*forge, ()).expect("one rewrite per forge");
            let bytes = skein_forge_setup::item::accepted_item(remaining.as_slice());
            keep(engine, *forge, Item::Accepted, bytes.expose(), Purpose::Deleted(*forge));
        } else if engine.setup.lifecycle.reconcile.contains_key(forge)
            && connected_in_root(&engine.domain, *forge)
            && engine.setup.runs.len() < engine.setup.env.limits.runs
        {
            engine.setup.lifecycle.reconcile.remove(forge);
            assert!(engine.setup.lifecycle.next_run < (1_u64 << 60_u32), "internal setup names");
            let run = Token::new(crate::tokens::SETUP | engine.setup.lifecycle.next_run);
            engine.setup.lifecycle.next_run = engine.setup.lifecycle.next_run.checked_add(1).expect("setup run names");
            if !engine.setup.served.contains_key(forge) {
                let reason = *engine.setup.unserved.get(forge).expect("configured setup exclusion");
                ended(engine, run, root::SetupEnd::Stopped { forge: *forge, reason: Box::from(reason) });
                continue;
            }
            engine
                .setup
                .runs
                .insert(
                    run,
                    Run {
                        forge: *forge,
                        kind: Kind::Reconcile,
                        internal: true,
                        previous: domain::Connection::NotConnected,
                    },
                )
                .expect("admitted reconcile");
            engine
                .setup
                .down(host::Request::Run { run, plan: host::Plan::Reconcile { forge: *forge, adopted: Box::new([]) } });
        }
    }
}

#[expect(clippy::disallowed_macros, reason = "Stopped has no credential; diagnostic boundary")]
pub(crate) fn stopped_text(why: connection::Stopped) -> Box<[u8]> {
    alloc::format!("{why:?}").into_bytes().into_boxed_slice()
}
