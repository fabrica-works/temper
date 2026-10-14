//! Drain before closing children, close the store last (protocol.md, 6).
use crate::{End, Stop, engine::Engine};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_lib::Queue;
use temper_engine_domain as root;

pub(crate) struct Drain {
    closing: bool,
    pub(crate) busy: Queue<Box<temper_engine_domain_forge::Event>>,
}
pub(crate) fn shutdown(engine: &mut Engine) {
    if engine.drain.is_some() {
        engine.end = Some(End::Aborted);
        return;
    }
    engine.drain = Some(Drain { closing: false, busy: Queue::with_capacity(engine.events.capacity()) });
    engine.workers.down(engine.env.now, jig_protocol_engine_workers::Event::Stop, &mut engine.events);
    engine.events.push(root::Event::Stop);
}
pub(crate) fn drained(engine: &mut Engine) {
    let Some(drain) = &mut engine.drain else {
        return;
    };
    if drain.closing {
        return;
    }
    drain.closing = true;
    engine.workers.close(engine.env.now, &mut engine.events);
    if engine.accounts.is_some() {
        engine.step(root::Event::Account(jig_domain_accounts::Event::Close));
    }
}
pub(crate) fn advance(engine: &mut Engine) {
    let Some(drain) = &mut engine.drain else {
        return;
    };
    if !drain.closing {
        return;
    }
    let closed = match &engine.accounts {
        Some(accounts) => accounts.is_empty(),
        None => true,
    };
    if closed
        && engine.workers.closed()
        && engine.store.idle()
        && engine.events.is_empty()
        && engine.requests.is_empty()
    {
        engine.store.close();
        if engine.store.closed() {
            for _ in 0..drain.busy.len() {
                let _held = drain.busy.pop();
            }
            engine.end = Some(End::Drained);
        }
    }
}
pub(crate) fn failed(engine: &mut Engine, number: u64) {
    engine.step(root::Event::Store(root::StoreInput::Uncommitted { number }));
    engine.end = Some(End::Stopped(Stop::CommitFailed { number }));
}
pub(crate) fn unavailable(engine: &mut Engine, member: &[u8]) {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(member), What::Refers);
    engine.end = Some(End::Stopped(Stop::Start(problems)));
}
