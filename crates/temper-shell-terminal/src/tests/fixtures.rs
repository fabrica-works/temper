use jig_domain_client::contract;
use jig_protocol_client_link as link;
use skein_lib::{RequestRecord, Token};
use temper_client_domain as client;
pub const ATTEMPT: Token = Token::new(101);
pub const READ: Token = Token::new(102);
pub const WATCH: Token = Token::new(103);
pub const SERVICE: u64 = 7;
pub const CREDENTIAL: &[u8] = b"one-time-service-credential";
pub fn jig_record() -> jig_domain_client::store::Record {
    jig_domain_client::store::Record::Place {
        version: jig_domain_client::store::STORE_VERSION,
        party: SERVICE,
        address: jig_domain_client::address::Address::Inbox { project: None },
    }
}
pub fn table_record() -> client::store::Record {
    client::store::Record::Table {
        version: client::store::VERSION,
        record: RequestRecord {
            key: [9; 16],
            scope: SERVICE,
            first_sent: None,
            request: client::Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })),
        },
    }
}
pub fn inbox_read() -> contract::Read {
    contract::Read::Inbox { project: None, group: contract::InboxReadGroup::Needs, kind: None, goal: None, after: None }
}
pub fn family() -> crate::route::family::Family {
    crate::limits::family(&crate::limits::native()).expect("bounded fixture family")
}
pub fn wire_limits() -> temper_protocol_client_wire::v1::Limits {
    family().wire
}
pub fn adoption() -> client::forge::Adoption {
    client::forge::Adoption {
        project: 1,
        forge: 1,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        prefix: Box::from(*b"temper/"),
        landing: Box::from(*b"main"),
        role: client::forge::Role::Owned,
        home: true,
        ci: true,
        checks: Box::from([1, 2]),
    }
}
pub fn adopted(protection: client::forge::Protection) -> client::forge::Adopted {
    client::forge::Adopted {
        forge: 1,
        repository: 7,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        role: client::forge::Role::Owned,
        home: true,
        kinds: client::forge::Kinds {
            read: true,
            push: false,
            open: true,
            land: false,
            review: true,
            status: false,
            comment: true,
            issue: false,
            branch: true,
        },
        protection,
    }
}
pub fn send_adoption(attempt: Token) -> client::Request {
    client::Request::Send { attempt, key: [9; 16], ask: client::Ask::Forge(client::forge::Ask::Adopt(adoption())) }
}
pub fn final_answer(attempt: Token, answer: link::Answered) -> link::Request {
    link::Request::Answer { attempt, envelope: skein_lib::RequestEnvelope::Final, answer: Some(answer) }
}
pub fn role_refusal() -> contract::Refusal {
    contract::Refusal::Role { held: None }
}
pub fn entrance(refusal: contract::Refusal) -> contract::EntranceRefusal {
    contract::EntranceRefusal::new(refusal).expect("fixed entrance facts")
}
pub fn core_refusal() -> Box<[u8]> {
    use jig_protocol_client_wire as w;
    let limits = family().link.wire;
    let value = w::Refusal::Role(w::RefusalRole::new(&limits, w::RefusalRoleParts { held: None }).expect("role fits"));
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measure fits"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

pub fn tree_limits() -> jig_view::Limits {
    jig_view::Limits {
        nodes: 1024,
        depth: 48,
        text_per_node: 1024,
        text_bytes: 1_048_576,
        drawing_bytes: 16,
        styles_per_node: 2,
        patches: 5120,
        bindings: 1024,
        slots: 256,
        markdown_depth: 3,
    }
}
pub fn bounds() -> crate::render::Bounds {
    crate::render::bounds(&tree_limits()).expect("fixture print bounds")
}
pub fn frame() -> crate::render::Frame {
    crate::render::Frame { behind: Box::from(*b"Behind: waiting for current facts") }
}
pub fn golden(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/golden").join(format!("{name}.txt"))
}
pub fn visible(mut tree: jig_view::Tree) -> jig_view::Tree {
    let limits = tree_limits();
    jig_view::diff(
        &jig_view::Tree::new(limits.nodes),
        &mut tree,
        &limits,
        &mut 0,
        &mut skein_lib::Queue::with_capacity(limits.patches),
    );
    tree
}
pub fn copy_tree(source: &jig_view::Tree) -> jig_view::Tree {
    let mut builder = jig_view::Builder::new(&tree_limits());
    let mut open = Vec::new();
    for (index, node) in source.nodes().iter().enumerate() {
        while open.last().is_some_and(|end| *end == index) {
            builder.close();
            open.pop();
        }
        builder.open(node.role);
        if let Some(name) = &node.name {
            builder.name(name);
        }
        for state in &node.states {
            builder.state(*state);
        }
        for run in &node.text {
            builder.run(run.run, &run.text);
        }
        if let Some(value) = &node.value {
            builder.value(&value.text, value.written);
        }
        if let Some(binding) = node.binding {
            builder.bind(binding);
        }
        if let Some(link) = node.link {
            builder.link(link);
        }
        if let Some(external) = &node.external {
            builder.external(external);
        }
        open.push(index + usize::try_from(node.size).expect("subtree fits"));
    }
    for _ in open {
        builder.close();
    }
    visible(builder.finish())
}
pub fn page(address: jig_domain_client::address::Address, signed_out: bool) -> jig_view::Tree {
    use jig_domain_client::{
        Event as JigEvent,
        contract::{Delivery, ReadResult, Subject, WatchEvent},
    };
    use skein_lib::{Env, Queue, Wall};
    use temper_client_domain::{Event, JigOut, Request};
    let mut limits = crate::limits::native().client;
    limits.jig.text = 32;
    limits.jig.rows = 8;
    let env = Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits };
    let (jig, forge) = page_words();
    let mut view =
        temper_client_view::View::new(&temper_client_view::Limits { tree: tree_limits() }, &limits, jig, forge, 1);
    let mut root = client::new(&limits, 16, client::forge::Config { connector: 1, version: 1 });
    root_navigation(&mut root, &limits);
    let mut out = Queue::with_capacity(client::max_out(&limits));
    client::step(
        &mut root,
        &env,
        Event::Start {
            address: client::Address::Jig(address),
            kept: skein_lib::List::with_capacity(0),
            offset: jig_domain_client::boundary::Offset { minutes: 0 },
        },
        &mut out,
    );
    if signed_out {
        client::step(&mut root, &env, Event::SignedOut, &mut out);
    } else {
        client::step(
            &mut root,
            &env,
            Event::Jig(JigEvent::Hello { versions: contract::VersionRange { first: 7, last: 7 } }),
            &mut out,
        );
        for _ in 0..128 {
            let Some(request) = out.pop() else {
                break;
            };
            match request {
                Request::Jig(JigOut::Read { read, what }) => {
                    let value = read_value(&what);
                    if let Some(value) = value {
                        let result = ReadResult::Value(value);
                        client::step(&mut root, &env, Event::Jig(JigEvent::Read { read, result }), &mut out);
                    }
                }
                Request::Jig(JigOut::Open { watch, subject } | JigOut::Continue { watch, subject }) => {
                    client::step(
                        &mut root,
                        &env,
                        Event::Jig(JigEvent::Watch { watch, event: WatchEvent::Opened }),
                        &mut out,
                    );
                    let delivery = match subject {
                        Subject::Frame => Some(Delivery::Frame(Box::new(jig_client_world::stories::frame(1)))),
                        Subject::Task { .. } => Some(Delivery::Task(Box::new(jig_client_world::stories::task()))),
                        Subject::Deployment => {
                            Some(Delivery::Deployment(Box::new(jig_client_world::stories::deployment())))
                        }
                        Subject::Tree { .. }
                        | Subject::Chips { .. }
                        | Subject::Inbox { .. }
                        | Subject::Goals { .. }
                        | Subject::Tasks { .. }
                        | Subject::Conversations
                        | Subject::Run { .. } => None,
                    };
                    if let Some(delivery) = delivery {
                        client::step(
                            &mut root,
                            &env,
                            Event::Jig(JigEvent::Watch { watch, event: WatchEvent::Delivered(delivery) }),
                            &mut out,
                        );
                    }
                }
                Request::Show { address: next, .. } => {
                    if next != client::Address::Jig(address) {
                        client::step(&mut root, &env, Event::Went { place: client::Place::Address(next) }, &mut out);
                    }
                }
                Request::Save { .. }
                | Request::Erase { .. }
                | Request::Send { .. }
                | Request::Jig(JigOut::Close { .. } | JigOut::Answered { .. } | JigOut::Unknown { .. })
                | Request::SignIn { .. }
                | Request::SignOut
                | Request::Fact { .. } => {}
            }
        }
        assert!(out.is_empty(), "fixture settles within its step budget: {out:?}");
    }
    temper_client_view::build(&mut view, &root, &mut Queue::with_capacity(tree_limits().patches));
    copy_tree(view.tree())
}
fn root_navigation(root: &mut client::Domain, limits: &client::Limits) {
    client::configure_navigation(root, client::navigation(limits)).expect("fixture navigation");
}
fn page_words() -> (jig_view::words::Words, temper_client_view::words::Words) {
    static WORDS: std::sync::OnceLock<(jig_view::words::Words, temper_client_view::words::Words)> =
        std::sync::OnceLock::new();
    WORDS
        .get_or_init(|| {
            (
                jig_view::words::Words {
                    book: jig_view::phrases::english::english(32).expect("fixture English"),
                    language: 0,
                    offset_minutes: 0,
                },
                temper_client_view::words::Words::english(1, 32).expect("fixture forge English"),
            )
        })
        .clone()
}
pub fn inbox_document(group: contract::InboxReadGroup) -> contract::InboxPage {
    use contract::{InboxEntry, InboxGroup, ObjectKey};
    let mut rows = skein_lib::List::with_capacity(6);
    for (kind, object) in [
        (InboxGroup::Questions, ObjectKey::Question { task: 9, message: 1 }),
        (
            InboxGroup::Proposals,
            ObjectKey::Proposal(contract::ProposalKey { proposer: contract::Proposer::Task(9), number: 1 }),
        ),
        (InboxGroup::Escalations, ObjectKey::Escalation(9)),
        (InboxGroup::PersonTasks, ObjectKey::PersonTask(9)),
        (InboxGroup::Results, ObjectKey::Result(9)),
        (InboxGroup::Replies, ObjectKey::Task(10)),
    ] {
        if contract::InboxReadGroup::of(kind) == group {
            rows.push(InboxEntry {
                reply: None,
                lineage: skein_lib::List::with_capacity(0),
                goal: None,
                recipient: None,
                stall: None,
                object,
                project: 1,
                group: kind,
                position: 1,
                at: skein_lib::Wall::EPOCH,
            })
            .expect("six inbox groups");
        }
    }
    contract::InboxPage { cut: false, rows, before: None, after: None }
}
pub fn inbox_page() -> jig_view::Tree {
    page(jig_domain_client::address::Address::Inbox { project: None }, false)
}
pub fn task_page() -> jig_view::Tree {
    page(
        jig_domain_client::address::Address::Task {
            task: 9,
            section: jig_domain_client::address::TaskSection::Activity,
        },
        false,
    )
}
pub fn project_page() -> jig_view::Tree {
    page(
        jig_domain_client::address::Address::Project {
            project: 1,
            section: jig_domain_client::address::ProjectSection::Overview,
        },
        false,
    )
}
pub fn deployment_page() -> jig_view::Tree {
    page(jig_domain_client::address::Address::Deployment, false)
}
pub fn signed_out_page() -> jig_view::Tree {
    page(jig_domain_client::address::Address::Inbox { project: None }, true)
}

pub fn synthetic_header(builder: &mut jig_view::Builder, title: Option<&[u8]>, behind: bool) {
    use jig_view::{Role, Run};
    builder.open(Role::Page);
    builder.open(Role::Region);
    builder.name(b"Frame");
    builder.open(Role::Status);
    builder.text(Run::Plain, b"Live");
    builder.close();
    if behind {
        builder.open(Role::Status);
        builder.text(Run::Plain, &frame().behind);
        builder.close();
    }
    builder.open(Role::Status);
    builder.text(Run::Plain, b"Signed in as Pat");
    builder.close();
    builder.close();
    if let Some(title) = title {
        builder.open(Role::Heading { level: 1 });
        builder.text(Run::Plain, title);
        builder.close();
    }
}
pub fn header_page(title: Option<&[u8]>, behind: bool) -> jig_view::Tree {
    let mut builder = jig_view::Builder::new(&tree_limits());
    synthetic_header(&mut builder, title, behind);
    builder.close();
    visible(builder.finish())
}
pub fn binding() -> jig_view::Binding {
    jig_view::Binding { owner: jig_view::Owner::Jig, token: Token::new(1) }
}
pub fn every_role() -> jig_view::Tree {
    use jig_view::{Role, Run};
    let mut builder = jig_view::Builder::new(&tree_limits());
    synthetic_header(&mut builder, Some(b"Every role"), false);
    for (role, label) in [
        (Role::Region, b"Region".as_slice()),
        (Role::Figure, b"Figure"),
        (Role::Dialog, b"Dialog"),
        (Role::List, b"List"),
        (Role::Tree, b"Tree"),
        (Role::ChoiceGroup, b"Choices"),
        (Role::Tabs, b"Tabs"),
        (Role::Log, b"Log"),
    ] {
        builder.open(role);
        builder.name(label);
        builder.text(Run::Plain, b"child");
        builder.close();
    }
    for (role, text) in [
        (Role::Heading { level: 2 }, b"Heading".as_slice()),
        (Role::Paragraph, b"Paragraph"),
        (Role::Time, b"Time"),
        (Role::Cell, b"Cell"),
        (Role::Term, b"Term"),
        (Role::Description, b"Description"),
        (Role::Alert, b"Alert"),
        (Role::Code, b"code\nsecond"),
        (Role::Quote, b"quote\nsecond"),
        (Role::Status, b"Status"),
    ] {
        builder.open(role);
        builder.text(Run::Plain, text);
        builder.close();
    }
    builder.text(Run::Plain, b"Text");
    builder.open(Role::Text);
    builder.run(Run::Code, b"run");
    builder.run(Run::Strong, b" strong");
    builder.close();
    for role in [Role::Item, Role::TreeItem] {
        builder.open(role);
        builder.open(Role::Paragraph);
        builder.text(Run::Plain, b"item\ncontinued");
        builder.close();
        builder.close();
    }
    builder.open(Role::Table);
    builder.open(Role::Row);
    for text in [b"one".as_slice(), b"two"] {
        builder.open(Role::Cell);
        builder.text(Run::Plain, text);
        builder.close();
    }
    builder.close();
    builder.close();
    role_controls(&mut builder);
    builder.close();
    visible(builder.finish())
}
fn role_controls(builder: &mut jig_view::Builder) {
    use jig_view::{Role, Run, State};
    for (role, name) in [
        (Role::Button, b"Button".as_slice()),
        (Role::Tab, b"Tab"),
        (Role::Link, b"Link"),
        (Role::TextField, b"Field"),
        (Role::TextArea, b"Area"),
        (Role::Choice, b"Choice"),
        (Role::Check, b"Check"),
    ] {
        builder.open(role);
        builder.name(name);
        builder.bind(binding());
        if role == Role::TextField {
            builder.value(b"value", 1);
            builder.state(State::SubmitsOnEnter);
        }
        if role == Role::TextArea {
            builder.value(b"", 1);
        }
        builder.close();
    }
    for (role, name, state) in
        [(Role::Choice, b"Chosen".as_slice(), State::Selected), (Role::Check, b"Checked", State::Pressed)]
    {
        builder.open(role);
        builder.name(name);
        builder.state(state);
        builder.bind(binding());
        builder.close();
    }
    builder.open(Role::Button);
    builder.name(b"All states");
    builder.bind(binding());
    for state in [
        State::Selected,
        State::Pressed,
        State::Invalid,
        State::Busy,
        State::Current,
        State::Expanded,
        State::Disabled,
        State::Provisional,
    ] {
        builder.state(state);
    }
    builder.close();
    builder.open(Role::Button);
    builder.name(b"Unbound");
    builder.close();
    builder.open(Role::Link);
    builder.text(Run::Plain, b"Outside");
    builder.external(b"https://example.test/");
    builder.close();
    builder.open(Role::Status);
    builder.name(b"Loading");
    builder.close();
    builder.open(Role::Status);
    builder.close();
    builder.open(Role::Progress);
    builder.name(b"Progress");
    builder.text(Run::Plain, b"half");
    builder.close();
    builder.open(Role::Separator);
    builder.close();
    builder.open(Role::Slot);
    builder.text(Run::Plain, b"Slot");
    builder.close();
    builder.drawing(jig_view::DrawingKind { owner: jig_view::Owner::Jig, kind: 1 }, b"Drawing", b"image", 16);
    builder.text(Run::Plain, b"fallback");
    builder.close();
    builder.open(Role::Paragraph);
    builder.text(Run::Plain, b"before");
    builder.open(Role::Link);
    builder.text(Run::Plain, b"Nested");
    builder.bind(binding());
    builder.close();
    builder.text(Run::Plain, b"after");
    builder.close();
}
pub fn control_text() -> jig_view::Tree {
    use jig_view::{Role, Run};
    let mut builder = jig_view::Builder::new(&tree_limits());
    synthetic_header(&mut builder, Some(b"Text\tchecked"), false);
    builder.open(Role::Button);
    builder.name(b"Label");
    builder.text(Run::Plain, b"Label");
    builder.bind(binding());
    builder.close();
    builder.open(Role::Paragraph);
    builder.text(Run::Plain, b"a\t b\nnext\x1b\0\x7f\xc2\x80\xff\xc0\xaf\xe2\x82");
    builder.close();
    builder.open(Role::Text);
    builder.run(Run::Emphasis, "café ".as_bytes());
    builder.run(Run::Code, b"x\ty\nz");
    builder.close();
    builder.close();
    visible(builder.finish())
}
pub fn controls() -> (jig_view::Tree, crate::render::Numbers) {
    let tree = visible(control_tree());
    let numbers = crate::render::print(&tree, &frame(), &bounds()).numbers;
    (tree, numbers)
}
pub fn control_tree() -> jig_view::Tree {
    control_tree_with(None, true)
}
pub fn control_tree_with(check_state: Option<jig_view::State>, check_bound: bool) -> jig_view::Tree {
    use jig_view::{Role, State};
    let mut builder = jig_view::Builder::new(&tree_limits());
    synthetic_header(&mut builder, Some(b"Controls"), false);
    for (role, name) in [
        (Role::Button, b"Button".as_slice()),
        (Role::Link, b"Link"),
        (Role::Tab, b"Tab"),
        (Role::Check, b"Check"),
        (Role::Choice, b"Choice"),
        (Role::TextField, b"Field"),
        (Role::TextArea, b"Area"),
    ] {
        builder.open(role);
        builder.name(name);
        if role != Role::Check || check_bound {
            builder.bind(binding());
        }
        if role == Role::Check
            && let Some(state) = check_state
        {
            builder.state(state);
        }
        if role == Role::TextField {
            builder.state(State::SubmitsOnEnter);
        }
        builder.close();
    }
    builder.open(Role::Button);
    builder.name(b"Disabled");
    builder.bind(binding());
    builder.state(State::Disabled);
    builder.close();
    builder.close();
    builder.finish()
}
fn read_value(what: &contract::Read) -> Option<contract::ReadValue> {
    use contract::{Read, ReadValue};
    match what {
        Read::Frame => Some(ReadValue::Frame(Box::new(jig_client_world::stories::frame(1)))),
        Read::Task { .. } => Some(ReadValue::Task(Box::new(jig_client_world::stories::task()))),
        Read::Project { .. } => Some(ReadValue::Project(Box::new(jig_client_world::stories::project()))),
        Read::Deployment => Some(ReadValue::Deployment(Box::new(jig_client_world::stories::deployment()))),
        Read::Proposal { .. } => Some(ReadValue::Proposal(Box::new(jig_client_world::stories::proposal()))),
        Read::Inbox { group, .. } => Some(ReadValue::Inbox(Box::new(inbox_document(*group)))),
        Read::Activity { .. } => Some(ReadValue::Activity(Box::new(activity_document()))),
        Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Conversations { .. }
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Notes { .. } => None,
    }
}
fn activity_document() -> contract::ActivityPage {
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(contract::ActivityLine {
        position: 1,
        at: skein_lib::Wall::EPOCH,
        event: contract::Activity::Words {
            by: jig_client_world::stories::party(1),
            message: 1,
            words: jig_client_world::stories::text(b"A fixture conversation"),
        },
    })
    .expect("one conversation entry");
    contract::ActivityPage { cut: false, rows, before: None, after: None }
}
