use crate::{exchange, fixtures};
use skein_lib::{Env, Queue, Rng, Secret, Time, Wall};
use std::net::SocketAddr;
use temper_fake_forge_protocol::{Service, Tally, setup::Store};
use temper_forge_forgejo::{document::Document, request::Request};
use temper_forge_setup::{
    place::File,
    reconcile::{self, Event, Failure, Need, Reconcile, Reply},
    setup::Entry,
};
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ended {
    Done,
    Stop(Failure),
}
pub struct World {
    entry: Entry,
    listener: Option<SocketAddr>,
    service: Service,
    env: Env<temper_fake_forge_protocol::Limits>,
    files: Vec<(File, Secret)>,
    rng: Rng,
    trace: Vec<String>,
    requests: Vec<Request>,
    secrets: Vec<Secret>,
}
impl World {
    #[must_use]
    pub fn managed(seed: u64) -> Self {
        Self::new(seed, false)
    }
    #[must_use]
    pub fn owned(seed: u64) -> Self {
        Self::new(seed, true)
    }
    fn new(seed: u64, owned: bool) -> Self {
        let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
        let mut service = Service::new(fixtures::config(), &limits, seed).expect("world service");
        let admin = service.setup_mut().account(b"temper-admin", true, b"");
        service.setup_mut().token(admin, b"setup-admin", &[b"all"], fixtures::secret(fixtures::ADMIN));
        if owned {
            service.setup_mut().account(b"ada", true, b"Ada");
        } else {
            service.setup_mut().org(b"acme", b"temper-admin");
            service.setup_mut().org(b"work", b"temper-admin");
        }
        Self {
            entry: fixtures::entry(owned),
            listener: None,
            service,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            files: vec![(File::AdminToken, fixtures::secret(fixtures::ADMIN))],
            rng: Rng::new(seed),
            trace: Vec::new(),
            requests: Vec::new(),
            secrets: vec![fixtures::secret(fixtures::ADMIN)],
        }
    }
    pub fn missing_org(&mut self, name: &[u8]) {
        self.entry.orgs = Box::new([name.into()]);
    }
    pub fn listening(&mut self, address: SocketAddr) {
        self.listener = Some(address);
    }
    #[must_use]
    pub fn file(&self, file: File) -> Option<&Secret> {
        self.files.iter().find(|(f, _)| *f == file).map(|(_, value)| value)
    }
    pub fn put(&mut self, file: File, bytes: Secret) {
        if matches!(file, File::AdminToken | File::ApiToken | File::GitToken | File::WebhookSecret) {
            self.secrets.push(bytes.clone());
        }
        if let Some((_, value)) = self.files.iter_mut().find(|(f, _)| *f == file) {
            *value = bytes;
        } else {
            self.files.push((file, bytes));
        }
    }
    #[must_use]
    pub fn store(&self) -> &Store {
        self.service.setup()
    }
    pub fn store_mut(&mut self) -> &mut Store {
        self.service.setup_mut()
    }
    #[must_use]
    pub fn tally(&self) -> &Tally {
        self.service.tally()
    }
    #[must_use]
    pub fn trace(&self) -> &[String] {
        &self.trace
    }
    #[must_use]
    pub fn requests(&self) -> &[Request] {
        &self.requests
    }
    #[must_use]
    pub fn secrets_absent(&self) -> bool {
        let mut checked = Vec::new();
        for secret in self.secrets.iter().chain(self.service.setup().issued().iter().map(|token| &token.secret)) {
            if secret.is_empty() || checked.contains(&secret) {
                continue;
            }
            if self.trace.iter().any(|text| text.as_bytes().windows(secret.len()).any(|bytes| bytes == secret.expose()))
            {
                return false;
            }
            checked.push(secret);
        }
        true
    }
    pub fn reconcile(&mut self) -> Ended {
        self.secrets.extend(self.service.setup().issued().iter().map(|token| token.secret.clone()));
        for org in &self.entry.orgs {
            self.secrets.extend(self.service.setup().subscriptions(org).map(|hook| hook.secret.clone()));
        }
        let mut machine = Reconcile::new(&self.entry, self.listener);
        let mut out = Queue::with_capacity(reconcile::MAX_OUT);
        let mut event = Event::Begin;
        for _ in 0..4096 {
            self.trace.push(format!("{event:?}"));
            reconcile::step(&mut machine, event, &mut out);
            assert_eq!(out.len(), 1, "one outstanding need");
            let need = out.pop().expect("one need");
            self.trace.push(format!("{need:?}"));
            event = match need {
                Need::Send { request, token } => {
                    if let Request::CreateUser { password, .. } = &request {
                        self.secrets.push(password.clone());
                    }
                    self.requests.push(request.clone());
                    let reply = exchange::send(&mut self.service, &self.env, &request, &token, self.rng.next_u64());
                    if let Reply::Document(Document::Token(token)) = &reply
                        && let Some(secret) = &token.value
                    {
                        self.secrets.push(secret.clone());
                    }
                    Event::Reply(reply)
                }
                Need::Read(file) => Event::Read(self.file(file).cloned()),
                Need::Write { file, bytes } => {
                    self.put(file, bytes);
                    Event::Written
                }
                Need::Draw(count) => {
                    let mut bytes = Vec::new();
                    for _ in 0..count {
                        bytes.push(self.rng.next_u64().to_le_bytes()[0]);
                    }
                    Event::Drawn(Secret::new(bytes.into_boxed_slice()))
                }
                Need::Done => return Ended::Done,
                Need::Stop(failure) => return Ended::Stop(failure),
            };
        }
        panic!("reconcile did not end")
    }
}
