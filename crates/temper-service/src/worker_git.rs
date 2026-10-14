//! Git's records at the worker layer, with identity values below the root.
use crate::{limits, settings};
use jig_protocol_config::{Path, Problems, Segment, What};
use skein_io::{self as io, kernel};
use skein_lib::{Queue, Writer};
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git as component;

#[derive(Debug)]
pub(crate) struct Git {
    pub component: component::Component,
    pub out: component::Outputs,
    pub started: bool,
    pub ready: bool,
    pub refused: Option<Problems>,
}
impl Git {
    pub fn new(
        worker: &settings::Worker,
        own: &limits::Limits,
        path: &[u8],
        root: kernel::Fd,
        seed: u64,
    ) -> Result<Self, Problems> {
        let mut home = Writer::new(path.len().checked_add(5).ok_or(problem(b"git", What::Bound))?);
        home.put(path).expect("home path");
        home.put(b"/home").expect("home suffix");
        let component = match component::Component::new(
            &own.git_component,
            &own.checkout,
            component::Settings {
                git: worker.git.clone(),
                home: home.finish(),
                root,
                prefix: crate::tokens::git(0).ok_or(problem(b"git", What::Bound))?,
                serial: seed,
                largest_read: worker.jig.limits.io.intake,
                largest_room: worker.jig.limits.io.output,
            },
        ) {
            Ok(component) => component,
            Err(why) => return Err(unfit(why)),
        };
        let slots = own.git_component.operations.checked_add(1).ok_or(problem(b"git", What::Bound))?;
        let io = slots.checked_mul(component::MAX_OUT.io).ok_or(problem(b"git", What::Bound))?;
        Ok(Self {
            component,
            out: component::Outputs {
                events: Queue::with_capacity(slots),
                io: Queue::with_capacity(io),
                kernel: Queue::with_capacity(component::MAX_OUT.kernel),
            },
            started: false,
            ready: false,
            refused: None,
        })
    }
}
pub(crate) fn problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(member), what);
    problems
}
fn unfit(unfit: component::Unfit) -> Problems {
    let name: &[u8] = match unfit {
        component::Unfit::Operations => b"operations",
        component::Unfit::StdoutBytes => b"stdout_bytes",
        component::Unfit::StderrBytes => b"stderr_bytes",
        component::Unfit::ReadBytes => b"read_bytes",
        component::Unfit::SweepEntries => b"sweep_entries",
        component::Unfit::SweepBytes => b"sweep_bytes",
        component::Unfit::Memory => b"memory_bytes",
    };
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"limits").member(b"git_component").member(name), What::Bound);
    problems
}
#[must_use]
pub(crate) fn identity(op: &git::Op) -> Option<u32> {
    match op {
        git::Op::Clone { identity, .. }
        | git::Op::Fetch { identity, .. }
        | git::Op::Create { identity, .. }
        | git::Op::Commit { identity, .. }
        | git::Op::Push { identity, .. } => Some(*identity),
        git::Op::Make { .. } | git::Op::CheckOut { .. } | git::Op::Merge { .. } => None,
    }
}
#[must_use]
pub(crate) fn worst_case(own: &limits::Limits) -> Option<u64> {
    component::check(&own.git_component, &own.checkout).ok()?;
    let slots = own.git_component.operations.checked_add(1)?;
    let bytes = component::worst_case(&own.git_component, &own.checkout)?;
    bytes
        .checked_add(Queue::<component::Event>::worst_case(slots)?)?
        .checked_add(Queue::<io::Request>::worst_case(slots.checked_mul(component::MAX_OUT.io)?)?)?
        .checked_add(Queue::<kernel::Submit>::worst_case(component::MAX_OUT.kernel)?)?
        .checked_add(Queue::<kernel::Submit>::worst_case(slots.checked_add(2)?)?)?
        .checked_add(bytes.checked_mul(u64::from(slots))?)
}

impl crate::Worker {
    pub(crate) fn git_drain(&mut self) {
        for _ in 0..self.git.out.events.len() {
            match self.git.out.events.pop().expect("git events") {
                component::Event::Started => self.git.ready = true,
                component::Event::Unusable { .. } => self.git.refused = Some(problem(b"git", What::Form)),
                component::Event::Done { owner, done } => {
                    if self.aborted.is_none() {
                        self.events.push(temper_worker_domain::Event::Done { owner, done });
                    }
                }
            }
        }
        for _ in 0..self.git.out.io.len() {
            self.io.push(self.git.out.io.pop().expect("git io"));
        }
        for _ in 0..self.git.out.kernel.len() {
            self.submitted.push(self.git.out.kernel.pop().expect("git kernel"));
        }
    }
    pub(crate) fn git_request(&mut self, request: component::Request) {
        self.git.component.down(self.env.now, request, &mut self.git.out);
        self.git_drain();
    }
}
