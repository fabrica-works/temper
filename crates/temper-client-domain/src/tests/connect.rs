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
    Row { number, kind, mode, connection, offer, first: None, installations: Box::new([]), ended: None }
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
    assert_eq!(
        out.pop(),
        Some(Request::Show { address: Address::Welcome { step: Step::Model { way: None } }, replace: false })
    );
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

#[test]
fn the_forges_page_lists_the_persons_forges_not_fakes_or_the_demos() {
    use forge::listed;
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    let connections = [
        C::NotConnected,
        C::NotSetUp,
        C::SettingUp { step: None },
        C::SetUpFailed { why: Box::default() },
        C::Connecting,
        C::Connected { id: 42, login: Box::default() },
        C::Stopped { reason: Box::default() },
    ];
    for connection in connections {
        let expected = match connection {
            C::NotConnected | C::NotSetUp => false,
            C::SettingUp { .. } | C::SetUpFailed { .. } | C::Connecting | C::Connected { .. } | C::Stopped { .. } => {
                true
            }
        };
        let candidate = row(HERE, Kind::Forgejo, Mode::Owned, connection, Offer::Offered);
        assert_eq!(listed::personal(&candidate, Some(&demo)), expected);
        let mut fake = candidate.clone();
        fake.mode = Mode::Fake;
        assert!(!listed::personal(&fake, None));
        fake.mode = Mode::Owned;
        fake.number = demo.forge;
        assert!(!listed::personal(&fake, Some(&demo)));
    }
    let state = State::Rows(Box::new([
        row(demo.forge, Kind::Forgejo, Mode::Connected, C::Connecting, Offer::NotOffered),
        row(2, Kind::GitHub, Mode::Managed, C::Connecting, Offer::NotOffered),
        row(3, Kind::Forgejo, Mode::Fake, C::Connecting, Offer::NotOffered),
        row(4, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::Offered),
        row(5, Kind::Forgejo, Mode::Owned, C::Stopped { reason: Box::default() }, Offer::Offered),
    ]));
    let listed = listed::rows(&state, Some(&demo));
    assert_eq!(listed.len(), 2_u32);
    assert_eq!(listed.as_slice()[0].number, 2_u16);
    assert_eq!(listed.as_slice()[1].number, 5_u16);
    assert!(listed::rows(&State::Unknown, None).is_empty());
    assert!(listed::rows(&State::Ended, None).is_empty());
}
#[test]
fn the_dot_is_red_for_a_forge_that_needs_the_person_else_amber_while_one_sets_up() {
    use forge::listed::{self, Dot};
    for (connection, expected) in [
        (C::NotSetUp, Dot::None),
        (C::Connected { id: 42, login: Box::default() }, Dot::None),
        (C::Connecting, Dot::Amber),
        (C::SettingUp { step: None }, Dot::Amber),
        (C::SetUpFailed { why: Box::default() }, Dot::Red),
        (C::Stopped { reason: Box::default() }, Dot::Red),
    ] {
        assert_eq!(listed::dot(&rows(connection), None), expected);
    }
    for connections in
        [[C::Connecting, C::Stopped { reason: Box::default() }], [C::Stopped { reason: Box::default() }, C::Connecting]]
    {
        let state = State::Rows(Box::new([
            row(2, Kind::GitHub, Mode::Managed, connections[0].clone(), Offer::NotOffered),
            row(HERE, Kind::Forgejo, Mode::Owned, connections[1].clone(), Offer::Offered),
        ]));
        assert_eq!(listed::dot(&state, None), Dot::Red);
        assert_eq!(
            connect::underway_forge(&state, None).unwrap().number,
            if connections[0] == C::Connecting { 2 } else { HERE }
        );
    }
    let demo = forge::family::demo(&demo::answer(b"title"), &w::CEILINGS).unwrap();
    let state = State::Rows(Box::new([
        row(demo.forge, Kind::GitHub, Mode::Managed, C::Connecting, Offer::NotOffered),
        row(HERE, Kind::Forgejo, Mode::Fake, C::Stopped { reason: Box::default() }, Offer::NotOffered),
    ]));
    assert_eq!(listed::dot(&state, Some(&demo)), Dot::None);
    assert!(connect::underway_forge(&state, Some(&demo)).is_none());
}

fn sent(mut out: Queue<Request>) -> (Token, skein_lib::RequestKey, w::Request) {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                client::Request::Down { down: Down::Send { attempt, key, request: client::Ask::Family { record, .. }, .. } },
        } = request
        {
            assert!(found.is_none(), "one family request");
            found = Some((
                attempt,
                key,
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request,
            ));
        }
    }
    found.expect("sent family request")
}
fn reply(root: &mut crate::Domain, attempt: Token, outcome: w::Outcome) -> Queue<Request> {
    up(
        root,
        Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(client::Answered::Family(client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
            })),
            why: None,
        },
    )
}
fn first(state: forges::FirstState) -> forges::First {
    forges::First {
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: forges::SetUpProject::Existing(1),
        state,
    }
}
fn with_first(connection: C, state: forges::FirstState) -> State {
    let mut row = row(HERE, Kind::Forgejo, Mode::Owned, connection, Offer::Offered);
    row.first = Some(first(state));
    State::Rows(Box::new([row]))
}
fn deliver_first(
    root: &mut crate::Domain,
    watch: Token,
    connection: w::Connection,
    state: w::FirstState,
) -> Queue<Request> {
    let first = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: w::SetUpProject::Existing(
                w::ExistingProject::new(&w::CEILINGS, w::ExistingProjectParts { project: 1 }).unwrap(),
            ),
            state,
        },
    )
    .unwrap();
    deliver_first_record(root, watch, connection, first)
}
fn deliver_first_record(
    root: &mut crate::Domain,
    watch: Token,
    connection: w::Connection,
    first: w::FirstRepository,
) -> Queue<Request> {
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                installations: List::with_capacity(0),
                ended: None,
                number: HERE,
                kind: w::Kind::Forgejo,
                mode: w::Mode::Owned,
                offer: w::Offer::Offered,
                connection,
                first: Some(first),
            },
        )
        .unwrap(),
    )
    .unwrap();
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
            },
        },
    )
}
fn made() -> w::FirstState {
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 42 }).unwrap())
}
fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 7, login: b"temper".as_slice().into() }).unwrap(),
    )
}
fn write_setup(root: &mut crate::Domain, field: forges::SetupField, text: &[u8]) {
    assert!(f::form_act(root, forge::Act::SetupWrite { field, text: text.into() }).is_empty());
}
#[test]
fn a_set_up_asks_for_work_hello_in_the_persons_project_else_one() {
    for project in [None, Some(7_u32)] {
        let (mut root, me, _) = opened(true);
        if let Some(project) = project {
            let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
            frame.projects = List::with_capacity(1);
            frame
                .projects
                .push(c::MeProject {
                    project,
                    name: None,
                    role: c::HeldRole::Owner,
                    pool: None,
                    templates: List::with_capacity(0),
                    creating: None,
                    models: List::with_capacity(0),
                })
                .unwrap();
            up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(frame)) });
        }
        let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
        assert_eq!((value.org(), value.repository()), (b"work".as_slice(), b"hello".as_slice()));
        let w::SetUpProject::Existing(value) = value.project() else { panic!("existing project") };
        assert_eq!(value.project(), project.unwrap_or(1));
    }
}
#[test]
fn a_set_up_sends_the_fields_the_person_gave() {
    let (mut root, _, _) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"team");
    write_setup(&mut root, forges::SetupField::Repository, b"widget");
    assert_eq!(root.forge().setup_form().text(forges::SetupField::Project), b"widget");
    write_setup(&mut root, forges::SetupField::Project, b"Next");
    write_setup(&mut root, forges::SetupField::Repository, b"app");
    f::form_act(&mut root, forge::Act::SetupNew { new: true });
    let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!((value.org(), value.repository()), (b"team".as_slice(), b"app".as_slice()));
    let w::SetUpProject::New(value) = value.project() else { panic!("new project") };
    assert_eq!(value.name(), b"Next");
    let last = root.forge().set_up().unwrap();
    assert_eq!(
        (&*last.org, &*last.repository, &last.project),
        (b"team".as_slice(), b"app".as_slice(), &forges::SetUpProject::New(b"Next".as_slice().into()))
    );
}
#[test]
fn an_empty_field_is_required_and_sends_nothing() {
    for field in [forges::SetupField::Organization, forges::SetupField::Repository, forges::SetupField::Project] {
        let (mut root, _, _) = opened(true);
        f::form_act(&mut root, forge::Act::SetupNew { new: true });
        write_setup(&mut root, field, b"");
        assert!(setup(&mut root).is_empty());
        assert!(root.forge().setup_form().required(field));
        assert!(root.forge().set_up().is_none());
    }
}
#[test]
fn set_up_again_sends_the_same_fields() {
    let (mut root, _, watch) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"team");
    let (attempt, _, w::Request::SetUp(original)) = sent(setup(&mut root)) else { panic!("setup") };
    reply(&mut root, attempt, w::Outcome::SettingUp);
    deliver_first(
        &mut root,
        watch,
        w::Connection::Connecting,
        w::FirstState::Failed(
            w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::File }).unwrap(),
        ),
    );
    write_setup(&mut root, forges::SetupField::Organization, b"changed");
    let (_, _, w::Request::SetUp(retry)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!(original, retry);
}
#[test]
fn ready_waits_on_the_first_repository_made() {
    for (state, expected) in [
        (forges::FirstState::Making, Shown::SettingUp(HERE)),
        (forges::FirstState::Made { id: 42 }, Shown::Ready(HERE)),
    ] {
        let rows = with_first(C::Connected { id: 7, login: b"temper".as_slice().into() }, state);
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None), expected);
        assert_eq!(connect::personal(&rows, None).unwrap().number, HERE);
    }
    assert_eq!(
        connect::step(Some(Choice::Forge(HERE)), &with_first(C::Connecting, forges::FirstState::Made { id: 42 }), None),
        Shown::SettingUp(HERE)
    );
}
#[test]
fn a_first_repository_failed_shows_its_step() {
    for (step, words) in [
        (forges::CreateStep::Admin, b"Set up failed: checking the admin's token".as_slice()),
        (forges::CreateStep::Org, b"Set up failed: making the organization"),
        (forges::CreateStep::Repository, b"Set up failed: making the repository"),
        (forges::CreateStep::File, b"Set up failed: adding the starter workflow"),
    ] {
        let rows = with_first(C::Connected { id: 7, login: Box::default() }, forges::FirstState::Failed { step });
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None), Shown::SettingUp(HERE));
        let row = connect::personal(&rows, None).unwrap();
        assert_eq!(forge::words::forge_connection(row).as_ref(), words);
        assert_eq!(forge::listed::dot(&rows, None), forge::listed::Dot::Red);
    }
    let rows = with_first(C::Connected { id: 7, login: Box::default() }, forges::FirstState::Making);
    assert_eq!(forge::listed::dot(&rows, None), forge::listed::Dot::Amber);
}
#[test]
fn the_client_that_set_up_adopts_the_made_repository_once_on_another_page() {
    let (mut root, _, watch) = opened(true);
    let (attempt, _, _) = sent(setup(&mut root));
    reply(&mut root, attempt, w::Outcome::SettingUp);
    event(&mut root, Event::Went { place: Place::Address(Address::Chats) });
    assert!(deliver_first(&mut root, watch, connected(), w::FirstState::Making).is_empty());
    assert!(deliver_first(&mut root, watch, w::Connection::Connecting, made()).is_empty());
    let (_, key, w::Request::Adopt(adoption)) = sent(deliver_first(&mut root, watch, connected(), made())) else {
        panic!("adoption")
    };
    assert_eq!(key, forge::first_repository_key(HERE, false));
    assert_eq!((adoption.project(), adoption.forge(), adoption.home(), adoption.ci()), (1, HERE, true, true));
    assert_eq!(
        (adoption.owner(), adoption.name(), adoption.prefix(), adoption.landing()),
        (b"work".as_slice(), b"hello".as_slice(), b"temper/".as_slice(), b"main".as_slice())
    );
    assert_eq!(adoption.role(), &w::Role::Owned);
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
}
#[test]
fn a_client_that_did_not_set_up_adopts_nothing() {
    let (mut root, _, watch) = opened(true);
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
    assert!(root.forge().set_up().is_none());
}
#[test]
fn the_adoption_is_keyed_as_ctls_and_retried_once_without_home() {
    let (mut root, _, watch) = opened(true);
    let (attempt, _, _) = sent(setup(&mut root));
    reply(&mut root, attempt, w::Outcome::SettingUp);
    let (attempt, _, _) = sent(deliver_first(&mut root, watch, connected(), made()));
    let refused = w::Outcome::ForgeRefused(
        w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::Refused }).unwrap(),
    );
    let (attempt, key, w::Request::Adopt(adoption)) = sent(reply(&mut root, attempt, refused.clone())) else {
        panic!("retry")
    };
    assert_eq!(key, forge::first_repository_key(HERE, true));
    assert!(!adoption.home());
    let mut out = reply(&mut root, attempt, refused);
    while let Some(request) = out.pop() {
        if let Request::Client { request: client::Request::Down { down: Down::Send { .. } } } = request {
            panic!("no third adoption");
        }
    }
    assert_eq!(
        root.forge().set_up().unwrap().adoption.as_ref().unwrap().outcome,
        Some(forge::Answer::ForgeRefused(forge::Reason::Refused))
    );
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
}
#[test]
fn new_project_and_invalid_refusals_say_their_words() {
    for (why, expected) in [
        (w::SetUpRefusal::NewProject, b"a new project".as_slice()),
        (w::SetUpRefusal::Invalid, b"invalid: the organization, repository or project name is empty or too long"),
    ] {
        let (mut root, _, _) = opened(true);
        let (attempt, _, _) = sent(setup(&mut root));
        reply(
            &mut root,
            attempt,
            w::Outcome::SetUpRefused(w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why }).unwrap()),
        );
        let Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Refused(why))) =
            root.forge().set_up().unwrap().answer
        else {
            panic!("refusal")
        };
        assert_eq!(forge::words::set_up_refusal(why), expected);
    }
}

#[test]
fn set_up_again_on_another_client_uses_the_rows_fields() {
    let (mut root, _, watch) = opened(true);
    let value = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            org: b"team".as_slice().into(),
            repository: b"widget".as_slice().into(),
            project: w::SetUpProject::Existing(
                w::ExistingProject::new(&w::CEILINGS, w::ExistingProjectParts { project: 7 }).unwrap(),
            ),
            state: w::FirstState::Failed(
                w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::File }).unwrap(),
            ),
        },
    )
    .unwrap();
    deliver_first_record(&mut root, watch, w::Connection::Connecting, value);
    let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!((value.org(), value.repository()), (b"team".as_slice(), b"widget".as_slice()));
    let w::SetUpProject::Existing(project) = value.project() else { panic!("existing project") };
    assert_eq!(project.project(), 7_u32);
}
#[test]
fn signing_out_forgets_the_automatic_adoption_intent() {
    let (mut root, _, _) = opened(true);
    setup(&mut root);
    event(&mut root, Event::SignedOut);
    assert!(root.forge().set_up().is_none());
}
