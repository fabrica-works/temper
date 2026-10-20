use super::fixtures;
use crate::{
    Action, Address, Event, Request,
    pages::{
        self,
        accounts::{self, Act, Operation, Selection, SignInState},
    },
};
use jig_client::{Down, contract as c};
use skein_lib::{List, Queue, Secret, Token};
const PROVIDER: u32 = 17;
const ACCOUNT: u32 = 29;
const SOURCE: u32 = 43;
const NAME: [u8; 16] = [97; 16];
fn event(root: &mut crate::Domain, event: Event) -> Queue<Request> {
    let limits = fixtures::limits();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(root, &fixtures::env(&limits), event, &mut out);
    crate::take(root, &fixtures::env(&limits), &mut out);
    out
}
fn act(root: &mut crate::Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::Accounts(act)) })
}
fn frame(role: c::HeldRole, party: u64) -> c::WatchEvent {
    let c::WatchEvent::Delivered(c::Delivery::Me(mut me)) = fixtures::frame(party) else { unreachable!() };
    me.deployment_role = Some(role);
    c::WatchEvent::Delivered(c::Delivery::Me(me))
}
fn opened(role: c::HeldRole) -> (crate::Domain, Token, Token) {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let (me, accounts) = fixtures::start_accounts(&mut root, List::with_capacity(0));
    deliver(&mut root, me, frame(role, fixtures::PARTY));
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    (root, me, accounts)
}
fn deliver(root: &mut crate::Domain, watch: Token, event: c::WatchEvent) -> Queue<Request> {
    self::event(root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Watched { watch, event } } })
}
fn document(signing: bool) -> c::WatchEvent {
    let mut rows = List::with_capacity(1);
    if signing {
        rows.push(c::OwnerAccountSignIn {
            source: SOURCE,
            provider: PROVIDER,
            holder: c::OwnerAccountHolder::Party(fixtures::PARTY),
        })
        .unwrap();
    }
    c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(c::OwnerAccounts {
        accounts: List::with_capacity(0),
        signing: rows,
        last_end: None,
        usable: false,
        settled: !signing,
        cut: false,
    })))
}
fn select(root: &mut crate::Domain, holder: c::AccountHolder) {
    act(
        root,
        Act::Select {
            selection: Selection { holder, provider: Some(PROVIDER), account: Some(ACCOUNT), source: Some(SOURCE) },
        },
    );
}
fn key(root: &mut crate::Domain, bytes: &[u8]) {
    event(root, Event::AccountKey { value: Secret::new(Box::from(bytes)) });
}
fn submit(root: &mut crate::Domain, operation: Operation) -> Queue<Request> {
    act(root, Act::Submit { name: NAME, operation })
}
type Sent = (Token, [u8; 16], jig_client::Ask, Option<Secret>);
fn sends(mut out: Queue<Request>) -> Queue<Sent> {
    let mut sent = Queue::with_capacity(4);
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Save { .. } } = &request {
            panic!("a value is never saved");
        }
        if let Request::Client {
            request: jig_client::Request::Down { down: Down::Send { attempt, key, request, value } },
        } = request
        {
            sent.push((attempt, key, request, value));
        }
    }
    sent
}
#[test]
fn accounts_path_and_global_unknown_lifecycle() {
    assert_eq!(crate::address::print(&Address::Accounts).as_ref(), b"/accounts");
    for path in [b"/accounts".as_slice(), b"/accounts/?x=1"] {
        assert_eq!(crate::address::parse(path), crate::Place::Address(Address::Accounts));
    }
    assert_eq!(crate::address::parse(b"/accounts/2"), crate::Place::Missing);
    let (mut root, _, watch) = opened(c::HeldRole::Member);
    assert_eq!(root.pages().availability(root.client()), None);
    deliver(&mut root, watch, document(false));
    assert_eq!(root.pages().availability(root.client()), Some(false));
    let c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(mut usable)) = document(false) else { unreachable!() };
    usable.usable = true;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(usable)));
    assert_eq!(root.pages().availability(root.client()), Some(true));
    deliver(&mut root, watch, document(false));
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Adopt { project: fixtures::PROJECT }) });
    assert_eq!(root.pages().availability(root.client()), Some(false));
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
        },
    );
    assert_eq!(root.client().document(accounts::handle()).unwrap().state, jig_client::State::Behind);
    assert_eq!(root.pages().availability(root.client()), None);
    event(&mut root, Event::SignedOut);
    assert_eq!(root.pages().availability(root.client()), None);
    assert!(accounts::document(root.client()).is_none());
    let mut opened =
        event(&mut root, Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    let mut me = None;
    let mut accounts = None;
    while let Some(request) = opened.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Open { watch, subject } } } = request
        {
            if subject == jig_client::Watchable::Jig(c::Subject::Me) {
                me = Some(watch);
            } else if subject == jig_client::Watchable::Jig(c::Subject::Accounts) {
                accounts = Some(watch);
            }
        }
    }
    deliver(&mut root, me.expect("me reopened"), frame(c::HeldRole::Member, fixtures::PARTY));
    deliver(&mut root, accounts.expect("accounts reopened"), document(false));
    assert_eq!(root.pages().availability(root.client()), Some(false));
    event(
        &mut root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } } },
    );
    assert_eq!(root.pages().availability(root.client()), None);
    assert!(accounts::document(root.client()).is_none());
}
#[test]
fn deployment_is_page_scoped_and_role_loss_and_party_change_clear_drafts() {
    let (mut root, me, _) = opened(c::HeldRole::Owner);
    assert!(root.pages().accounts().unwrap().deployment.is_some());
    select(&mut root, c::AccountHolder::Deployment);
    key(&mut root, b"private-draft");
    deliver(&mut root, me, frame(c::HeldRole::Member, fixtures::PARTY));
    let page = root.pages().accounts().unwrap();
    assert!(page.deployment.is_none());
    assert!(!page.has_key());
    select(&mut root, c::AccountHolder::Deployment);
    key(&mut root, b"private-draft");
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    deliver(&mut root, me, frame(c::HeldRole::Owner, fixtures::PARTY + 1));
    let page = root.pages().accounts().unwrap();
    assert!(!page.has_key());
    assert!(page.pending.is_none());
    key(&mut root, b"draft");
    event(&mut root, Event::Went { place: crate::Place::Missing });
    assert!(root.pages().accounts().is_none());
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    assert!(!root.pages().accounts().unwrap().has_key());
    key(&mut root, b"draft");
    event(&mut root, Event::SignedOut);
    assert!(root.pages().accounts().is_none());
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the test captures Debug to verify secret redaction")]
fn key_bounds_joined_press_and_same_name_retry_never_keep_or_leak_a_value() {
    let (mut root, _, _) = opened(c::HeldRole::Member);
    select(&mut root, c::AccountHolder::Own);
    for bytes in [b"".as_slice(), &[b'x'; 4097]] {
        key(&mut root, bytes);
        assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    }
    let drawn = root.pages().account_name();
    key(&mut root, b"private-once");
    assert_eq!(root.pages().account_name(), drawn, "the name is independent of the value");
    let mut sent = sends(submit(&mut root, Operation::AddKey));
    let (attempt, name, ask, value) = sent.pop().expect("one value send");
    assert_eq!(name, NAME);
    assert_eq!(value.unwrap().expose(), b"private-once");
    assert_eq!(
        ask,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::AddKey {
                provider: PROVIDER,
                join: Some(ACCOUNT),
                digest: skein_lib::sha256::digest(b"private-once")
            },
        }))
    );
    assert!(!root.pages().accounts().unwrap().has_key());
    assert!(!format!("{:?}", root.pages()).contains("private-once"));
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Lost,
                    answer: None,
                    why: None,
                },
            },
        },
    );
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    assert_eq!(root.pages().account_name(), NAME);
    key(&mut root, b"private-once");
    assert!(sends(act(&mut root, Act::Submit { name: [98; 16], operation: Operation::AddKey })).is_empty());
    key(&mut root, b"private-once");
    let retry = sends(submit(&mut root, Operation::AddKey)).pop().expect("explicit retry");
    assert_eq!(retry.1, NAME);
    assert_eq!(retry.2, ask);
    assert_eq!(retry.3.unwrap().expose(), b"private-once");
    let out = event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: retry.0,
                    envelope: skein_lib::RequestEnvelope::Again,
                    answer: None,
                    why: None,
                },
            },
        },
    );
    assert!(sends(out).is_empty());
    assert_eq!(root.pages().accounts().unwrap().pending.as_ref().unwrap().outcome, Some(jig_client::Outcome::Unsent));
}
#[test]
fn a_restored_sign_in_address_is_live_only_while_the_watch_names_its_source() {
    let (mut root, _, watch) = opened(c::HeldRole::Member);
    let pending = jig_client::Key { asker: pages::me_handle().asker, number: 991 };
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Kept {
            key: pending,
            request: jig_client::Ask::Jig(Box::new(c::Ask::SignIn {
                holder: c::AccountHolder::Own,
                provider: PROVIDER,
            })),
        },
        &mut Queue::with_capacity(crate::max_out(&fixtures::limits())),
    );
    started(&mut root, pending);
    deliver(&mut root, watch, document(true));
    let page = root.pages().accounts().unwrap();
    assert_eq!(page.sign_in_state(root.client()), SignInState::Signing { source: SOURCE });
    assert_eq!(page.sign_in_address(root.client()), Some(b"https://issuer/visit".as_slice()));
    let c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(mut cut)) = document(false) else { unreachable!() };
    cut.cut = true;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(cut)));
    assert_eq!(root.pages().accounts().unwrap().sign_in_state(root.client()), SignInState::Unknown);
    deliver(&mut root, watch, document(false));
    let page = root.pages().accounts().unwrap();
    assert_eq!(page.sign_in_state(root.client()), SignInState::Ended { source: SOURCE });
    assert_eq!(page.sign_in_address(root.client()), None);
    select(&mut root, c::AccountHolder::Own);
    submit(&mut root, Operation::SignIn);
    let key = root.pages().accounts().unwrap().pending.as_ref().unwrap().key;
    started(&mut root, key);
    let page = root.pages().accounts().unwrap();
    assert_eq!(page.sign_in_state(root.client()), SignInState::Started { source: SOURCE });
    assert!(page.sign_in_address(root.client()).is_some(), "an older watch does not end a new sign-in");
}
#[test]
fn every_supported_operation_preserves_holder_numbers_and_one_based_rank() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    let digest = skein_lib::sha256::digest(b"private-once");
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        for (operation, ask, confirms) in [
            (Operation::SignIn, c::Ask::SignIn { holder, provider: PROVIDER }, false),
            (Operation::SignOut, c::Ask::SignOut { holder, source: SOURCE }, true),
            (
                Operation::AddKey,
                change(holder, c::AccountChange::AddKey { provider: PROVIDER, join: Some(ACCOUNT), digest }),
                false,
            ),
            (Operation::ReplaceKey, change(holder, c::AccountChange::ReplaceKey { source: SOURCE, digest }), false),
            (Operation::RemoveKey, change(holder, c::AccountChange::RemoveKey { source: SOURCE }), true),
            (Operation::RemoveAccount, change(holder, c::AccountChange::Remove { account: ACCOUNT }), true),
            (Operation::Resume, change(holder, c::AccountChange::Resume { source: SOURCE }), false),
            (
                Operation::MoveAccount { place: 1 },
                change(holder, c::AccountChange::MoveAccount { account: ACCOUNT, place: 1 }),
                false,
            ),
            (
                Operation::MoveSource { place: 2 },
                change(holder, c::AccountChange::MoveSource { account: ACCOUNT, source: SOURCE, place: 2 }),
                false,
            ),
        ] {
            let mut page = accounts::Accounts::default();
            let mut out = Queue::with_capacity(8);
            let mut serial = 0;
            page.act(
                root.client(),
                Act::Select {
                    selection: Selection {
                        holder,
                        provider: Some(PROVIDER),
                        account: Some(ACCOUNT),
                        source: Some(SOURCE),
                    },
                },
                &mut serial,
                &mut out,
            );
            page.key(Secret::new(Box::from(b"private-once".as_slice())));
            page.act(root.client(), Act::Submit { name: NAME, operation }, &mut serial, &mut out);
            if confirms {
                assert!(out.is_empty());
                assert_eq!(page.confirm, Some((NAME, ask.clone())));
                page.act(root.client(), Act::Confirm, &mut serial, &mut out);
            }
            let request = out.pop().expect("one exact operation");
            let expected = jig_client::Ask::Jig(Box::new(ask));
            if let pages::Request::Client(jig_client::Event::Send { name, request, .. }) = request {
                assert_eq!(name, Some(NAME));
                assert_eq!(request, expected);
            } else if let pages::Request::Client(jig_client::Event::Carry { name, request, value, .. }) = request {
                assert_eq!(name, NAME);
                assert_eq!(request, expected);
                assert_eq!(value.expose(), b"private-once");
            } else {
                panic!("keyed operation");
            }
            assert!(out.is_empty());
            assert!(!page.has_key());
            page.act(root.client(), Act::Confirm, &mut serial, &mut out);
            assert!(out.is_empty());
        }
    }
    let mut page = accounts::Accounts::default();
    let mut out = Queue::with_capacity(4);
    for operation in [Operation::MoveAccount { place: 0 }, Operation::MoveSource { place: 0 }] {
        page.act(root.client(), Act::Submit { name: NAME, operation }, &mut 0, &mut out);
        assert_eq!(page.broken, Some(accounts::Broken::Rank));
        assert!(out.is_empty());
    }
    page.selection.provider = Some(PROVIDER);
    page.key(Secret::new(Box::new([b'x'; 4096])));
    page.act(root.client(), Act::Submit { name: NAME, operation: Operation::AddKey }, &mut 0, &mut out);
    let c::Ask::AccountChange { change: c::AccountChange::AddKey { join: None, .. }, .. } = page.pending.unwrap().ask
    else {
        panic!("key-only account");
    };
}

fn change(holder: c::AccountHolder, change: c::AccountChange) -> c::Ask {
    c::Ask::AccountChange { holder, change }
}
#[test]
fn another_partys_restored_account_work_is_not_shown() {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut kept = List::with_capacity(1);
    kept.push(skein_lib::RequestRecord {
        key: NAME,
        scope: fixtures::PARTY.checked_add(1).unwrap(),
        first_sent: Some(skein_lib::Wall::EPOCH),
        request: jig_client::Held {
            key: jig_client::Key { asker: pages::me_handle().asker, number: 991 },
            request: jig_client::Ask::Jig(Box::new(c::Ask::SignIn {
                holder: c::AccountHolder::Own,
                provider: PROVIDER,
            })),
        },
    })
    .unwrap();
    let (me, _) = fixtures::start_accounts(&mut root, kept);
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    assert!(root.pages().accounts().is_none());
    deliver(&mut root, me, frame(c::HeldRole::Member, fixtures::PARTY));
    assert!(root.pages().accounts().unwrap().pending.is_none());
}

fn started(root: &mut crate::Domain, pending: jig_client::Key) {
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Answered {
            key: pending,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountSignInStarted {
                    source: SOURCE,
                    address: Box::from(b"https://issuer/visit".as_slice()),
                },
            )))),
        },
        &mut Queue::with_capacity(crate::max_out(&fixtures::limits())),
    );
}
