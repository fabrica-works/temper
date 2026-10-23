//! GitHub's setup visit, after jig's claim (client/forge.md, 9.1).
use crate::engine::Engine;
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge_setup::host;
use skein_http::{Method, server::Call};
use skein_lib::{Decimal, Token, Writer};

fn number(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn split(target: &[u8]) -> (&[u8], &[u8]) {
    match skein_lib::bytes::find(target, b"?") {
        Some(at) => (target.get(..at).expect("query"), target.get(at.saturating_add(1)..).expect("query")),
        None => (target, b""),
    }
}
fn forge(path: &[u8]) -> Option<u16> {
    let digits = path.strip_prefix(b"/forges/")?.strip_suffix(b"/github/setup")?;
    u16::try_from(number(digits)?).ok()
}
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        b'A'..=b'F' => byte.checked_sub(b'A')?.checked_add(10),
        _ => None,
    }
}
fn value(bytes: &[u8]) -> Option<Box<[u8]>> {
    let mut escapes = 0_usize;
    for byte in bytes {
        if *byte == b'%' {
            escapes = escapes.checked_add(1)?;
        }
    }
    let mut writer = Writer::new(bytes.len().checked_sub(escapes.checked_mul(2)?)?);
    let mut at = 0_usize;
    for _ in 0..bytes.len() {
        if at == bytes.len() {
            break;
        }
        let byte = *bytes.get(at)?;
        let decoded = if byte == b'%' {
            let hi = hex(*bytes.get(at.checked_add(1)?)?)?;
            let lo = hex(*bytes.get(at.checked_add(2)?)?)?;
            at = at.checked_add(2)?;
            hi.checked_mul(16)?.checked_add(lo)?
        } else if byte == b'+' {
            b' '
        } else {
            byte
        };
        if !(b'!'..=b'~').contains(&decoded) {
            return None;
        }
        writer.put(core::slice::from_ref(&decoded)).ok()?;
        at = at.checked_add(1)?;
    }
    Some(writer.finish())
}
struct Query {
    installation: Option<NonZeroU64>,
    action: Box<[u8]>,
    state: Option<Box<[u8]>>,
}
fn query(bytes: &[u8]) -> Option<Query> {
    let mut installation = None;
    let mut action = None;
    let mut state = None;
    let mut rest = Some(bytes);
    for _ in 0_u32..3 {
        let Some(bytes) = rest.take() else {
            break;
        };
        let field = match skein_lib::bytes::find(bytes, b"&") {
            Some(at) => {
                rest = Some(bytes.get(at.checked_add(1)?..)?);
                bytes.get(..at)?
            }
            None => bytes,
        };
        let eq = skein_lib::bytes::find(field, b"=")?;
        let name = value(field.get(..eq)?)?;
        let bytes = value(field.get(eq.checked_add(1)?..)?)?;
        match name.as_ref() {
            b"installation_id" if installation.is_none() => installation = Some(NonZeroU64::new(number(&bytes)?)?),
            b"setup_action" if action.is_none() => action = Some(bytes),
            b"state" if state.is_none() && !bytes.is_empty() => state = Some(bytes),
            _ => return None,
        }
    }
    if rest.is_some() {
        return None;
    }
    let action = action?;
    match action.as_ref() {
        b"install" | b"update" if installation.is_some() => {}
        b"request" if installation.is_none() => {}
        _ => return None,
    }
    Some(Query { installation, action, state })
}
fn fallback(forge: u16) -> Box<[u8]> {
    let digits = Decimal::of(u64::from(forge));
    let mut writer =
        Writer::new(b"/forges/connect?forge=".len().checked_add(digits.as_bytes().len()).expect("bounded fallback"));
    writer.put(b"/forges/connect?forge=").expect("fallback room");
    writer.put(digits.as_bytes()).expect("forge room");
    writer.finish()
}
fn escaped(byte: &u8) -> &[u8] {
    match byte {
        b'&' => b"&amp;",
        b'"' => b"&quot;",
        b'<' => b"&lt;",
        b'>' => b"&gt;",
        _ => core::slice::from_ref(byte),
    }
}
fn page(engine: &mut Engine, call: Token, status: u16, heading: &[u8], why: &[u8], back: &[u8]) {
    let parts = [
        b"<!doctype html><title>temper</title><h1>temper</h1><main><h2>".as_slice(),
        heading,
        b"</h2><p>",
        why,
        b"</p><a href=\"",
    ];
    let suffix = b"\">Back to temper</a></main>";
    let mut length = suffix.len();
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded page");
    }
    for byte in back {
        length = length.checked_add(escaped(byte).len()).expect("bounded escaped return");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured page");
    }
    for byte in back {
        writer.put(escaped(byte)).expect("measured return");
    }
    writer.put(suffix).expect("measured suffix");
    engine.clients.github_answer(call, status, None, &writer.finish());
}
pub(super) fn ended(engine: &mut Engine, call: Token, forge: u16) {
    page(
        engine,
        call,
        410,
        b"This link has ended",
        b"It was used already, it expired, or temper restarted since. Start again from temper.",
        &fallback(forge),
    );
}
fn route(engine: &mut Engine, call: Token, head: Call) {
    let (path, bytes) = split(&head.target);
    let Some(forge) = forge(path) else {
        engine.clients.github_answer(call, 404, None, b"Forge not found");
        return;
    };
    let mut configured = false;
    for config in &engine.workers.forges {
        if config.number == forge && config.kind == crate::settings::Kind::GitHub {
            configured = true;
            break;
        }
    }
    if !configured {
        engine.clients.github_answer(call, 404, None, b"Forge not found");
        return;
    }
    if head.method != Method::Get {
        engine.clients.github_answer(call, 405, None, b"Use GET");
        return;
    }
    let parsed = if bytes.len() <= usize::try_from(engine.clients.query_bytes).expect("query bound") {
        query(bytes)
    } else {
        None
    };
    let Some(query) = parsed else {
        page(engine, call, 400, b"This visit could not be read", b"Start again from temper.", &fallback(forge));
        return;
    };
    if query.action.as_ref() == b"request" {
        let Some(state) = query.state else {
            ended(engine, call, forge);
            return;
        };
        let Some(back) = super::public::take_back(engine, forge, &state) else {
            ended(engine, call, forge);
            return;
        };
        page(engine, call, 200, b"GitHub sent a request, not an installation", "Only an owner of an organization can install temper’s App there, and temper can’t wait for an owner to approve it yet: an installation approved later isn’t used. Install it on your own account, or on an organization you own.".as_bytes(), back.back.as_deref().unwrap_or(&fallback(forge)));
        return;
    }
    let installation = query.installation.expect("validated install or update");
    let Some(state) = query.state else {
        if super::lifecycle::accepted(engine, forge).contains(&installation.get()) {
            engine.clients.github_answer(call, 303, Some(b"/forges"), b"");
        } else {
            ended(engine, call, forge);
        }
        return;
    };
    // The host alone authenticates/consumes the state; this map holds presentation only.
    let back = match super::public::take_back(engine, forge, &state) {
        Some(return_to) => return_to.back,
        None => None,
    };
    if super::lifecycle::busy(engine, forge)
        || engine.setup.runs.len() == engine.setup.env.limits.runs
        || engine.setup.public.len() == engine.setup.public.capacity()
    {
        ended(engine, call, forge);
        return;
    }
    let run = super::lifecycle::next_run(&mut engine.setup.lifecycle);
    engine.setup.public.insert(run, super::public::Caller::Accept { forge, call, back }).expect("public room checked");
    engine.setup.staged.insert(run, host::Plan::Accept { forge, installation, state }).expect("staged room");
    super::request(engine, run);
}
pub(super) fn advance(engine: &mut Engine) {
    for _ in 0..engine.clients.github_calls.len() {
        if engine.events.room() < 4
            || !engine.setup.room()
            || !engine.store.room()
            || !engine.forge.room(2)
            || engine.notes.room() == 0
        {
            break;
        }
        let (call, head) = engine.clients.github_calls.pop().expect("route count");
        route(engine, call, head);
    }
}
