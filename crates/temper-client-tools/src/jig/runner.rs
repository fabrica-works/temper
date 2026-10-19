//! Calls share an asker and partition their local numbers (tools contract, 4).
use crate::jig::{
    call::{Call, Value, Values},
    render,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{
    Asker, Client, Event, Handle, Latest, Readable, State, Told, Watchable,
    contract::{Delivery, Read, Subject},
};
use skein_lib::{Env, Queue};
#[derive(Debug)]
pub struct Printed {
    pub status: u8,
    pub stdout: Box<[u8]>,
    pub stderr: Box<[u8]>,
}
#[derive(Debug)]
pub struct Runner {
    asker: Asker,
    waiting: BTreeMap<u32, Called>,
    active: BTreeMap<u32, Called>,
    ready: bool,
    watching: bool,
    owner_part: Option<fn(&jig_client::contract::OwnerPart, &mut render::Text)>,
}
#[derive(Debug)]
pub struct Called {
    pub number: u32,
    pub call_number: u32,
    pub call: Call,
    pub json: bool,
}
#[derive(Debug)]
pub enum Out {
    Client { event: Event },
    Ready,
    Kept { line: Box<[u8]> },
    Printed { number: u32, printed: Printed },
}
impl Runner {
    #[must_use]
    pub fn new(asker: Asker, _salt: &'static [u8]) -> Self {
        Self {
            asker,
            waiting: BTreeMap::new(),
            active: BTreeMap::new(),
            ready: false,
            watching: false,
            owner_part: None,
        }
    }
    pub fn parts(&mut self, owner_part: fn(&jig_client::contract::OwnerPart, &mut render::Text)) {
        self.owner_part = Some(owner_part);
    }
    fn me(&self) -> Handle {
        Handle { asker: self.asker, number: 0 }
    }
    pub(crate) fn start(&mut self, out: &mut Queue<Out>) {
        if !self.watching {
            self.watching = true;
            out.push(Out::Client { event: Event::Watch { handle: self.me(), subject: Watchable::Jig(Subject::Me) } });
        }
    }
}
pub fn call(runner: &mut Runner, client: &Client, env: &Env<crate::jig::Limits>, called: Called, out: &mut Queue<Out>) {
    let count = runner.waiting.len().checked_add(runner.active.len()).expect("bounded calls");
    if count >= usize::try_from(env.limits.rows).expect("rows fit") {
        let printed = render::failure(b"call limit", called.json, 1);
        finish(called, printed, out);
    } else if !live(runner, client) {
        runner.waiting.insert(called.call_number, called);
    } else {
        run(runner, client, env, called, out);
    }
}
fn live(runner: &Runner, client: &Client) -> bool {
    let Some(document) = client.document(runner.me()) else { return false };
    document.state == State::Live
}
fn run(runner: &mut Runner, client: &Client, env: &Env<crate::jig::Limits>, called: Called, out: &mut Queue<Out>) {
    match called.call.tool {
        0 => {
            let Some(document) = client.document(runner.me()) else { return };
            let Some(Latest::Jig(Delivery::Me(me))) = document.latest else { return };
            let printed =
                render::me(me, called.json, &env.limits, runner.owner_part.expect("host supplies part vocabulary"));
            finish(called, printed, out);
        }
        3 => {
            let Some(Some(Values::One(Value::Number(task)))) = called.call.values.first() else {
                let printed = render::failure(b"invalid task call", called.json, 1);
                finish(called, printed, out);
                return;
            };
            let handle = Handle { asker: runner.asker, number: (u64::from(called.call_number) << 32u32) | 1 };
            let read = Readable::Jig(Read::Task { task: *task });
            runner.active.insert(called.call_number, called);
            out.push(Out::Client { event: Event::Read { handle, read } });
        }
        _ => {
            let printed = render::failure(b"call not supported", called.json, 1);
            finish(called, printed, out);
        }
    }
}
pub fn told(runner: &mut Runner, client: &Client, env: &Env<crate::jig::Limits>, told: Told, out: &mut Queue<Out>) {
    match told {
        Told::Changed { handle } if handle == runner.me() => {
            if live(runner, client) {
                if !runner.ready {
                    runner.ready = true;
                    out.push(Out::Ready);
                }
                for _ in 0..runner.waiting.len() {
                    let Some((_, called)) = runner.waiting.pop_first() else { break };
                    run(runner, client, env, called, out);
                }
            }
        }
        Told::Page { handle, page } => {
            let prefix = u32::try_from(handle.number >> 32u32).expect("high half fits");
            if let Some(called) = runner.active.remove(&prefix) {
                let printed = render::read(
                    &page,
                    client,
                    runner.me(),
                    called.json,
                    &env.limits,
                    runner.owner_part.expect("host supplies part vocabulary"),
                );
                finish(called, printed, out);
            }
        }
        Told::Changed { .. }
        | Told::Link { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Answered { .. }
        | Told::Pending { .. } => {}
    }
}
fn finish(called: Called, printed: Printed, out: &mut Queue<Out>) {
    out.push(Out::Printed { number: called.number, printed });
}
