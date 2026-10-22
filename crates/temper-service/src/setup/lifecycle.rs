//! Connection sequencing and serialized installation rewrites (protocol/forge.md, 2.2–2.6).
use super::{Write, answer, ended, failure_text};
use alloc::boxed::Box;
use core::num::NonZeroU64;
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
    LocalReconcile,
    LocalConnect,
    Adopt,
    Follow,
}
#[derive(Debug)]
pub(super) struct Run {
    pub(super) previous: domain::Connection,
    pub(super) forge: u16,
    pub(super) kind: Kind,
    pub(super) internal: bool,
}
#[derive(Debug)]
pub(super) struct Adoption {
    pub(super) next: NonZeroU64,
    pub(super) repositories: u64,
    pub(super) admin_token: Secret,
    pub(super) access: connection::Access,
}
#[derive(Debug)]
pub(super) struct Local {
    pub(super) pending: bool,
    pub(super) restored: bool,
    pub(super) limits: skein_forge_setup::limits::Limits,
    pub(super) webhooks: bool,
    pub(super) bot_login: Box<[u8]>,
    pub(super) identity: crate::settings::Identity,
}
#[derive(Debug)]
pub(super) enum Reconcile {
    GitHub,
    Local(Local),
}
pub(super) struct Lifecycle {
    pub(super) adoptions: Map<u16, Adoption>,
    pub(super) reconcile: Map<u16, Reconcile>,
    deleted: Map<(u16, u64), ()>,
    deleting: Map<u16, ()>,
    pub(super) installations: Map<(u16, u64), domain::Installation>,
    next_run: u64,
}
impl Lifecycle {
    pub(super) fn local(&self, forge: u16) -> Option<&Local> {
        match self.reconcile.get(&forge) {
            Some(Reconcile::Local(local)) => Some(local),
            Some(Reconcile::GitHub) | None => None,
        }
    }
    pub(super) fn local_mut(&mut self, forge: u16) -> Option<&mut Local> {
        match self.reconcile.get_mut(&forge) {
            Some(Reconcile::Local(local)) => Some(local),
            Some(Reconcile::GitHub) | None => None,
        }
    }
    pub(super) fn ready(&self, runs: &Map<Token, Run>, limit: u32, domain: &root::Domain) -> bool {
        for (_, pending) in &self.reconcile {
            if let Reconcile::Local(local) = pending
                && local.pending
                && local.restored
            {
                return true;
            }
        }
        for (forge, _) in &self.adoptions {
            if runs.len() < limit && !running(runs, *forge) {
                return true;
            }
        }
        for (forge, pending) in &self.reconcile {
            match pending {
                Reconcile::Local(_) => continue,
                Reconcile::GitHub => {}
            }
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
            adoptions: Map::with_capacity(forges),
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
        domain::Connection::NotConnected
        | domain::Connection::NotSetUp
        | domain::Connection::SettingUp { .. }
        | domain::Connection::SetUpFailed { .. }
        | domain::Connection::Connecting
        | domain::Connection::Stopped { .. } => false,
    }
}
pub(crate) fn set_up_refused(
    engine: &mut crate::engine::Engine,
    forge: u16,
    request: Token,
    why: domain::SetUpRefusal,
) {
    // The link already named this fault. Ordinary refusals stay in the connector.
    engine.events.push(temper_engine_domain::Event::Forge(Box::new(domain::Event::SetUpAnswered {
        forge,
        request,
        answer: domain::SetUpAnswer::Refused(why),
    })));
}
pub(crate) fn stopped_connection(engine: &mut crate::engine::Engine, forge: u16) {
    if engine.setup.lifecycle.local(forge).is_none() {
        engine.setup.lifecycle.reconcile.remove(&forge);
    }
}
pub(super) fn kind(plan: &host::Plan) -> Kind {
    match plan {
        host::Plan::Manifest { .. } => Kind::Manifest,
        host::Plan::Convert { .. } => Kind::Convert,
        host::Plan::Install { .. } => Kind::Install,
        host::Plan::Accept { .. } => Kind::Accept,
        host::Plan::Reconcile { .. } => Kind::Reconcile,
        host::Plan::Adopt { .. } => Kind::Adopt,
        host::Plan::Paste { .. } => unreachable!("temper-13 stages Paste"),
        // 14a1-iii introduces the handover and TakeOver; Check remains a later plan.
        host::Plan::Check { .. } | host::Plan::TakeOver { .. } => {
            unreachable!("Check is not staged; 14a1-iii introduces TakeOver")
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
    LocalOrigin(Token),
    Deleted(u16),
}
pub(super) fn done(engine: &mut crate::engine::Engine, token: Token, outcome: reconcile::Outcome) {
    let run = engine.setup.runs.get(&token).expect("ended setup run");
    let (forge, kind) = (run.forge, run.kind);
    if kind == Kind::LocalReconcile {
        engine.setup.runs.get_mut(&token).expect("reconcile done").kind = Kind::LocalConnect;
        let origin = records::origin(connection::settings::Kind::Forgejo, &configured(engine, forge).api).encode();
        keep(engine, forge, Item::Origin, &origin, Purpose::LocalOrigin(token));
        return;
    }
    if kind == Kind::Adopt {
        let adoption = engine.setup.lifecycle.adoptions.get_mut(&forge).expect("live start-time adoption");
        if adoption.next.get() < adoption.repositories {
            adoption.next = NonZeroU64::new(adoption.next.get().checked_add(1).expect("bounded repositories"))
                .expect("next repository");
        } else {
            let adoption = engine.setup.lifecycle.adoptions.remove(&forge).expect("last adoption");
            crate::forge::connect(engine, forge, Ok(crate::forge::Start::Adopted(adoption.access)));
        }
    }
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
    if run.kind == Kind::Adopt {
        engine.setup.lifecycle.adoptions.remove(&run.forge);
    }
    if run.kind == Kind::LocalReconcile {
        let forge = run.forge;
        engine.notes.push(crate::Note::ForgeSetupStopped {
            number: forge,
            why: crate::SetupStopped::Stopped {
                plan: crate::SetupPlan::Reconcile,
                step: crate::SetupStep { stage: failure.step, why: failure.why },
            },
        });
        engine.setup.runs.remove(&token);
        state(engine, forge, domain::Connection::NotConnected);
        return;
    }
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
        Purpose::LocalOrigin(run) => {
            let active = engine.setup.runs.remove(&run).expect("reconcile waits for Origin Kept");
            connect_local(engine, active.forge);
        }
        Purpose::Host(run) => engine.setup.down(host::Request::Answer { run, answer: host::Answer::Written }),
        Purpose::Origin(run) => {
            let active = engine.setup.runs.get_mut(&run).expect("conversion waits on Origin");
            active.kind = Kind::Follow;
            let forge = active.forge;
            super::run_plan(engine, run, host::Plan::Install { forge });
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
    let start = match access {
        Ok(start) => Ok(crate::forge::Start::Restored(start)),
        Err(why) => Err(why),
    };
    crate::forge::connect(engine, forge, start);
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
pub(super) fn installations(engine: &mut crate::engine::Engine, forge: u16) {
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
    crate::forge::tokens::accepted(engine, forge, &accepted);
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
    } else if let Some(local) = engine.setup.lifecycle.local(forge) {
        let rows = engine.forge_records.rows.forge(forge).expect("Connected has records");
        let id =
            skein_forge_setup::item::user(rows.get(Item::User).expect("connected User")).expect("valid User").get();
        (id, local.bot_login.clone())
    } else {
        // Forgejo still starts from private files (temper-15); its login is the git identity's.
        let mut login = Box::from([]);
        if let Some((_, value)) = engine.values.identities.newest(config.identity) {
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
        crate::forge::tokens::connected(engine, forge);
        engine.setup.lifecycle.reconcile.insert(forge, Reconcile::GitHub).expect("one reconcile per forge");
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
    if change == connection::Change::Deleted || change == connection::Change::Suspended {
        crate::forge::tokens::removed(engine, forge, installation);
    }
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
fn next_run(lifecycle: &mut Lifecycle) -> Token {
    assert!(lifecycle.next_run < (1_u64 << 60_u32), "internal setup names");
    let run = Token::new(crate::tokens::SETUP | lifecycle.next_run);
    lifecycle.next_run = lifecycle.next_run.checked_add(1).expect("setup run names");
    run
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
            || engine.notes.room() == 0
        {
            continue;
        }
        if start_pending(engine, *forge) {
            continue;
        }
        if busy(engine, *forge) {
            continue;
        }
        if let Some(adoption) = engine.setup.lifecycle.adoptions.get(forge) {
            if engine.setup.runs.len() == engine.setup.env.limits.runs {
                continue;
            }
            let repository = adoption.next;
            let run = next_run(&mut engine.setup.lifecycle);
            if !engine.setup.served.contains_key(forge) {
                let reason = *engine.setup.unserved.get(forge).expect("configured setup exclusion");
                engine.setup.lifecycle.adoptions.remove(forge);
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
                        kind: Kind::Adopt,
                        internal: true,
                        previous: domain::Connection::NotConnected,
                    },
                )
                .expect("admitted adoption");
            super::run_plan(engine, run, host::Plan::Adopt { forge: *forge, repository });
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
        } else if engine.setup.lifecycle.local(*forge).is_none()
            && engine.setup.lifecycle.reconcile.contains_key(forge)
            && connected_in_root(&engine.domain, *forge)
            && engine.setup.runs.len() < engine.setup.env.limits.runs
        {
            engine.setup.lifecycle.reconcile.remove(forge);
            let run = next_run(&mut engine.setup.lifecycle);
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
            super::run_plan(engine, run, host::Plan::Reconcile { forge: *forge, adopted: Box::new([]) });
        }
    }
}

#[expect(clippy::disallowed_macros, reason = "Stopped has no credential; diagnostic boundary")]
pub(crate) fn stopped_text(why: connection::Stopped) -> Box<[u8]> {
    alloc::format!("{why:?}").into_bytes().into_boxed_slice()
}

pub(super) fn refused(engine: &mut crate::engine::Engine, run: Token, forge: u16, reason: Box<[u8]>) {
    if engine.setup.runs.contains_key(&run) {
        engine.setup.lifecycle.adoptions.remove(&forge);
        finish(engine, run, root::SetupEnd::Stopped { forge, reason });
    } else {
        ended(engine, run, root::SetupEnd::Stopped { forge, reason });
    }
}

/// Called only once the root has completed its restore, never by the records loader.
pub(crate) fn restored(engine: &mut crate::engine::Engine) {
    for forge in &engine.workers.forges {
        if let Some(local) = engine.setup.lifecycle.local_mut(forge.number) {
            local.restored = true;
        }
    }
}
fn local_stopped(engine: &mut crate::engine::Engine, forge: u16, why: crate::SetupStopped) {
    engine.notes.push(crate::Note::ForgeSetupStopped { number: forge, why });
    state(engine, forge, domain::Connection::NotConnected);
}
fn start_pending(engine: &mut crate::engine::Engine, forge: u16) -> bool {
    if let Some(local) = engine.setup.lifecycle.local(forge)
        && local.restored
        && local.pending
    {
        start_local(engine, forge);
        return true;
    }
    false
}
fn start_local(engine: &mut crate::engine::Engine, forge: u16) {
    engine.setup.lifecycle.local_mut(forge).expect("local start").pending = false;
    if busy(engine, forge) || engine.setup.runs.len() == engine.setup.env.limits.runs {
        local_stopped(engine, forge, crate::SetupStopped::Busy);
        return;
    }
    if !engine.setup.served.contains_key(&forge) || engine.forge.unreachable(forge).is_some() {
        local_stopped(engine, forge, crate::SetupStopped::Missing);
        return;
    }
    // 14a1-iii introduces TakeOver; only an already kept AdminToken can start here.
    let Some(rows) = engine.forge_records.rows.forge(forge) else {
        local_stopped(engine, forge, crate::SetupStopped::NotPrepared);
        return;
    };
    if !rows.has(Item::AdminToken) {
        local_stopped(engine, forge, crate::SetupStopped::NotPrepared);
        return;
    }
    let valid = match rows.get(Item::AdminToken) {
        Ok(bytes) => skein_forge_setup::item::fits(
            skein_forge_setup::item::Item::AdminToken,
            bytes,
            &engine.forge_records.bounds,
        ),
        Err(_) => false,
    };
    if !valid {
        engine.notes.push(crate::Note::ForgeNotConnected { number: forge, why: records::NotConnected::Unreadable });
        return;
    }
    let expected = records::origin(connection::settings::Kind::Forgejo, &configured(engine, forge).api);
    if rows.has(Item::Origin) {
        let origin = match rows.get(Item::Origin) {
            Ok(bytes) => records::Origin::decode(bytes).ok(),
            Err(_) => None,
        };
        let why = match origin {
            Some(found) if found != expected => {
                Some(records::NotConnected::OtherForge { kind: found.kind, api: found.api })
            }
            None => Some(records::NotConnected::Unreadable),
            Some(_) => None,
        };
        if let Some(why) = why {
            engine.notes.push(crate::Note::ForgeNotConnected { number: forge, why });
            return;
        }
    }
    let adopted = engine.domain.adopted_ids(forge);
    let local = engine.setup.lifecycle.local(forge).expect("local start");
    let limits = &local.limits;
    assert!(
        skein_forge_setup::given::check_adopted(&adopted, limits).is_ok(),
        "restored adopted ids are checked at their entrance"
    );
    let run = next_run(&mut engine.setup.lifecycle);
    engine
        .setup
        .runs
        .insert(
            run,
            Run { forge, kind: Kind::LocalReconcile, internal: true, previous: domain::Connection::NotConnected },
        )
        .expect("admitted local reconcile");
    state(engine, forge, domain::Connection::Connecting);
    super::run_plan(engine, run, host::Plan::Reconcile { forge, adopted });
}
fn connect_local(engine: &mut crate::engine::Engine, forge: u16) {
    let expected = records::origin(connection::settings::Kind::Forgejo, &configured(engine, forge).api);
    let rows = engine.forge_records.rows.forge(forge).expect("reconciled records");
    let local = engine.setup.lifecycle.local_mut(forge).expect("local connection");
    let start = records::access(
        connection::settings::Kind::Forgejo,
        &expected,
        rows,
        &engine.forge_records.bounds,
        local.webhooks,
    );
    let start = match start {
        Ok(start) => start,
        Err(why) => {
            crate::forge::connect(engine, forge, Err(why));
            return;
        }
    };
    let token = rows.get(Item::GitToken).expect("reconciled GitToken");
    assert!(
        skein_forge_setup::item::fits(skein_forge_setup::item::Item::GitToken, token, &engine.forge_records.bounds),
        "kept GitToken validated"
    );
    if engine.values.identities.newest(local.identity.number).is_none() {
        local.identity.token = Secret::new(Box::from(token));
        let value = crate::translate::identity(&local.identity, &engine.workers.encoding.limits)
            .expect("checked identity bounds");
        local.identity.token = Secret::new(Box::new([]));
        let number = local.identity.number;
        let generation = engine.values.identities.put(number, value);
        engine.events.push(root::Event::Credential {
            number,
            generation,
            valid: skein_lib::Duration::from_nanos(u64::MAX),
        });
    }
    crate::forge::connect(engine, forge, Ok(crate::forge::Start::Restored(start)));
}
