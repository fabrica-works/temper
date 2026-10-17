use crate::{Address, Event, JigOut, Limits, Request, forge, store::Record};
use jig_client::contract::{self, Delivery, DocumentText, Frame, FrameParty, HeldRole, Needs, PartyKind, WatchEvent};
use jig_client_view::boundary::Offset;
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Token, Wall};
pub const FORGE: u16 = 1;
pub const OTHER: u16 = 2;
pub const PROJECT: u32 = 1;
pub const SEED: u64 = 11;
pub fn limits() -> Limits {
    Limits {
        jig: jig_client_view::Limits {
            watches: 8,
            notices: 2,
            notice_life: Duration::from_secs(5),
            // A finite fixture keeps the same before/after-linger behavior at a short scale.
            linger: Duration::from_millis(250),
            save_delay: Duration::from_millis(10),
            intents: 2,
            fields: 4,
            providers: 1,
            objects: 3,
            sources: 8,
            reads: 8,
            parts: 8,
            delegates: 2,
            document_bytes: 65536,
            records: 2,
            kept: 4,
            drafts: 1,
            projects: 2,
            roles: 2,
            charters: 2,
            text: 32,
            rows: 2,
            conversation_words: 2,
            navigation_entries: 8,
            updates_count: 99,
            backoff: jig_client_view::limits::ui::Backoff {
                first: Duration::from_millis(10),
                most: Duration::from_millis(80),
            },
            heartbeat: Duration::from_millis(100),
        },
        forge: forge::Limits { parts: 8, field_bytes: 256, checks: 8 },
        table: RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        },
    }
}
pub fn env(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}
pub fn connector() -> contract::Part {
    contract::Part::Connector { connector: FORGE }
}
pub fn resource() -> contract::Part {
    let mut path = List::with_capacity(2);
    path.push(Box::from(*b"acme")).expect("fixture path fits");
    path.push(Box::from(*b"shop")).expect("fixture path fits");
    contract::Part::Resource { connector: FORGE, path }
}
pub fn frame(party: u64) -> WatchEvent {
    WatchEvent::Delivered(Delivery::Frame(Box::new(Frame {
        party: FrameParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(contract::Text::new(Box::from(*b"fixture-login"), 32).expect("login fits")),
            name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits")),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: contract::Count { value: 0, limited: false },
        results_seen: 0,
        charters: List::with_capacity(0),
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    })))
}
pub const PARTY: u64 = 73;
pub fn on_adoption_page() -> crate::Domain {
    let limits = limits();
    let mut root = crate::new(&limits, SEED, forge::Config { connector: FORGE, version: 1 });
    let frame = start(&mut root, List::with_capacity(0));
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    confirm(&mut root, frame, PARTY, &mut out);
    crate::step(
        &mut root,
        &env(&limits),
        Event::Went { place: crate::Place::Address(Address::Forge(forge::Address::Adopt { project: PROJECT })) },
        &mut out,
    );
    root
}
pub fn form_act(root: &mut crate::Domain, act: forge::Act) -> Queue<Request> {
    let limits = limits();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(root, &env(&limits), Event::Act { action: crate::Action::Forge(act) }, &mut out);
    crate::take(root, &env(&limits), &mut out);
    out
}
pub fn write(field: forge::Text, text: &[u8], typed: forge::Typed) -> forge::Act {
    forge::Act::Write { field, text: Box::from(text), typed }
}
pub fn valid_fields() -> Box<[forge::Act]> {
    Box::new([
        write(forge::Text::Owner, b"acme", forge::Typed::Bytes),
        write(forge::Text::Name, b"shop", forge::Typed::Bytes),
        write(forge::Text::Landing, b"main", forge::Typed::Bytes),
    ])
}
pub fn filled() -> crate::Domain {
    let mut root = on_adoption_page();
    for act in valid_fields() {
        assert!(form_act(&mut root, act).is_empty());
    }
    root
}
pub fn act_then_adopt(root: &mut crate::Domain, acts: &[forge::Act]) -> u32 {
    for act in valid_fields() {
        assert!(form_act(root, act).is_empty());
    }
    for act in acts {
        assert!(form_act(root, act.clone()).is_empty());
    }
    let mut out = form_act(root, forge::Act::Adopt);
    sends(&mut out)
}
pub fn sends(out: &mut Queue<Request>) -> u32 {
    let mut sent = 0_u32;
    for _ in 0..out.len() {
        if let Some(Request::Send { .. }) = out.pop() {
            sent = sent.checked_add(1).expect("bounded output count");
        }
    }
    sent
}
pub fn forge_event(root: &mut crate::Domain, event: forge::Event) -> Queue<Request> {
    let limits = limits();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(root, &env(&limits), Event::Forge(event), &mut out);
    out
}
pub fn marked(root: &crate::Domain, place: Option<forge::Text>) -> Option<forge::Broken> {
    let form = root.forge().form().expect("on adoption page");
    match place {
        Some(field) => form.texts[field.index()].broken,
        None => form.home.broken,
    }
}
pub fn owner_empty() -> Box<[forge::Act]> {
    Box::new([write(forge::Text::Owner, b"", forge::Typed::Bytes)])
}
pub fn forge_zero() -> Box<[forge::Act]> {
    Box::new([write(forge::Text::Forge, b"0", forge::Typed::Forge(Some(0)))])
}
pub fn checks_twice() -> Box<[forge::Act]> {
    Box::new([write(forge::Text::Checks, b"1, 1", forge::Typed::Checks(None))])
}
pub fn context_home() -> Box<[forge::Act]> {
    Box::new([forge::Act::Choose { role: forge::Role::Context }])
}
pub fn no_ci_no_check() -> Box<[forge::Act]> {
    Box::new([forge::Act::Ci])
}
pub fn adopted() -> forge::Answer {
    forge::Answer::Adopted(forge::Adopted {
        forge: FORGE,
        repository: 7,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        role: forge::Role::Owned,
        home: true,
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: false,
            status: false,
            comment: false,
            issue: false,
            branch: true,
        },
        protection: forge::Protection::Unknown,
    })
}
pub fn start(root: &mut crate::Domain, kept: List<Record>) -> Token {
    let limits = limits();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        root,
        &env(&limits),
        Event::Start {
            address: Address::Jig(jig_client_view::address::Address::Inbox { project: None }),
            kept,
            offset: Offset { minutes: 0 },
        },
        &mut out,
    );
    let mut frame = None;
    for _ in 0..out.len() {
        match out.pop().expect("counted output") {
            Request::Jig(JigOut::Open { watch, subject }) => match subject {
                contract::Subject::Frame => frame = Some(watch),
                contract::Subject::Inbox { .. }
                | contract::Subject::Task { .. }
                | contract::Subject::Conversations
                | contract::Subject::Tree { .. }
                | contract::Subject::Chips { .. }
                | contract::Subject::Goals { .. }
                | contract::Subject::Tasks { .. }
                | contract::Subject::Run { .. }
                | contract::Subject::Deployment => {}
            },
            Request::Jig(
                JigOut::Read { .. }
                | JigOut::Continue { .. }
                | JigOut::Close { .. }
                | JigOut::Answered { .. }
                | JigOut::Unknown { .. },
            )
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Send { .. }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut
            | Request::Fact { .. } => {}
        }
    }
    frame.expect("start opens a frame watch")
}
pub fn confirm(root: &mut crate::Domain, watch: Token, party: u64, out: &mut Queue<Request>) {
    let limits = limits();
    let env = env(&limits);
    crate::step(root, &env, Event::Jig(jig_client_view::Event::Hello { served: true }), out);
    crate::step(root, &env, Event::Jig(jig_client_view::Event::Watch { watch, event: frame(party) }), out);
}
