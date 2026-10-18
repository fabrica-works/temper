use jig_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Wall, Writer};
use temper_client_domain::{Limits, forge};
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const PROJECT: u32 = 1;
pub const SECOND_PROJECT: u32 = 2;
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
            client: temper_client_domain::limits::client(
                &jig,
                requests,
                12,
                8,
                jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                Duration::from_millis(100),
            )
            .expect("client profile fits"),
        }
    }
}
pub fn words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| crate::words::Words::english(FORGE, 64).expect("forge English fits")).clone()
}
pub fn jig_words() -> jig_client_view::words::Words {
    static WORDS: std::sync::OnceLock<jig_client_view::words::Words> = std::sync::OnceLock::new();
    WORDS
        .get_or_init(|| jig_client_view::words::Words {
            book: jig_client_view::phrases::english::english(32).expect("jig English fits"),
            language: 0,
            offset_minutes: 0,
        })
        .clone()
}
pub fn tree_limits() -> skein_ui::Limits {
    skein_ui::Limits {
        nodes: 128,
        depth: 32,
        text_per_node: 512,
        text_bytes: 65536,
        drawing_bytes: 0,
        scene_marks: 0,
        scene_bytes: 0,
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
pub fn text(tree: &skein_ui::Tree) -> Box<[u8]> {
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
    let mut builder = skein_ui::Builder::new(&tree_limits());
    crate::forge::owner_part(
        &mut builder,
        &OwnerPart { connector: FORGE, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path(segments)) },
        words,
    );
    text(&builder.finish())
}
pub fn part_tree(words: &crate::words::Words, part: Part, held: bool) -> skein_ui::Tree {
    let limits = forge::Limits { parts: 1, field_bytes: 256, checks: 8 };
    let mut side = forge::Side::new(&limits, SEED);
    if held {
        forge::step(
            &mut side,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            forge::Event::Show { part: part.clone() },
            &mut Queue::with_capacity(1),
        );
    }
    let mut builder = skein_ui::Builder::new(&tree_limits());
    crate::forge::part(&mut builder, &side, &part, words);
    builder.finish()
}
pub fn part_text(words: &crate::words::Words, part: Part, held: bool) -> Box<[u8]> {
    text(&part_tree(words, part, held))
}
pub fn built() -> crate::View {
    let domain = domain_limits();
    let root = temper_client_domain::unlinked(&domain, SEED);
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
    let limits = crate::Limits { tree: tree_limits() };
    crate::View::new(&limits, &domain, jig, words, FORGE)
}
pub fn build<'a>(view: &'a mut crate::View, root: &temper_client_domain::Domain) -> &'a skein_ui::Tree {
    crate::build(view, root, &mut Queue::with_capacity(view.limits().tree.patches));
    view.tree()
}
pub fn find(tree: &skein_ui::Tree, role: skein_ui::Role, name: &[u8]) -> Option<skein_ui::NodeId> {
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return Some(node.id);
        }
    }
    None
}
pub fn root_in_project(project: Option<u32>) -> temper_client_domain::Domain {
    root_with_watch(project).0
}
pub fn root_with_watch(project: Option<u32>) -> (temper_client_domain::Domain, skein_lib::Token) {
    let (root, me, _) = root_at(
        temper_client_domain::Address::Jig(jig_client_view::address::Address::Inbox { project }),
        &domain_limits(),
    );
    (root, me)
}
fn root_at(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, skein_lib::Token, Option<skein_lib::Token>) {
    use jig_client::contract::{
        self, Delivery, DocumentText, HeldRole, Me, MeParty, MeProject, Needs, PartyKind, Subject, WatchEvent,
    };
    use temper_client_domain::{Event, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut root = temper_client_domain::unlinked(limits, SEED);
    let mut out = Queue::with_capacity(temper_client_domain::max_out(limits));
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Start {
            address,
            kept: List::with_capacity(0),
            offset: jig_client_view::boundary::Offset { minutes: 0 },
        },
        &mut out,
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        temper_client_domain::step(&mut root, &env, Event::Client { event }, &mut out);
    }
    let mut watch = None;
    let mut inbox_watch = None;
    for _ in 0..out.len() {
        if let Some(Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(subject) },
                },
        }) = out.pop()
        {
            if subject == Subject::Me {
                watch = Some(token);
            } else if let Subject::Inbox { .. } = subject {
                inbox_watch = Some(token);
            }
        }
    }
    let mut projects = List::with_capacity(2);
    projects
        .push(MeProject {
            project: PROJECT,
            name: Some(DocumentText::complete(contract::Text::new(Box::from(*b"Shop"), 32).expect("name fits"))),
            role: HeldRole::Policy { role: 1 },
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
        })
        .expect("one project");
    let mut second = projects.as_slice()[0].clone();
    second.project = SECOND_PROJECT;
    second.name = Some(DocumentText::complete(contract::Text::new(Box::from(*b"Second"), 32).expect("name fits")));
    projects.push(second).expect("second project");
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("frame opened"),
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(Me {
                        party: MeParty {
                            number: 73,
                            kind: PartyKind::Person,
                            login: DocumentText::complete(
                                contract::Text::new(Box::from(*b"pat"), 32).expect("login fits"),
                            ),
                            name: DocumentText::complete(
                                contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits"),
                            ),
                        },
                        projects,
                        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                        needs_total: 0,
                        updates: contract::Count { value: 0, limited: false },
                        results_seen: 0,
                        cut: false,
                        retention: Duration::from_secs(1000),
                        deployment_role: None,
                    }))),
                },
            },
        },
        &mut out,
    );
    (root, watch.expect("me opened"), inbox_watch)
}
pub fn adoption_root() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(
        &mut root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Adopt { project: PROJECT }),
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

pub fn signed_in_with_project() -> crate::View {
    let mut view = view();
    build(&mut view, &root_in_project(Some(PROJECT)));
    view
}
pub fn link_names(view: &crate::View) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(view.limits().tree.nodes);
    for node in view.tree().nodes() {
        if node.role == skein_ui::Role::Link
            && let Some(name) = &node.name
        {
            names.push(skein_lib::bytes::copy_of(name)).expect("tree bounds link names");
        }
    }
    names.into_boxed()
}
pub fn banner_statuses(tree: &skein_ui::Tree) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(tree.len());
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == skein_ui::Role::Banner {
            let end = index.checked_add(usize::try_from(node.size).expect("banner fits")).expect("banner end fits");
            for child in &tree.nodes()[index..end] {
                if child.role == skein_ui::Role::Status
                    && let Some(name) = &child.name
                {
                    names.push(skein_lib::bytes::copy_of(name)).expect("tree bounds statuses");
                }
            }
        }
    }
    names.into_boxed()
}

pub const INBOX_TASK: u64 = 9;
pub const INBOX_REPLY_TASK: u64 = 10;
pub const INBOX_QUESTION: u64 = 1;
pub const INBOX_POSITION: u64 = 1;
pub fn inbox_root() -> temper_client_domain::Domain {
    inbox_root_filtered(temper_client_domain::address::Filter::ALL)
}
pub fn inbox_root_filtered(filter: temper_client_domain::address::Filter) -> temper_client_domain::Domain {
    inbox_with_document(filter, None)
}
pub fn inbox_with_document(
    filter: temper_client_domain::address::Filter,
    document: Option<(jig_client::contract::Read, jig_client::contract::ReadValue)>,
) -> temper_client_domain::Domain {
    use jig_client::contract::{self, InboxGroup as Group, ObjectKey as Object};
    let mut limits = domain_limits();
    limits.jig.objects = 6;
    let (mut root, _, watch) = root_at(temper_client_domain::Address::Inbox { filter }, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits));
    let mut entries = List::with_capacity(6);
    for (object, group) in [
        (Object::Question { task: INBOX_TASK, message: INBOX_QUESTION }, Group::Questions),
        (
            Object::Proposal(contract::ProposalKey {
                proposer: contract::Proposer::Task(INBOX_TASK),
                number: INBOX_QUESTION,
            }),
            Group::Proposals,
        ),
        (Object::Escalation(INBOX_TASK), Group::Escalations),
        (Object::PersonTask(INBOX_TASK), Group::PersonTasks),
        (Object::Result(INBOX_TASK), Group::Results),
        (Object::Task(INBOX_REPLY_TASK), Group::Replies),
    ] {
        entries
            .push(contract::InboxEntry {
                reply: None,
                lineage: List::with_capacity(0),
                goal: None,
                recipient: None,
                stall: None,
                object,
                project: PROJECT,
                group,
                position: INBOX_POSITION,
                at: Wall::EPOCH,
            })
            .expect("six kinds");
    }
    let inbox = contract::Inbox {
        entries,
        needs: contract::Needs { questions: 1, proposals: 1, escalations: 1, person_tasks: 1 },
        updates: contract::Count { value: 2, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: false,
    };
    temper_client_domain::step(
        &mut root,
        &env,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("inbox watch"),
                    event: contract::WatchEvent::Delivered(contract::Delivery::Inbox(Box::new(inbox))),
                },
            },
        },
        &mut out,
    );
    if let Some((read, page)) = document {
        while let Some(request) = out.pop() {
            if let temper_client_domain::Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(what) },
                    },
            } = request
                && what == read
            {
                root_event(
                    &mut root,
                    temper_client_domain::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read: token,
                                result: jig_client::Readout::Jig(contract::ReadResult::Value(page)),
                            },
                        },
                    },
                );
                break;
            }
        }
    }
    root
}
