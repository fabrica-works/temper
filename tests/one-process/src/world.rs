//! One real startup, native person, hosted agent and git, and loopback byte peers.
use crate::{
    fixtures, forge,
    machine::{self, Machine, SignalHost},
    referee::{self, Capture, Store, Wire},
    stories,
};
use jig_fake_person::{Doing, Find, Next, Step, TreeFace};
use skein_io::kernel::{Complete, Exit, Fd, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, Memory};
use std::{cell::RefCell, rc::Rc};
use temper_shell_terminal::client::{NativeClient, Settings as ClientSettings, Start};
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub config: skein_sim::Config,
}
#[derive(Clone, Copy, Debug)]
pub enum Story {
    Chat { words: &'static [u8] },
    Adopt,
    Merge,
}
#[derive(Debug)]
pub struct Outcome {
    pub verdict: skein_world::domain::Verdict,
    pub signed_in: u32,
    pub chats_answered: u32,
    pub adopted: u32,
    pub pushed: Vec<Pushed>,
}
#[derive(Debug)]
pub struct Pushed {
    pub branch: Box<[u8]>,
    pub commit: u64,
    pub parents: Vec<u64>,
}
thread_local! { static STORE: RefCell<Option<Rc<RefCell<Store>>>> = const { RefCell::new(None) }; }
pub(crate) struct ServiceHost {
    pub(crate) service: temper_service::Service,
    capture: Capture,
    store: Rc<RefCell<Store>>,
    pub(crate) commits: Vec<crate::referee::durable::Commit>,
    pub(crate) wire: Wire,
    codec: jig_protocol_store::codec::Limits,
    families: temper_protocol_store::Bounds,
}
impl Host for ServiceHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for complete in self.service.completions().iter() {
            self.capture.complete(complete);
            self.wire.complete(complete);
        }
        self.commits.extend(self.store.borrow_mut().observe(&mut self.capture.events, &self.codec, &self.families));
        temper_service::iterate(&mut self.service, now, wall);
        for submit in self.service.submissions().iter() {
            self.capture.pending(&submit.kind);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.service.worst_case() + 8 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
}
pub(crate) enum Proc {
    Service(Box<ServiceHost>),
    Agent(Box<skein_world::PipeHost<smith_agent_shell::Agent>>),
    Git(Box<temper_fake_git::Git>),
    Shell(Box<stories::Shell>),
    Peer(Box<SignalHost<skein_fake_peers::llm::Peer>>),
    Forge(Box<forge::ForgeHost>),
    Client(Box<Person>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Service(h) => h.as_ref(),
            Self::Agent(h) => h.as_ref(),
            Self::Git(h) => h.as_ref(),
            Self::Shell(h) => h.as_ref(),
            Self::Peer(h) => h.as_ref(),
            Self::Forge(h) => h.as_ref(),
            Self::Client(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Service(h) => h.as_mut(),
            Self::Agent(h) => h.as_mut(),
            Self::Git(h) => h.as_mut(),
            Self::Shell(h) => h.as_mut(),
            Self::Peer(h) => h.as_mut(),
            Self::Forge(h) => h.as_mut(),
            Self::Client(h) => h.as_mut(),
        }
    }
}
impl Host for Proc {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
pub(crate) struct Person {
    pub(crate) wire: referee::client::Wire,
    client: Option<NativeClient>,
    root: Option<Fd>,
    actor: jig_fake_person::Person<temper_client_domain::Address>,
    wake: Option<Time>,
    pub(crate) signed_in: u32,
    pub(crate) finished: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Person {
    fn new(root: Fd, scenario: Story) -> Self {
        let steps = match scenario {
            Story::Chat { words } => vec![
                Step::Go {
                    address: temper_client_domain::Address::Jig(jig_domain_client::address::Address::Conversations {
                        project: None,
                    }),
                },
                Step::See {
                    find: Find::named(jig_view::Role::TextArea, b"First words"),
                    within: Duration::from_secs(5),
                },
                Step::Type { find: Find::named(jig_view::Role::TextArea, b"First words"), words: words.into() },
                Step::Press { find: Find::named(jig_view::Role::Button, b"Start a conversation") },
                Step::See { find: Find::text(fixtures::ANSWER), within: Duration::from_secs(5) },
            ],
            Story::Adopt | Story::Merge => stories::person(),
        };
        Self {
            wire: referee::client::Wire::default(),
            client: None,
            root: Some(root),
            actor: jig_fake_person::Person::new(steps.into_boxed_slice()),
            wake: None,
            signed_in: 0,
            finished: false,
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }
    }
    pub(crate) fn sign_in(&mut self, link: &[u8]) {
        assert!(self.client.is_none(), "one printed link");
        self.client = Some(
            NativeClient::new(
                ClientSettings {
                    engine: skein_io::Addr::Ip((std::net::Ipv4Addr::LOCALHOST, 7100).into()),
                    directory: b"private".as_slice().into(),
                    limits: temper_shell_terminal::limits::native(),
                },
                self.root.take().expect("private root"),
                1000,
                fixtures::SEED,
                Start::Link(link.into()),
            )
            .expect("service's printed link"),
        );
    }
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let Some(client) = &mut self.client else {
            return;
        };
        while let Some(complete) = self.completions.pop() {
            self.wire.complete(&complete);
            client.completions().push(complete);
        }
        client.iterate(now, wall);
        while let Some(signing) = client.signing() {
            assert!(matches!(signing, jig_shell_native::Signing::SignedIn { .. }), "sign-in failed: {signing:?}");
            self.signed_in += 1;
        }
        while let Some(notice) = client.notices() {
            assert!(
                !matches!(
                    notice,
                    temper_shell_terminal::client::Shown::Notice(temper_shell_terminal::route::Notice::Fact(_))
                ),
                "client protocol fact: {notice:?}"
            );
        }
        self.wake = None;
        if client.started() && !self.finished {
            let before = self.actor.at();
            match self.actor.next(&mut TreeFace::new(client.tree()), now) {
                Next::Do(Doing::Act { node, act }) => assert!(client.act(node, act), "offered person act"),
                Next::Do(Doing::Go { address }) => client.go(address),
                Next::Do(Doing::Reload) => panic!("chat script has no reload"),
                Next::Wait { until } => self.wake = until,
                Next::Done => {
                    self.finished = true;
                    client.close();
                }
                Next::Failed { step, why } => panic!("person step {step}: {why:?}"),
            }
            if self.actor.at() != before {
                self.wake = Some(now);
            }
        }
        while let Some(submit) = client.submissions().pop() {
            self.submissions.push(submit);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.wake.is_some_and(|at| at <= now)
            || self.client.as_ref().is_some_and(|client| client.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_deadline), self.wake].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_policy_deadline), self.wake].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.finished && self.client.as_ref().is_some_and(Host::is_empty)
    }
    fn worst_case(&self) -> u64 {
        self.client.as_ref().map_or(64 * 1024 * 1024, Host::worst_case)
    }
    fn operations(&self) -> u32 {
        512
    }
}
fn service(inherited: &Inherited) -> Proc {
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("state root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut service = temper_service::startup::startup(
        &fixtures::document(),
        &fixtures::files(),
        &fixtures::agent(),
        &fixtures::seeds(),
        None,
        temper_service::WorkerRuntime { root: worker, path: b"/worker".as_slice().into() },
        fixtures::forges(),
    )
    .expect("one-process startup");
    service.adopt_store(root, 1000).expect("deployment store");
    service.adopt_signals(inherited.signal).expect("service signal");
    let mut capture = Capture::default();
    capture.root(root);
    let families =
        temper_protocol_store::bounds(&temper_service::limits::profile(temper_service::limits::Profile::Small).forge)
            .expect("application codec bounds");
    Proc::Service(Box::new(ServiceHost {
        service,
        capture,
        store: STORE.with(|store| store.borrow().as_ref().expect("world store").clone()),
        commits: Vec::new(),
        wire: Wire::new(STORE.with(|store| store.borrow().as_ref().expect("world store").clone())),
        codec: fixtures::codec(),
        families,
    }))
}
fn agent(_spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let stderr = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(stderr);
    Proc::Agent(Box::new(
        capture.host(
            smith_agent_shell::Agent::new(
                smith_agent_shell::config::Configuration {
                    service: fixtures::agent_configuration(),
                    memory: fixtures::agent_profile().declared.memory,
                    trace: None,
                    profile: fixtures::agent_profile(),
                    declarations: fixtures::agent_declarations(),
                },
                smith_agent_shell::Resources {
                    input: inherited.pipes.iter().find(|(child, _)| *child == 0).expect("stdin").1,
                    output: inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1,
                    error: Some(stderr),
                    signals: inherited.signal,
                    seed: fixtures::SEED,
                    pid: 101,
                    trace: None,
                    roots: Some(inherited.roots.iter().map(|(_, fd)| *fd).collect()),
                },
                Box::new(errors),
            )
            .expect("hosted agent"),
        ),
    ))
}
fn git(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    Proc::Git(Box::new(temper_fake_git::Git::new(spawn, inherited, forge::transport())))
}
fn shell(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    Proc::Shell(Box::new(stories::Shell::new(spawn, inherited)))
}
fn worker_root(_spawn: &skein_io::kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    vec![skein_world::StartupRoot { name: b"worker".as_slice().into(), path: b"worker".as_slice().into() }]
}
fn agent_root(_spawn: &skein_io::kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    vec![skein_world::StartupRoot { name: b"worker".as_slice().into(), path: b"agent".as_slice().into() }]
}
#[must_use]
pub fn run(settings: Settings, scenario: Story) -> Outcome {
    stories::set(scenario);
    let mut disk = machine::fresh();
    let client = disk.reopen_root(2);
    let store = Rc::new(RefCell::new(Store::new()));
    STORE.with(|cell| {
        *cell.borrow_mut() = Some(store.clone());
    });
    let transport = stories::forge(scenario);
    let observed_forge = Rc::new(RefCell::new(referee::forge::Forge::new(transport.clone(), scenario)));
    forge::set(Some(transport.clone()));
    let mut world = skein_world::World::new_controlled(settings.seed, settings.config, Memory::Unchecked, |controls| {
        referee::Referee::new(controls, store.clone(), scenario, observed_forge.clone())
    })
    .with_machine(Machine(disk));
    world.host_roots(
        skein_world::HostedProgram {
            program: b"/opt/temper/bin/temper".as_slice().into(),
            make: agent,
            instances: fixtures::AGENTS,
            operations: fixtures::agent_limits().routes + 3,
        },
        agent_root,
    );
    world.host_roots(
        skein_world::HostedProgram {
            program: b"/usr/bin/git".as_slice().into(),
            make: git,
            instances: 1,
            operations: 1,
        },
        worker_root,
    );
    world.host_roots(
        skein_world::HostedProgram { program: b"/bin/sh".as_slice().into(), make: shell, instances: 1, operations: 1 },
        agent_root,
    );
    world.spawn_restartable(
        vec![
            skein_world::StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() },
            skein_world::StartupRoot { name: b"worker".as_slice().into(), path: b"worker".as_slice().into() },
        ],
        service,
    );
    world.spawn_root(skein_sim::Handle::new(client.raw()), |root| Proc::Client(Box::new(Person::new(root, scenario))));
    world.spawn(|| Proc::Peer(Box::new(SignalHost::new(stories::provider(scenario), None))));
    world.spawn(|| Proc::Forge(Box::new(forge::ForgeHost::new())));
    let outcome = world.run();
    let signed_in = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Client(person) => Some(person.signed_in),
            Proc::Service(_) | Proc::Agent(_) | Proc::Git(_) | Proc::Shell(_) | Proc::Peer(_) | Proc::Forge(_) => None,
        })
        .sum();
    let chats_answered = referee::terminals(&store.borrow());
    let adopted = stories::adopted(&store.borrow().rows);
    let pushed = std::mem::take(&mut observed_forge.borrow_mut().pushed);
    assert!(outcome.procs.iter().all(Host::is_empty), "all processes drain");
    forge::set(None);
    STORE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    Outcome { verdict: skein_world::domain::Verdict::Passed, signed_in, chats_answered, adopted, pushed }
}
