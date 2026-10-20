//! Area 5 translates supplied holder/source facts; the engine owns account policy.
use super::{Called, Out, Runner, acts, finish, reads};
use crate::jig::{
    Limits,
    call::{Call, Value, number, value},
    render,
};
use jig_client::{Client, contract as c};
use skein_lib::{Env, Queue};

pub(super) fn start(runner: &mut Runner, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    let ask = match called.call.tool {
        24 => Ok(c::Ask::SignIn { holder: c::AccountHolder::Own, provider: integer(&called.call, 0) }),
        26 => match digest(&called) {
            Err(why) => Err(why),
            Ok(digest) => Ok(c::Ask::AccountChange {
                holder: if value(&called.call, 3).is_some() {
                    c::AccountHolder::Deployment
                } else {
                    c::AccountHolder::Own
                },
                change: c::AccountChange::AddKey {
                    provider: integer(&called.call, 0),
                    join: match number(&called.call, 2) {
                        Some(n) => Some(u32::try_from(n).expect("checked number")),
                        None => None,
                    },
                    digest,
                },
            }),
        },
        _ => unreachable!("direct account act"),
    };
    send(runner, client, env, called, ask, out);
}
fn integer(call: &Call, slot: usize) -> u32 {
    u32::try_from(number(call, slot).expect("parsed number")).expect("checked account number")
}
fn digest(called: &Called) -> Result<[u8; 32], &'static [u8]> {
    let value = called.value.as_ref().ok_or(b"key file must be supplied again".as_slice())?;
    Ok(skein_lib::sha256::digest(value.expose()))
}
pub(super) fn resolved(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    reading: reads::Reading,
    out: &mut Queue<Out>,
) {
    let ask = build(&reading);
    send(runner, client, env, reading.called, ask, out);
}
fn build(reading: &reads::Reading) -> Result<c::Ask, &'static [u8]> {
    let call = &reading.called.call;
    let account = integer(call, 0);
    let mut found = None;
    for delivery in reading.documents.values() {
        match delivery {
            c::Delivery::OwnerAccounts(doc) => {
                for row in &doc.accounts {
                    if row.number == account {
                        if found.is_some() {
                            return Err(b"account number is ambiguous; read accounts again");
                        }
                        found = Some((row.holder, &row.sources, row.cut));
                    }
                }
            }
            c::Delivery::OwnerDeployment(doc) => {
                for row in &doc.accounts {
                    if row.number == account {
                        if found.is_some() {
                            return Err(b"account number is ambiguous; read accounts again");
                        }
                        found = Some((row.holder, &row.sources, row.cut));
                    }
                }
            }
            _ => {}
        }
    }
    let (holder, sources, cut) = found.ok_or(b"account unavailable; read accounts again".as_slice())?;
    let holder = match holder {
        c::OwnerAccountHolder::Party(_) => c::AccountHolder::Own,
        c::OwnerAccountHolder::Deployment => c::AccountHolder::Deployment,
    };
    let source_slot = if call.tool == 29 { 2 } else { 1 };
    let selected = if call.tool == 27 {
        Some(1)
    } else {
        match value(call, source_slot) {
            Some(Value::Word(kind)) => Some(*kind),
            _ => None,
        }
    };
    let change = if call.tool == 28 && selected.is_none() {
        c::AccountChange::Remove { account }
    } else if call.tool == 29 && selected.is_none() {
        c::AccountChange::MoveAccount { account, place: integer(call, 1) }
    } else {
        if cut {
            return Err(b"source list is incomplete; read accounts again");
        }
        let mut source = None;
        for candidate in sources {
            let kind = match candidate.kind {
                c::OwnerAccountKind::SignIn => 0,
                c::OwnerAccountKind::Key => 1,
                c::OwnerAccountKind::Program => 2,
                c::OwnerAccountKind::None => 3,
            };
            if selected.is_none() || selected == Some(kind) {
                if source.is_some() {
                    return Err(b"source is ambiguous; name one unique --source kind");
                }
                source = Some(candidate);
            }
        }
        let source = source.ok_or(b"source unavailable; read accounts again".as_slice())?;
        if call.tool == 25 || call.tool == 28 && source.kind != c::OwnerAccountKind::Key {
            return Ok(c::Ask::SignOut { holder, source: source.number });
        }
        match call.tool {
            27 => c::AccountChange::ReplaceKey { source: source.number, digest: digest(&reading.called)? },
            28 => c::AccountChange::RemoveKey { source: source.number },
            29 => c::AccountChange::MoveSource { account, source: source.number, place: integer(call, 1) },
            30 => c::AccountChange::Resume { source: source.number },
            _ => unreachable!("account act"),
        }
    };
    Ok(c::Ask::AccountChange { holder, change })
}
fn send(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    called: Called,
    ask: Result<c::Ask, &'static [u8]>,
    out: &mut Queue<Out>,
) {
    match ask {
        Ok(ask) => acts::prepared(runner, client, env, called, Ok((ask, None)), out),
        Err(why) => {
            let printed = render::failure(why, called.json, 1);
            finish(called, printed, out);
        }
    }
}
