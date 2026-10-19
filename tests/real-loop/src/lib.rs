//! The standalone stack on one checked real loop (domain/testing.md, 2.4).
#![forbid(unsafe_code)]

pub mod fixtures;
pub mod llm;
pub mod scripts;

/// A real deadline bounds every story, including startup and drain.
pub const PATIENCE: skein_lib::Duration = skein_lib::Duration::from_secs(10);
mod agent;
pub mod calls;
pub mod service;
mod world;
pub use skein_io::kernel::Exit;
pub use skein_lib::Time;
pub use skein_scratch::Scratch;
pub use temper_service::End;

/// The checked result retains its owners until assertions finish.
pub struct Ran {
    outcome: skein_world::real::CheckedOutcome<world::Proc>,
    scratch: Scratch,
}
impl Ran {
    #[must_use]
    pub fn calls(&self) -> &[calls::Called] {
        let world::Proc::Caller(caller) = &self.outcome.procs[2] else { panic!("caller root") };
        &caller.calls
    }
    fn service(&self) -> &service::ServiceHost {
        let world::Proc::Service(service) = &self.outcome.procs[1] else { panic!("service root") };
        service
    }
    #[must_use]
    pub fn served(&self) -> Option<&End> {
        self.service().service.as_ref().and_then(temper_service::Service::end)
    }
    #[must_use]
    pub fn link(&self) -> &[u8] {
        &self.service().link
    }
    #[must_use]
    pub fn llm(&self) -> &[skein_fake_peers::llm::Observation] {
        let world::Proc::Llm(peer) = &self.outcome.procs[0] else { panic!("LLM root") };
        peer.observations()
    }
    #[must_use]
    pub fn killed(&self) -> &[skein_world::real::Killed] {
        &self.outcome.killed
    }
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        self.scratch.path()
    }
    #[must_use]
    pub fn into_scratch(self) -> Scratch {
        drop(self.outcome);
        self.scratch
    }
}
#[must_use]
pub fn run(
    scripts: impl FnOnce() -> Box<[skein_fake_llm_domain::api::Script]>,
    steps: impl FnOnce() -> Vec<calls::Step>,
) -> Ran {
    run_on(Scratch::new("real-loop"), || llm::scripted(scripts()), steps)
}
#[must_use]
pub fn run_on(scratch: Scratch, recipe: impl FnOnce() -> llm::Recipe, steps: impl FnOnce() -> Vec<calls::Step>) -> Ran {
    let outcome = world::run(&scratch, recipe, steps);
    let ran = Ran { outcome, scratch };
    assert_eq!(ran.served(), Some(&End::Drained), "service end {:?}", ran.served());
    assert!(ran.killed().is_empty(), "drain killed {:?}", ran.killed());
    ran
}
