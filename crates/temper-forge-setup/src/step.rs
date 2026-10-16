//! The box's ordered commands; each hides whether it is once or long.
use crate::{
    check::joined,
    setup::{Mode, Setup},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, bytes};
/// One command of the temper binary that the box runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    name: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    long: bool,
}
impl Step {
    /// Constructs a step from its display name, arguments and lifetime.
    #[must_use]
    pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, long: bool) -> Self {
        Self { name, arguments, long }
    }
    /// Whether this step runs until stopped.
    #[must_use]
    pub fn is_long(&self) -> bool {
        self.long
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
fn forge(action: &[u8], config: &[u8], number: u16, run: Option<&[u8]>) -> Step {
    let digits = Decimal::of(u64::from(number));
    let n = digits.as_bytes();
    let mut args = List::with_capacity(5);
    for a in [b"forge".as_slice(), action, config, n] {
        args.push(bytes::copy_of(a)).expect("five arguments");
    }
    let name = match run {
        Some(which) => {
            args.push(bytes::copy_of(which)).expect("fifth argument");
            joined(&[which, b" ", n])
        }
        None => joined(&[action, b" ", n]),
    };
    Step::new(name, args.into_boxed(), run.is_some())
}
/// Commands in setup order: owned processes, every reconcile, then serve.
#[must_use]
pub fn steps(setup: &Setup, configuration: &[u8]) -> Box<[Step]> {
    let count = u32::try_from(setup.entries.len()).expect("bounded entries");
    let mut out = List::with_capacity(
        count.checked_mul(4).expect("bounded step count").checked_add(1).expect("bounded step count"),
    );
    for e in &setup.entries {
        match &e.mode {
            Mode::Managed { .. } => {}
            Mode::Owned(o) => {
                out.push(forge(b"prepare", configuration, e.forge, None)).expect("step capacity");
                out.push(forge(b"run", configuration, e.forge, Some(b"forgejo"))).expect("step capacity");
                if o.runner.is_some() {
                    out.push(forge(b"run", configuration, e.forge, Some(b"runner"))).expect("step capacity");
                }
            }
        }
    }
    for e in &setup.entries {
        out.push(forge(b"reconcile", configuration, e.forge, None)).expect("step capacity");
    }
    out.push(Step::new(
        bytes::copy_of(b"serve"),
        Box::new([bytes::copy_of(b"serve"), bytes::copy_of(configuration)]),
        true,
    ))
    .expect("serve capacity");
    out.into_boxed()
}
