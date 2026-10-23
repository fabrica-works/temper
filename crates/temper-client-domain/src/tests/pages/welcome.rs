use crate::tests::{demo, fixtures as f};
use crate::{
    Action, Address, Domain, Event, Place, Request, Step, forge,
    pages::{self, Act},
};
use jig_client::{Down, Link, Readout, Told, Up, contract as c};
use skein_lib::{List, Queue, Secret, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn go(root: &mut Domain, step: Step) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(Address::Welcome { step }) })
}
fn deliver(root: &mut Domain, watch: Token, delivery: c::Delivery) -> Queue<Request> {
    up(root, Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) })
}
fn answer(root: &mut Domain, present: bool) {
    let mut out = Queue::with_capacity(1);
    forge::demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut out);
    forge::demo::told(&mut root.forge, &Told::Link { link: Link::Live }, f::limits().forge, &mut out);
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, .. } }) = out.pop() else {
        panic!("demo read")
    };
    let page = if present {
        Readout::Family { family: wire::FORGE, record: Some(demo::answer(b"Change the title")) }
    } else {
        Readout::NotServed { family: Some(wire::FORGE), served: None }
    };
    crate::route::told(
        root,
        &f::env(&f::limits()),
        Told::Page { handle, page },
        &mut Queue::with_capacity(crate::max_out(&f::limits())),
    );
}
fn opened(owner: bool, step: Step) -> (Domain, Token, Token, Option<Token>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, accounts) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
    frame.projects = List::with_capacity(1);
    frame
        .projects
        .push(c::MeProject {
            project: f::PROJECT,
            name: None,
            role: c::HeldRole::Member,
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
            models: List::with_capacity(0),
        })
        .unwrap();
    frame.providers = List::with_capacity(1);
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(c::Text::new(b"openai".as_slice().into(), 32).unwrap()),
            key_bytes: 256,
            sign_in: true,
            signed_in_plan: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .unwrap();
    frame.deployment_role = Some(if owner { c::HeldRole::Owner } else { c::HeldRole::Member });
    deliver(&mut root, me, c::Delivery::Me(frame));
    let mut out = go(&mut root, step);
    let mut forges = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Open { watch, subject: jig_client::Watchable::Family { .. } } },
        } = request
        {
            forges = Some(watch);
        }
    }
    (root, me, accounts, forges)
}
fn rows(root: &mut Domain, watch: Token, number: u16) -> Queue<Request> {
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    let mut forges = List::with_capacity(1);
    forges
        .push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    number,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Connected,
                    connection: w::Connection::Connected(
                        w::Connected::new(&limits, w::ConnectedParts { id: 42, login: b"bot".as_slice().into() })
                            .unwrap(),
                    ),
                    offer: w::Offer::NotOffered,
                },
            )
            .unwrap(),
        )
        .unwrap();
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges }).unwrap()),
            },
        },
    )
}
fn account(root: &mut Domain, watch: Token, provider: u32, usable: bool) -> Queue<Request> {
    deliver(root, watch, c::Delivery::OwnerAccounts(Box::new(demo::accounts(Some(provider), usable, false))))
}
fn moved(mut out: Queue<Request>, replace: bool) -> bool {
    while let Some(request) = out.pop() {
        if request == (Request::Show { address: Address::Chats, replace }) {
            return true;
        }
    }
    false
}
fn decided(root: &Domain) -> bool {
    root.pages().welcome().unwrap().decided
}
#[test]
fn the_welcome_moves_on_once_an_account_and_a_forge_are_the_persons() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, true);
    account(&mut root, accounts, 18, true);
    assert!(moved(rows(&mut root, forges.unwrap(), wire::FORGE + 1), true));
    assert!(decided(&root));
}
#[test]
fn the_demos_account_and_forge_keep_the_welcome() {
    for (provider, number) in [(17, wire::FORGE), (18, wire::FORGE), (17, wire::FORGE + 1)] {
        let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
        answer(&mut root, true);
        account(&mut root, accounts, provider, true);
        assert!(!moved(rows(&mut root, forges.unwrap(), number), true));
        assert!(decided(&root));
    }
}
#[test]
fn an_account_alone_decides_for_a_person_without_the_forges_role() {
    let (mut root, _, accounts, forges) = opened(false, Step::Welcome);
    assert!(forges.is_none());
    answer(&mut root, true);
    assert!(moved(account(&mut root, accounts, 18, true), true));
}
#[test]
fn the_welcome_waits_for_its_facts() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    account(&mut root, accounts, 18, true);
    up(&mut root, Up::Watched { watch: forges.unwrap(), event: c::WatchEvent::Opened });
    assert!(!decided(&root));
    rows(&mut root, forges.unwrap(), wire::FORGE + 1);
    assert!(!decided(&root));
    answer(&mut root, true);
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::fire(&mut root, &f::env(&f::limits()), &mut out);
    assert!(decided(&root));
    assert!(moved(out, true));
}
#[test]
fn a_refused_forges_watch_counts_as_no_forge() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::NotServed, c::WatchEnd::SubjectEnded] {
        let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
        answer(&mut root, true);
        account(&mut root, accounts, 18, true);
        assert!(!moved(up(&mut root, Up::WatchEnded { watch: forges.unwrap(), end }), true));
        assert!(decided(&root));
    }
}
#[test]
fn an_unserved_demo_read_counts_as_no_demo() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    assert!(root.demo_answered());
    assert!(root.forge().demo().is_none());
    account(&mut root, accounts, 17, true);
    assert!(moved(rows(&mut root, forges.unwrap(), wire::FORGE), true));
}
#[test]
fn an_ended_accounts_watch_counts_as_no_account() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, true);
    rows(&mut root, forges.unwrap(), wire::FORGE + 1);
    assert!(!moved(up(&mut root, Up::WatchEnded { watch: accounts, end: c::WatchEnd::NotAllowed }), true));
    assert!(decided(&root));
}
#[test]
fn a_cut_accounts_document_counts_as_an_answer() {
    let (mut root, _, accounts, _) = opened(false, Step::Welcome);
    answer(&mut root, false);
    let mut document = demo::accounts(Some(18), true, true);
    document.cut = true;
    assert!(moved(deliver(&mut root, accounts, c::Delivery::OwnerAccounts(Box::new(document))), true));
    assert!(decided(&root));
}
#[test]
fn the_welcome_decides_once_each_time_it_opens() {
    let (mut root, _, accounts, _) = opened(false, Step::Welcome);
    answer(&mut root, true);
    assert!(!moved(account(&mut root, accounts, 18, false), true));
    assert!(decided(&root));
    assert!(!moved(account(&mut root, accounts, 18, true), true));
    go(&mut root, Step::Forge);
    assert!(moved(go(&mut root, Step::Welcome), true));
}
#[test]
fn a_later_step_never_moves_on() {
    for step in [Step::Forge, Step::Model, Step::Ready] {
        let (mut root, _, accounts, forges) = opened(true, step);
        answer(&mut root, true);
        account(&mut root, accounts, 18, true);
        assert!(!moved(rows(&mut root, forges.unwrap(), wire::FORGE + 1), true));
        assert!(!decided(&root));
    }
}
#[test]
fn try_the_playground_opens_the_chats_in_the_demos_project() {
    let (mut root, _, _, _) = opened(false, Step::Welcome);
    assert!(!moved(event(&mut root, Event::Act { action: Action::Pages(Act::Playground) }), false));
    answer(&mut root, true);
    assert!(moved(event(&mut root, Event::Act { action: Action::Pages(Act::Playground) }), false));
    assert_eq!(root.pages().project(), Some(f::PROJECT));
}
#[test]
fn the_model_step_sends_the_accounts_acts() {
    use pages::accounts::{Act as A, Operation, Way};
    let (mut root, _, _, _) = opened(false, Step::Model);
    event(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::Accounts(A::Choose { name: [1; 16], way: Way::Key { provider: 17 } })),
        },
    );
    event(&mut root, Event::AccountKey { value: Secret::new(b"private-test-key".as_slice().into()) });
    let mut out = event(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::Accounts(A::Submit { name: [1; 16], operation: Operation::AddAccountKey })),
        },
    );
    let mut sent = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { value: Some(value), .. } } } =
            request
        {
            assert_eq!(value.expose(), b"private-test-key");
            sent = true;
        }
    }
    assert!(sent);
    assert!(!root.pages().accounts().unwrap().has_key());
}
#[test]
fn leaving_the_welcome_unwatches_its_forges() {
    let (mut root, _, _, forges) = opened(true, Step::Welcome);
    let watch = forges.unwrap();
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let mut out = event(&mut root, Event::Went { place: Place::Address(Address::Chats) });
    let mut closed = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Close { watch: closing } } } = request
        {
            closed |= closing == watch;
        }
    }
    assert!(closed);
    assert!(root.pages().welcome().is_none());
}
#[test]
fn losing_the_owner_role_closes_the_welcomes_watch() {
    let (mut root, me, _, forges) = opened(true, Step::Welcome);
    let watch = forges.unwrap();
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let c::WatchEvent::Delivered(frame) = f::frame(f::PARTY) else { panic!("frame") };
    let mut out = deliver(&mut root, me, frame);
    let mut closed = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Close { watch: closing } } } = request
        {
            closed |= closing == watch;
        }
    }
    assert!(closed);
}
#[test]
fn an_invalid_forges_document_counts_as_no_forge() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, true);
    account(&mut root, accounts, 18, true);
    assert!(!moved(
        up(
            &mut root,
            Up::FamilyDelivered {
                watch: forges.unwrap(),
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: Box::new([255])
                }
            }
        ),
        true
    ));
    assert!(decided(&root));
    let pages::welcome::Forges::Ended = root.pages().welcome().unwrap().forges else {
        panic!("invalid rows end the watch")
    };
}
