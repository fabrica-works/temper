//! The promised names, in client/browser.md, 7.2's order.
use crate::driver::{limits, step};
use jig_client::contract as c;
use skein_lib::{Queue, Token};
use temper_client_domain::{Action, Address, Domain, Event, Request, address::Filter, pages::Act};
use temper_client_domain::{ChatsFilter, PlanFilter};

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
    GoogleSignIn,
    OutOfDate,
    Closing,
    Rejecting,
    Held,
    Releasing,
    SigningIn,
    AccountRefused,
    KeyReady,
    SignInReady,
    Declined,
    Renewed,
    SignInRefused,
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub name: &'static str,
    pub world: World,
    pub address: Address,
    pub then: Then,
}
pub const N: usize = 46;
#[must_use]
fn task(task: u64) -> Address {
    Address::Task { task, filter: PlanFilter::All }
}

#[must_use]
pub fn samples() -> [Sample; N] {
    const CHATS: Address = Address::Chats { filter: ChatsFilter::All, start: false };
    use Then::{
        AccountRefused, Closing, Declined, Filled, Held, KeyReady, OutOfDate, Refused, Rejecting, Releasing, Renewed,
        Shown, SignInReady, SignInRefused, SignedOut, SigningIn,
    };
    use World::{Bare, FirstRun, Reference, Solo};
    [
        Sample { name: "chats-solo", world: Solo, address: CHATS, then: Shown },
        Sample { name: "chat-solo-T10", world: Solo, address: Address::Chat { task: 10 }, then: Shown },
        Sample { name: "inbox-solo", world: Solo, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "task-solo-T3", world: Solo, address: task(3), then: Shown },
        Sample { name: "accounts-solo", world: Solo, address: Address::Accounts, then: Shown },
        Sample {
            name: "welcome-first-run-0",
            world: FirstRun { step: 0 },
            address: Address::Welcome { step: temper_client_domain::Step::Welcome },
            then: Shown,
        },
        Sample { name: "chats-first-run-1", world: FirstRun { step: 1 }, address: CHATS, then: Shown },
        Sample {
            name: "connect-forge-first-run-6",
            world: FirstRun { step: 6 },
            address: Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(3)) },
            then: Shown,
        },
        Sample { name: "forges-first-run-7", world: FirstRun { step: 7 }, address: Address::Forges, then: Shown },
        Sample {
            name: "connect-forge-choose",
            world: FirstRun { step: 0 },
            address: Address::ConnectForge { forge: None },
            then: Shown,
        },
        Sample { name: "chats-setup-strip", world: Bare, address: CHATS, then: Shown },
        Sample { name: "adoption", world: Reference, address: Address::Adopt { project: 1 }, then: Shown },
        Sample { name: "adoption-filled", world: Reference, address: Address::Adopt { project: 1 }, then: Filled },
        Sample { name: "adoption-refused", world: Reference, address: Address::Adopt { project: 1 }, then: Refused },
        Sample { name: "chats-reference", world: Reference, address: CHATS, then: Shown },
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
        Sample { name: "google-sign-in", world: Bare, address: CHATS, then: Then::GoogleSignIn },
        Sample { name: "signed-out", world: Bare, address: CHATS, then: SignedOut },
        Sample { name: "out-of-date", world: Bare, address: CHATS, then: OutOfDate },
        Sample { name: "chats-empty", world: Bare, address: CHATS, then: Shown },
        Sample { name: "inbox-empty", world: Bare, address: Address::Inbox { filter: Filter::ALL }, then: Shown },
        Sample { name: "chat-closing", world: Solo, address: Address::Chat { task: 6 }, then: Closing },
        Sample { name: "chat-rejecting", world: Solo, address: Address::Chat { task: 10 }, then: Rejecting },
        Sample { name: "task-held", world: Solo, address: task(3), then: Held },
        Sample { name: "task-releasing", world: Solo, address: task(3), then: Releasing },
        Sample {
            name: "connect-model-choose",
            world: Reference,
            address: Address::ConnectModel { way: None },
            then: Shown,
        },
        model_sample("connect-model-key-details", temper_client_domain::WayKind::Key, Shown),
        model_sample("connect-model-program-details", temper_client_domain::WayKind::Program, Shown),
        model_sample("connect-model-signing-in", temper_client_domain::WayKind::SignIn, SigningIn),
        model_sample("connect-model-key-ready", temper_client_domain::WayKind::Key, KeyReady),
        model_sample("connect-model-signin-ready", temper_client_domain::WayKind::SignIn, SignInReady),
        model_sample("connect-model-declined", temper_client_domain::WayKind::SignIn, Declined),
        model_sample("connect-model-renewed", temper_client_domain::WayKind::SignIn, Renewed),
        model_sample("connect-model-refused", temper_client_domain::WayKind::SignIn, SignInRefused),
        model_sample("connect-model-key-lost", temper_client_domain::WayKind::Key, AccountRefused),
        Sample { name: "task-reference-T28", world: Reference, address: task(28), then: Shown },
        Sample { name: "task-reference-T24", world: Reference, address: task(24), then: Shown },
        Sample { name: "task-reference-T12", world: Reference, address: task(12), then: Shown },
        Sample { name: "task-solo-T9", world: Solo, address: task(9), then: Shown },
    ]
}
fn model_sample(name: &'static str, kind: temper_client_domain::WayKind, then: Then) -> Sample {
    let way = temper_client_domain::ModelWay {
        source: None,
        provider: if kind == temper_client_domain::WayKind::Program { 17 } else { 18 },
        kind,
    };
    Sample { name, world: World::Reference, address: Address::ConnectModel { way: Some(way) }, then }
}
#[must_use]
pub fn mockup(sample: &Sample) -> Option<String> {
    if sample.then == Then::GoogleSignIn {
        return Some("sign-in.html?world=google-sign-in&bare".to_owned());
    }
    if sample.then != Then::Shown {
        if !matches!(sample.address, Address::ConnectModel { .. }) {
            return None;
        }
        // The mockup draws these endings in the first run after ChatGPT is connected.
        // It draws no sign-in under way, key Ready, or lost key answer.
        let anchor = match sample.then {
            Then::SignInReady => "",
            Then::Declined => "#other-states-declined",
            Then::Renewed => "#other-states-renewed",
            Then::SignInRefused => "#other-states-refused",
            Then::Shown
            | Then::Filled
            | Then::Refused
            | Then::SignedOut
            | Then::GoogleSignIn
            | Then::OutOfDate
            | Then::Closing
            | Then::Rejecting
            | Then::Held
            | Then::Releasing
            | Then::SigningIn
            | Then::AccountRefused
            | Then::KeyReady => return None,
        };
        return Some(format!("connect-model.html?world=first-run&step=5&way=chatgpt&bare{anchor}"));
    }
    let world = match sample.world {
        World::Reference => "reference".to_owned(),
        World::Solo => "solo".to_owned(),
        World::FirstRun { step } => format!("first-run&step={step}"),
        World::Bare => return None,
    };
    let (page, task) = match sample.address {
        Address::Chats { .. } => ("chats", None),
        Address::Chat { task } => ("chat", Some(task)),
        Address::Inbox { .. } => ("inbox", None),
        Address::Accounts => ("accounts", None),
        Address::ConnectModel { .. } => ("connect-model", None),
        Address::Welcome { .. } => ("welcome", None),
        Address::Task { task, .. } => ("task", Some(task)),
        Address::ConnectForge { .. } => ("connect-forge", None),
        Address::Forges => ("forges", None),
        Address::Adopt { .. } => return None,
    };
    let task = match task {
        Some(task) => format!("&task=T{task}"),
        None => String::new(),
    };
    let forge = match sample.address {
        Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(number)) } => {
            format!("&forge={number}")
        }
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Forges
        | Address::ConnectForge { forge: None | Some(temper_client_domain::Choice::Elsewhere) }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => String::new(),
    };
    let way = if let Address::ConnectModel { way: Some(way) } = sample.address {
        match way.kind {
            temper_client_domain::WayKind::SignIn => "&way=chatgpt",
            temper_client_domain::WayKind::Key => "&way=key",
            temper_client_domain::WayKind::Program => "&way=program",
        }
    } else {
        ""
    };
    Some(format!("{page}.html?world={world}{task}{forge}{way}&bare"))
}
#[must_use]
pub fn domain(sample: &Sample) -> Domain {
    let mut root = crate::worlds::world(sample.world, sample.address.clone(), sample.then);
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits()));
    match sample.then {
        Then::Shown | Then::Held => {}
        Then::Filled | Then::Refused => adoption(&mut root, sample.then == Then::Refused, &mut out),
        Then::SignedOut | Then::GoogleSignIn => step(&mut root, Event::SignedOut, &mut out),
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
        Then::SigningIn
        | Then::AccountRefused
        | Then::KeyReady
        | Then::SignInReady
        | Then::Declined
        | Then::Renewed
        | Then::SignInRefused => account(&mut root, sample.then, &mut out),
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
fn account(root: &mut Domain, then: Then, out: &mut Queue<Request>) {
    use temper_client_domain::pages::{accounts::Way, connect_model::Act as A};
    let refused = then == Then::AccountRefused;
    let key = refused || then == Then::KeyReady;
    let way = if key { Way::Key { provider: 18 } } else { Way::SignIn { provider: 18, plan: true } };
    step(
        root,
        Event::Went {
            place: temper_client_domain::Place::Address(Address::ConnectModel {
                way: Some(temper_client_domain::pages::connect_model::model_way(way)),
            }),
        },
        out,
    );
    if !key {
        step(root, Event::Act { action: Action::Pages(Act::ConnectModel(A::SignIn { name: [97; 16], way })) }, out);
    }
    if key {
        step(
            root,
            Event::AccountKey { value: skein_lib::Secret::new(Box::from(b"gallery-example-key".as_slice())) },
            out,
        );
    }
    if key {
        step(root, Event::Act { action: Action::Pages(Act::ConnectModel(A::Submit { name: [97; 16] })) }, out);
    }
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let attempt = sent_attempt(out);
    // The tool branch's carried-key refusal: its final answer was lost, so
    // the account form explains that the key must be supplied again.
    let answer = if refused {
        None
    } else if key {
        Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept {
            source: 3,
        }))))
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
