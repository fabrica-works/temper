use jig_domain_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Wall, Writer};
use temper_client_domain::{Limits, forge};
pub const FORGE: u16 = 1;
pub const OTHER: u16 = 2;
pub const PROJECT: u32 = 1;
pub const SEED: u64 = 11;
pub fn limits() -> Limits {
    Limits {
        jig: jig_domain_client::Limits {
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
            backoff: jig_domain_client::limits::Backoff {
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
pub fn words() -> crate::words::Words {
    crate::words::Words::english(FORGE, 64).expect("forge English fits")
}
pub fn jig_words() -> jig_view::words::Words {
    jig_view::words::Words {
        book: jig_view::phrases::english::english(32).expect("jig English fits"),
        language: 0,
        offset_minutes: 0,
    }
}
pub fn tree_limits() -> jig_view::Limits {
    jig_view::Limits {
        nodes: 128,
        depth: 32,
        text_per_node: 512,
        text_bytes: 65536,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 640,
        bindings: 128,
        slots: 32,
        markdown_depth: 3,
    }
}
pub fn domain_limits() -> Limits {
    limits()
}
pub fn path(segments: &[&[u8]]) -> List<Box<[u8]>> {
    let mut path = List::with_capacity(u32::try_from(segments.len()).expect("fixture segments fit"));
    for segment in segments {
        path.push(skein_lib::bytes::copy_of(segment)).expect("fixture path fits");
    }
    path
}
pub fn text(tree: &jig_view::Tree) -> Box<[u8]> {
    let mut bytes = 0_usize;
    for node in tree.nodes() {
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).expect("fixture text fits");
        }
    }
    let mut writer = Writer::new(bytes);
    for node in tree.nodes() {
        for run in &node.text {
            writer.put(&run.text).expect("text counted");
        }
    }
    writer.finish()
}
pub fn owner_part_text(words: &crate::words::Words, segments: &[&[u8]]) -> Box<[u8]> {
    let mut builder = jig_view::Builder::new(&tree_limits());
    crate::forge::owner_part(
        &mut builder,
        &OwnerPart { connector: FORGE, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path(segments)) },
        words,
    );
    text(&builder.finish())
}
pub fn part_tree(words: &crate::words::Words, part: Part, held: bool) -> jig_view::Tree {
    let limits = forge::Limits { parts: 1, field_bytes: 256, checks: 8 };
    let mut side = forge::Side::new(&limits, forge::Config { connector: FORGE, version: 1 });
    if held {
        forge::step(
            &mut side,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            forge::Event::Show { part: part.clone(), generation: 1 },
            &mut Queue::with_capacity(1),
        );
    }
    let mut builder = jig_view::Builder::new(&tree_limits());
    crate::forge::part(&mut builder, &side, &part, 1, words);
    builder.finish()
}
pub fn part_text(words: &crate::words::Words, part: Part, held: bool) -> Box<[u8]> {
    text(&part_tree(words, part, held))
}
pub fn built(root_forge: u16) -> crate::View {
    let domain = domain_limits();
    let root = temper_client_domain::new(&domain, SEED, forge::Config { connector: root_forge, version: 1 });
    let jig = jig_words();
    let forge = words();
    let envelope = crate::Limits::of(&domain, &jig, &forge).expect("composed envelope fits");
    assert!(envelope.tree.nodes >= 128);
    let limits = crate::Limits { tree: tree_limits() };
    let mut view = crate::View::new(&limits, &domain, jig, forge, FORGE);
    crate::build(&mut view, &root, &mut Queue::with_capacity(limits.tree.patches));
    view
}
pub fn view() -> crate::View {
    let domain = domain_limits();
    let jig = jig_words();
    let words = words();
    let limits = crate::Limits::of(&domain, &jig, &words).expect("view envelope fits");
    crate::View::new(&limits, &domain, jig, words, FORGE)
}
pub fn build<'a>(view: &'a mut crate::View, root: &temper_client_domain::Domain) -> &'a jig_view::Tree {
    crate::build(view, root, &mut Queue::with_capacity(view.limits().tree.patches));
    view.tree()
}
pub fn find(tree: &jig_view::Tree, role: jig_view::Role, name: &[u8]) -> Option<jig_view::NodeId> {
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return Some(node.id);
        }
    }
    None
}
pub fn root_in_project(project: Option<u32>) -> temper_client_domain::Domain {
    use jig_domain_client::contract::{
        self, Delivery, DocumentText, Frame, FrameParty, HeldRole, Needs, PartyKind, ProjectRole, Subject, WatchEvent,
    };
    use temper_client_domain::{Address, Event, JigOut, Request};
    let limits = domain_limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut root = temper_client_domain::new(&limits, SEED, forge::Config { connector: FORGE, version: 1 });
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits));
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Start {
            address: Address::Jig(jig_domain_client::address::Address::Inbox { project }),
            kept: List::with_capacity(0),
            offset: jig_domain_client::boundary::Offset { minutes: 0 },
        },
        &mut out,
    );
    let mut watch = None;
    for _ in 0..out.len() {
        if let Some(Request::Jig(JigOut::Open { watch: token, subject: Subject::Frame })) = out.pop() {
            watch = Some(token);
        }
    }
    let mut projects = List::with_capacity(1);
    projects
        .push(ProjectRole {
            project: PROJECT,
            name: Some(DocumentText::complete(contract::Text::new(Box::from(*b"Shop"), 32).expect("name fits"))),
            role: HeldRole::Policy { role: 1 },
            pool: None,
            chat_budget: 0,
        })
        .expect("one project");
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Jig(jig_domain_client::Event::Hello { versions: contract::VersionRange { first: 7, last: 7 } }),
        &mut out,
    );
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Jig(jig_domain_client::Event::Watch {
            watch: watch.expect("frame opened"),
            event: WatchEvent::Delivered(Delivery::Frame(Box::new(Frame {
                party: FrameParty {
                    number: 73,
                    kind: PartyKind::Person,
                    login: DocumentText::complete(contract::Text::new(Box::from(*b"pat"), 32).expect("login fits")),
                    name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits")),
                },
                projects,
                needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                needs_total: 0,
                updates: contract::Count { value: 0, limited: false },
                results_seen: 0,
                charters: List::with_capacity(0),
                cut: false,
                retention: Duration::from_secs(1000),
                deployment_role: None,
            }))),
        }),
        &mut out,
    );
    root
}
pub fn adoption_root() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(
        &mut root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Forge(forge::Address::Adopt {
                project: PROJECT,
            })),
        },
    );
    root
}
pub fn root_event(root: &mut temper_client_domain::Domain, event: temper_client_domain::Event) {
    let limits = domain_limits();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        event,
        &mut Queue::with_capacity(temper_client_domain::max_out(&limits)),
    );
}
