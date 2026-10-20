//! The clients child and secret-free root handoffs (client/README.md, 2 and 8; jig's
//! protocol/service.md, 3).
use crate::settings::Seeds;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_engine_clients as kit;
use jig_protocol_engine_clients_http as http;
use skein_io as io;
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
use temper_engine_domain as root;
pub mod bundle;
pub(crate) mod policy;
pub(crate) mod printed;
pub mod translation;
const PREFIX: u64 = crate::tokens::CLIENTS;

#[derive(Debug)]
pub struct Clients {
    component: kit::Component,
    initial: Option<kit::Settings>,
    server: crate::server::Server,
    mount: http::Mount,
    mount_env: Env<http::Limits>,
    mount_out: Queue<http::Request>,
    server_out: Queue<crate::server::Routed>,
    replies: Map<Token, bundle::Reply>,
    send: u32,
    bundle: Option<bundle::Bundle>,
    component_closed: bool,
    stopping: Option<()>,

    secret: Option<skein_lib::Secret>,
    pub(crate) env: Env<kit::Limits>,
    out: Queue<kit::Request>,
    pub(crate) io: Queue<io::Request>,
    pub(crate) links: Queue<Box<[u8]>>,
    pub(crate) secrets: Queue<kit::SecretsRequest>,
    pub(crate) closed: bool,
    pending: bool,
    pub(crate) starting: Option<()>,
    pub(crate) early: Queue<root::Event>,
    endings: Map<u64, Endings>,
    durable: u64,
    writes: u32,
    secret_owners: Map<Token, Asked>,
    next_secret: u64,
}
/// What a secrets request in flight was for: a kept value's owner, or the
/// exchange whose carried value the store numbers itself.
#[derive(Clone, Copy, Debug)]
enum Asked {
    Owner(kit::SecretOwner),
    Carry(Token),
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
    pub fn new(
        config: &config::Engine,
        listener: &config::Address,
        seeds: &Seeds,
        room: u32,
        journal: &root::JournalLimits,
    ) -> Result<Self, Problems> {
        let config::Address::Addr(listener) = listener else {
            return Err(invalid(What::Form));
        };
        let mut providers = List::with_capacity(u32::try_from(config.providers.len()).expect("bounded providers"));
        let mut link_provider = None;
        let mut deployment_provider = 0;
        for provider in &config.providers {
            match provider.kind {
                config::ProviderKind::Link => link_provider = Some(provider.number),
                config::ProviderKind::Deployment => deployment_provider = provider.number,
                config::ProviderKind::Application => {
                    providers.push(provider.number).expect("provider count");
                }
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
                first: jig_client_protocol_wire::VERSION,
                last: jig_client_protocol_wire::VERSION,
            })
            .or(Err(invalid(What::Bound)))?;
        served
            .push(kit::Served {
                family: Some(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)),
                first: temper_client_protocol_wire::CLIENT_VERSION,
                last: temper_client_protocol_wire::CLIENT_VERSION,
            })
            .or(Err(invalid(What::Bound)))?;
        let settings = kit::Settings {
            link,
            deployment_provider,
            heartbeat: config.clients.heartbeat,
            rows: config.limits.core.documents.rows,
            served,
            providers,
            seed: seeds.crypto.clone(),
            stream: seeds.seed,
            models: config::clients::models(&config.llm),
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients };
        let mut out = Queue::with_capacity(room);
        let component = kit::Component::new(settings.clone(), &env, &mut out).or(Err(invalid(What::Bound)))?;
        Ok(Self {
            component,
            initial: Some(settings),
            server: crate::server::Server::new(*listener, &crate::limits::server(&config.limits)),
            mount: http::Mount::new(http_settings(), &config.limits.clients_http),
            mount_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients_http },
            mount_out: Queue::with_capacity(room),
            server_out: Queue::with_capacity(room),
            replies: Map::with_capacity(crate::limits::server(&config.limits).connections),
            send: crate::limits::server(&config.limits).http.send,
            bundle: None,
            component_closed: false,
            stopping: None,

            secret: None,
            env,
            out,
            io: Queue::with_capacity(room),
            links: Queue::with_capacity(room),
            secrets: Queue::with_capacity(room),
            closed: false,
            pending: true,
            starting: Some(()),
            early: Queue::with_capacity(room),
            endings: Map::with_capacity(journal.commits),
            secret_owners: Map::with_capacity(room),
            next_secret: 1,
            durable: 0,
            writes: journal.writes,
        })
    }
    /// Hold the bundle the binary read at start.
    pub(crate) fn adopt_bundle(&mut self, bundle: bundle::Bundle) {
        self.bundle = Some(bundle);
    }
    fn print_link(&mut self) {
        if let Some(listener) = self.server.listening()
            && let Some(secret) = self.secret.take()
        {
            self.links.push(printed::link(listener, &secret));
        }
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
        crate::accounts::earliest(self.component.next_deadline(), self.server.next_deadline())
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
        if let kit::Event::Stop | kit::Event::Close | kit::Event::Abort = &event {
            self.stopping = Some(());
            self.initial = None;
        }
        if let kit::Event::Stop = &event {
            self.server.stop(&mut self.server_out);
        }
        if let kit::Event::Close = &event {
            self.server.close(&mut self.server_out);
        }
        if let kit::Event::Abort = &event {
            self.server.abort(&mut self.server_out);
        }
        if !self.component_closed {
            self.component.down(&self.env, event, &mut self.out);
            self.pending = true;
        }
    }
    pub(crate) fn abort(&mut self, unstarted: bool) {
        self.down(kit::Event::Abort);
        if unstarted {
            self.io = Queue::with_capacity(self.io.capacity());
            self.closed = true;
        }
    }
    #[must_use]
    pub fn opened(&self) -> bool {
        self.server.opened()
    }
    pub fn up(&mut self, event: io::Event) -> bool {
        let event = untag(event);
        if let io::Event::Failed { owner, .. } | io::Event::Closed { owner } = &event
            && *owner == Token::new(0)
            && self.stopping.is_none()
        {
            return false;
        }
        if let io::Event::Listening { addr, .. } = &event
            && let Some(address) = addr.socket()
        {
            let mut hosts = List::with_capacity(1);
            hosts.push(printed::address_text(address).expect("bound listener address")).expect("one listener host");
            self.mount.down(&self.mount_env, http::Event::Listening { hosts }, &mut self.mount_out);
        }
        self.server.up(event, &mut self.server_out);
        self.print_link();
        self.pending = true;
        true
    }
    fn server_requests(&mut self) {
        for _ in 0..self.server_out.len() {
            if self.io.room() == 0 || self.out.room() < kit::MAX_OUT_DOWN {
                break;
            }
            match self.server_out.pop().expect("server requests") {
                crate::server::Routed::Io(request) => self.io.push(tag(request)),
                crate::server::Routed::Call { call, head } => {
                    let claimed = http::claims(&http_settings(), &head);
                    if claimed && self.mount.room() > 0 {
                        self.mount.down(&self.mount_env, http::Event::Call { call, head }, &mut self.mount_out);
                    } else {
                        self.serve_call(call, &head, claimed);
                    }
                }
                crate::server::Routed::Http { call, event } => {
                    if self.replies.get(&call).is_some() {
                        self.serve_event(call, event);
                    } else {
                        self.mount.down(&self.mount_env, http::Event::Http { call, event }, &mut self.mount_out);
                    }
                }
            }
        }
        self.closed = self.component_closed && self.server.closed();
    }
    fn root(&mut self, value: root::Event, events: &mut Queue<root::Event>) {
        if self.starting.is_some() {
            self.early.push(value);
        } else {
            events.push(value);
        }
    }
    pub fn advance(&mut self, events: &mut Queue<root::Event>, forges: &[crate::settings::Forge]) {
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
                kit::Request::Transport(message) => {
                    self.mount.down(&self.mount_env, http::Event::Component(message), &mut self.mount_out);
                }
                kit::Request::Link { secret } => {
                    self.secret = Some(secret);
                    self.print_link();
                }
                kit::Request::Secrets(value) => self.secrets.push(value),
                kit::Request::Core(value) => match translation::to_root(*value) {
                    Ok(value) => self.root(root::Event::Party(value), events),
                    Err(value) => self.down(*value),
                },
                kit::Request::Application(value) => {
                    match crate::family::to_root(value, forges, &temper_client_protocol_wire::v1::CEILINGS) {
                        Ok(value) => self.root(value, events),
                        Err(value) => self.down(*value),
                    }
                }
                kit::Request::Minted { .. } | kit::Request::MintRefused { .. } => {
                    unreachable!("temper mints no application sign-ins");
                }
                kit::Request::Closed => self.component_closed = true,
            }
        }
    }
    pub fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        events: &mut Queue<root::Event>,
        forges: &[crate::settings::Forge],
    ) {
        self.env.now = now;
        self.env.wall = wall;
        if let Some(settings) = self.initial.take() {
            // Link lifetimes start on the supplied clock, which need not start at zero.
            self.out = Queue::with_capacity(self.out.capacity());
            self.component =
                kit::Component::new(settings, &self.env, &mut self.out).expect("validated client settings");
        }
        self.mount_env.now = now;
        self.mount_env.wall = wall;
        self.server.clock(now, wall);
        if self.closed || !self.room(1) || self.early.room() < kit::MAX_OUT_DOWN || events.room() < kit::MAX_OUT_DOWN {
            return;
        }
        self.server.fire(&mut self.server_out);
        let server_pending = self.server.advance(&mut self.server_out);
        self.server_requests();
        self.endings_step(events, forges);
        self.advance(events, forges);
        if self.closed {
            return;
        }
        if !self.component_closed {
            self.component.fire(&self.env, &mut self.out);
        }
        self.advance(events, forges);
        self.pending = server_pending || !self.server_out.is_empty();
        let mount_pending = self.mount.up(&self.mount_env, &mut self.mount_out);
        self.pending |= mount_pending;
        if self.component_closed && !mount_pending && self.mount_out.is_empty() && self.out.is_empty() {
            self.server.finish();
            self.pending = true;
        }
        if let Some(request) = self.mount_out.pop() {
            self.pending = true;
            match request {
                http::Request::Component(message) => self.down(kit::Event::Transport(message)),
                http::Request::Http { call, request } => self.server.answer(call, request, &mut self.server_out),
            }
        }
        if !self.component_closed {
            self.pending |= self.component.up(&self.env, &mut self.out);
        }

        self.advance(events, forges);
    }
}
#[must_use]
pub fn owns(token: Token) -> bool {
    token.raw() & (15_u64 << 60) == crate::tokens::SERVER
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
            *owner = Token::new(owner.raw() | crate::tokens::SERVER);
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
pub fn worst_case(
    limits: &kit::Limits,
    transport: &crate::server::Limits,
    mapping: &http::Limits,
    room: u32,
    journal: &root::JournalLimits,
) -> Option<u64> {
    kit::worst_case(limits)?
        // First-clock initialization briefly owns both validated component instances.
        .checked_add(kit::worst_case(limits)?)?
        .checked_add(http::worst_case(mapping)?)?
        .checked_add(Map::<Token, bundle::Reply>::worst_case(transport.connections)?)?
        .checked_add(u64::from(transport.connections).checked_mul(u64::from(transport.http.head))?)?
        .checked_add(crate::server::worst_case(transport)?)?
        .checked_add(Queue::<crate::server::Routed>::worst_case(room)?)?
        .checked_add(Queue::<http::Request>::worst_case(room)?)?
        .checked_add(List::<Box<[u8]>>::worst_case(1)?.checked_add(64)?)?
        .checked_add(Queue::<kit::Request>::worst_case(room)?)?
        .checked_add(Queue::<io::Request>::worst_case(room)?)?
        .checked_add(Queue::<root::Event>::worst_case(room)?)?
        .checked_add(u64::from(limits.message_bytes).checked_mul(64)?.checked_mul(u64::from(room).checked_mul(2)?)?)?
        .checked_add(u64::from(transport.http.send).checked_mul(u64::from(room))?)?
        .checked_add(Queue::<Box<[u8]>>::worst_case(room)?)?
        .checked_add(u64::from(room).checked_mul(512)?)?
        .checked_add(Queue::<kit::SecretsRequest>::worst_case(room)?)?
        .checked_add(Map::<Token, Asked>::worst_case(room)?)?
        .checked_add(u64::from(room).checked_mul(40)?)?
        // A carried value waits here on its way to the store.
        .checked_add(u64::from(room).checked_mul(u64::from(limits.wire.keyed_value_some))?)?
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
    fn endings_step(&mut self, events: &mut Queue<root::Event>, forges: &[crate::settings::Forge]) {
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
            self.advance(events, forges);
        }
    }
}

fn ended(key: &root::Key) -> Option<u64> {
    match key {
        root::Key::Core(core::Key::People(people::Key::SignIn(number))) => Some(*number),
        root::Key::Core(
            core::Key::People(_)
            | core::Key::Core(_)
            | core::Key::Tasks(_)
            | core::Key::Notes(_)
            | core::Key::Accounts(_),
        )
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
            Event::Kept { owner } => kit::SecretsAnswer::Kept { owner: self.owner(owner) },
            Event::Erased { owner } => kit::SecretsAnswer::Erased { owner: self.owner(owner) },
            Event::Carried { owner, number } => {
                kit::SecretsAnswer::Carried { exchange: self.carry(owner), number: Some(number) }
            }
            Event::Stale { owner } => match self.secret_owners.remove(&owner).expect("secret stale owner") {
                Asked::Carry(exchange) => kit::SecretsAnswer::Carried { exchange, number: None },
                Asked::Owner(_) => kit::SecretsAnswer::Failed,
            },
            Event::Secret { owner, value } => {
                let owner = self.owner(owner);
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
    fn owner(&mut self, token: Token) -> kit::SecretOwner {
        match self.secret_owners.remove(&token).expect("secret request owner") {
            Asked::Owner(owner) => owner,
            Asked::Carry(_) => unreachable!("the store answers a carry Carried or Stale"),
        }
    }
    fn carry(&mut self, token: Token) -> Token {
        match self.secret_owners.remove(&token).expect("secret carry owner") {
            Asked::Carry(exchange) => exchange,
            Asked::Owner(_) => unreachable!("the store answers Carried only to a carry"),
        }
    }
}
fn secret_request(request: kit::SecretsRequest) -> (Asked, jig_protocol_store::Secrets) {
    use jig_protocol_store::Secrets;
    match request {
        kit::SecretsRequest::Keep { owner, value } => {
            let (kind, number) = place(owner);
            let mut writer = skein_lib::Writer::new(40);
            writer.put(&value.expires.as_nanos().to_be_bytes()).expect("expiry bytes");
            writer.put(value.digest.expose()).expect("32-byte digest");
            (Asked::Owner(owner), Secrets::Keep { owner: kind, number, value: skein_lib::Secret::new(writer.finish()) })
        }
        kit::SecretsRequest::Erase { owner } => {
            let (kind, number) = place(owner);
            (Asked::Owner(owner), Secrets::Erase { owner: kind, number })
        }
        kit::SecretsRequest::Get { owner } => {
            let (kind, number) = place(owner);
            (Asked::Owner(owner), Secrets::Get { owner: kind, number })
        }
        kit::SecretsRequest::Carry { exchange, value } => (Asked::Carry(exchange), Secrets::Carry { value }),
    }
}
/// Where the store keeps an owner's value; a carried value is the
/// application's, under the number the store drew (jig's store.md 9).
fn place(owner: kit::SecretOwner) -> (jig_protocol_store::SecretOwner, u64) {
    use jig_protocol_store::SecretOwner;
    match owner {
        kit::SecretOwner::SignIn(number) => (SecretOwner::SignIn, number),
        kit::SecretOwner::Service(number) => (SecretOwner::Service, number),
        kit::SecretOwner::Carried(number) => (SecretOwner::Application, number),
    }
}
fn kept(value: skein_lib::Secret) -> Option<kit::Kept> {
    let bytes = value.expose();
    if bytes.len() != 40 {
        return None;
    }
    let expires = u64::from_be_bytes(bytes.get(..8)?.try_into().ok()?);
    Some(kit::Kept { digest: skein_lib::Secret::new(Box::from(bytes.get(8..)?)), expires: Wall::from_nanos(expires) })
}

fn http_settings() -> http::Settings {
    http::Settings { prefix: Box::from(b"/jig".as_slice()), loopback: true }
}
