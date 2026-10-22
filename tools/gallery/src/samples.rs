//! The promised names, in client/browser.md, 7.2's order.
use crate::driver::{limits, step};
use jig_client::contract as c;
use skein_lib::{Queue, Token};
use temper_client_domain::{Action, Address, Domain, Event, Request, address::Filter, pages::Act};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum World {
    Reference,
    Solo,
    FirstRun { step: u8 },
    Bare,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Then {
    Shown,
    Filled,
    Refused,
    SignedOut,
    OutOfDate,
    Closing,
    Rejecting,
    Held,
    Releasing,
    SigningIn,
    AccountRefused,
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub name: &'static str,
    pub world: World,
    pub address: Address,
    pub then: Then,
}
pub const N: usize = 28;
#[must_use]
pub fn samples() -> [Sample; N] {
    use Then::{
        AccountRefused, Closing, Filled, Held, OutOfDate, Refused, Rejecting, Releasing, Shown, SignedOut, SigningIn,
    };
    use World::{Bare, FirstRun, Reference, Solo};
    [
        Sample { name: "chats-solo", world: Solo, address: Address::Chats, then: Shown },
        Sample { name: "chat-solo-T10", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "inbox-solo", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "task-solo-T3", world: Solo, address: Address::Task { task: 3 }, then: Shown },
        Sample { name: "accounts-solo", world: Solo, address: Address::Accounts, then: Shown },
        Sample { name: "chats-first-run-1", world: FirstRun { step: 1 }, address: Address::Chats, then: Shown },
        Sample { name: "adoption", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "adoption-filled", world: Reference, address: Address::Adopt { project: 1 }, then: Filled },
        Sample { name: "adoption-refused", world: Reference, address: Address::Adopt { project: 1 }, then: Refused },
        Sample { name: "chats-reference", world: Reference, address: Address::Chats, then: Shown },
        Sample { name: "chat-reference-T1", world: Reference, address: Address::Chat { task: 1 }, then: Shown },
        Sample { name: "chat-solo-T6", world: Solo, address: Address::Chat { task: 6 }, then: Shown },
        Sample {
            name: "chat-first-run-2-T1",
            world: FirstRun { step: 2 },
            address: Address::Chat { task: 1 },
            then: Shown,
        },
        Sample {
            name: "chat-first-run-3-T1",
            world: FirstRun { step: 3 },
            address: Address::Chat { task: 1 },
            then: Shown,
        },
        Sample {
            name: "chat-first-run-4-T4",
            world: FirstRun { step: 4 },
            address: Address::Chat { task: 4 },
            then: Shown,
        },
        Sample {
            name: "inbox-reference",
            world: Reference,
            address: Address::Inbox { filter: Filter::ALL },
            then: Shown,
        },
        Sample { name: "accounts-reference", world: Reference, address: Address::Accounts, then: Shown },
        Sample { name: "accounts-first-run-1", world: FirstRun { step: 1 }, address: Address::Accounts, then: Shown },
        Sample { name: "signed-out", world: Bare, address: Address::Chats, then: SignedOut },
        Sample { name: "out-of-date", world: Bare, address: Address::Chats, then: OutOfDate },
        Sample { name: "chats-empty", world: Bare, address: Address::Chats, then: Shown },
        Sample { name: "inbox-empty", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "chat-closing", world: Solo, address: Address::Chat { task: 6 }, then: Closing },
        Sample { name: "chat-rejecting", world: Solo, address: Address::Chat { task: 10 }, then: Rejecting },
        Sample { name: "task-held", world: Solo, address: Address::Task { task: 3 }, then: Held },
        Sample { name: "task-releasing", world: Solo, address: Address::Task { task: 3 }, then: Releasing },
        Sample { name: "accounts-signin", world: Reference, address: Address::Accounts, then: SigningIn },
        Sample { name: "accounts-refused", world: Reference, address: Address::Accounts, then: AccountRefused },
    ]
}
#[must_use]
pub fn mockup(sample: &Sample) -> Option<String> {
    if sample.then != Then::Shown {
        return None;
    }
    let world = match sample.world {
        World::Reference => "reference".to_owned(),
        World::Solo => "solo".to_owned(),
        World::FirstRun { step } => format!("first-run&step={step}"),
        World::Bare => return None,
    };
    let (page, task) = match sample.address {
        Address::Chats => ("chats", None),
        Address::Chat { task } => ("chat", Some(task)),
        Address::Inbox { .. } => ("inbox", None),
        Address::Accounts => ("accounts", None),
        Address::Task { task } => ("task", Some(task)),
        Address::Adopt { .. } => return None,
    };
    let task = match task {
        Some(task) => format!("&task=T{task}"),
        None => String::new(),
    };
    Some(format!("{page}.html?world={world}{task}&bare"))
}
#[must_use]
pub fn domain(sample: &Sample) -> Domain {
    let mut root = crate::worlds::world(sample.world, sample.address.clone(), sample.then);
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits()));
    match sample.then {
        Then::Shown | Then::Held => {}
        Then::Filled | Then::Refused => adoption(&mut root, sample.then == Then::Refused, &mut out),
        Then::SignedOut => step(&mut root, Event::SignedOut, &mut out),
        Then::OutOfDate => step(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
            },
            &mut out,
        ),
        Then::Closing => step(&mut root, Event::Act { action: Action::Pages(Act::Close) }, &mut out),
        Then::Rejecting => step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Decide {
                    object: c::ObjectKey::Proposal(crate::worlds::PROPOSAL),
                    revision: None,
                    act: c::ActKind::Reject,
                }),
            },
            &mut out,
        ),
        Then::Releasing => step(
            &mut root,
            Event::Act {
                action: Action::Pages(Act::Decide {
                    object: c::ObjectKey::Task(3),
                    revision: Some(c::Revision { value: 7 }),
                    act: c::ActKind::Release,
                }),
            },
            &mut out,
        ),
        Then::SigningIn | Then::AccountRefused => account(&mut root, sample.then == Then::AccountRefused, &mut out),
    }
    root
}
fn sent_attempt(out: &mut Queue<Request>) -> Token {
    for _ in 0..out.len() {
        if let Some(request) = out.pop()
            && let Some(down) = crate::driver::down(request)
        {
            match down {
                jig_client::Down::Send { attempt, .. } => return attempt,
                jig_client::Down::Open { .. }
                | jig_client::Down::Close { .. }
                | jig_client::Down::Continue { .. }
                | jig_client::Down::Read { .. }
                | jig_client::Down::Save { .. }
                | jig_client::Down::Erase { .. } => {}
            }
        }
    }
    unreachable!("the sample's request was stored and sent")
}
fn adoption(root: &mut Domain, refused: bool, out: &mut Queue<Request>) {
    use temper_client_domain::forge::{self, Act as A, Text as T, Typed};
    for (field, value, typed) in [
        (T::Forge, "1", Typed::Forge(Some(1))),
        (T::Owner, "acme", Typed::Bytes),
        (T::Name, "shop", Typed::Bytes),
        (T::Prefix, "temper/", Typed::Bytes),
        (T::Landing, "main", Typed::Bytes),
        (T::Checks, "1, 2", Typed::Checks(Some(Box::new([1, 2])))),
    ] {
        step(
            root,
            Event::Act { action: Action::Forge(A::Write { field, text: Box::from(value.as_bytes()), typed }) },
            out,
        );
    }
    step(root, Event::Act { action: Action::Forge(A::Choose { role: forge::Role::Owned }) }, out);
    if refused {
        step(root, Event::Act { action: Action::Forge(A::Adopt) }, out);
        step(root, Event::Client { event: jig_client::Event::Stored }, out);
        let attempt = sent_attempt(out);
        step(
            root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                            refusal: c::Refusal::NotOffered,
                            at: None,
                        })),
                        why: None,
                    },
                },
            },
            out,
        );
    }
}
fn account(root: &mut Domain, refused: bool, out: &mut Queue<Request>) {
    use temper_client_domain::pages::accounts::{Act as A, Operation, Selection};
    step(
        root,
        Event::Act {
            action: Action::Pages(Act::Accounts(A::Select {
                selection: Selection { provider: Some(18), ..Default::default() },
            })),
        },
        out,
    );
    if refused {
        step(
            root,
            Event::AccountKey { value: skein_lib::Secret::new(Box::from(b"gallery-example-key".as_slice())) },
            out,
        );
    }
    step(
        root,
        Event::Act {
            action: Action::Pages(Act::Accounts(A::Submit {
                name: [97; 16],
                operation: if refused { Operation::AddKey } else { Operation::SignIn },
            })),
        },
        out,
    );
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let attempt = sent_attempt(out);
    // The tool branch's carried-key refusal: its final answer was lost, so
    // the account form explains that the key must be supplied again.
    let answer = if refused {
        None
    } else {
        Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted {
            source: 43,
            address: Box::from(b"https://issuer.example/visit/gallery-sign-in".as_slice()),
        }))))
    };
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: if refused {
                        skein_lib::RequestEnvelope::Lost
                    } else {
                        skein_lib::RequestEnvelope::Final
                    },
                    answer,
                    why: None,
                },
            },
        },
        out,
    );
}
