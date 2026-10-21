//! Area 5 translates supplied holder/source facts; the engine owns account policy.
use super::{Called, Out, Runner, acts, reads};
use crate::jig::{
    Limits,
    call::{Call, Value, number, value},
};
use jig_client::{Client, contract as c};
use skein_lib::{Env, Queue};

pub(super) fn start(runner: &mut Runner, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.call.tool == 32 {
        let Some(me) = acts::me(runner, client) else { return };
        let request = declare(&called.call, me);
        acts::prepared(runner, client, env, called, request, out);
        return;
    }
    let ask = match called.call.tool {
        24 => acts::Request::Ask(c::Ask::SignIn { holder: c::AccountHolder::Own, provider: integer(&called.call, 0) }),
        26 => acts::Request::AddKey {
            holder: if value(&called.call, 3).is_some() { c::AccountHolder::Deployment } else { c::AccountHolder::Own },
            provider: integer(&called.call, 0),
            join: join(&called.call),
        },
        _ => unreachable!("direct account act"),
    };
    acts::prepared(runner, client, env, called, Ok((ask, None)), out);
}
fn integer(call: &Call, slot: usize) -> u32 {
    u32::try_from(number(call, slot).expect("parsed number")).expect("checked account number")
}
pub(super) fn resolved(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    reading: reads::Reading,
    out: &mut Queue<Out>,
) {
    let request = match build(&reading) {
        Ok(request) => Ok((request, None)),
        Err(why) => Err(acts::Failed::Fixed(why)),
    };
    acts::prepared(runner, client, env, reading.called, request, out);
}
fn build(reading: &reads::Reading) -> Result<acts::Request, &'static [u8]> {
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
            c::Delivery::OwnerProject(_)
            | c::Delivery::Me(_)
            | c::Delivery::Task(_)
            | c::Delivery::OwnerProposal(_)
            | c::Delivery::Tree(_)
            | c::Delivery::OwnerTree(_)
            | c::Delivery::Chips(_)
            | c::Delivery::Inbox(_)
            | c::Delivery::OwnerInbox(_)
            | c::Delivery::Tasks(_)
            | c::Delivery::Run(_)
            | c::Delivery::RunState(_)
            | c::Delivery::Deployment(_) => {}
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
        c::AccountChange::MoveAccount { account, place: integer(call, 1).checked_sub(1).expect("one-based position") }
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
            return Ok(acts::Request::Ask(c::Ask::SignOut { holder, source: source.number }));
        }
        match call.tool {
            33 => {
                if source.kind != c::OwnerAccountKind::Program {
                    return Err(b"verify needs a program source; sign-in and key sources have no host login to check");
                }
                return Ok(acts::Request::CheckProgram { holder, source: source.number });
            }
            27 => return Ok(acts::Request::ReplaceKey { holder, source: source.number }),
            28 => c::AccountChange::RemoveKey { source: source.number },
            29 => c::AccountChange::MoveSource {
                account,
                source: source.number,
                place: integer(call, 1).checked_sub(1).expect("one-based position"),
            },
            30 => c::AccountChange::Resume { source: source.number },
            _ => unreachable!("account act"),
        }
    };
    Ok(acts::Request::Ask(c::Ask::AccountChange { holder, change }))
}
fn join(call: &Call) -> Option<u32> {
    Some(u32::try_from(number(call, 2)?).expect("checked number"))
}

fn declare(call: &Call, me: &c::Me) -> Result<(acts::Request, Option<c::Recipient>), acts::Failed> {
    let Some(Value::Text(adapter)) = value(call, 1) else { unreachable!("parsed adapter") };
    let Some(Value::Text(name)) = value(call, 2) else { unreachable!("parsed program name") };
    let provider = integer(call, 0);
    let mut offered = false;
    let mut cut = me.providers_cut;
    for offer in &me.providers {
        if offer.provider == provider {
            cut |= offer.programs_cut || offer.name.cut;
            for program in &offer.programs {
                offered |= program.bytes() == adapter.as_ref();
            }
        }
    }
    if !offered {
        if cut {
            return Err(acts::Failed::Fixed(b"provider offers cut in me; adapter unchecked; read me again"));
        }
        return Err(acts::Failed::Fixed(b"adapter not offered by this provider in me; read me again"));
    }
    // DeclareProgram carries no adapter: this validates the offer locally, without binding one in the ask.
    Ok((
        acts::Request::DeclareProgram {
            holder: c::AccountHolder::Own,
            provider,
            join: match number(call, 3) {
                Some(n) => Some(u32::try_from(n).expect("checked account")),
                None => None,
            },
            name: name.clone(),
        },
        None,
    ))
}
