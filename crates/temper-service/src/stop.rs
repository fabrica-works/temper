//! Drain before closing children, close the store last (protocol/README.md, 6).
use crate::{End, Stop, engine::Engine};
use jig_protocol_config::{Path, Problems, What};
use temper_engine_domain as root;

pub(crate) struct Drain {
    pub(crate) closing: bool,
    stopped: Option<Stop>,
}
pub(crate) fn shutdown(engine: &mut Engine) {
    if engine.drain.is_some() {
        engine.end = Some(End::Aborted);
        return;
    }
    engine.drain = Some(Drain { closing: false, stopped: None });
    engine.workers.down(engine.env.now, jig_protocol_engine_workers::Event::Stop, &mut engine.events);
    engine.clients.down(jig_protocol_engine_clients::Event::Stop);
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
    engine.setup.close();
    engine.forge.down(skein_forge_connection::Request::Close);
    engine.clients.down(jig_protocol_engine_clients::Event::Close);
    engine.workers.close(engine.env.now, &mut engine.events);
    if engine.accounts.is_some() {
        engine.step(root::Event::Account(jig_domain_accounts::Event::Close));
    }
}
pub(crate) fn advance(engine: &mut Engine) {
    if engine.end.is_some() {
        return;
    }
    let Some(drain) = &mut engine.drain else {
        return;
    };
    if !drain.closing {
        return;
    }
    engine.setup.close();
    let closed = match &engine.accounts {
        Some(accounts) => accounts.is_empty(),
        None => true,
    };
    if closed
        && engine.setup.closed
        && engine.setup.io.is_empty()
        && engine.forge.closed
        && engine.forge.io.is_empty()
        && engine.workers.closed()
        && engine.clients.closed
        && engine.clients.io.is_empty()
        && engine.store.idle()
        && engine.events.is_empty()
        && engine.requests.is_empty()
    {
        engine.store.close();
        if engine.store.closed() {
            engine.end = Some(match drain.stopped.take() {
                Some(why) => End::Stopped(why),
                None => End::Drained,
            });
        }
    }
}
pub(crate) fn failed(engine: &mut Engine, number: u64) {
    engine.step(root::Event::Store(root::StoreInput::Uncommitted { number }));
    engine.end = Some(End::Stopped(Stop::CommitFailed { number }));
}
pub(crate) fn version(engine: &mut Engine) {
    engine.end = Some(End::Stopped(Stop::StoreVersion));
}
pub(crate) fn unavailable(engine: &mut Engine, member: &[u8]) {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(member), What::Refers);
    engine.end = Some(End::Stopped(Stop::Start(problems)));
}

/// Losing the webhook listener drains the engine (protocol/forge.md, 2.3).
pub(crate) fn hooks(engine: &mut Engine) {
    if engine.drain.is_none() {
        shutdown(engine);
    }
    if let Some(drain) = &mut engine.drain
        && drain.stopped.is_none()
    {
        drain.stopped = Some(Stop::Hooks);
    }
}
