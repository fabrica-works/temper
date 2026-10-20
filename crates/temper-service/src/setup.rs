//! Durable setup needs and held manifest pages (protocol/forge.md, 2.6).
mod settings;
use alloc::boxed::Box;
use jig_protocol_store::{Event as StoreEvent, SecretOwner, Secrets};
pub(crate) use settings::prepare;
use skein_forge_setup::{host, reconcile};
use skein_lib::{CryptoRng, Duration, Env, Map, Queue, Secret, Time, Token, Wall};
use temper_engine_domain as root;
use temper_protocol_records::{self as records, Item};

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
    runs: Map<Token, u16>,
    writes: Map<Token, Write>,
    next_write: u64,
    pages: Queue<Page>,
    life: Duration,
    value_bytes: u32,
    #[cfg(test)]
    pub(crate) answers: Queue<(Token, host::Answer)>,
}
struct Write {
    run: Token,
    number: u64,
    value: Secret,
}
struct Page {
    forge: u16,
    state: Box<[u8]>,
    bytes: Box<[u8]>,
    until: Time,
}
impl Layer {
    pub(crate) fn new(
        settings: host::Settings,
        limits: host::Limits,
        crypto: CryptoRng,
        life: Duration,
        value_bytes: u32,
        batch: u32,
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
            writes: Map::with_capacity(limits.runs),
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
    } else if engine.setup.runs.len() == engine.setup.env.limits.runs {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped { forge: number, reason: Box::from(b"setup host: busy".as_slice()) },
        );
    } else {
        engine.setup.runs.insert(run, number).expect("admitted host run");
        engine.setup.down(host::Request::Run { run, plan });
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
        if !engine.setup.room() || !engine.store.room() || engine.events.room() == 0 {
            break;
        }
        let event = engine.setup.above.pop().expect("host output count");
        if engine.setup.closing && event != host::Event::Closed {
            continue;
        }
        match event {
            host::Event::Need { run, need } => {
                let forge = *engine.setup.runs.get(&run).expect("live setup run");
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
                        let number = records::number(forge, item_record(item));
                        let value = Secret::new(records::encode(bytes.expose()).into_boxed_slice());
                        if value.expose().len() > usize::try_from(engine.setup.value_bytes).expect("value bound") {
                            crate::stop::unavailable(engine, b"forge records");
                            return;
                        }
                        assert!(engine.setup.next_write < (1_u64 << 60_u32), "setup write names fit prefix");
                        let owner = Token::new(crate::tokens::SETUP | engine.setup.next_write);
                        engine.setup.next_write = engine.setup.next_write.checked_add(1).expect("setup write names");
                        engine
                            .setup
                            .writes
                            .insert(owner, Write { run, number, value: value.clone() })
                            .ok()
                            .expect("one write per run");
                        if let Some(terminal) = engine
                            .store
                            .secrets(owner, Secrets::Keep { owner: SecretOwner::Application, number, value })
                        {
                            answer(engine, terminal);
                        }
                    }
                }
            }
            host::Event::Done { run, outcome } => {
                let forge = engine.setup.runs.remove(&run).expect("ended setup run");
                let outcome = match outcome {
                    reconcile::Outcome::Manifest { state, page } => {
                        engine.setup.hold(forge, state.clone(), page);
                        root::SetupOutcome::Manifest { state }
                    }
                    reconcile::Outcome::Install { state, url } => root::SetupOutcome::Install { state, url },
                    reconcile::Outcome::Nothing | reconcile::Outcome::Lacking { .. } => root::SetupOutcome::Done, // 05b logs the names; temper-13 shows them.
                    reconcile::Outcome::Identity { .. } | reconcile::Outcome::Reachable { .. } => {
                        unreachable!("later plans are not staged")
                    }
                };
                ended(engine, run, root::SetupEnd::Done { forge, outcome });
            }
            host::Event::Stop { run, failure } => {
                let _ended = engine.setup.runs.remove(&run).expect("stopped setup run");
                ended(engine, run, root::SetupEnd::Stopped { forge: failure.forge, reason: failure_text(&failure) });
            }
            host::Event::Closed => engine.setup.closed = true,
        }
        if engine.end.is_some() {
            break;
        }
    }
}
pub(crate) fn answer(engine: &mut crate::engine::Engine, event: StoreEvent) {
    if let StoreEvent::Kept { owner } = event {
        let write = engine.setup.writes.remove(&owner).expect("setup keep owner");
        engine.forge_records.rows.push(write.number, write.value, engine.setup.value_bytes);
        if !engine.setup.closing {
            engine.setup.down(host::Request::Answer { run: write.run, answer: host::Answer::Written });
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
