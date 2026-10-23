//! 16:52Z ruling: the host owns secret/clearing/target evidence; Chromium owns navigation and re-entry.
use super::*;
use jig_client::contract as c;
use skein_ui::{EntryKind, Role};
#[path = "host_tests_departure.rs"]
mod departure;
const KEY: &[u8] = b"private-host-key";
const ADDRESS: &[u8] = b"http://127.0.0.1:9/authorize?state=test";
fn fixture() -> Host {
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH);
    host.root = root::unlinked(&host.env.limits, 11);
    host.event(Event::Start { address: Address::Accounts, kept: List::with_capacity(0), with: None });
    host.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    host.client(jig_client::Event::SignIn { with: jig_client::With::Transport });
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    let me = host
        .out
        .iter()
        .find_map(|request| match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                    },
            } => Some(*watch),
            Request::Client { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        })
        .expect("me watch");
    let link =
        jig_client_protocol_link::Limits::new(host.env.limits.client.domain, limits::EXCHANGES).expect("link bounds");
    let mut frame =
        jig_client_protocol_link::translate::documents::frame(jig_client_test_records::frame_record(), &link)
            .expect("frame");
    frame.providers = List::with_capacity(1);
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(
                c::Text::new(Box::from(b"openai".as_slice()), 32).expect("provider name fits"),
            ),
            key_bytes: 4096,
            sign_in: true,
            signed_in_plan: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .expect("one offered provider");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    });
    host.requests();
    host
}
fn fixture_at(path: &[u8]) -> Host {
    let mut host = fixture();
    host.went(path);
    host.requests();
    host
}
fn build(host: &mut Host) -> Vec<Patch> {
    host.patches()
}
fn node(host: &Host, role: Role, name: &[u8]) -> NodeId {
    host.view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("account control")
        .id
}
fn input(host: &mut Host, name: &[u8], value: &[u8]) -> Vec<Patch> {
    let field = node(host, Role::TextField, name);
    host.act(field, Act::Input { value: Box::from(value) });
    build(host)
}
fn send(host: &mut Host, name: &[u8]) -> Token {
    let button = node(host, Role::Button, name);
    host.act(button, Act::Press);
    host.client(jig_client::Event::Stored);
    host.out
        .iter()
        .find_map(|request| match request {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                Some(*attempt)
            }
            Request::Client { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        })
        .expect("account request left the host")
}
#[test]
fn the_key_patch_declares_a_secret_entry() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Use an API key");
    host.act(way, Act::Press);
    host.requests();
    let patches = build(&mut host);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    let made =
        patches
            .iter()
            .find_map(|patch| {
                if let Patch::Insert { nodes, .. } = patch { nodes.iter().find(|made| made.id == key) } else { None }
            })
            .expect("inserted key field");
    assert_eq!(made.attribute.entry, Some(EntryKind::Secret));
    assert!(made.value.as_ref().expect("empty field").text.is_empty());
    let patches = input(&mut host, b"Key (pasted once)", KEY);
    assert!(!format!("{patches:?} {:?}", host.view).contains("private-host-key"));
}
#[test]
fn sending_a_key_emits_the_empty_value_patch() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Use an API key");
    host.act(way, Act::Press);
    host.requests();
    build(&mut host);
    input(&mut host, b"Key (pasted once)", KEY);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    send(&mut host, b"Add key");
    let patches = build(&mut host);
    assert!(
        patches
            .iter()
            .any(|patch| matches!(patch,Patch::Value {node,value:Some(value)} if *node==key && value.text.is_empty()))
    );
}
#[test]
fn a_key_never_reaches_the_small_store_output() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Use an API key");
    host.act(way, Act::Press);
    host.requests();
    build(&mut host);
    input(&mut host, b"Key (pasted once)", KEY);
    send(&mut host, b"Add key");
    let requests = host.requests();
    assert!(
        !requests.iter().any(|request| matches!(request, Asked::Save { .. })),
        "carried requests are not small-store writes"
    );
}
#[test]
fn the_external_href_is_the_supplied_sign_in_address() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let attempt = send(&mut host, b"Continue with ChatGPT");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountSignInStarted { source: 43, address: Box::from(ADDRESS) },
            )))),
            why: None,
        },
    });
    build(&mut host);
    let link = host
        .view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.external.as_deref() == Some(ADDRESS))
        .expect("sign-in address")
        .id;
    assert_eq!(host.href(link).as_deref(), Some(ADDRESS));
}
#[test]
fn the_accounts_href_navigates_inside_the_app() {
    let mut host = fixture();
    build(&mut host);
    let link = node(&host, Role::Link, b"Accounts");
    assert_eq!(host.href(link).as_deref(), Some(b"/accounts".as_slice()));
    host.went(b"/chats");
    build(&mut host);
    let link = node(&host, Role::Link, b"Accounts");
    host.act(link, Act::Press);
    host.requests();
    assert_eq!(host.show, Some((Box::from(b"/accounts".as_slice()), false)));
}

#[test]
fn a_filter_chosen_pushes_the_inbox_at_its_query() {
    let mut host = fixture_at(b"/");
    build(&mut host);
    let question = node(&host, Role::Button, b"Question");
    host.act(question, Act::Press);
    host.requests();
    assert_eq!(host.show.take(), Some((Box::from(b"/?kind=question".as_slice()), false)));
    build(&mut host);
    let all = node(&host, Role::Button, b"All kinds");
    host.act(all, Act::Press);
    host.requests();
    assert_eq!(host.show, Some((Box::from(b"/".as_slice()), false)));
}
#[test]
fn a_load_at_a_filtered_address_opens_the_inbox_filtered() {
    let mut host = fixture_at(b"/?kind=question");
    build(&mut host);
    assert_eq!(
        host.address,
        Address::Inbox { filter: Filter { kind: Some(root::address::Kind::Question), ..Filter::ALL } }
    );
    assert_eq!(host.show, None);
    node(&host, Role::Disclosure, "Kind · Question".as_bytes());
}
#[test]
fn a_stale_filter_link_opens_less_filtered_at_its_printed_address() {
    let host = fixture_at(b"/?kind=reply&project=3");
    assert_eq!(host.address, Address::Inbox { filter: Filter { project: Some(3), ..Filter::ALL } });
    assert_eq!(host.show, Some((Box::from(b"/?project=3".as_slice()), true)));
}

#[path = "../../../testing/memory_literal.rs"]
mod memory_literal;
// Retained domain draft: 3 x 1,048,576 document bytes + 280 B size_of::<Draft>()
// = 3,146,008 B. Base 87c82f753: 119,483,766,536 + 3,146,008 B domain draft
// + 304 added view nodes x 574,364 B = 119,661,519,200 B. A chat's 12 delegations
// (client/README.md, 5.7): 12 x 1,048,576 document bytes + 1,392 B of lists
// = 12,584,304 B, so 119,674,103,504 B. Their 12 cards (5.7): 12 x (514 markdown
// + 14) nodes x 574,364 B + 12 x 8 B of carded tasks = 3,639,170,400 B,
// so 123,313,273,904 B.
// Cards remembered across the window: 12 x 8 B = 96 B, so 123,313,274,000 B.
// Live delegates add 12 x 8 B of scratch and a 574,364 B region: 123,313,848,460 B.
// A goal's result, contract and budget card add 392,869,712 B: 123,706,718,172 B.
// Plan rows add 32 x 574,364 B, phrases 2,368 B and a tool name 256 B,
// less 8 B of root stack: +18,382,264 B, giving 123,725,100,436 B.
// Lower-case phase words add 16 phrases' room, 2,368 B: 123,725,102,804 B.
// Smaller receiving bounds and the full chat window price each turn's shared budget (client/README.md, 5.4).
// Agent page: 557 added nodes x 574,364 B + 16,782,488 B for the bounded thread
// and borrowed view lines = 336,703,236 B, giving 124,061,806,040 B.
// The context line adds one repository/forge text node; chat words use existing phrase room.
// Your chats adds filter/empty nodes and five phrases to the checked envelope.
// Procedure goal slot: five added nodes including its paragraph and link text.
// The chat projection adds seven phrases while retaining the agent's work-line words.
// Amendment editor: sixteen nodes and eleven phrases; the web root also retains
// its bounded draft, typed specification and separate amendment outcome.
// Added dependency marks (client/README.md, 5.8): two nodes per dependency,
// across two plan rows and the brief: (2 * 2 + 2) * 2 = 12 nodes x 1,674,892 B.
// Their five phrases add five entries' room, 740 B.
// Naming navigation and cards adds two phrases (296 B).
// The forge side's kept change panel (client/README.md 4.6): its handle, its box
// and the panel's bounded heap add 4,976 B.
// A task's name over its words in a chat adds one phrase (148 B).
// The four time forms add 47 phrases x 148 B = 6,956 B.
// jig 388a71ec counts reads in max_out and prices each as a PendingRead: +12,483,662,288 B.
// The change panel adds 86 forge phrases (12,728 B); its full address and
// combined pull/bounds texts fit the existing maximum text envelope.
// The procedure's ended page and its three phrases move the checked envelope.
// Parent-based plan ordering adds 56 B per bounded tree row: 3 rows, 168 B.
// skein f043ea16: b6ea7b9c's placeholder and completed state; boxed patch attributes, no slotless splice room.
// The page reserves its patch queue once, at four patches a node.
// A chat's composer placeholder adds one phrase (148 B).
// Closed-chat, confirmation and refusal words add 29 phrases (4,292 B).
// Exceeded adds nine jig phrases: 9 * 148 B = 1,332 B.
// The flow's three state words (444 B).
// The Accounts navigation's two state words (296 B).
// ConnectForge's and the welcome's forge words and nodes, and 128 more phrase segments (77,364 B).
// The chat's saved run state adds 320 B inline and three 512 B call texts (1,856 B).
// The Forges page, the setup strip and the dot's words (2,664 B).
// Wire 6's SetUp fields and the row's first repository, written by SKEIN_CODEGEN_WRITE=1 (+27,648 B).
// The setup draft, retained intent and adoption, shared keys and Ready words add 70,031 B.
// The demo start's retained state, words, failure text and retry nodes (2,509,493 B).
#[expect(clippy::unreadable_literal, reason = "the drift test regenerates this decimal memory count")]
const WEB_BYTES: u64 = 274394275519;
#[test]
#[cfg(target_pointer_width = "64")]
fn the_web_preset_has_a_literal_price() {
    let actual = limits::worst_case(&limits::client()).expect("priced web preset");
    memory_literal::check(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/host_tests.rs"),
        "const WEB_BYTES: u64 = ",
        WEB_BYTES,
        actual,
    );
}

#[test]
fn the_web_preset_sends_a_whole_suggested_request() {
    assert!(limits::client().client.domain.text >= temper_client_protocol_wire::CUE_BYTES);
}

// Wire fixtures keep the regression on Host's linked root and public transport surface.
fn encoded(size: u32, write: impl FnOnce(&mut skein_lib::Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("fixture fits"));
    write(&mut writer).expect("measured fixture");
    writer.finish()
}
fn owned(record: Box<[u8]>) -> jig_client_protocol_wire::v13::Owned {
    use jig_client_protocol_wire::{VERSION, v13 as w};
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: VERSION, record })
        .expect("fixture envelope")
}
fn streamed(host: &mut Host, event: &jig_client_protocol_wire::v13::StreamEvent) {
    use skein_lib::base64::{self, Alphabet};
    let bytes = encoded(event.measure(), |writer| event.encode(writer));
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture fits"))
        .expect("base64 fits");
    let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("fixture fits"));
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("measured base64");
    let mut piece = b"data: ".to_vec();
    piece.extend_from_slice(&writer.finish());
    piece.extend_from_slice(b"\n\n");
    host.heard(Heard::Piece { bytes: piece.into_boxed_slice() });
}
#[test]
fn the_first_task_activity_answer_survives_transport_sign_in() {
    use jig_client_protocol_wire::{VERSION, v13 as w};
    use skein_lib::{Duration, Reader};
    // task_record's chip is task 9; the answer carries one committed run marker.
    const TASK_PATH: &[u8] = b"/tasks/9";
    const RUN: u64 = 44;
    const ONE_LINE: u32 = 1;
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH);
    host.start(TASK_PATH, Vec::new(), None);
    let activity = host
        .requests()
        .into_iter()
        .find_map(|request| match request {
            Asked::Exchange { id, call, .. } if call.path.as_ref() == b"/jig/v1/read" => Some(id),
            Asked::Exchange { .. }
            | Asked::OpenStream { .. }
            | Asked::CloseStream
            | Asked::Save { .. }
            | Asked::Erase { .. } => None,
        })
        .expect("first activity read left the host");
    host.heard(Heard::StreamOpened);
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: VERSION, last: VERSION })
                .expect("served version"),
        )
        .expect("one served family");
    streamed(
        &mut host,
        &w::StreamEvent::Hello(
            w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
                .expect("hello"),
        ),
    );
    let watch = host
        .requests()
        .into_iter()
        .find_map(|request| {
            let Asked::Exchange { call, .. } = request else { return None };
            if call.path.as_ref() != b"/jig/v1/watch" {
                return None;
            }
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            if matches!(subject, w::Subject::Task(_)) { Some(open.watch()) } else { None }
        })
        .expect("task watch");
    let mut task = jig_client_test_records::task_record().into_parts();
    task.chip = jig_client_test_records::owner_waiting_chip_record();
    let document = w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).expect("agent task")));
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(
            w::Delivery::new(
                &w::CEILINGS,
                w::DeliveryParts {
                    watch,
                    document: owned(encoded(document.measure(), |writer| document.encode(writer))),
                },
            )
            .expect("task delivery"),
        ),
    );
    let event = w::Activity::RunBegan(
        w::ActivityRunBegan::new(
            &w::CEILINGS,
            w::ActivityRunBeganParts { attempt: RUN, how: w::RunBeginning::Resumed, why: Box::from([]) },
        )
        .expect("run marker"),
    );
    let line =
        w::ActivityLine::new(&w::CEILINGS, w::ActivityLineParts { position: 1, at: 1, event }).expect("activity line");
    let page = jig_client_test_records::activity_rows(&[line]);
    let value = w::ReadValue::Activity(page);
    let answer = w::ReadAnswer::Value(owned(encoded(value.measure(), |writer| value.encode(writer))));
    host.heard(Heard::Answered {
        exchange: activity,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
    });
    build(&mut host);
    let shown = host.view.tree().nodes().iter().filter(|node| node.name.as_deref() == Some(b"Run 44 began")).count();
    assert_eq!(shown, usize::try_from(ONE_LINE).expect("one line fits"), "first activity answer must reach the page");
}

#[test]
fn the_deadline_takes_the_earlier_domain_or_view_time_from_the_same_clocks() {
    let now = Time::from_nanos(100);
    let wall = Wall::from_nanos(1000);
    for (domain, next, expected) in [
        (Some(105), Some(1020), Some(105)),
        (Some(130), Some(1020), Some(120)),
        (None, Some(1020), Some(120)),
        (Some(130), None, Some(130)),
        (None, None, None),
        (None, Some(990), Some(100)),
    ] {
        assert_eq!(
            deadline(domain.map(Time::from_nanos), next.map(Wall::from_nanos), now, wall),
            expected.map(Time::from_nanos)
        );
    }
}

#[test]
fn the_patch_queue_is_reserved_once_at_start() {
    let mut host = fixture();
    assert!(!host.patches().is_empty());
    assert!(host.patches.is_empty());
    assert_eq!(host.patches.room(), host.bounds.tree.patches);
    host.went(b"/chats");
    assert!(!host.patches().is_empty());
    assert!(host.patches.is_empty());
    assert_eq!(host.patches.room(), host.bounds.tree.patches);
}
