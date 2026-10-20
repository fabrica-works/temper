//! Admission order and cues on one checked real world.
use crate::{agent, calls::Caller, fixtures, forge, llm, service::ServiceHost};
use skein_io::kernel::{Complete, Exit, ServiceSignal, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{
    HostedProgram, Referee,
    real::{CheckedOutcome, Controls, World},
};

pub(crate) enum Proc {
    Llm(Box<Llm>),
    Forge(Box<skein_fake_peers::forge::Peer>),
    Service(Box<ServiceHost>),
    Caller(Box<Caller>),
    Ctl(Box<crate::calls::Recorded>),
    Agent(Box<skein_world::PipeHost<smith_agent_shell::Agent>>),
    Git(Box<temper_fake_git::Git>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Llm(h) => &h.peer,
            Self::Forge(h) => h.as_ref(),
            Self::Service(h) => h.as_ref(),
            Self::Caller(h) => h.as_ref(),
            Self::Ctl(h) => h.as_ref(),
            Self::Agent(h) => h.as_ref(),
            Self::Git(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Llm(h) => &mut h.peer,
            Self::Forge(h) => h.as_mut(),
            Self::Service(h) => h.as_mut(),
            Self::Caller(h) => h.as_mut(),
            Self::Ctl(h) => h.as_mut(),
            Self::Agent(h) => h.as_mut(),
            Self::Git(h) => h.as_mut(),
        }
    }
}
impl Host for Proc {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if !matches!(self, Self::Llm(llm) if llm.scripts.is_some()) {
            self.host_mut().iterate(now, wall);
        }
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !matches!(self, Self::Llm(llm) if llm.scripts.is_some()) && self.host().work_pending(now)
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
pub(crate) fn run(
    scratch: &skein_scratch::Scratch,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> CheckedOutcome<Proc> {
    let mut world = World::new_controlled_checked(|controls| Judge {
        controls,
        stopping: false,
        finished: false,
        armed: false,
        wake: None,
        forge: None,
    });
    world
        .spawn(|| Proc::Llm(Box::new(Llm { peer: llm::peer(recipe()).expect("independent LLM peer"), scripts: None })));
    world.spawn_signals(|signals| Proc::Service(Box::new(ServiceHost::new(scratch.path().into(), signals))));
    host(&mut world, scratch, steps);
    world.run(&skein_shell::Clock::new(), crate::PATIENCE)
}
type Scripts = fn(std::net::SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]>;
pub(crate) struct Llm {
    pub(crate) peer: skein_fake_peers::llm::Peer,
    scripts: Option<Scripts>,
}
impl std::ops::Deref for Llm {
    type Target = skein_fake_peers::llm::Peer;
    fn deref(&self) -> &Self::Target {
        &self.peer
    }
}
impl std::ops::DerefMut for Llm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.peer
    }
}
pub(crate) fn run_forge(
    scratch: &skein_scratch::Scratch,
    fixture: forge::Fixture,
    scripts: Scripts,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> skein_world::real::Outcome<Proc> {
    let state = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = World::new_controlled(|controls| Judge {
        controls,
        stopping: false,
        finished: false,
        armed: false,
        wake: None,
        forge: Some(state.clone()),
    });
    world.spawn(|| {
        Proc::Llm(Box::new(Llm {
            peer: llm::peer(llm::scripted(Box::new([]))).expect("deferred peer"),
            scripts: Some(scripts),
        }))
    });
    world.spawn_signals(|signals| {
        Proc::Service(Box::new(ServiceHost::configured(scratch.path().into(), signals, Some(fixture))))
    });
    host(&mut world, scratch, steps);
    world.spawn(|| {
        let (forge, peer) = forge::State::new(fixture);
        *state.borrow_mut() = Some(forge);
        Proc::Forge(Box::new(peer))
    });
    world.host_roots(crate::git::program(), crate::git::roots);
    let outcome = world.run(&skein_shell::Clock::new(), crate::PATIENCE);
    crate::git::share(None);
    outcome
}
fn host<R: Referee<Proc>>(
    world: &mut World<Proc, R>,
    scratch: &skein_scratch::Scratch,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) {
    let root = skein_shell::open_root(scratch.path()).expect("caller launch root");
    world.spawn_with_fds(vec![root], || Proc::Caller(Box::new(Caller::new(steps(), scratch.path().into(), root))));
    let limits = temper_client_shell_cli::limits::native();
    world.host_roots(
        HostedProgram {
            program: fixtures::CTL.into(),
            make: fixtures::ctl,
            instances: fixtures::CALLS_AT_ONCE,
            operations: skein_io::operations(&limits.io).expect("tools operations") * 2 + 5,
        },
        fixtures::ctl_roots,
    );
    let configuration =
        smith_agent_shell::config::read(&scratch.path().join("agent.json")).expect("agent operation bound");
    world.host_roots(
        HostedProgram {
            program: fixtures::AGENT.into(),
            make: agent::agent,
            instances: 1,
            operations: configuration.service.limits.routes + 3,
        },
        agent::roots,
    );
}
struct Judge {
    controls: Controls,
    stopping: bool,
    armed: bool,
    finished: bool,
    wake: Option<Time>,
    forge: Option<std::rc::Rc<std::cell::RefCell<Option<forge::State>>>>,
}
impl Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.wake = None;
        let (roots, children) = procs.split_at_mut(3);
        let [Proc::Llm(peer), Proc::Service(service), Proc::Caller(caller)] = roots else {
            panic!("root admission order")
        };
        if !self.armed {
            for name in caller.held_names() {
                let name = temper_client_tools::jig::key::Name::new(name).expect("held request name");
                service.arm(temper_client_tools::key::engine(&name));
            }
            self.armed = true;
        }
        for proc in children.iter_mut() {
            if let Proc::Ctl(recorded) = proc {
                let result = &mut caller.calls[recorded.call];
                result.locked = recorded.locked;
                result.released = recorded.released;
                result.busy = recorded.busy;
            }
        }
        if let Some(state) = &self.forge {
            let mut borrowed = state.borrow_mut();
            let state = borrowed.as_mut().expect("spawned forge");
            let Proc::Forge(forge) = &mut children[0] else { panic!("forge admission order") };
            service.forge_address = forge.address();
            if let Some(address) = forge.address()
                && let Some(scripts) = peer.scripts.take()
            {
                peer.peer = llm::peer(llm::scripted(scripts(address))).expect("bound forge scripts");
            }
            if let Some(hooks) = service.hooks
                && !state.hooked
            {
                state.hand_hooks(hooks);
            }
            if caller.advancing() && state.hooked && !state.moved {
                state.advance(now, skein_shell::Clock::new().now().wall);
            }
            service.seen.collect(forge, &state.handle);
            caller
                .advanced
                .set(state.moved && !service.seen.deliveries().is_empty() && !service.seen.responses().is_empty());
            if service.is_empty() {
                forge.shutdown();
            }
        }
        service.address = peer.address();
        if service.links() != 0 {
            caller.released = true;
        }
        if caller.watching.is_some() {
            service.arm_watch();
            caller.watched = service.delivered();
        }
        if caller.is_empty() && !self.stopping {
            self.controls.signal(1, ServiceSignal::Terminate);
            self.stopping = true;
        }
        if service.is_empty() && !self.finished {
            peer.shutdown();
            self.finished = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Proc]) {
        let [Proc::Llm(peer), Proc::Service(service), Proc::Caller(caller), ..] = procs else {
            panic!("root admission order")
        };
        caller.held.set(service.held_at(caller.held_index()));
        caller.links.set(service.links());
        let lock_changed = procs[3..].iter().any(|proc| match proc {
            Proc::Ctl(recorded) => {
                caller.calls[recorded.call].busy != recorded.busy
                    || caller.calls[recorded.call].locked != recorded.locked
                    || caller.calls[recorded.call].released != recorded.released
            }
            Proc::Llm(_) | Proc::Forge(_) | Proc::Service(_) | Proc::Caller(_) | Proc::Agent(_) | Proc::Git(_) => false,
        });
        let forge_changed = self.forge.as_ref().is_some_and(|state| {
            let borrowed = state.borrow();
            let state = borrowed.as_ref().expect("spawned forge");
            let Proc::Forge(forge) = &procs[3] else { panic!("forge root") };
            (forge.address().is_some() && service.forge_address.is_none())
                || (service.hooks.is_some() && !state.hooked)
                || (caller.advancing() && state.hooked && !state.moved)
                || (caller.advancing()
                    && !caller.advanced.get()
                    && (!forge.observations().is_empty() || !service.seen.responses().is_empty()))
        });
        if forge_changed
            || lock_changed
            || (peer.address().is_some() && service.address.is_none())
            || (service.links() != 0 && !caller.released)
            || (caller.watching.is_some() && (!service.watching() || service.delivered() && !caller.watched))
            || (caller.is_empty() && !self.stopping)
            || (service.is_empty() && !self.finished)
        {
            self.wake = Some(now);
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.finished
    }
}
