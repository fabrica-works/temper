//! Keep: bounded parts held until their page generation leaves.
//! Replace: Ops connector subjects with the forge's parts, which need no reads.
//! `Show` retains a part; `PartsLeft` frees that generation and every older one.
use jig_domain_client::contract::Part;
use skein_lib::{Env, List, Queue, Time};
mod vocabulary;
pub use vocabulary::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub parts: u32,
}
#[derive(Debug)]
pub enum Event {
    Went { address: Address },
    Left,
    Served { versions: jig_domain_client::contract::VersionRange },
    Show { part: Part, generation: u32 },
    PartsLeft { generation: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Full,
}
#[derive(Debug)]
pub enum Request {
    Fact { fact: Fact },
}
#[derive(Debug)]
struct Held {
    part: Part,
    generation: u32,
}
#[derive(Debug)]
pub struct Side {
    config: Config,
    address: Option<Address>,
    served: Served,
    held: List<Held>,
    shown: u64,
}
impl Side {
    #[must_use]
    pub fn new(limits: &Limits, config: Config) -> Side {
        Side { config, address: None, served: Served::Unknown, held: List::with_capacity(limits.parts), shown: 0 }
    }
    #[must_use]
    pub const fn address(&self) -> Option<Address> {
        self.address
    }
    #[must_use]
    pub const fn served(&self) -> Served {
        self.served
    }
    #[must_use]
    pub fn contains(&self, part: &Part, generation: u32) -> bool {
        for held in &self.held {
            if held.generation == generation && held.part == *part {
                return true;
            }
        }
        false
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("forge shown count fits");
    }
}
pub fn step(side: &mut Side, _env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Went { address } => {
            side.address = Some(address);
            side.changed();
        }
        Event::Left => {
            if side.address.take().is_some() {
                side.changed();
            }
        }
        Event::Served { versions } => {
            side.served = if versions.contains(side.config.version) { Served::Yes } else { Served::OutOfDate };
            side.changed();
        }
        Event::Show { part, generation } => {
            if side.contains(&part, generation) {
                return;
            }
            if side.held.push(Held { part, generation }).is_err() {
                out.push(Request::Fact { fact: Fact::Full });
            } else {
                side.changed();
            }
        }
        Event::PartsLeft { generation } => {
            let mut kept = List::with_capacity(side.held.capacity());
            let old = core::mem::replace(&mut side.held, List::with_capacity(0));
            let count = old.len();
            for held in old.into_boxed() {
                if held.generation > generation {
                    kept.push(held).expect("surviving parts fit their old bound");
                }
            }
            if count != kept.len() {
                side.changed();
            }
            side.held = kept;
        }
    }
}
#[must_use]
pub const fn next_deadline(_side: &Side) -> Option<Time> {
    None
}
pub const fn reclaim(_side: &mut Side) {}
#[must_use]
pub const fn shown(side: &Side) -> u64 {
    side.shown
}
#[must_use]
pub const fn max_out(_limits: &Limits) -> u32 {
    1
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    List::<Held>::worst_case(limits.parts)?.checked_mul(2)
}
