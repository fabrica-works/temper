//! Public Paste and Install; carried bytes stay beside the host (protocol/forge.md, 2.6).
use crate::engine::Engine;
use alloc::boxed::Box;
use core::num::NonZeroU64;
use jig_domain_core::Carried;
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_forge_setup::host;
use skein_lib::{Secret, Time, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;

#[derive(Debug)]
pub(super) enum Caller {
    Connect { forge: u16, request: Token, number: u64, pem: Option<Secret> },
    Install { forge: u16, request: Token, back: Option<Box<[u8]>> },
    Accept { forge: u16, call: Token, back: Option<Box<[u8]>> },
}
pub(super) struct ReturnTo {
    pub(super) forge: u16,
    pub(super) state: Box<[u8]>,
    pub(super) back: Option<Box<[u8]>>,
    pub(super) until: Time,
}
fn event(engine: &mut Engine, event: domain::Event) {
    engine.events.push(root::Event::Forge(Box::new(event)));
}
fn full(engine: &Engine, forge: u16) -> bool {
    super::lifecycle::busy(engine, forge)
        || engine.setup.runs.len() == engine.setup.env.limits.runs
        || engine.setup.public.len() == engine.setup.public.capacity()
}
fn stage(engine: &mut Engine, run: Token, plan: host::Plan) {
    engine.setup.staged.insert(run, plan).expect("public run room checked");
    super::request(engine, run);
}
pub(crate) fn connect(engine: &mut Engine, forge: u16, request: Token, app: NonZeroU64, carried: Carried) {
    if full(engine, forge) {
        event(
            engine,
            domain::Event::ConnectAnswered {
                forge,
                request,
                answer: domain::ConnectAnswer::Refused(domain::ForgeRefusal::Busy),
            },
        );
        return;
    }
    let run = super::lifecycle::next_run(&mut engine.setup.lifecycle);
    event(engine, domain::Event::ConnectAnswered { forge, request, answer: domain::ConnectAnswer::Connecting });
    let value = engine.store.secrets(run, Secrets::Get { owner: SecretOwner::Application, number: carried.number });
    let parsed = match value {
        Some(Event::Secret { value: Some(value), .. }) => {
            if skein_lib::sha256::digest(value.expose()) == carried.digest {
                pasted(value.expose()).ok_or(b"the pasted value is not a key".as_slice())
            } else {
                Err(b"the pasted value changed".as_slice())
            }
        }
        Some(_) | None => Err(b"the pasted value changed".as_slice()),
    };
    let (pem, failure) = match parsed {
        Ok(pem) => (Some(pem), None),
        Err(reason) => (None, Some(reason)),
    };
    engine
        .setup
        .public
        .insert(run, Caller::Connect { forge, request, number: carried.number, pem })
        .expect("public room checked");
    if let Some(reason) = failure {
        super::ended(engine, run, root::SetupEnd::Stopped { forge, reason: Box::from(reason) });
    } else {
        stage(engine, run, host::Plan::Paste { forge, app });
    }
}
fn pasted(bytes: &[u8]) -> Option<Secret> {
    if bytes.len() > 4096 {
        return None;
    }
    let mut reader = skein_lib::Reader::new(bytes);
    if reader.u8()? != 1 || reader.u8()? != 1 || u16::from_le_bytes(reader.bytes(2)?.try_into().ok()?) != 11 {
        return None;
    }
    let size = u32::from(u16::from_le_bytes(reader.bytes(2)?.try_into().ok()?));
    let pem = reader.bytes(size)?;
    if pem.is_empty() || !reader.is_empty() {
        return None;
    }
    Some(Secret::new(Box::from(pem)))
}
pub(crate) fn install(engine: &mut Engine, forge: u16, request: Token, back: Option<Box<[u8]>>) {
    if full(engine, forge) {
        event(
            engine,
            domain::Event::InstallAnswered {
                forge,
                request,
                answer: domain::InstallAnswer::Refused(domain::ForgeRefusal::Busy),
                stopped: None,
            },
        );
        return;
    }
    let run = super::lifecycle::next_run(&mut engine.setup.lifecycle);
    engine.setup.public.insert(run, Caller::Install { forge, request, back }).expect("public room checked");
    stage(engine, run, host::Plan::Install { forge });
}
fn done(end: &root::SetupEnd) -> bool {
    match end {
        root::SetupEnd::Done { .. } => true,
        root::SetupEnd::Stopped { .. } => false,
    }
}
pub(super) fn ended(engine: &mut Engine, run: Token, end: &root::SetupEnd) -> bool {
    let Some(caller) = engine.setup.public.remove(&run) else { return false };
    if let root::SetupEnd::Stopped { reason, .. } = end {
        engine.notes.push(crate::Note::BoxLink { line: reason.clone() });
    }
    let stopped = match end {
        root::SetupEnd::Stopped { reason, .. } => Some(reason.clone()),
        root::SetupEnd::Done { .. } => None,
    };
    match caller {
        Caller::Connect { forge, request, number, .. } => {
            if done(end) {
                engine.setup.pastes.insert(number, forge).expect("one pending connect per forge");
            }
            event(engine, domain::Event::ConnectEnded { forge, request, connected: done(end), why: stopped });
        }
        Caller::Install { forge, request, back } => {
            let answer = match end {
                root::SetupEnd::Done { outcome: root::SetupOutcome::Install { state, url }, .. } => {
                    hold(engine, forge, state.clone(), back);
                    domain::InstallAnswer::Link(url.clone())
                }
                root::SetupEnd::Done { .. } | root::SetupEnd::Stopped { .. } => domain::InstallAnswer::Connecting,
            };
            event(engine, domain::Event::InstallAnswered { forge, request, answer, stopped });
        }
        Caller::Accept { forge, call, back } => match end {
            root::SetupEnd::Done { .. } => engine.clients.github_answer(
                call,
                if back.is_some() { 303 } else { 200 },
                back.as_deref(),
                b"temper has the installation; you may close this tab.",
            ),
            root::SetupEnd::Stopped { .. } => super::routes::ended(engine, call, forge),
        },
    }
    true
}
pub(crate) fn erase(engine: &mut Engine, number: u64) {
    assert!(number >= temper_protocol_records::CARRIED, "only carried rows erased");
    let owner = if engine.forge_records.erasing(number) {
        crate::forge::records::LOAD
    } else {
        super::lifecycle::next_run(&mut engine.setup.lifecycle)
    };
    if let Some(forge) = engine.setup.pastes.remove(&number) {
        engine.setup.erases.insert(owner, forge).expect("one pending erase per forge");
    }
    if let Some(event) = engine.store.secrets(owner, Secrets::Erase { owner: SecretOwner::Application, number }) {
        super::answer(engine, event);
    }
}
pub(super) fn hold(engine: &mut Engine, forge: u16, state: Box<[u8]>, back: Option<Box<[u8]>>) {
    if engine.setup.backs.room() == 0 {
        let _oldest = engine.setup.backs.pop();
    }
    engine.setup.backs.push(ReturnTo { forge, state, back, until: engine.env.now.saturating_add(engine.setup.life) });
}
pub(super) fn take_back(engine: &mut Engine, forge: u16, state: &[u8]) -> Option<ReturnTo> {
    let mut found = None;
    for _ in 0..engine.setup.backs.len() {
        let back = engine.setup.backs.pop().expect("back count");
        if back.until <= engine.env.now {
            continue;
        }
        if back.forge == forge && back.state.as_ref() == state {
            found = Some(back);
        } else {
            engine.setup.backs.push(back);
        }
    }
    found
}
