use super::fixtures::{self as f, panel as p};
use crate::{
    Address, Domain, Event, Place, Request,
    forge::{
        self,
        panel::{State, Subject},
    },
};
use crate::{ChatsFilter, PlanFilter};
use jig_client::{Down, Handle, Told, Up, Watchable, Why, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

const TASK: u64 = 17;
const NEXT: u64 = 19;
const PROJECT: u32 = f::PROJECT + 1;
const SUBJECT: Subject = Subject { task: TASK, project: PROJECT };
const PROCEDURE: c::Executor = c::Executor::Procedure { connector: f::FORGE, kind: 1234 };

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn went(root: &mut Domain, place: Place) -> Queue<Request> {
    event(root, Event::Went { place })
}
fn task_page(root: &mut Domain, task: u64) -> Token {
    let mut out = went(root, Place::Address(Address::Task { task, filter: PlanFilter::All }));
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Open { watch, subject } } } = request
        {
            if subject == Watchable::Jig(c::Subject::Task { task }) {
                found = Some(watch);
            }
            up(root, Up::Watched { watch, event: c::WatchEvent::Opened });
        }
    }
    found.expect("task watch")
}
fn before_document() -> (Domain, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    f::confirm(&mut root, frame, f::PARTY, &mut Queue::with_capacity(crate::max_out(&f::limits())));
    let watch = task_page(&mut root, TASK);
    (root, watch)
}
fn document(root: &mut Domain, watch: Token, task: u64, executor: c::Executor) -> Queue<Request> {
    up(
        root,
        Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(p::task(task, PROJECT, executor)))),
        },
    )
}
fn family_open(out: &mut Queue<Request>, task: u64) -> Option<Token> {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down: Down::Open { watch, subject: Watchable::Family { scope, window, record } },
                },
        } = request
        {
            assert!(found.is_none(), "one panel watch");
            assert_eq!(scope, c::Scope::Project(PROJECT));
            assert_eq!(window, c::Window { rows: 0, after: None });
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let watching =
                wire::decode_watching(&record.bytes, &forge::wire_limits(&f::limits().forge).unwrap()).unwrap();
            let w::Watch::Change(subject) = watching.watch() else { panic!("change watch") };
            assert_eq!(subject.task(), task);
            found = Some(watch);
        }
    }
    found
}
fn opened() -> (Domain, Token, Handle) {
    let (mut root, task) = before_document();
    let watch = family_open(&mut document(&mut root, task, TASK, PROCEDURE), TASK).expect("panel watch");
    let panel = root.forge().panel().expect("panel");
    assert_eq!(panel.subject, SUBJECT);
    assert_eq!(panel.state, State::Loading);
    let handle = panel.handle.expect("open handle");
    assert_eq!(handle.asker, f::FORGE_ASKER);
    assert_ne!(handle.number, 0);
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    (root, watch, handle)
}
fn deliver(root: &mut Domain, watch: Token, panel: &w::ChangePanel) -> Queue<Request> {
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::change_panel(panel),
            },
        },
    )
}
fn closes(out: &mut Queue<Request>) -> List<Token> {
    let mut watches = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Close { watch } } } = request {
            watches.push(watch).unwrap();
        }
    }
    watches
}
#[test]
fn only_a_forge_procedure_document_opens_its_projects_panel() {
    for (executor, watches) in [
        (PROCEDURE, true),
        (c::Executor::Procedure { connector: f::FORGE, kind: 1 }, true),
        (c::Executor::Agent { charter: 1 }, false),
        (c::Executor::Party { party: f::PARTY }, false),
        (c::Executor::Role { project: PROJECT, role: 1 }, false),
        (c::Executor::Procedure { connector: f::FORGE + 1, kind: 1 }, false),
    ] {
        let (mut root, task) = before_document();
        assert!(root.forge().panel().is_none(), "before its document");
        assert_eq!(
            family_open(&mut document(&mut root, task, TASK, executor), TASK).is_some(),
            watches,
            "{executor:?}"
        );
        assert_eq!(root.forge().panel().is_some(), watches);
        assert!(family_open(&mut document(&mut root, task, TASK, executor), TASK).is_none(), "same subject");
    }
}
#[test]
fn leaving_closes_the_panel_and_reopening_uses_a_fresh_handle() {
    for place in [
        Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }),
        Place::Address(Address::Adopt { project: PROJECT }),
    ] {
        let (mut root, watch, first) = opened();
        let closed = closes(&mut went(&mut root, place.clone()));
        assert!(closed.as_slice().contains(&watch), "{place:?}");
        assert!(root.forge().panel().is_none());
        for &watch in &closed {
            up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
        }
        let task = task_page(&mut root, TASK);
        let next_watch = family_open(&mut document(&mut root, task, TASK, PROCEDURE), TASK).expect("reopened panel");
        assert_ne!(root.forge().panel().unwrap().handle, Some(first));
        assert_ne!(next_watch, watch);
    }
}
#[test]
fn another_tasks_page_closes_the_old_panel_and_watches_its_own_subject() {
    let (mut root, watch, first) = opened();
    // task_page acknowledges the task and tree openings; the old panel closes on navigation.
    let mut out = went(&mut root, Place::Address(Address::Task { task: NEXT, filter: PlanFilter::All }));
    let mut task = None;
    let mut closed = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        match request {
            Request::Client { request: jig_client::Request::Down { down: Down::Close { watch } } } => {
                closed.push(watch).unwrap();
            }
            Request::Client { request: jig_client::Request::Down { down: Down::Open { watch, subject } } } => {
                if subject == Watchable::Jig(c::Subject::Task { task: NEXT }) {
                    task = Some(watch);
                }
                up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
            }
            Request::Client { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => {}
        }
    }
    assert!(closed.as_slice().contains(&watch));
    assert!(root.forge().panel().is_none());
    for &watch in &closed {
        up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
    }
    let next =
        family_open(&mut document(&mut root, task.expect("next task"), NEXT, PROCEDURE), NEXT).expect("next panel");
    assert_ne!(root.forge().panel().unwrap().handle, Some(first));
    assert_ne!(watch, next);
    assert_eq!(root.forge().panel().unwrap().subject, Subject { task: NEXT, project: PROJECT });
}
#[test]
fn deliveries_replace_the_whole_decoded_panel() {
    let (mut root, watch, _) = opened();
    for step in [
        w::Step::Producing,
        p::queued(),
        w::Step::Landed(w::Landed::new(&w::CEILINGS, w::LandedParts { merge: [0x47; 32] }).unwrap()),
    ] {
        let panel = p::panel(TASK, step);
        let before = forge::shown(root.forge());
        assert!(deliver(&mut root, watch, &panel).is_empty());
        assert_eq!(root.forge().panel().unwrap().state, State::Shown(Box::new(panel.clone())));
        assert_ne!(forge::shown(root.forge()), before);
        let shown = forge::shown(root.forge());
        assert!(deliver(&mut root, watch, &panel).is_empty());
        assert_eq!(forge::shown(root.forge()), shown, "identical delivery");
    }
}
#[test]
fn invalid_records_end_unknown_and_unwatch_once() {
    let panel = p::panel(TASK, w::Step::Opening);
    let bytes = wire::change_panel(&panel);
    let mut version = bytes.clone();
    version[1] = u8::try_from(wire::CLIENT_VERSION + 1).unwrap();
    let mut trailing = skein_lib::bytes::zeroed(bytes.len() + 1);
    for (target, byte) in trailing.iter_mut().zip(bytes.iter()) {
        *target = *byte;
    }
    for (row, family, version, bytes) in [
        ("version metadata", wire::FORGE, wire::CLIENT_VERSION + 1, bytes.clone()),
        ("version bytes", wire::FORGE, wire::CLIENT_VERSION, version),
        ("family", wire::FORGE + 1, wire::CLIENT_VERSION, bytes),
        ("undecodable", wire::FORGE, wire::CLIENT_VERSION, Box::from([255])),
        ("trailing", wire::FORGE, wire::CLIENT_VERSION, trailing),
        ("other task", wire::FORGE, wire::CLIENT_VERSION, wire::change_panel(&p::panel(NEXT, w::Step::Opening))),
    ] {
        let (mut root, watch, _) = opened();
        let out =
            &mut up(&mut root, Up::FamilyDelivered { watch, record: jig_client::Family { family, version, bytes } });
        assert_eq!(closes(out).as_slice(), &[watch], "{row}");
        assert_eq!(root.forge().panel().unwrap().state, State::Unknown, "{row}");
        assert_eq!(root.forge().panel().unwrap().handle, None);
        assert!(deliver(&mut root, watch, &panel).is_empty(), "late delivery");
        assert_eq!(root.forge().panel().unwrap().state, State::Unknown);
        assert!(
            family_open(
                &mut went(&mut root, Place::Address(Address::Task { task: TASK, filter: PlanFilter::All })),
                TASK
            )
            .is_none()
        );
        up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
        let closed =
            closes(&mut went(&mut root, Place::Address(Address::Chats { filter: ChatsFilter::All, start: false })));
        assert!(!closed.as_slice().contains(&watch), "already unwatched");
        for &watch in &closed {
            up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
        }
        assert!(root.forge().panel().is_none());
        let task = task_page(&mut root, TASK);
        assert!(family_open(&mut document(&mut root, task, TASK, PROCEDURE), TASK).is_some(), "opened afresh");
        assert_eq!(root.forge().panel().unwrap().state, State::Loading);
    }
}
#[test]
fn jig_endings_keep_unknown_and_other_reasons_apart_from_loading() {
    for (end, expected) in [
        (c::WatchEnd::Unknown, State::Unknown),
        (c::WatchEnd::NotAllowed, State::Ended(Why::NotAllowed)),
        (c::WatchEnd::SignedOut, State::Ended(Why::SignedOut)),
        (c::WatchEnd::SubjectEnded, State::Unknown),
        (c::WatchEnd::NotServed, State::Ended(Why::NotServed)),
    ] {
        let (mut root, watch, _) = opened();
        let shown = forge::shown(root.forge());
        assert!(up(&mut root, Up::WatchEnded { watch, end }).is_empty());
        assert_eq!(root.forge().panel().unwrap().state, expected);
        assert_ne!(forge::shown(root.forge()), shown);
        assert!(root.forge().panel().unwrap().handle.is_none());
        assert!(
            family_open(
                &mut went(&mut root, Place::Address(Address::Task { task: TASK, filter: PlanFilter::All })),
                TASK
            )
            .is_none()
        );
        let closed =
            closes(&mut went(&mut root, Place::Address(Address::Chats { filter: ChatsFilter::All, start: false })));
        assert!(!closed.as_slice().contains(&watch), "jig already ended it");
        for &watch in &closed {
            up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
        }
        assert!(root.forge().panel().is_none());
        let task = task_page(&mut root, TASK);
        assert!(family_open(&mut document(&mut root, task, TASK, PROCEDURE), TASK).is_some(), "opened afresh");
        assert_eq!(root.forge().panel().unwrap().state, State::Loading);
    }
}
#[test]
fn a_late_tell_from_the_previous_subject_changes_nothing() {
    let (mut root, watch, first) = opened();
    let mut out =
        event(&mut root, Event::Forge(forge::Event::Panel { subject: Some(Subject { task: NEXT, project: PROJECT }) }));
    // This event exercises the side's two-output replacement through the root.
    assert!(out.len() <= crate::max_out(&f::limits()));
    let next = family_open(&mut out, NEXT).expect("replacement");
    let handle = root.forge().panel().unwrap().handle.unwrap();
    assert_ne!(handle, first);
    let shown = forge::shown(root.forge());
    for told in [Told::Changed { handle: first }, Told::Ended { handle: first, why: Why::Unknown }] {
        let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
        crate::route::told(&mut root, &f::env(&f::limits()), told, &mut out);
        assert!(out.is_empty());
        assert_eq!(forge::shown(root.forge()), shown);
        assert_eq!(root.forge().panel().unwrap().handle, Some(handle));
        assert_eq!(root.forge().panel().unwrap().state, State::Loading);
    }
    deliver(&mut root, watch, &p::panel(TASK, w::Step::Producing));
    deliver(&mut root, next, &p::panel(NEXT, w::Step::Producing));
    assert_eq!(root.forge().panel().unwrap().state, State::Shown(Box::new(p::panel(NEXT, w::Step::Producing))));
}
#[test]
fn output_and_memory_bounds_cover_replacement_and_a_full_panel() {
    const PANEL_REPLACEMENT_REQUESTS: u32 = 2; // Unwatch followed by watch.
    let limits = f::limits();
    let mut side = forge::Side::new(&limits.forge, f::SEED);
    let mut out = Queue::with_capacity(forge::max_out(&limits.forge));
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits.forge };
    forge::step(&mut side, &env, forge::Event::Panel { subject: Some(SUBJECT) }, &mut out);
    let Some(forge::Request::Client { asked: jig_client::Event::Watch { handle: first, .. } }) = out.pop() else {
        panic!("watch")
    };
    forge::step(
        &mut side,
        &env,
        forge::Event::Panel { subject: Some(Subject { task: NEXT, project: PROJECT }) },
        &mut out,
    );
    assert_eq!(out.len(), PANEL_REPLACEMENT_REQUESTS);
    let Some(forge::Request::Client { asked: jig_client::Event::Unwatch { handle } }) = out.pop() else {
        panic!("unwatch first")
    };
    assert_eq!(handle, first);
    let Some(forge::Request::Client { asked: jig_client::Event::Watch { handle, .. } }) = out.pop() else {
        panic!("watch next")
    };
    assert_ne!(handle, first);
    let (mut root, watch, _) = opened();
    let panel = p::panel(TASK, p::queued());
    deliver(&mut root, watch, &panel);
    assert_eq!(root.forge().panel().unwrap().state, State::Shown(Box::new(panel.clone())));
    let panel_heap = size_of::<w::ChangePanel>()
        + panel.owner().len()
        + panel.name().len()
        + panel.branch().len()
        + panel.landing().len()
        + panel.pull().as_ref().unwrap().address().len()
        + usize::try_from(wire::GATES).unwrap() * size_of::<w::Gate>()
        + usize::try_from(wire::TASKS_AHEAD).unwrap() * size_of::<u64>();
    let wire = forge::wire_limits(&limits.forge).unwrap();
    assert_eq!(
        u64::try_from(panel_heap).unwrap(),
        u64::try_from(size_of::<w::ChangePanel>()).unwrap() + w::ChangePanel::worst_case_heap(&wire).unwrap()
    );
    let existing = u64::from(limits.forge.field_bytes) * 6
        + u64::from(limits.forge.checks) * 4
        + w::DemoAnswer::worst_case_heap(&wire).unwrap()
        + w::Answer::worst_case_heap(&wire).unwrap()
        + u64::from(wire.group_failed_why)
        + forge::forges::worst_case(&limits.forge).unwrap()
        + forge::picker::worst_case(limits.forge).unwrap()
        // Admin token, four fields and its answer.
        + 4090 + u64::from(limits.forge.field_bytes) * 3
        + u64::from(limits.forge.field_bytes.max(wire.first_repository_label))
        + w::Answer::worst_case_heap(&wire).unwrap()
        // App draft key, answered install link and editable App id.
        + 4097 + 2048 + u64::from(limits.forge.field_bytes)
        + u64::try_from(size_of::<Handle>()).unwrap();
    assert_eq!(forge::worst_case(&limits.forge), Some(existing + u64::try_from(panel_heap).unwrap()));
    assert!(crate::worst_case(&limits).unwrap() >= forge::worst_case(&limits.forge).unwrap());
}
