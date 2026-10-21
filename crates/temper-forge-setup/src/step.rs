//! The box's ordered commands; each hides whether it is once, long or link.
use crate::{
    check::joined,
    setup::{Mode, Setup},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, bytes};
/// The word carried by a group's step lines (protocol/setup.md, 7.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Fetch,
    Prepare,
    Run,
}
impl Stage {
    #[must_use]
    pub fn word(self) -> &'static [u8] {
        match self {
            Self::Fetch => b"fetch",
            Self::Prepare => b"prepare",
            Self::Run => b"run",
        }
    }
}
/// A dormant forge's steps and their place in the always chain.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Group {
    pub forge: u16,
    pub before: u32,
    pub steps: Box<[(Stage, Step)]>,
}
/// Commands run at startup, and groups run when asked.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Steps {
    pub always: Box<[Step]>,
    pub asked: Box<[Group]>,
}
/// How the box runs a step (protocol/setup.md, 7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Once,
    Long,
    Link,
}
/// One command of the temper binary that the box runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    name: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    kind: Kind,
}
impl Step {
    /// Constructs a step from its display name, arguments and lifetime.
    #[must_use]
    pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, kind: Kind) -> Self {
        Self { name, arguments, kind }
    }
    /// How the box runs this step.
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }
    /// The step's name for prefixed output.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    /// Arguments passed to temper.
    #[must_use]
    pub fn arguments(&self) -> &[Box<[u8]>] {
        &self.arguments
    }
}
fn forge(action: &[u8], flags: &[Box<[u8]>], number: u16, run: Option<&[u8]>) -> Step {
    let digits = Decimal::of(u64::from(number));
    let n = digits.as_bytes();
    let mut args = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(4).expect("bounded arguments"),
    );
    for a in [b"forge".as_slice(), action] {
        args.push(bytes::copy_of(a)).expect("command words");
    }
    for flag in flags {
        args.push(flag.clone()).expect("flags counted");
    }
    args.push(bytes::copy_of(n)).expect("forge number");
    let name = match run {
        Some(which) => {
            args.push(bytes::copy_of(which)).expect("run argument");
            joined(&[which, b" ", n])
        }
        None => joined(&[action, b" ", n]),
    };
    Step::new(name, args.into_boxed(), if run.is_some() { Kind::Long } else { Kind::Once })
}
/// Carries the box's absolute directory flags after each command's words.
/// Commands in setup order: owned processes, every reconcile, then serve;
/// an asked forge's processes apart, in a group placed in that order.
#[must_use]
pub fn steps(setup: &Setup, flags: &[Box<[u8]>]) -> Steps {
    let count = u32::try_from(setup.entries.len()).expect("bounded entries");
    let mut out = List::with_capacity(
        count.checked_mul(4).expect("bounded step count").checked_add(1).expect("bounded step count"),
    );
    let mut asked = List::with_capacity(count);
    for e in &setup.entries {
        match &e.mode {
            // Fake processes and their preparation come later; keep their reconcile below.
            Mode::Managed { .. } | Mode::Fake(_) => {}
            Mode::Owned(o) => {
                let mut group = List::with_capacity(3);
                group.push((Stage::Prepare, forge(b"prepare", flags, e.forge, None))).expect("prepare");
                group.push((Stage::Run, forge(b"run", flags, e.forge, Some(b"forgejo")))).expect("forgejo");
                if o.runner.is_some() {
                    group.push((Stage::Run, forge(b"run", flags, e.forge, Some(b"runner")))).expect("runner");
                }
                if e.asked {
                    asked
                        .push(Group { forge: e.forge, before: out.len(), steps: group.into_boxed() })
                        .expect("entry capacity");
                } else {
                    for (_, step) in group.into_boxed() {
                        out.push(step).expect("step capacity");
                    }
                }
            }
        }
    }
    for e in &setup.entries {
        if !e.asked {
            out.push(forge(b"reconcile", flags, e.forge, None)).expect("step capacity");
        }
    }
    let mut serve = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(1).expect("bounded arguments"),
    );
    serve.push(bytes::copy_of(b"serve")).expect("command word");
    for flag in flags {
        serve.push(flag.clone()).expect("flags counted");
    }
    out.push(Step::new(bytes::copy_of(b"serve"), serve.into_boxed(), Kind::Long)).expect("serve capacity");
    Steps { always: out.into_boxed(), asked: asked.into_boxed() }
}
