//! Admission order and cues on one checked real world.
use crate::{agent, calls::Caller, fixtures, llm, service::ServiceHost};
use skein_io::kernel::{Complete, Exit, ServiceSignal, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{
    HostedProgram, Referee,
    real::{CheckedOutcome, Controls, World},
};

pub(crate) enum Proc {
    Llm(Box<skein_fake_peers::llm::Peer>),
    Service(Box<ServiceHost>),
    Caller(Caller),
    Agent(Box<skein_world::PipeHost<smith_agent_shell::Agent>>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Llm(h) => h.as_ref(),
            Self::Service(h) => h.as_ref(),
            Self::Caller(h) => h,
            Self::Agent(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Llm(h) => h.as_mut(),
            Self::Service(h) => h.as_mut(),
            Self::Caller(h) => h,
            Self::Agent(h) => h.as_mut(),
        }
    }
}
impl Host for Proc {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
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
pub(crate) fn run(
    scratch: &skein_scratch::Scratch,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> CheckedOutcome<Proc> {
    let mut world =
        World::new_controlled_checked(|controls| Judge { controls, stopping: false, finished: false, wake: None });
    world.spawn(|| Proc::Llm(Box::new(llm::peer(recipe()).expect("independent LLM peer"))));
    world.spawn_signals(|signals| Proc::Service(Box::new(ServiceHost::new(scratch.path().into(), signals))));
    world.spawn(|| Proc::Caller(Caller::new(steps())));
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
    world.run(&skein_shell::Clock::new(), crate::PATIENCE)
}
struct Judge {
    controls: Controls,
    stopping: bool,
    finished: bool,
    wake: Option<Time>,
}
impl Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        self.wake = None;
        let [Proc::Llm(peer), Proc::Service(service), Proc::Caller(caller), ..] = procs else {
            panic!("root admission order")
        };
        service.address = peer.address();
        if !service.link.is_empty() {
            caller.released = true;
        }
        if caller.released && !self.stopping {
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
        if (peer.address().is_some() && service.address.is_none())
            || (!service.link.is_empty() && !caller.released)
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
