use jig_client::contract as c;
use jig_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Token, Wall, Writer};
use temper_client_domain::{Limits, forge};
use temper_client_protocol_wire::{self as wire, v1 as w};
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const PROJECT: u32 = 1;
pub const SECOND_PROJECT: u32 = 2;
pub const SEED: u64 = 11;
pub fn limits() -> Limits {
    {
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        let client = jig_client::Limits {
            domain: jig_client::DomainLimits {
                owners: &[jig_client::Spoken {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                }],
                requests,
                ask_bytes: requests.bytes,
                handles: 9,
                watches: 9,
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 8,
                parts: 8,
                delegates: 2,
                document_bytes: 65536,
                records: 2,
                kept: 4,
                projects: 2,
                text: 32,
                rows: 2,
                fields: 4,
                segments: 9,
            },
            exchanges: 8,
        };

        Limits {
            forge: forge::Limits { field_bytes: 256, checks: 8 },
            client,
            accounts: temper_client_domain::limits::AccountLimits { accounts: 16, sources: 8, uses: 8, sign_ins: 4 },
        }
    }
}
pub fn words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Terminal)).clone()
}
pub fn browser_words() -> crate::words::Words {
    static WORDS: std::sync::OnceLock<crate::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| english(crate::words::Shell::Browser)).clone()
}
fn english(shell: crate::words::Shell) -> crate::words::Words {
    crate::words::Words::english(FORGE, 64, shell).expect("forge English fits")
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
pub fn part_tree(words: &crate::words::Words, part: Part) -> skein_ui::Tree {
    let mut builder = skein_ui::Builder::new(&tree_limits());
    crate::forge::drawn(&mut builder, &part, words);
    builder.finish()
}
pub fn part_text(words: &crate::words::Words, part: Part) -> Box<[u8]> {
    text(&part_tree(words, part))
}
pub fn built() -> crate::View {
    let domain = domain_limits();
    let mut root = temper_client_domain::unlinked(&domain, SEED);
    root_event(
        &mut root,
        temper_client_domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } },
    );
    let forge = words();
    let envelope = crate::Limits::of(&domain, &forge).expect("composed envelope fits");
    assert!(envelope.tree.nodes >= 128);
    let limits = crate::Limits { tree: tree_limits() };
    let mut view = crate::View::new(&limits, &domain, forge, FORGE);
    crate::build(&mut view, &root, &mut Queue::with_capacity(limits.tree.patches));
    view
}
pub fn view() -> crate::View {
    view_of(words())
}
fn view_of(words: crate::words::Words) -> crate::View {
    let domain = domain_limits();
    let limits = crate::Limits { tree: tree_limits() };
    crate::View::new(&limits, &domain, words, FORGE)
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
pub fn root_with_watch(project: Option<u32>) -> (temper_client_domain::Domain, Token) {
    let (root, me, _) = root_at(
        temper_client_domain::Address::Inbox {
            filter: temper_client_domain::address::Filter { project, ..temper_client_domain::address::Filter::ALL },
        },
        &domain_limits(),
    );
    (root, me)
}
pub(super) fn root_at(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, Token, Option<Token>) {
    let (root, me, page, _) = root_at_with_tree(address, limits);
    (root, me, page)
}
pub(super) fn root_at_with_tree(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>) {
    root_at_with_tree_named(address, limits, b"Shop")
}
fn root_at_with_tree_named(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>) {
    let (root, me, page, tree, _) = root_at_full(address, limits, project_name);
    (root, me, page, tree)
}
pub fn account_root() -> (temper_client_domain::Domain, Token) {
    let (root, _, _, _, accounts) = root_at_full(temper_client_domain::Address::Accounts, &domain_limits(), b"Shop");
    (root, accounts.expect("global accounts watch"))
}
pub(super) fn root_at_full(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>) {
    root_at_with_demo(address, limits, project_name, None)
}
fn root_at_with_demo(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    demo: Option<(u32, bool)>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>) {
    use jig_client::contract::{Delivery, Subject, WatchEvent};
    use temper_client_domain::{Event, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut root = temper_client_domain::unlinked(limits, SEED);
    let mut out = Queue::with_capacity(temper_client_domain::max_out(limits));
    temper_client_domain::step(&mut root, &env, Event::Start { address, kept: List::with_capacity(0) }, &mut out);
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        temper_client_domain::step(&mut root, &env, Event::Client { event }, &mut out);
    }
    let mut watch = None;
    let mut inbox_watch = None;
    let mut tree_watch = None;
    let mut account_watch = None;
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
            } else if subject == Subject::Accounts {
                account_watch = Some(token);
            } else if let Subject::Tree { .. } = subject {
                tree_watch = Some(token);
            } else if let Subject::Inbox { .. } | Subject::Task { .. } = subject {
                inbox_watch = Some(token);
            }
        }
    }
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("frame opened"),
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(fixture_me(project_name)))),
                },
            },
        },
        &mut out,
    );
    if let Some((project, model)) = demo {
        let mut read = None;
        while let Some(request) = out.pop() {
            if let Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read: token, what: jig_client::Readable::Family { .. } },
                    },
            } = request
            {
                read = Some(token);
            }
        }
        deliver_demo(
            &mut root,
            read.expect("live link reads demo"),
            account_watch.expect("accounts watch"),
            project,
            model,
        );
    }
    (root, watch.expect("me opened"), inbox_watch, tree_watch, account_watch)
}
pub fn adoption_root() -> temper_client_domain::Domain {
    adoption_page(PROJECT, b"shop")
}
pub fn adoption_page(project: u32, name: &[u8]) -> temper_client_domain::Domain {
    root_at_with_tree_named(temper_client_domain::Address::Adopt { project }, &domain_limits(), name).0
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
    // Six card reads beside the frame's own bounded reads.
    limits.client.domain.reads = 8;
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

pub mod task;

pub fn signed_out() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(&mut root, temper_client_domain::Event::SignedOut);
    root
}
pub fn out_of_date() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
        },
    );
    root
}
pub fn main_text(tree: &skein_ui::Tree) -> Box<[u8]> {
    let mut main = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == skein_ui::Role::Main {
            let end = index
                .checked_add(usize::try_from(node.size).expect("fixture subtree fits"))
                .expect("fixture index fits");
            main = tree.nodes().get(index..end);
        }
    }
    let main = main.expect("fixture main");
    let mut bytes = 0_usize;
    for node in main {
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).expect("fixture text fits");
        }
    }
    let mut writer = Writer::new(bytes);
    for node in main {
        for run in &node.text {
            writer.put(&run.text).expect("main text counted");
        }
    }
    writer.finish()
}

/// Shared chat setup for the focused view stories.
pub mod chat {
    use super as fixtures;
    use jig_client::{contract as c, thread::Thread};
    use skein_lib::{List, Wall};
    use skein_ui::Builder;
    pub const TASK: u64 = 9;
    pub const ATTEMPT: u64 = 23;
    pub const TURN: u32 = 1;
    pub const RETRY: u32 = 0;
    pub const OFFSET: u32 = 0;
    pub const COMMITTED: u32 = 0;
    pub const LINES: u32 = 4;
    pub const BYTES: u64 = 65536;
    pub const TEXT: u32 = 32;
    pub const MESSAGE: u64 = 3;
    pub const PARTY: u64 = 73;
    pub const DELEGATE: u64 = 12;
    pub const OTHER_DELEGATE: u64 = 13;
    pub const CLOSE_GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(5);
    pub const CLOSE_OPERATIONS: u32 = 3;
    pub const FRACTIONAL_GRACE: skein_lib::Duration = skein_lib::Duration::from_millis(125);
    pub const GRACES: [(Option<skein_lib::Duration>, &[u8]); 3] = [
        (Some(CLOSE_GRACE), b"allow 5 seconds to finish, then cancel"),
        (Some(FRACTIONAL_GRACE), b"allow 125000000 nanoseconds to finish, then cancel"),
        (None, b"Close chat now"),
    ];
    pub const CHAT_LABEL: u32 = 7;
    pub const CHAT_TEMPLATE: u32 = 1;
    pub const CHAT_CHARTER: u32 = 1;
    pub const SINGLE_TEMPLATE: u32 = 1;
    pub const EMPTY_PARAMETERS: u32 = 0;
    const MESSAGING_OFFERS: u32 = 2;
    pub const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(DELEGATE), number: 3 };
    pub const PROPOSAL_OUTPUTS: u32 = 256;
    pub const PROPOSAL_NOW: skein_lib::Time = skein_lib::Time::ZERO;
    pub const PROPOSAL_OFFERS: u32 = 2;
    pub const EMPTY_BATCH: u32 = 0;

    pub fn chat_header() -> skein_ui::Tree {
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
        crate::pages::chat::main(&mut b, &chat_root(), &fixtures::words(), &mut bindings, LINES, TEXT);
        b.finish()
    }
    pub fn provisional() -> Thread {
        provisional_with_text(b"Hello")
    }
    pub fn provisional_with_text(words: &[u8]) -> Thread {
        let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT });
        let text = c::RunText {
            attempt: ATTEMPT,
            began: Some(Wall::EPOCH),
            turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: TURN, retry: RETRY },
            from: OFFSET,
            through: u32::try_from(words.len()).expect("run text size fits"),
            text: c::Text::new(Box::from(words), u32::try_from(BYTES).expect("byte bound fits")).expect("text"),
            cut: false,
            call: None,
        };
        thread.run(
            &c::RunState { task: TASK, attempt: Some(ATTEMPT), committed_turn: COMMITTED, writing: None, call: None },
            Some(&text),
        );
        thread
    }
    pub fn tree(thread: &Thread) -> skein_ui::Tree {
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::pages::chat::conversation(&mut b, thread, &fixtures::words(), LINES, TEXT);
        b.finish()
    }
    pub fn closing(grace: Option<skein_lib::Duration>) -> skein_ui::Tree {
        let mut operations = List::with_capacity(CLOSE_OPERATIONS);
        let reason = c::Text::new(Box::from(*b"Chat closed"), TEXT).expect("reason");
        operations
            .push(c::Operation::Cancel {
                project: fixtures::PROJECT,
                task: DELEGATE,
                reason: reason.clone(),
                grace: None,
            })
            .expect("delegate cancel");
        operations
            .push(c::Operation::Move { project: fixtures::PROJECT, task: OTHER_DELEGATE, reason: reason.clone() })
            .expect("delegate move");
        operations
            .push(c::Operation::Cancel { project: fixtures::PROJECT, task: TASK, reason, grace })
            .expect("chat close");
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::confirm::close_choices(
            &mut b,
            TASK,
            &operations,
            &fixtures::words(),
            &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        );
        b.finish()
    }
    pub fn chat_root() -> temper_client_domain::Domain {
        chat_offering(false)
    }
    /// `chat_root`'s, with the message act offered beside the close.
    pub fn messaging_root() -> temper_client_domain::Domain {
        chat_offering(true)
    }
    fn chat_offering(message: bool) -> temper_client_domain::Domain {
        use temper_client_domain::{Address, Event};
        let (mut root, me_watch, task_watch) =
            fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
        let mut me = root.pages().me(root.client()).expect("me").clone();
        let member = me.projects.get_mut(EMPTY_PARAMETERS).expect("project");
        member.templates = List::with_capacity(SINGLE_TEMPLATE);
        member
            .templates
            .push(c::Template {
                number: CHAT_TEMPLATE,
                name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), TEXT).expect("name")),
                executor: c::TemplateExecutor::Charter(CHAT_CHARTER),
                contract: c::TaskContractDocument::Report { words: TEXT },
                shallower: SINGLE_TEMPLATE,
                budget: c::TemplateBudget::AsAllowed { cap: None },
                deadline: c::TemplateDeadline::None,
                tracked: None,
                flags: c::TaskFlags { conversation: true, listed: false, retention: c::Retention::Kept },
                label: CHAT_LABEL,
                grace: Some(CLOSE_GRACE),
                llm: None,
            })
            .expect("template");
        let mut task = fixtures::task::document();
        task.chip.task = TASK;
        task.project = fixtures::PROJECT;
        task.acts = List::with_capacity(if message { MESSAGING_OFFERS } else { SINGLE_TEMPLATE });
        if message {
            task.acts
                .push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None })
                .expect("message");
        }
        task.acts
            .push(c::Offered {
                act: c::ActKind::Cancel,
                admission: c::Admission::Allowed,
                effect: Some(c::Effect::Cancel(c::Impact {
                    tasks: c::PhaseCounts {
                        waiting: 0,
                        running: 1,
                        held: 0,
                        closing: 0,
                        ended: 0,
                        done: 0,
                        failed: 0,
                        cancelled: 0,
                    },
                    live_runs: 1,
                    proposals: 0,
                })),
            })
            .expect("close");
        for (watch, document) in [
            (me_watch, c::Delivery::Me(Box::new(me))),
            (task_watch.expect("chat watch"), c::Delivery::Task(Box::new(task))),
        ] {
            fixtures::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(document) },
                    },
                },
            );
        }
        root
    }
    pub fn proposal_root() -> temper_client_domain::Domain {
        use temper_client_domain::Event;
        let limits = fixtures::domain_limits();
        let (mut root, _, watch) = fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &limits);
        let mut task = fixtures::task::document();
        task.chip.task = TASK;
        task.proposals = List::with_capacity(SINGLE_TEMPLATE);
        task.proposals.push(PROPOSAL).expect("one proposal");
        let mut out = skein_lib::Queue::with_capacity(PROPOSAL_OUTPUTS);
        temper_client_domain::step(
            &mut root,
            &skein_lib::Env { now: PROPOSAL_NOW, wall: Wall::EPOCH, limits },
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.expect("task watch"),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
            &mut out,
        );
        let mut acts = List::with_capacity(PROPOSAL_OFFERS);
        for act in [c::ActKind::Accept, c::ActKind::Reject] {
            acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("offer");
        }
        let page = c::ReadValue::OwnerProposal(Box::new(c::OwnerProposal {
            key: PROPOSAL,
            project: fixtures::PROJECT,
            reason: c::DocumentText::complete(c::Text::new(Box::from(*b"Make a goal"), TEXT).expect("reason")),
            action: c::OwnerProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
            as_holder: true,
            waiting: c::OwnerProposalWaiting { holder: c::OwnerHistoryHolder::Person(PARTY), since: Wall::EPOCH },
            funding: List::with_capacity(EMPTY_BATCH),
            acts,
        }));
        let mut answered = false;
        while let Some(request) = out.pop() {
            if let temper_client_domain::Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            jig_client::Down::Read {
                                read,
                                what: jig_client::Readable::Jig(c::Read::Proposal { proposal: PROPOSAL }),
                            },
                    },
            } = request
            {
                fixtures::root_event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(page)),
                            },
                        },
                    },
                );
                answered = true;
                break;
            }
        }
        assert!(answered, "proposal read emitted");
        root
    }
    pub fn rejecting_chat() -> temper_client_domain::Domain {
        use temper_client_domain::{Action, Event, pages::Act};
        let mut root = proposal_root();
        let object = c::ObjectKey::Proposal(PROPOSAL);
        for act in [
            Act::CardWords { object, words: Box::from(*b"Too costly") },
            Act::Decide { object, act: c::ActKind::Reject, revision: None },
        ] {
            fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
        }
        root
    }
}

pub fn chats_root() -> temper_client_domain::Domain {
    root_at(temper_client_domain::Address::Chats, &domain_limits()).0
}
/// Act `Input` on the named text area, as a person typing, and return the area's value after the root took it.
pub fn type_into(root: &mut temper_client_domain::Domain, name: &[u8], words: &[u8]) -> Box<[u8]> {
    let mut view = page_view(root);
    let area = find(view.tree(), skein_ui::Role::TextArea, name).expect("text area");
    let action = crate::act(&mut view, area, skein_ui::Act::Input { value: Box::from(words) }).expect("input action");
    root_event(root, temper_client_domain::Event::Act { action });
    let view = page_view(root);
    for node in view.tree().nodes() {
        if node.role == skein_ui::Role::TextArea && node.name.as_deref() == Some(name) {
            return node.value.as_ref().expect("area value").text.clone();
        }
    }
    panic!("text area");
}
/// The styles of each node of `role`, in tree order.
pub fn styles(tree: &skein_ui::Tree, role: skein_ui::Role) -> Box<[Box<[skein_ui::Style]>]> {
    let mut found = List::with_capacity(u32::try_from(tree.nodes().len()).expect("nodes fit"));
    for node in tree.nodes() {
        if node.role == role {
            found.push(Box::from(node.styles.as_slice())).expect("a node per style list");
        }
    }
    found.into_boxed()
}
/// Build a page once for the status-only page tests.
pub fn page_view(root: &temper_client_domain::Domain) -> crate::View {
    let mut view = view();
    build(&mut view, root);
    view
}
/// `page_view`'s, with the browser's words.
pub fn browser_page_view(root: &temper_client_domain::Domain) -> crate::View {
    let mut view = view_of(browser_words());
    build(&mut view, root);
    view
}

fn fixture_me(project_name: &[u8]) -> c::Me {
    use jig_client::contract::{self, DocumentText, HeldRole, Me, MeParty, MeProject, Needs, PartyKind};
    let mut projects = List::with_capacity(2);
    projects
        .push(MeProject {
            project: PROJECT,
            name: Some(DocumentText::complete(
                contract::Text::new(skein_lib::bytes::copy_of(project_name), 32).expect("name fits"),
            )),
            role: HeldRole::Policy { role: 1 },
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
            models: List::with_capacity(0),
        })
        .expect("one project");
    let mut second = projects.as_slice()[0].clone();
    second.project = SECOND_PROJECT;
    second.name = Some(DocumentText::complete(contract::Text::new(Box::from(*b"Second"), 32).expect("name fits")));
    projects.push(second).expect("second project");
    Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        party: MeParty {
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
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: None,
    }
}
pub const DEMO_CUE: &[u8] = b"Change the title";
pub const DEMO_OTHER_CUE: &[u8] = b"Add a test";
pub fn demo_root(address: temper_client_domain::Address, project: u32, model: bool) -> temper_client_domain::Domain {
    let (mut root, me_watch, _, _, _) =
        root_at_with_demo(address, &domain_limits(), b"Project", Some((project, model)));
    let template = chat::messaging_root();
    let mut me = template.pages().me(template.client()).expect("chat template").clone();
    let mut charters = List::with_capacity(1);
    charters.push(1).expect("chat charter");
    me.projects.get_mut(0).expect("demo project").creating = Some(c::Creating {
        within: c::CreateBounds {
            authority: Box::new(c::OwnerAuthorityDocument::complete(c::OwnerAuthority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::OwnerDelegation { kinds: List::with_capacity(0), tasks: 1, depth: 2 },
                spend: 100,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            })),
            pool_left: 100,
            charters,
            procedures: List::with_capacity(0),
            latest: None,
            budget: 100,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: c::Beyond::Refused(c::Refusal::NotOffered),
    });
    root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    root
}

fn deliver_demo(root: &mut temper_client_domain::Domain, read: Token, watch: Token, project: u32, model: bool) {
    let mut cues = List::with_capacity(2);
    cues.push(Box::from(DEMO_CUE)).expect("first cue");
    cues.push(Box::from(DEMO_OTHER_CUE)).expect("second cue");
    let answer = w::DemoAnswer::new(
        &w::CEILINGS,
        w::DemoAnswerParts {
            project,
            forge: wire::FORGE,
            provider: 17,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            cues,
        },
    )
    .expect("demo answer");
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::demo_answer(&answer)),
                    },
                },
            },
        },
    );
    let accounts = c::OwnerAccounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts: List::with_capacity(0),
        signing: List::with_capacity(0),
        last_end: None,
        usable: false,
        settled: true,
        cut: !model,
    };
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(accounts))),
                },
            },
        },
    );
}
