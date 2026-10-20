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
        readiness: List::with_capacity(0),
        readiness_cut: false,
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
fn accounts_address_prints_the_canonical_accounts_page_path() {
    assert_eq!(crate::address::print(&Address::Accounts).as_ref(), b"/accounts");
}
#[test]
fn accounts_addresses_parse_the_supported_accounts_page_forms() {
    for path in [b"/accounts".as_slice(), b"/accounts/?x=1"] {
        assert_eq!(crate::address::parse(path), crate::Place::Address(Address::Accounts));
    }
}
#[test]
fn accounts_addresses_refuse_a_path_with_an_extra_segment() {
    assert_eq!(crate::address::parse(b"/accounts/2"), crate::Place::Missing);
}
#[test]
fn account_availability_is_unknown_before_the_first_account_delivery() {
    let (root, _, _) = opened(c::HeldRole::Member);
    assert_eq!(root.pages().availability(root.client()), None);
}
#[test]
fn account_availability_reports_the_delivered_usable_account_projection() {
    let (mut root, _, watch) = opened(c::HeldRole::Member);
    let c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(mut usable)) = document(false) else { unreachable!() };
    usable.usable = true;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(usable)));
    assert_eq!(root.pages().availability(root.client()), Some(true));
}
#[test]
fn account_availability_reports_unusable_accounts_outside_the_accounts_page() {
    let (mut root, _, _) = current_accounts(c::HeldRole::Member);
    assert_eq!(root.pages().availability(root.client()), Some(false));
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Adopt { project: fixtures::PROJECT }) });
    assert_eq!(root.pages().availability(root.client()), Some(false));
}
#[test]
fn account_availability_becomes_unknown_when_its_watch_is_lost() {
    let (mut root, _, watch) = current_accounts(c::HeldRole::Member);
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
        },
    );
    assert_eq!(root.client().document(accounts::handle()).unwrap().state, jig_client::State::Behind);
    assert_eq!(root.pages().availability(root.client()), None);
}
#[test]
fn account_availability_forgets_its_document_when_the_person_signs_out() {
    let (mut root, _, _) = current_accounts(c::HeldRole::Member);
    event(&mut root, Event::SignedOut);
    assert_eq!(root.pages().availability(root.client()), None);
    assert!(accounts::document(root.client()).is_none());
}
#[test]
fn account_availability_returns_after_signing_in_with_fresh_deliveries() {
    let (mut root, _, _) = current_accounts(c::HeldRole::Member);
    event(&mut root, Event::SignedOut);
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
}
#[test]
fn account_availability_forgets_its_document_when_service_is_withdrawn() {
    let (mut root, _, _) = current_accounts(c::HeldRole::Member);
    event(
        &mut root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } } },
    );
    assert_eq!(root.pages().availability(root.client()), None);
    assert!(accounts::document(root.client()).is_none());
}
#[test]
fn owners_see_deployment_accounts_on_the_open_accounts_page() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    assert!(root.pages().accounts().unwrap().deployment.is_some());
}
#[test]
fn losing_the_owner_role_clears_the_deployment_key_draft() {
    let (mut root, me) = deployment_draft();
    deliver(&mut root, me, frame(c::HeldRole::Member, fixtures::PARTY));
    let page = root.pages().accounts().unwrap();
    assert!(page.deployment.is_none());
    assert!(!page.has_key());
}
#[test]
fn members_cannot_submit_a_key_for_the_deployment_holder() {
    let (mut root, _, _) = opened(c::HeldRole::Member);
    select(&mut root, c::AccountHolder::Deployment);
    key(&mut root, b"private-draft");
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
}
#[test]
fn changing_the_party_clears_the_deployment_account_draft() {
    let (mut root, me) = deployment_draft();
    deliver(&mut root, me, frame(c::HeldRole::Member, fixtures::PARTY));
    select(&mut root, c::AccountHolder::Deployment);
    key(&mut root, b"private-draft");
    submit(&mut root, Operation::AddKey);
    deliver(&mut root, me, frame(c::HeldRole::Owner, fixtures::PARTY + 1));
    let page = root.pages().accounts().unwrap();
    assert!(!page.has_key());
    assert!(page.pending.is_none());
}
#[test]
fn leaving_the_accounts_page_discards_its_private_key_draft() {
    let (mut root, _) = deployment_draft();
    event(&mut root, Event::Went { place: crate::Place::Missing });
    assert!(root.pages().accounts().is_none());
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    assert!(!root.pages().accounts().unwrap().has_key());
}
#[test]
fn signing_out_closes_the_accounts_page_with_its_draft() {
    let (mut root, _) = deployment_draft();
    event(&mut root, Event::SignedOut);
    assert!(root.pages().accounts().is_none());
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the test captures Debug to verify secret redaction")]
fn adding_a_joined_key_sends_its_value_once_privately() {
    let mut root = own_accounts();
    let drawn = root.pages().account_name();
    key(&mut root, b"private-once");
    assert_eq!(root.pages().account_name(), drawn, "the name is independent of the value");
    let mut sent = sends(submit(&mut root, Operation::AddKey));
    let (_, name, ask, value) = sent.pop().expect("one value send");
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
}
#[test]
fn account_keys_outside_the_value_bounds_are_never_sent() {
    let mut root = own_accounts();
    for bytes in [b"".as_slice(), &[b'x'; 4097]] {
        key(&mut root, bytes);
        assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    }
}
#[test]
fn a_pending_joined_key_does_not_send_a_second_press() {
    let (mut root, _) = submitted_key();
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
}
#[test]
fn a_lost_key_send_requires_the_value_to_be_reentered() {
    let (mut root, _) = lost_key();
    assert!(sends(submit(&mut root, Operation::AddKey)).is_empty());
    assert_eq!(root.pages().account_name(), NAME);
}
#[test]
fn a_lost_key_send_refuses_a_different_retry_name() {
    let (mut root, _) = lost_key();
    key(&mut root, b"private-once");
    assert!(sends(act(&mut root, Act::Submit { name: [98; 16], operation: Operation::AddKey })).is_empty());
}
#[test]
fn a_same_name_key_retry_preserves_the_original_request() {
    let (mut root, ask) = lost_key();
    key(&mut root, b"private-once");
    let retry = sends(submit(&mut root, Operation::AddKey)).pop().expect("explicit retry");
    assert_eq!(retry.1, NAME);
    assert_eq!(retry.2, ask);
    assert_eq!(retry.3.unwrap().expose(), b"private-once");
}
#[test]
fn an_again_answer_leaves_the_key_retry_unsent() {
    let (mut root, _) = lost_key();
    key(&mut root, b"private-once");
    let retry = sends(submit(&mut root, Operation::AddKey)).pop().expect("explicit retry");
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
fn account_operations_send_the_selected_holder_numbers_without_confirmation() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        for (operation, ask) in [
            (Operation::SignIn, c::Ask::SignIn { holder, provider: PROVIDER }),
            (Operation::Resume, change(holder, c::AccountChange::Resume { source: SOURCE })),
            (
                Operation::MoveAccount { place: 1 },
                change(holder, c::AccountChange::MoveAccount { account: ACCOUNT, place: 1 }),
            ),
            (
                Operation::MoveSource { place: 2 },
                change(holder, c::AccountChange::MoveSource { account: ACCOUNT, source: SOURCE, place: 2 }),
            ),
        ] {
            let (mut page, mut serial, mut out) = operation_page(root.client(), holder);
            page.act(root.client(), Act::Submit { name: NAME, operation }, &mut serial, &mut out);
            assert_operation_send(&mut out, ask);
            assert_finished_operation(&mut page, root.client(), &mut serial, &mut out);
        }
    }
}
#[test]
fn destructive_account_operations_wait_for_confirmation_before_sending() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        for (operation, ask) in [
            (Operation::SignOut, c::Ask::SignOut { holder, source: SOURCE }),
            (Operation::RemoveKey, change(holder, c::AccountChange::RemoveKey { source: SOURCE })),
            (Operation::RemoveAccount, change(holder, c::AccountChange::Remove { account: ACCOUNT })),
        ] {
            let (mut page, mut serial, mut out) = operation_page(root.client(), holder);
            page.act(root.client(), Act::Submit { name: NAME, operation }, &mut serial, &mut out);
            assert!(out.is_empty());
            assert_eq!(page.confirm, Some((NAME, ask.clone())));
            page.act(root.client(), Act::Confirm, &mut serial, &mut out);
            assert_operation_send(&mut out, ask);
            assert_finished_operation(&mut page, root.client(), &mut serial, &mut out);
        }
    }
}
#[test]
fn key_operations_carry_the_selected_holder_numbers_with_the_value() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    let digest = skein_lib::sha256::digest(b"private-once");
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        for (operation, ask) in [
            (
                Operation::AddKey,
                change(holder, c::AccountChange::AddKey { provider: PROVIDER, join: Some(ACCOUNT), digest }),
            ),
            (Operation::ReplaceKey, change(holder, c::AccountChange::ReplaceKey { source: SOURCE, digest })),
        ] {
            let (mut page, mut serial, mut out) = operation_page(root.client(), holder);
            page.act(root.client(), Act::Submit { name: NAME, operation }, &mut serial, &mut out);
            let request = out.pop().expect("one exact operation");
            let pages::Request::Client(jig_client::Event::Carry { name, request, value, .. }) = request else {
                panic!("keyed operation");
            };
            assert_eq!(name, NAME);
            assert_eq!(request, jig_client::Ask::Jig(Box::new(ask)));
            assert_eq!(value.expose(), b"private-once");
            assert_finished_operation(&mut page, root.client(), &mut serial, &mut out);
        }
    }
}
#[test]
fn account_move_operations_refuse_a_zero_based_rank() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    for operation in [Operation::MoveAccount { place: 0 }, Operation::MoveSource { place: 0 }] {
        let mut page = accounts::Accounts::default();
        let mut out = Queue::with_capacity(4);
        page.act(root.client(), Act::Submit { name: NAME, operation }, &mut 0, &mut out);
        assert_eq!(page.broken, Some(accounts::Broken::Rank));
        assert!(out.is_empty());
    }
}
#[test]
fn a_key_at_capacity_can_create_a_standalone_account() {
    let (root, _, _) = opened(c::HeldRole::Owner);
    let mut page = accounts::Accounts::default();
    let mut out = Queue::with_capacity(4);
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

fn current_accounts(role: c::HeldRole) -> (crate::Domain, Token, Token) {
    let (mut root, me, watch) = opened(role);
    deliver(&mut root, watch, document(false));
    (root, me, watch)
}
fn deployment_draft() -> (crate::Domain, Token) {
    let (mut root, me, _) = opened(c::HeldRole::Owner);
    select(&mut root, c::AccountHolder::Deployment);
    key(&mut root, b"private-draft");
    (root, me)
}
fn own_accounts() -> crate::Domain {
    let (mut root, _, _) = opened(c::HeldRole::Member);
    select(&mut root, c::AccountHolder::Own);
    root
}
fn submitted_key() -> (crate::Domain, Sent) {
    let mut root = own_accounts();
    key(&mut root, b"private-once");
    let sent = sends(submit(&mut root, Operation::AddKey)).pop().expect("one value send");
    (root, sent)
}
fn lost_key() -> (crate::Domain, jig_client::Ask) {
    let (mut root, (attempt, _, ask, _)) = submitted_key();
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
    (root, ask)
}
fn operation_page(
    client: &jig_client::Client,
    holder: c::AccountHolder,
) -> (accounts::Accounts, u64, Queue<pages::Request>) {
    let mut page = accounts::Accounts::default();
    let mut out = Queue::with_capacity(8);
    let mut serial = 0;
    page.act(
        client,
        Act::Select {
            selection: Selection { holder, provider: Some(PROVIDER), account: Some(ACCOUNT), source: Some(SOURCE) },
        },
        &mut serial,
        &mut out,
    );
    page.key(Secret::new(Box::from(b"private-once".as_slice())));
    (page, serial, out)
}
fn assert_operation_send(out: &mut Queue<pages::Request>, ask: c::Ask) {
    let request = out.pop().expect("one exact operation");
    let pages::Request::Client(jig_client::Event::Send { name, request, .. }) = request else {
        panic!("keyed operation");
    };
    assert_eq!(name, Some(NAME));
    assert_eq!(request, jig_client::Ask::Jig(Box::new(ask)));
}
fn assert_finished_operation(
    page: &mut accounts::Accounts,
    client: &jig_client::Client,
    serial: &mut u64,
    out: &mut Queue<pages::Request>,
) {
    assert!(out.is_empty());
    assert!(!page.has_key());
    page.act(client, Act::Confirm, serial, out);
    assert!(out.is_empty());
}

#[test]
fn cancel_abandons_a_named_lost_account_act() {
    for envelope in [skein_lib::RequestEnvelope::Lost, skein_lib::RequestEnvelope::Again] {
        let (mut root, _, _) = opened(c::HeldRole::Member);
        select(&mut root, c::AccountHolder::Own);
        key(&mut root, b"first-key");
        let attempt = sends(submit(&mut root, Operation::AddKey)).pop().unwrap().0;
        event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer { attempt, envelope, answer: None, why: None },
                },
            },
        );
        act(&mut root, Act::Cancel);
        assert!(root.pages().accounts().unwrap().pending.is_none());
        key(&mut root, b"another-key");
        assert_eq!(sends(act(&mut root, Act::Submit { name: [99; 16], operation: Operation::AddKey })).len(), 1);
    }
}
#[test]
fn cancel_abandons_a_restored_lost_account_act() {
    for outcome in [jig_client::Outcome::Lost, jig_client::Outcome::Unsent] {
        let (mut root, _, _) = opened(c::HeldRole::Member);
        let key = jig_client::Key { asker: pages::me_handle().asker, number: 991 };
        let mut out = Queue::with_capacity(crate::max_out(&fixtures::limits()));
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Kept {
                key,
                request: jig_client::Ask::Jig(Box::new(c::Ask::SignIn {
                    holder: c::AccountHolder::Own,
                    provider: PROVIDER,
                })),
            },
            &mut out,
        );
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Answered { key, outcome },
            &mut out,
        );
        assert_eq!(root.pages().accounts().unwrap().pending.as_ref().unwrap().name, None);
        act(&mut root, Act::Cancel);
        select(&mut root, c::AccountHolder::Own);
        assert_eq!(sends(submit(&mut root, Operation::Resume)).len(), 1);
    }
}
#[test]
fn typed_account_lists_outlive_the_task_window() {
    let (mut root, _, watch) = opened(c::HeldRole::Member);
    let count = fixtures::limits().client.domain.rows.checked_add(1).unwrap();
    let mut rows = List::with_capacity(count);
    let mut signing = List::with_capacity(count);
    for number in 1..=count {
        let mut sources = List::with_capacity(count);
        for source in 1..=count {
            sources
                .push(c::OwnerAccountSource {
                    place: source,
                    number: source,
                    kind: c::OwnerAccountKind::Key,
                    state: c::OwnerAccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .unwrap();
        }
        rows.push(c::OwnerAccountLine {
            number,
            place: number,
            uses: None,
            plan: false,
            provider: Some(PROVIDER),
            provider_name: Some(c::DocumentText::complete(
                c::Text::new(Box::from(b"Scripted".as_slice()), 32).unwrap(),
            )),
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(Box::from(b"Account".as_slice()), 32).unwrap()),
            holder: c::OwnerAccountHolder::Party(fixtures::PARTY),
            sources,
            cut: false,
        })
        .unwrap();
        signing
            .push(c::OwnerAccountSignIn {
                source: number,
                provider: PROVIDER,
                holder: c::OwnerAccountHolder::Party(fixtures::PARTY),
            })
            .unwrap();
    }
    let doc = c::OwnerAccounts {
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts: rows,
        signing,
        last_end: None,
        usable: true,
        settled: false,
        cut: false,
    };
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(doc.clone()))));
    assert_eq!(accounts::document(root.client()), Some(&doc));
}
