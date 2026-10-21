//! Durable setup needs and held manifest pages (protocol/forge.md, 2.6).
pub(crate) mod diagnostics;
pub(crate) mod lifecycle;
mod settings;
use alloc::boxed::Box;
use jig_protocol_store::Event as StoreEvent;
use lifecycle::{Kind, Purpose, Run};
pub(crate) use settings::prepare;
use skein_forge_setup::{host, reconcile};
use skein_lib::{CryptoRng, Duration, Env, Map, Queue, Secret, Time, Token, Wall};
use temper_engine_domain as root;
use temper_protocol_records::Item;

/// Plans are staged in the protocol layer: the root names them, never their credentials.
pub(crate) struct Layer {
    component: Option<host::Host>,
    pending: Option<(host::Settings, CryptoRng)>,
    pub(crate) env: Env<host::Limits>,
    pub(crate) above: Queue<host::Event>,
    pub(crate) io: Queue<skein_io::Request>,
    pub(crate) closed: bool,
    closing: bool,
    served: Map<u16, ()>,
    unserved: Map<u16, &'static [u8]>,
    pub(crate) staged: Map<Token, host::Plan>,
    runs: Map<Token, Run>,
    lifecycle: lifecycle::Lifecycle,
    writes: Map<Token, Write>,
    next_write: u64,
    pages: Queue<Page>,
    life: Duration,
    value_bytes: u32,
    #[cfg(test)]
    pub(crate) answers: Queue<(Token, host::Answer)>,
}
struct Write {
    purpose: Purpose,
    number: u64,
    value: Secret,
}
struct Page {
    forge: u16,
    state: Box<[u8]>,
    bytes: Box<[u8]>,
    until: Time,
}
// skein host/states.rs:53 spells its 16 random bytes as 32 hex bytes.
pub(crate) const STATE_TEXT_BYTES: u64 = 32;
/// Whole host plus the allocations retained by this owner (protocol/forge.md, 2.6).
pub(crate) fn worst_case(
    config: &crate::settings::Engine,
    forges: &skein_forge_connection::Settings,
    batch: u32,
) -> Option<u64> {
    use temper_engine_domain_forge::{BOT_LOGIN_SUFFIX, STOPPED_TEXT_BYTES};
    let (settings, l) = settings::fixed(config, forges).ok()?;
    let host = host::worst_case(&l, &settings)?; // Includes pending fixed Settings, moved into Host::new.
    let installations = config.limits.forge_component.installations;
    let count = l.forges.checked_mul(installations)?;
    let above = batch.checked_mul(host::MAX_OUT.above)?;
    let below = batch.checked_mul(host::MAX_OUT.below)?;
    let writes = l.runs.checked_add(l.forges)?;
    let name = u64::from(config.limits.forge.name_bytes);
    let page = u64::from(l.body_bytes).checked_add(STATE_TEXT_BYTES)?;
    let wanted = skein_forge::github::setup::PERMISSIONS.len().checked_add(skein_forge::github::setup::EVENTS.len())?;
    let lacking = skein_lib::List::<skein_forge_setup::github::Lacking>::worst_case(u32::try_from(wanted).ok()?)?
        .checked_add(diagnostics::worst_case()?)?;
    // Handed-up items, pages/links with state, failure subjects, or missing access.
    let payload =
        u64::from(l.body_bytes_in).max(page).max(u64::from(config.limits.records.value_bytes)).max(name).max(lacking);
    host.checked_add(Map::<u16, ()>::worst_case(l.forges)?)? // served
        .checked_add(Map::<u16, ()>::worst_case(l.forges)?)? // lifecycle.reconcile
        .checked_add(Map::<u16, ()>::worst_case(l.forges)?)? // lifecycle.deleting
        .checked_add(Map::<u16, &'static [u8]>::worst_case(l.forges)?)? // unserved
        // BASE stages only auto-Reconcile with empty adopted data; 07/08/09 must bound inputs first.
        .checked_add(Map::<Token, host::Plan>::worst_case(l.runs)?)?
        .checked_add(Map::<Token, Run>::worst_case(l.runs)?)?
        .checked_add(
            name.checked_add(u64::try_from(BOT_LOGIN_SUFFIX.len()).ok()?)?
                .max(STOPPED_TEXT_BYTES)
                .checked_mul(u64::from(l.runs))?,
        )?
        .checked_add(Map::<(u16, u64), ()>::worst_case(count)?)? // lifecycle.deleted
        .checked_add(Map::<(u16, u64), temper_engine_domain_forge::Installation>::worst_case(count)?)?
        .checked_add(name.checked_mul(u64::from(count))?)? // installation account logins
        .checked_add(Map::<Token, Write>::worst_case(writes)?)?
        // Encoded record, including VERSION; the store's own copy stays in store::worst_case.
        .checked_add(u64::from(config.limits.records.value_bytes).checked_mul(u64::from(writes))?)?
        .checked_add(Queue::<Page>::worst_case(l.states)?)?
        .checked_add(page.checked_mul(u64::from(l.states))?)?
        .checked_add(Queue::<host::Event>::worst_case(above)?)?
        .checked_add(payload.checked_mul(u64::from(above))?)?
        .checked_add(Queue::<skein_io::Request>::worst_case(below)?)?
        .checked_add(u64::from(l.io.output).checked_mul(u64::from(below))?)?
        .checked_add(skein_lib::List::<u16>::worst_case(l.forges)?)? // advance's forge-number scratch
        .checked_add(skein_lib::List::<u64>::worst_case(installations)?)? // accepted()
        .checked_add(skein_lib::List::<u64>::worst_case(installations)?)? // deleted
        .checked_add(skein_lib::List::<u64>::worst_case(installations)?)? // remaining
        // accepted_item's raw little-endian ids coexist with the encoded Write above.
        .checked_add(u64::try_from(size_of::<u64>()).ok()?.checked_mul(u64::from(installations))?)
}

impl Layer {
    pub(crate) fn new(
        settings: host::Settings,
        limits: host::Limits,
        crypto: CryptoRng,
        life: Duration,
        value_bytes: u32,
        batch: u32,
        installations: u32,
    ) -> Self {
        let mut served = Map::with_capacity(limits.forges);
        for forge in &settings.forges {
            served.insert(forge.number(), ()).expect("configured host forges");
        }
        Self {
            component: None,
            pending: Some((settings, crypto)),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(host::MAX_OUT.above.checked_mul(batch).expect("host output room")),
            io: Queue::with_capacity(host::MAX_OUT.below.checked_mul(batch).expect("host IO room")),
            closed: false,
            closing: false,
            served,
            unserved: Map::with_capacity(limits.forges),
            staged: Map::with_capacity(limits.runs),
            runs: Map::with_capacity(limits.runs),
            lifecycle: lifecycle::Lifecycle::new(limits.forges, installations),
            writes: Map::with_capacity(limits.runs.checked_add(limits.forges).expect("setup writes")),
            next_write: 1,
            pages: Queue::with_capacity(limits.states),
            life,
            value_bytes,
            #[cfg(test)]
            answers: Queue::with_capacity(64),
        }
    }
    pub(crate) fn room(&self) -> bool {
        self.batch_room(1)
    }
    pub(crate) fn batch_room(&self, batch: u32) -> bool {
        self.above.room() >= host::MAX_OUT.above.checked_mul(batch).expect("host batch above")
            && self.io.room() >= host::MAX_OUT.below.checked_mul(batch).expect("host batch IO")
    }
    pub(crate) fn down(&mut self, request: host::Request) {
        #[cfg(test)]
        if let host::Request::Answer { run, answer } = &request {
            self.answers.push((*run, answer.clone()));
        }
        self.component.as_mut().expect("host started after records load").down(
            &self.env,
            request,
            &mut self.above,
            &mut self.io,
        );
    }
    pub(crate) fn up(&mut self, event: skein_io::Event) {
        self.component.as_mut().expect("host owns IO").up(
            &self.env,
            crate::accounts::untag(event),
            &mut self.above,
            &mut self.io,
        );
    }
    pub(crate) fn close(&mut self) {
        if self.closing || self.closed || !self.room() {
            return;
        }
        self.closing = true;
        self.pages = Queue::with_capacity(self.env.limits.states);
        self.staged = Map::with_capacity(self.env.limits.runs);
        if self.component.is_some() {
            self.down(host::Request::Close);
        } else {
            self.pending = None;
            self.closed = true;
        }
    }
    #[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let host = match &self.component {
            Some(host) => host.next_deadline(),
            None => None,
        };
        crate::accounts::earliest(
            host,
            match self.pages.iter().next() {
                Some(page) => Some(page.until),
                None => None,
            },
        )
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        !self.above.is_empty() || !self.io.is_empty() || due(self.next_deadline(), now)
    }
    pub(crate) fn lifecycle_ready(&self, engine: &crate::engine::Engine) -> bool {
        !self.closing
            && self.component.is_some()
            && self.lifecycle.ready(&self.runs, self.env.limits.runs, &engine.domain)
    }
    /// A served page stays available for reload, until its own clock/count evicts it.
    pub(crate) fn page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        for page in &self.pages {
            if page.forge == forge && page.state.as_ref() == state && now < page.until {
                return Some(&page.bytes);
            }
        }
        None
    }
    fn hold(&mut self, forge: u16, state: Box<[u8]>, bytes: Box<[u8]>) {
        if self.pages.room() == 0 {
            let _oldest = self.pages.pop();
        }
        self.pages.push(Page { forge, state, bytes, until: self.env.now.saturating_add(self.life) });
    }
}
/// Exhaustive ownership of the pinned plans, including the later tasks' arms.
fn forge(plan: &host::Plan) -> u16 {
    match plan {
        host::Plan::Manifest { forge, .. }
        | host::Plan::Convert { forge, .. }
        | host::Plan::Install { forge }
        | host::Plan::Accept { forge, .. }
        | host::Plan::Reconcile { forge, .. } => *forge,
        host::Plan::Paste { .. } => unreachable!("temper-13 stages Paste"),
        host::Plan::Check { .. } | host::Plan::TakeOver { .. } | host::Plan::Adopt { .. } => {
            unreachable!("temper-14 stages Forgejo plans")
        }
        host::Plan::Reachable { .. } => unreachable!("temper-10 stages Reachable"),
    }
}
pub(crate) fn request(engine: &mut crate::engine::Engine, run: Token) {
    let plan = engine.setup.staged.remove(&run).expect("root names a staged setup plan");
    let number = forge(&plan);
    if engine.drain.is_some() || engine.setup.closing || engine.setup.component.is_none() {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped { forge: number, reason: Box::from(b"setup host is not running".as_slice()) },
        );
    } else if !engine.setup.served.contains_key(&number) {
        let reason = match engine.setup.unserved.get(&number) {
            Some(reason) => *reason,
            None => b"setup host has no entry for this forge",
        };
        ended(engine, run, root::SetupEnd::Stopped { forge: number, reason: Box::from(reason) });
    } else if matches_app_plan(&plan) && lifecycle::has_app(engine, number) {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped {
                forge: number,
                reason: Box::from(b"forge holds App records; Forget before making a new App".as_slice()),
            },
        );
    } else if lifecycle::busy(engine, number) || engine.setup.runs.len() == engine.setup.env.limits.runs {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped { forge: number, reason: Box::from(b"setup host: busy".as_slice()) },
        );
    } else {
        let kind = lifecycle::kind(&plan);
        engine
            .setup
            .runs
            .insert(run, Run { forge: number, kind, internal: false, previous: engine.domain.forge_connection(number) })
            .expect("admitted host run");
        if kind == Kind::Convert {
            lifecycle::state(engine, number, temper_engine_domain_forge::Connection::Connecting);
        }
        engine.setup.down(host::Request::Run { run, plan });
    }
}
fn matches_app_plan(plan: &host::Plan) -> bool {
    match plan {
        host::Plan::Manifest { .. } | host::Plan::Convert { .. } => true,
        host::Plan::Install { .. }
        | host::Plan::Accept { .. }
        | host::Plan::Reconcile { .. }
        | host::Plan::Paste { .. }
        | host::Plan::Check { .. }
        | host::Plan::TakeOver { .. }
        | host::Plan::Adopt { .. }
        | host::Plan::Reachable { .. } => false,
    }
}
fn ended(engine: &mut crate::engine::Engine, run: Token, end: root::SetupEnd) {
    engine.events.push(root::Event::Setup { run, end });
}
fn begin(engine: &mut crate::engine::Engine) {
    if !engine.forge_records.loading()
        && engine.end.is_none()
        && engine.drain.is_none()
        && let Some((settings, crypto)) = engine.setup.pending.take()
    {
        match host::Host::new(&engine.setup.env.limits, settings, crypto) {
            Ok(host) => engine.setup.component = Some(host),
            Err(why) => {
                let mut found = jig_protocol_config::Problems::empty();
                settings::problems(why, &mut found);
                engine.end = Some(crate::End::Stopped(crate::Stop::Start(found)));
            }
        }
    }
}
#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
pub(crate) fn advance(engine: &mut crate::engine::Engine) {
    engine.setup.env.now = engine.env.now;
    engine.setup.env.wall = engine.env.wall;
    begin(engine);
    for _ in 0..engine.setup.pages.len() {
        if !due(
            match engine.setup.pages.iter().next() {
                Some(page) => Some(page.until),
                None => None,
            },
            engine.env.now,
        ) {
            break;
        }
        let _expired = engine.setup.pages.pop();
    }
    if engine.setup.room()
        && let Some(host) = &mut engine.setup.component
        && due(host.next_deadline(), engine.env.now)
    {
        host.fire(&engine.setup.env, &mut engine.setup.above, &mut engine.setup.io);
    }
    for _ in 0..engine.setup.above.len() {
        if !engine.setup.room()
            || !engine.store.room()
            || !engine.forge.room(2)
            || engine.events.room()
                < engine.forge_records.bounds.installations.checked_add(4).expect("setup event room")
            || engine.notes.room() == 0
        {
            break;
        }
        let event = engine.setup.above.pop().expect("host output count");
        if engine.setup.closing && event != host::Event::Closed {
            continue;
        }
        match event {
            host::Event::Need { run, need } => {
                let forge = engine.setup.runs.get(&run).expect("live setup run").forge;
                match need {
                    host::Need::Read(item) => {
                        let item = item_record(item);
                        let value = match engine.forge_records.rows.forge(forge) {
                            Some(rows) => rows.get(item).ok(),
                            None => None,
                        };
                        let answer = host::Answer::Read(match value {
                            Some(bytes) => Some(Secret::new(Box::from(bytes))),
                            None => None,
                        });
                        engine.setup.down(host::Request::Answer { run, answer });
                    }
                    host::Need::Write { item, bytes } => {
                        lifecycle::keep(engine, forge, item_record(item), bytes.expose(), Purpose::Host(run));
                    }
                }
            }
            host::Event::Done { run, outcome } => lifecycle::done(engine, run, outcome),
            host::Event::Stop { run, failure } => lifecycle::stopped(engine, run, failure),
            host::Event::Closed => engine.setup.closed = true,
        }
        if engine.end.is_some() {
            break;
        }
    }
    lifecycle::advance(engine);
}
pub(crate) fn answer(engine: &mut crate::engine::Engine, event: StoreEvent) {
    if let StoreEvent::Kept { owner } = event {
        let write = engine.setup.writes.remove(&owner).expect("setup keep owner");
        engine.forge_records.rows.push(write.number, write.value, engine.setup.value_bytes);
        if !engine.setup.closing {
            lifecycle::kept(engine, write.purpose);
        }
    } else {
        crate::stop::unavailable(engine, b"forge records");
    }
}
fn item_record(item: skein_forge_setup::item::Item) -> Item {
    use skein_forge_setup::item::Item as I;
    match item {
        I::AdminToken => Item::AdminToken,
        I::ApiToken => Item::ApiToken,
        I::GitToken => Item::GitToken,
        I::User => Item::User,
        I::WebhookSecret => Item::WebhookSecret,
        I::AppId => Item::AppId,
        I::AppSlug => Item::AppSlug,
        I::AppKey => Item::AppKey,
        I::Accepted => Item::Accepted,
        I::Pasted => unreachable!("temper-13 provides carried paste rows"),
    }
}
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "failure has no credential; this is the diagnostic boundary"
)]
fn failure_text(failure: &reconcile::Failure) -> Box<[u8]> {
    alloc::format!(
        "setup {:?} {}: {:?}",
        failure.step,
        core::str::from_utf8(&failure.subject).unwrap_or("subject"),
        failure.why
    )
    .into_bytes()
    .into_boxed_slice()
}
fn due(at: Option<Time>, now: Time) -> bool {
    match at {
        Some(at) => at <= now,
        None => false,
    }
}
pub(crate) fn tag(mut request: skein_io::Request) -> skein_io::Request {
    if let skein_io::Request::Connect { owner, .. } = &mut request {
        assert!(owner.raw() < (1_u64 << 60_u32), "setup token fits prefix");
        *owner = Token::new(owner.raw() | crate::tokens::SETUP);
    }
    request
}
#[cfg(test)]
#[path = "tests/setup.rs"]
mod tests;
