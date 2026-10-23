use super::{demo, fixtures as f};
use crate::{
    Address, Choice, Event, Place, Request, Step,
    forge::{
        self,
        connect::{self, Shown},
        forges::{self, Connection as C, Kind, Mode, Offer, Row, State},
    },
};
use jig_client::{self as client, Down, Told, Up, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const HERE: u16 = 3;
fn row(number: u16, kind: Kind, mode: Mode, connection: C, offer: Offer) -> Row {
    Row { number, kind, mode, connection, offer }
}
fn rows(connection: C) -> State {
    State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, connection, Offer::Offered)]))
}
#[test]
fn the_flow_shows_the_step_its_row_is_at() {
    for (connection, expected) in [
        (C::NotSetUp, Shown::Details(Choice::Forge(HERE))),
        (C::SettingUp { step: None }, Shown::SettingUp(HERE)),
        (C::SettingUp { step: Some(forges::Step::Prepare) }, Shown::SettingUp(HERE)),
        (C::SetUpFailed { why: b"fetch failed".as_slice().into() }, Shown::SettingUp(HERE)),
        (C::Connecting, Shown::SettingUp(HERE)),
        (C::Connected { id: 1, login: b"bot".as_slice().into() }, Shown::Ready(HERE)),
        (C::Stopped { reason: b"stopped".as_slice().into() }, Shown::Choose),
    ] {
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows(connection), None), expected);
    }
    assert_eq!(connect::step(None, &rows(C::NotSetUp), None), Shown::Choose);
    assert_eq!(connect::step(Some(Choice::Forge(9)), &rows(C::NotSetUp), None), Shown::Choose);
    assert_eq!(connect::step(Some(Choice::Elsewhere), &State::Unknown, None), Shown::Details(Choice::Elsewhere));
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    assert_eq!(connect::step(Some(Choice::Forge(demo.forge)), &rows(C::NotSetUp), Some(&demo)), Shown::Choose);
    let github = State::Rows(Box::new([row(2, Kind::GitHub, Mode::Managed, C::NotConnected, Offer::NotOffered)]));
    assert_eq!(connect::step(Some(Choice::Forge(2)), &github, None), Shown::Details(Choice::Forge(2)));
    let fake = State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Fake, C::NotSetUp, Offer::Offered)]));
    assert_eq!(connect::step(Some(Choice::Forge(HERE)), &fake, None), Shown::Choose);
}
#[test]
fn choose_offers_github_forgejo_here_and_elsewhere_by_offer() {
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    let state = State::Rows(Box::new([
        row(demo.forge, Kind::GitHub, Mode::Managed, C::NotConnected, Offer::NotOffered),
        row(2, Kind::GitHub, Mode::Managed, C::NotConnected, Offer::NotOffered),
        row(3, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::NotUnderTheBox),
        row(4, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::NotOffered),
        row(5, Kind::GitHub, Mode::Fake, C::NotConnected, Offer::NotOffered),
        row(6, Kind::Forgejo, Mode::Managed, C::NotConnected, Offer::NotOffered),
        row(7, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::Offered),
    ]));
    // Demo numbers belong to their own fixture; the other six numbers stay distinct.
    assert_eq!(
        connect::choices(&state, Some(&demo)).as_slice(),
        &[Choice::Forge(2), Choice::Forge(3), Choice::Forge(7), Choice::Elsewhere]
    );
    assert_eq!(connect::next_number(&State::Rows(Box::new([]))), 1);
    assert_eq!(connect::next_number(&rows(C::NotSetUp)), 4);
}
fn event(root: &mut crate::Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut crate::Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: client::Event::Up { up } })
}
fn watch(mut out: Queue<Request>) -> Option<Token> {
    while let Some(request) = out.pop() {
        if let Request::Client {
            request: client::Request::Down { down: Down::Open { watch, subject: client::Watchable::Family { .. } } },
        } = request
        {
            return Some(watch);
        }
    }
    None
}
fn opened(owner: bool) -> (crate::Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
    frame.deployment_role = Some(if owner { c::HeldRole::Owner } else { c::HeldRole::Member });
    let out = up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(frame)) });
    (root, me, watch(out).expect("session forges watch"))
}
#[test]
fn the_forges_watch_opens_once_me_is_delivered_for_every_person() {
    for owner in [false, true] {
        let (mut root, me, _) = opened(owner);
        assert!(watch(up(&mut root, Up::Watched { watch: me, event: f::frame(f::PARTY) })).is_none());
        assert!(
            watch(event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: None }) }))
                .is_none()
        );
    }
}
#[test]
fn a_refused_forges_watch_is_kept_until_the_next_sign_in() {
    let (mut root, me, token) = opened(false);
    up(&mut root, Up::WatchEnded { watch: token, end: c::WatchEnd::NotAllowed });
    let State::Ended = root.forges() else { panic!("refused watch ended") };
    assert!(watch(up(&mut root, Up::Watched { watch: me, event: f::frame(f::PARTY) })).is_none());
    event(&mut root, Event::SignedOut);
    let mut out = event(&mut root, Event::Client { event: client::Event::SignIn { with: client::With::Transport } });
    let mut served = up(&mut root, Up::Served { served: true, versions: None });
    while let Some(request) = served.pop() {
        out.push(request);
    }
    let mut me = None;
    let mut reopened = None;
    while let Some(request) = out.pop() {
        if let Request::Client { request: client::Request::Down { down: Down::Open { watch, subject } } } = request {
            match subject {
                client::Watchable::Jig(c::Subject::Me) => me = Some(watch),
                client::Watchable::Family { .. } => reopened = Some(watch),
                client::Watchable::Jig(_) => {}
            }
        }
    }
    let delivered = watch(up(&mut root, Up::Watched { watch: me.unwrap(), event: f::frame(f::PARTY) }));
    assert!(reopened.or(delivered).is_some());
}
#[test]
fn signing_out_unwatches_the_forges() {
    let (mut root, _, token) = opened(true);
    up(&mut root, Up::Watched { watch: token, event: c::WatchEvent::Opened });
    event(&mut root, Event::SignedOut);
    assert!(root.client().document(forges::handle()).is_none());
    let State::Unknown = root.forges() else { panic!("forges cleared") };
}
#[test]
fn the_forge_half_holds_until_a_forge_of_the_persons_connects() {
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    for (connection, expected) in [
        (C::NotSetUp, true),
        (C::SettingUp { step: None }, true),
        (C::Connecting, true),
        (C::Connected { id: 1, login: b"bot".as_slice().into() }, false),
    ] {
        assert_eq!(forge::demo::halves(Some(&demo), None, &rows(connection)).forge, expected);
    }
    let state = State::Rows(Box::new([row(
        demo.forge,
        Kind::Forgejo,
        Mode::Fake,
        C::Connected { id: 1, login: b"bot".as_slice().into() },
        Offer::NotOffered,
    )]));
    assert!(forge::demo::halves(Some(&demo), None, &state).forge);
}
#[test]
fn the_forge_half_does_not_hold_without_the_forges_rows() {
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    for state in [State::Unknown, State::Ended] {
        assert!(!forge::demo::halves(Some(&demo), None, &state).forge);
    }
}
fn setup(root: &mut crate::Domain) -> Queue<Request> {
    f::form_act(root, forge::Act::SetUp { forge: HERE })
}
fn answered(root: &mut crate::Domain, outcome: w::Outcome) -> Queue<Request> {
    let key =
        client::Key { asker: crate::owner::Owner::Forge.asker(), number: root.forge().set_up().unwrap().request.raw() };
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::route::told(
        root,
        &f::env(&f::limits()),
        Told::Answered {
            key,
            outcome: client::Outcome::Answered(client::Answered::Family(client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&w::Answer::new(&limits, w::AnswerParts { outcome }).unwrap()),
            })),
        },
        &mut out,
    );
    out
}
#[test]
fn set_up_sends_one_at_a_time() {
    let mut root = f::on_adoption_page();
    let mut out = setup(&mut root);
    let mut sends = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client { request: client::Request::Down { down: Down::Send { .. } } } = request {
            sends += 1;
        }
    }
    assert_eq!(sends, 1);
    assert!(setup(&mut root).is_empty());
    assert!(root.forge().form().unwrap().pending.is_none());
}
#[test]
fn a_refused_set_up_keeps_its_refusal() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    answered(
        &mut root,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
        ),
    );
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Refused(forges::SetUpRefusal::Busy)))
    );
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert!(root.forge().form().unwrap().outcome.is_none());
}
#[test]
fn an_accepted_set_up_moves_the_welcome_on_to_model() {
    let mut root = f::on_adoption_page();
    event(
        &mut root,
        Event::Went {
            place: Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }),
        },
    );
    setup(&mut root);
    let mut out = answered(&mut root, w::Outcome::SettingUp);
    assert_eq!(out.pop(), Some(Request::Show { address: Address::Welcome { step: Step::Model }, replace: false }));
    assert!(out.is_empty());
}

#[test]
fn a_set_up_survives_page_changes_and_parks_out_of_date() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let request = root.forge().set_up().unwrap().request;
    event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: None }) });
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert_eq!(root.forge().set_up().unwrap().request, request);
    assert_eq!(root.forge().set_up().unwrap().progress, Some(skein_lib::RequestProgress::Parked));
    assert!(setup(&mut root).is_empty());
    answered(&mut root, w::Outcome::SettingUp);
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::SettingUp))
    );
}

#[test]
fn an_unreadable_set_up_answer_is_unknown_and_allows_another_request() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let request = root.forge().set_up().unwrap().request;
    let key = client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() };
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Answered {
            key,
            outcome: client::Outcome::Answered(client::Answered::Family(client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: Box::new([255]),
            })),
        },
        &mut Queue::with_capacity(crate::max_out(&f::limits())),
    );
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Unknown))
    );
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert!(!setup(&mut root).is_empty());
    assert_ne!(root.forge().set_up().unwrap().request, request);
}

#[test]
fn an_out_of_date_link_unwatches_the_forges() {
    let (mut root, _, token) = opened(true);
    up(&mut root, Up::Watched { watch: token, event: c::WatchEvent::Opened });
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Link { link: client::Link::OutOfDate },
        &mut Queue::with_capacity(crate::max_out(&f::limits())),
    );
    assert!(root.client().document(forges::handle()).is_none());
    let State::Unknown = root.forges() else { panic!("forges cleared") };
}
