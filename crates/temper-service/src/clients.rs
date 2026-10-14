//! The clients child and secret-free root handoffs (client.md, 2 and 8; service.md, 3).
use crate::settings::Seeds;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_engine_clients as kit;
use skein_io as io;
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
use temper_engine_domain as root;
pub mod translation;
const PREFIX: u64 = crate::tokens::CLIENTS;

#[derive(Debug)]
pub struct Clients {
    component: kit::Component,
    pub(crate) env: Env<kit::Limits>,
    out: Queue<kit::Request>,
    pub(crate) io: Queue<io::Request>,
    pub(crate) links: Queue<Box<[u8]>>,
    pub(crate) secrets: Queue<kit::SecretsRequest>,
    pub(crate) opened: bool,
    pub(crate) closed: bool,
    pending: bool,
    closing: Option<()>,
    pub(crate) starting: Option<()>,
    pub(crate) early: Queue<root::Event>,
    endings: Map<u64, Endings>,
    durable: u64,
    writes: u32,
    secret_owners: Map<Token, kit::SecretOwner>,
    next_secret: u64,
}
#[derive(Debug)]
struct Endings {
    numbers: List<u64>,
    next: u32,
}
fn invalid(what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(b"clients"), what);
    problems
}
impl Clients {
    pub fn new(config: &config::Engine, seeds: &Seeds, room: u32) -> Result<Self, Problems> {
        let config::Address::Addr(listener) = config.clients.listener else {
            return Err(invalid(What::Form));
        };
        let mut link_provider = None;
        let mut deployment_provider = 0;
        for provider in &config.providers {
            match provider.kind {
                config::ProviderKind::Link => link_provider = Some(provider.number),
                config::ProviderKind::Deployment => deployment_provider = provider.number,
                config::ProviderKind::Application => {}
            }
        }
        let link = if config.clients.link {
            let provider = link_provider.ok_or(invalid(What::Refers))?;
            let mut identity = None;
            for owner in &config.owners {
                if owner.identity.provider == provider {
                    identity = Some(people::Identity {
                        key: owner.identity.clone(),
                        login: Box::from(b"owner".as_slice()),
                        name: Box::from(b"Owner".as_slice()),
                    });
                    break;
                }
            }
            Some(identity.ok_or(invalid(What::Refers))?)
        } else {
            None
        };
        let mut served = List::with_capacity(2);
        served
            .push(kit::Served {
                family: None,
                first: jig_protocol_client_wire::VERSION,
                last: jig_protocol_client_wire::VERSION,
            })
            .or(Err(invalid(What::Bound)))?;
        let settings = kit::Settings {
            listener,
            link,
            deployment_provider,
            heartbeat: config.clients.heartbeat,
            rows: config.limits.core.documents.rows,
            served,
            providers: List::with_capacity(0),
            seed: seeds.crypto.clone(),
            stream: seeds.seed,
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients };
        let mut out = Queue::with_capacity(room);
        let component = kit::Component::new(settings, &env, &mut out).or(Err(invalid(What::Bound)))?;
        Ok(Self {
            component,
            env,
            out,
            io: Queue::with_capacity(room),
            links: Queue::with_capacity(room),
            secrets: Queue::with_capacity(room),
            opened: false,
            closed: false,
            pending: true,
            closing: None,
            starting: Some(()),
            early: Queue::with_capacity(room),
            endings: Map::with_capacity(config.limits.journal.commits),
            secret_owners: Map::with_capacity(room),
            next_secret: 1,
            durable: 0,
            writes: config.limits.journal.writes,
        })
    }
    #[must_use]
    pub fn room(&self, batch: u32) -> bool {
        let count = batch.checked_mul(kit::MAX_OUT_DOWN).expect("client routing room");
        self.out.room() >= count
            && self.io.room() >= count
            && self.links.room() >= count
            && self.secrets.room() >= count
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        !self.closed
            && (self.pending
                || !self.secrets.is_empty()
                || self.endings_ready()
                || !self.out.is_empty()
                || !self.early.is_empty()
                || !self.io.is_empty()
                || !self.links.is_empty()
                || match self.next_deadline() {
                    Some(at) => at <= now,
                    None => false,
                })
    }
    pub fn down(&mut self, event: kit::Event) {
        if let kit::Event::Close | kit::Event::Abort = &event {
            self.closing = Some(());
        }
        if !self.closed {
            self.component.down(&self.env, event, &mut self.out);
            self.pending = true;
        }
    }
    pub fn up(&mut self, event: io::Event) -> bool {
        let event = untag(event);
        match &event {
            io::Event::Listening { .. } => self.opened = true,
            io::Event::Failed { owner, .. } | io::Event::Closed { owner }
                if *owner == Token::new(0) && self.closing.is_none() =>
            {
                return false;
            }
            io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. }
            | io::Event::Closed { .. }
            | io::Event::Failed { .. } => {}
        }
        self.down(kit::Event::Io(event));
        true
    }
    fn root(&mut self, value: root::Event, events: &mut Queue<root::Event>) {
        if self.starting.is_some() {
            self.early.push(value);
        } else {
            events.push(value);
        }
    }
    pub fn advance(&mut self, events: &mut Queue<root::Event>) {
        for _ in 0..self.out.len() {
            if self.io.room() == 0
                || self.links.room() == 0
                || self.secrets.room() == 0
                || (self.starting.is_some() && self.early.room() == 0)
                || (self.starting.is_none() && events.room() == 0)
            {
                break;
            }
            match self.out.pop().expect("client requests") {
                kit::Request::Io(value) => self.io.push(tag(value)),
                kit::Request::Link { link } => self.links.push(link),
                kit::Request::Secrets(value) => self.secrets.push(value),
                kit::Request::Core(value) => match translation::to_root(*value) {
                    Ok(value) => self.root(root::Event::Party(value), events),
                    Err(value) => self.down(*value),
                },
                kit::Request::Application(value) => self.down(application(value)),
                kit::Request::Provider { exchange, .. } => {
                    self.down(kit::Event::Provider { exchange, verdict: kit::Verdict::Refused });
                }
                kit::Request::Closed => self.closed = true,
            }
        }
    }
    pub fn iterate(&mut self, now: Time, wall: Wall, events: &mut Queue<root::Event>) {
        self.env.now = now;
        self.env.wall = wall;
        if self.closed || !self.room(1) || self.early.room() < kit::MAX_OUT_DOWN || events.room() < kit::MAX_OUT_DOWN {
            return;
        }
        self.endings_step(events);
        self.advance(events);
        if self.closed {
            return;
        }
        self.component.fire(&self.env, &mut self.out);
        self.advance(events);
        self.pending = self.component.up(&self.env, &mut self.out);
        self.advance(events);
    }
}
#[must_use]
pub fn owns(token: Token) -> bool {
    token.raw() & (15_u64 << 60) == PREFIX
}
#[must_use]
pub fn tag(mut request: io::Request) -> io::Request {
    match &mut request {
        io::Request::Listen { owner, .. }
        | io::Request::Connect { owner, .. }
        | io::Request::Bind { owner, .. }
        | io::Request::Spawn { owner, .. }
        | io::Request::Usage { owner }
        | io::Request::Window { owner, .. } => {
            assert!(owner.raw() < (1_u64 << 60_u32), "bounded client exchange names");
            *owner = Token::new(owner.raw() | PREFIX);
        }
        io::Request::Reject { .. }
        | io::Request::Stream { .. }
        | io::Request::Output { .. }
        | io::Request::Signal { .. }
        | io::Request::Close { .. }
        | io::Request::Abort { .. } => {}
    }
    request
}
#[must_use]
pub fn untag(mut event: io::Event) -> io::Event {
    match &mut event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => {
            *owner = Token::new(owner.raw() & !(15_u64 << 60));
        }
        io::Event::Resized | io::Event::Shutdown { .. } => {}
    }
    event
}

#[must_use]
pub fn worst_case(limits: &kit::Limits, room: u32, journal: &skein_lib::JournalLimits) -> Option<u64> {
    kit::worst_case(limits)?
        .checked_add(Queue::<kit::Request>::worst_case(room)?)?
        .checked_add(Queue::<io::Request>::worst_case(room)?)?
        .checked_add(Queue::<root::Event>::worst_case(room)?)?
        .checked_add(limits.http.body.checked_mul(64)?.checked_mul(u64::from(room).checked_mul(2)?)?)?
        .checked_add(u64::from(limits.http.send).checked_mul(u64::from(room))?)?
        .checked_add(Queue::<Box<[u8]>>::worst_case(room)?)?
        .checked_add(u64::from(room).checked_mul(512)?)?
        .checked_add(Queue::<kit::SecretsRequest>::worst_case(room)?)?
        .checked_add(Map::<Token, kit::SecretOwner>::worst_case(room)?)?
        .checked_add(u64::from(room).checked_mul(40)?)?
        .checked_add(Map::<u64, Endings>::worst_case(journal.commits)?)?
        .checked_add(List::<u64>::worst_case(journal.writes)?.checked_mul(u64::from(journal.commits))?)
}

impl Clients {
    pub fn commit(&mut self, number: u64, writes: &[root::Write]) {
        let mut endings = List::with_capacity(self.writes);
        for write in writes {
            match write {
                root::Write::Erase(key) => {
                    if let Some(number) = ended(key)
                        && !endings.as_slice().contains(&number)
                    {
                        endings.push(number).expect("bounded commit writes");
                    }
                }
                root::Write::Save(_) => {}
            }
        }
        if !endings.is_empty() {
            self.endings.insert(number, Endings { numbers: endings, next: 0 }).expect("bounded commits in flight");
        }
    }
    pub fn committed(&mut self, number: u64) {
        self.durable = self.durable.max(number);
    }
    #[must_use]
    pub fn commit_room(&self) -> bool {
        self.endings.len() < self.endings.capacity()
    }
    fn endings_ready(&self) -> bool {
        for (number, _) in &self.endings {
            if *number <= self.durable {
                return true;
            }
        }
        false
    }
    fn endings_step(&mut self, events: &mut Queue<root::Event>) {
        for _ in 0..self.endings.len().checked_mul(self.writes).expect("bounded sign-in endings") {
            if !self.room(1) || events.room() < kit::MAX_OUT_DOWN {
                break;
            }
            let mut ready = None;
            for (number, _) in &self.endings {
                if *number <= self.durable {
                    ready = Some(*number);
                    break;
                }
            }
            let Some(number) = ready else { return };
            let endings = self.endings.get_mut(&number).expect("durable endings");
            let sign_in = *endings.numbers.get(endings.next).expect("one sign-in per erased key");
            endings.next = endings.next.checked_add(1).expect("bounded erase index");
            if endings.next == endings.numbers.len() {
                self.endings.remove(&number);
            }
            self.down(kit::Event::SignInEnded { sign_in });
            self.advance(events);
        }
    }
}

fn ended(key: &root::Key) -> Option<u64> {
    match key {
        root::Key::Core(core::Key::People(people::Key::SignIn(number))) => Some(*number),
        root::Key::Core(core::Key::People(_) | core::Key::Core(_) | core::Key::Tasks(_) | core::Key::Notes(_))
        | root::Key::Forge(_) => None,
    }
}

impl Clients {
    pub fn not_ready(&mut self, value: root::PartyInput) {
        if let Some(event) = translation::not_ready(value) {
            self.down(event);
        }
    }
}

impl Clients {
    pub fn secrets_step(&mut self, store: &mut crate::store::Store) {
        for _ in 0..store.secrets.len() {
            if self.out.room() < kit::MAX_OUT_DOWN {
                break;
            }
            let answer = store.secrets.pop().expect("secrets terminals");
            self.secret_answer(answer);
        }
        self.advance_secrets(store);
    }
    fn advance_secrets(&mut self, store: &mut crate::store::Store) {
        for _ in 0..self.secrets.len() {
            if self.out.room() < kit::MAX_OUT_DOWN
                || !store.room()
                || self.secret_owners.len() == self.secret_owners.capacity()
            {
                break;
            }
            let request = self.secrets.pop().expect("client secrets requests");
            let (owner, request) = secret_request(request);
            let token = Token::new(PREFIX.checked_add(self.next_secret).expect("secret routing name"));
            self.next_secret = self.next_secret.checked_add(1).expect("secret routing sequence");
            self.secret_owners.insert(token, owner).expect("bounded secrets in flight");
            if let Some(answer) = store.secrets(token, request) {
                self.secret_answer(answer);
            }
        }
    }
    fn secret_answer(&mut self, answer: jig_protocol_store::Event) {
        use jig_protocol_store::Event;
        let event = match answer {
            Event::Kept { owner } => {
                kit::SecretsAnswer::Kept { owner: self.secret_owners.remove(&owner).expect("secret keep owner") }
            }
            Event::Erased { owner } => {
                kit::SecretsAnswer::Erased { owner: self.secret_owners.remove(&owner).expect("secret erase owner") }
            }
            Event::Secret { owner, value } => {
                let owner = self.secret_owners.remove(&owner).expect("secret get owner");
                let value = match value {
                    Some(value) => match kept(value) {
                        Some(value) => Some(value),
                        None => {
                            self.down(kit::Event::Secrets(kit::SecretsAnswer::Failed));
                            return;
                        }
                    },
                    None => None,
                };
                kit::SecretsAnswer::Got { owner, value }
            }
            Event::SecretsLoaded { .. }
            | Event::Opened { .. }
            | Event::OpenRefused { .. }
            | Event::Committed { .. }
            | Event::Failed { .. }
            | Event::Restored { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::Unread { .. }
            | Event::Corrupt { .. } => kit::SecretsAnswer::Failed,
        };
        self.down(kit::Event::Secrets(event));
    }
}
fn secret_request(request: kit::SecretsRequest) -> (kit::SecretOwner, jig_protocol_store::Secrets) {
    use jig_protocol_store::{SecretOwner, Secrets};
    let owner = match &request {
        kit::SecretsRequest::Keep { owner, .. }
        | kit::SecretsRequest::Erase { owner }
        | kit::SecretsRequest::Get { owner } => *owner,
    };
    let (kind, number) = match owner {
        kit::SecretOwner::SignIn(number) => (SecretOwner::SignIn, number),
        kit::SecretOwner::Service(number) => (SecretOwner::Service, number),
    };
    let request = match request {
        kit::SecretsRequest::Keep { value, .. } => {
            let mut writer = skein_lib::Writer::new(40);
            writer.put(&value.expires.as_nanos().to_be_bytes()).expect("expiry bytes");
            writer.put(value.digest.expose()).expect("32-byte digest");
            Secrets::Keep { owner: kind, number, value: skein_lib::Secret::new(writer.finish()) }
        }
        kit::SecretsRequest::Erase { .. } => Secrets::Erase { owner: kind, number },
        kit::SecretsRequest::Get { .. } => Secrets::Get { owner: kind, number },
    };
    (owner, request)
}
fn kept(value: skein_lib::Secret) -> Option<kit::Kept> {
    let bytes = value.expose();
    if bytes.len() != 40 {
        return None;
    }
    let expires = u64::from_be_bytes(bytes.get(..8)?.try_into().ok()?);
    Some(kit::Kept { digest: skein_lib::Secret::new(Box::from(bytes.get(8..)?)), expires: Wall::from_nanos(expires) })
}

fn application(value: kit::ToApplication) -> kit::Event {
    match value {
        kit::ToApplication::Request { reply_to, .. } => {
            kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })
        }
        kit::ToApplication::Read { reply, family, .. } => {
            kit::Event::Application(kit::FromApplication::Read { reply, family, record: None })
        }
        kit::ToApplication::Watch { watcher, .. } => {
            kit::Event::WatchRefused { watcher, why: core::WatchRefusal::Unknown }
        }
    }
}
