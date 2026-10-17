use crate::{Address, Event, Limits, Request, forge};
use jig_client::contract::{self, Delivery, DocumentText, Frame, FrameParty, HeldRole, Needs, PartyKind, WatchEvent};
use jig_client_view::boundary::Offset;
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Token, Wall};
pub const FORGE_ASKER: jig_client::Asker = jig_client::Asker(1);
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const OTHER: u16 = 2;
pub const PROJECT: u32 = 1;
pub const SEED: u64 = 11;
pub fn limits() -> Limits {
    {
        let jig = jig_client_view::Limits {
            segments: 9,
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
        };
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        Limits {
            jig,
            forge: forge::Limits { parts: 8, field_bytes: 256, checks: 8 },
            client: crate::limits::client(
                &jig,
                requests,
                8,
                jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                Duration::from_millis(100),
            )
            .expect("client profile fits"),
        }
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
    let mut root = crate::unlinked(&limits, SEED);
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
        if let Some(Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { .. } } }) =
            out.pop()
        {
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
pub fn start(root: &mut crate::Domain, kept: List<skein_lib::RequestRecord<jig_client::Held>>) -> Token {
    let limits = limits();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        root,
        &env(&limits),
        Event::Start {
            address: Address::Jig(jig_client_view::address::Address::Inbox { project: None }),
            kept: List::with_capacity(0),
            offset: Offset { minutes: 0 },
        },
        &mut out,
    );
    crate::step(root, &env(&limits), Event::Client { event: jig_client::Event::StartUnlinked { kept } }, &mut out);
    crate::step(
        root,
        &env(&limits),
        Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } },
        &mut out,
    );
    crate::step(
        root,
        &env(&limits),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Served {
                    served: true,
                    versions: Some(contract::VersionRange {
                        first: jig_client_protocol_wire::VERSION,
                        last: jig_client_protocol_wire::VERSION,
                    }),
                },
            },
        },
        &mut out,
    );
    let mut frame = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(contract::Subject::Frame) },
                },
        } = request
        {
            frame = Some(watch);
        }
    }
    frame.expect("start opens a frame watch")
}
pub fn confirm(root: &mut crate::Domain, watch: Token, party: u64, out: &mut Queue<Request>) {
    let limits = limits();
    crate::step(
        root,
        &env(&limits),
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Watched { watch, event: frame(party) } } },
        out,
    );
}

pub fn adopted_bytes() -> jig_client::Family {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = forge::wire_limits(&limits().forge).expect("wire limits");
    let kinds = w::Kinds::new(
        &limits,
        w::KindsParts { skein_bools: [true, true, true, true, false, false, false, false, true] },
    )
    .expect("kinds");
    let adopted = w::Adopted::new(
        &limits,
        w::AdoptedParts {
            forge: FORGE,
            repository: 7,
            owner: Box::from(*b"acme"),
            name: Box::from(*b"shop"),
            role: w::Role::Owned,
            home: true,
            kinds,
            protection: w::Protection::Unknown,
        },
    )
    .expect("adopted repository");
    let answer =
        w::AdoptAnswer::new(&limits, w::AdoptAnswerParts { outcome: w::Outcome::Adopted(adopted) }).expect("answer");
    jig_client::Family {
        family: wire::number(wire::Owner::Forge),
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&answer),
    }
}
