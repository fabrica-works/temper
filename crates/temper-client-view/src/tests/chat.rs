use super::fixtures;
use fixtures::chat::{
    ATTEMPT, BYTES, CLOSE_GRACE, COMMITTED, GRACES, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, chat_root,
    closing, messaging_root, proposal_root, provisional, provisional_with_text, rejecting_chat, tree,
};
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};
use skein_ui::Role;

#[test]
fn a_chats_header_says_its_status_and_spend() {
    let tree = chat_header();
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}
#[test]
fn a_chats_header_says_its_spend_including_delegates() {
    let tree = chat_header();
    assert!(fixtures::find(&tree, Role::Paragraph, b"Spent 7; delegates spent 2").is_some());
}

#[test]
fn provisional_text_is_marked_until_its_turn_commits() {
    let mut thread = provisional();
    assert!(fixtures::find(&tree(&thread), Role::Status, b"Writing...").is_some());
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(c::ActivityLine {
        position: TURN.into(),
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: ATTEMPT, number: TURN },
            turn: c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
        },
    })
    .expect("turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert_eq!(fixtures::find(&tree(&thread), Role::Status, b"Writing..."), None);
}

#[test]
fn an_owner_activity_message_is_shown_in_the_chat() {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(c::ActivityLine {
        position: TURN.into(),
        at: Wall::EPOCH,
        event: c::Activity::Owner {
            source: Box::new(c::OwnerActivityLine {
                task: TASK,
                position: TURN.into(),
                at: Wall::EPOCH,
                line: c::OwnerActivity::Words {
                    message: MESSAGE,
                    by: c::OwnerRequester::Person(PARTY),
                    kind: c::OwnerMessageKind::Words,
                    words: c::Text::new(Box::from(*b"An owner message"), TEXT).expect("words"),
                },
            }),
            decoded: None,
        },
    })
    .expect("line");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree(&thread)), b"An owner message").is_some());
}

#[test]
fn the_close_confirmation_names_each_live_delegates_fate() {
    let tree = closing(Some(CLOSE_GRACE));
    let text = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&text, b"Delegate 12 will be cancelled").is_some());
    assert!(skein_lib::bytes::find(&text, b"Delegate 13 will be moved to you").is_some());
}
#[test]
fn the_close_confirmation_offers_each_live_delegates_choice() {
    let tree = closing(Some(CLOSE_GRACE));
    assert!(fixtures::find(&tree, Role::Button, b"Cancel delegate 12").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Take delegate 13").is_some());
}
#[test]
fn the_close_confirmation_names_the_templates_grace() {
    for (grace, expected) in GRACES {
        assert!(skein_lib::bytes::find(&fixtures::text(&closing(grace)), expected).is_some(), "{grace:?}");
    }
}

#[test]
fn pressing_close_opens_the_chat_confirmation() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    assert!(root.pages().confirmation().is_some(), "Close prepared the compound request");
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Dialog, b"Close").is_some());
}

#[test]
fn the_close_confirmation_says_the_cancellations_facts() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    let mut view = fixtures::view();
    let said = fixtures::text(fixtures::build(&mut view, &root));
    let dangerous =
        fixtures::words().say_word(jig_client::words::Word::Decision(jig_client::words::DecisionWord::Dangerous), &[]);
    assert!(skein_lib::bytes::find(&said, &dangerous).is_some(), "irreversible cancellation");
    let impact = fixtures::words().say_word(
        jig_client::words::Word::Decision(jig_client::words::DecisionWord::TasksRunning),
        &[skein_ui::phrases::Hole::Number(1)],
    );
    assert!(skein_lib::bytes::find(&said, &impact).is_some(), "supplied running-task impact");
}

#[test]
fn a_proposal_waiting_in_the_chat_is_shown_after_its_conversation() {
    let root = proposal_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let mut conversation_end = None;
    let mut accept = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Log {
            conversation_end = Some(index + usize::try_from(node.size).expect("log size"));
        }
        if node.role == Role::Button && node.name.as_deref() == Some(b"Accept".as_slice()) {
            accept = Some(index);
        }
    }
    assert!(
        accept.expect("proposal's accept act") >= conversation_end.expect("conversation"),
        "card after conversation"
    );
}
#[test]
fn a_chat_proposal_offers_rejection_with_a_reason() {
    let root = proposal_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Button, b"Reject with reason").is_some());
}
#[test]
fn rejecting_in_the_chat_shows_the_reason_in_its_confirmation() {
    let root = rejecting_chat();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject with reason").is_some());
}
#[test]
fn a_chat_rejection_confirmation_keeps_the_persons_reason() {
    let root = rejecting_chat();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(skein_lib::bytes::find(&fixtures::text(tree), b"Too costly").is_some());
}

const LONG_PROVISIONAL: &[u8] = b"1234567890123456789012345678901\xc3\xa9 and more";
#[test]
fn long_provisional_text_says_that_earlier_text_was_omitted() {
    let thread = provisional_with_text(LONG_PROVISIONAL);
    let tree = tree(&thread);
    assert!(fixtures::find(&tree, Role::Status, b"Earlier text omitted").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"\xc3\xa9 and more").is_some());
}

#[test]
fn enter_in_the_message_composer_sends() {
    let mut view = fixtures::page_view(&messaging_root());
    let composer = fixtures::find(view.tree(), Role::TextArea, b"Message").expect("composer");
    assert_eq!(
        crate::act(&mut view, composer, skein_ui::Act::Submit),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Send)),
    );
}
#[test]
fn a_line_break_counts_within_the_message_bound() {
    let mut root = messaging_root();
    let mut over = [b'a'; 33];
    over[3] = b'\n';
    let words = &over[..32];
    assert_eq!(words.len(), usize::try_from(TEXT).expect("bound fits"));
    assert_eq!(&*fixtures::type_into(&mut root, b"Message", words), words);
    assert_eq!(&*fixtures::type_into(&mut root, b"Message", &over), words);
}
#[test]
fn enter_in_a_cards_reason_does_nothing() {
    let mut view = fixtures::page_view(&proposal_root());
    let reason = fixtures::find(view.tree(), Role::TextArea, b"Reason").expect("reason");
    assert_eq!(crate::act(&mut view, reason, skein_ui::Act::Submit), None);
}
#[test]
fn a_chats_card_carries_the_card_token() {
    let view = fixtures::page_view(&proposal_root());
    let cards = fixtures::styles(view.tree(), Role::Item);
    assert!(!cards.is_empty());
    for card in &cards {
        assert_eq!(card.as_ref(), [crate::look::CARD]);
    }
}
#[test]
fn each_turn_carries_the_turn_token() {
    let tree = tree(&provisional());
    let turns = fixtures::styles(&tree, Role::Region);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns.as_ref(), [Box::from([crate::look::TURN])]);
}

fn style_position(nodes: &[skein_ui::Node], token: skein_ui::Style) -> Option<usize> {
    for (index, node) in nodes.iter().enumerate() {
        if node.styles.as_slice().contains(&token) {
            return Some(index);
        }
    }
    None
}
fn named_position(nodes: &[skein_ui::Node], role: Role, name: Option<&[u8]>) -> Option<usize> {
    for (index, node) in nodes.iter().enumerate() {
        if node.role == role && (name.is_none() || node.name.as_deref() == name) {
            return Some(index);
        }
    }
    None
}
fn styled(tree: &skein_ui::Tree, token: skein_ui::Style) -> usize {
    style_position(tree.nodes(), token).expect("styled node")
}
fn subtree(tree: &skein_ui::Tree, at: usize) -> &[skein_ui::Node] {
    let from = at.checked_add(1).expect("child index");
    let through = at.checked_add(usize::try_from(tree.nodes()[at].size).expect("size")).expect("subtree end");
    &tree.nodes()[from..through]
}
#[test]
fn the_chat_bar_contains_identity_spend_and_acts() {
    use temper_client_domain::Event;
    let root = messaging_root();
    // The standard task fixture offers Close; a checked task delivery adds Stop.
    let mut task = root.pages().chat().expect("chat").task(root.client()).expect("task").clone();
    let mut offers = List::with_capacity(3);
    for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel] {
        offers.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("offer");
    }
    task.acts = offers;
    // Re-enter with a task watch whose token is supplied by the fixture.
    let (mut fresh, _, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    fixtures::root_event(
        &mut fresh,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    let view = fixtures::page_view(&fresh);
    let bar = styled(view.tree(), crate::look::CHATBAR);
    assert_eq!(view.tree().nodes()[bar].role, Role::Slot);
    let nodes = subtree(view.tree(), bar);
    for (role, name) in [
        (Role::Heading { level: 1 }, b"Chat 9".as_slice()),
        (Role::Paragraph, b"Spent 7; delegates spent 2"),
        (Role::Status, b"Parked"),
        (Role::Button, b"Stop"),
        (Role::Button, b"Close"),
        (Role::Button, b"Mark seen"),
    ] {
        assert!(named_position(nodes, role, Some(name)).is_some(), "{name:?}");
    }
    assert!(named_position(nodes, Role::Paragraph, Some(fixtures::task::document().chip.title.bytes())).is_some());
}
#[test]
fn the_chat_entry_has_the_true_hint_between_message_and_send() {
    let view = fixtures::page_view(&messaging_root());
    let tree = view.tree();
    let area = styled(tree, crate::look::COMPOSER_AREA);
    let entry = styled(tree, crate::look::ENTRY);
    assert_eq!(tree.nodes()[area].role, Role::Slot);
    assert!(entry > area);
    let nodes = subtree(tree, entry);
    assert_eq!(nodes[0].role, Role::TextArea);
    assert!(nodes[0].states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
    assert_eq!(nodes[1].role, Role::Paragraph);
    assert!(nodes[1].styles.as_slice().contains(&crate::look::HINT));
    let send = 1_usize.checked_add(usize::try_from(nodes[1].size).expect("hint size")).expect("send index");
    assert_eq!(nodes[send].role, Role::Button);
    assert_eq!(nodes[send].name.as_deref(), Some(b"Send".as_slice()));
    assert_eq!(
        fixtures::words().say(crate::words::Word::ChatSendHint, &[]).as_ref(),
        b"Enter to send \xc2\xb7 Shift+Enter for a new line"
    );
}
#[test]
fn a_chat_without_message_has_no_composer_area() {
    let view = fixtures::page_view(&chat_root());
    assert!(style_position(view.tree().nodes(), crate::look::COMPOSER_AREA).is_none());
}
#[test]
fn chat_cards_are_items_of_the_cards_list_after_the_log() {
    let view = fixtures::page_view(&proposal_root());
    let tree = view.tree();
    let cards = styled(tree, crate::look::CARDS);
    assert_eq!(tree.nodes()[cards].role, Role::List);
    let card = style_position(subtree(tree, cards), crate::look::CARD).expect("card");
    assert_eq!(subtree(tree, cards)[card].role, Role::Item);
    assert!(named_position(tree.nodes(), Role::Log, None).expect("log") < cards);
}

pub(super) fn structured_thread() -> Thread {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: 12, bytes: BYTES, text: 128 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut messages = List::with_capacity(4);
    for speaker in [c::Speaker::Person, c::Speaker::Assistant, c::Speaker::Tool, c::Speaker::System] {
        let mut blocks = List::with_capacity(2);
        blocks
            .push(c::Block::Call(c::Call {
                number: 1,
                tool: c::Text::new(Box::from(*b"Read"), TEXT).expect("tool"),
                input: c::Text::new(Box::from([]), TEXT).expect("input"),
                outcome: None,
                failed: false,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("call");
        blocks
            .push(c::Block::Result {
                call: 1,
                summary: c::Text::new(Box::from(*b"Done"), TEXT).expect("summary"),
                failed: false,
            })
            .expect("result");
        messages.push(c::Message { speaker, blocks }).expect("message");
    }
    let mut rows = List::with_capacity(5);
    for (position, event) in [
        (5, c::Activity::RunEnded { attempt: ATTEMPT, how: c::RunEnding::Parked, spent: 0 }),
        (
            4,
            c::Activity::Turn {
                name: c::TurnName { attempt: ATTEMPT, number: 1 },
                turn: c::TurnRead::Decoded(c::Turn {
                    messages,
                    used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                    spent: 0,
                    read_through: None,
                    cut: false,
                }),
            },
        ),
        (
            3,
            c::Activity::Words {
                by: c::Party {
                    number: PARTY,
                    kind: c::PartyKind::Person,
                    name: c::Text::new(Box::from(*b"Pat"), TEXT).expect("person"),
                },
                message: 1,
                words: c::Text::new(Box::from(*b"Hello"), TEXT).expect("words"),
            },
        ),
        (
            2,
            c::Activity::RunBegan {
                attempt: ATTEMPT,
                how: c::RunBeginning::Fresh { tail: 0 },
                why: c::Text::new(Box::from(*b"Hello"), TEXT).expect("why"),
            },
        ),
    ] {
        rows.push(c::ActivityLine { position, at: Wall::EPOCH, event }).expect("line");
    }
    thread.page(c::ActivityPage {
        rows,
        after: None,
        before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).expect("cursor")),
        cut: true,
    });
    thread.left = 1;
    thread
}
#[test]
fn messages_keep_speaker_who_says_and_work_slots() {
    use crate::look;
    let tree = tree(&structured_thread());
    for speaker in [look::BY_PERSON, look::BY_AGENT, look::BY_TOOL, look::BY_SYSTEM] {
        let at = styled(&tree, speaker);
        assert_eq!(tree.nodes()[at].role, Role::Slot);
        assert!(tree.nodes()[at].styles.as_slice().contains(&look::MESSAGE));
        let nodes = subtree(&tree, at);
        let who = style_position(nodes, look::WHO).expect("who");
        let says = style_position(nodes, look::SAYS).expect("says");
        assert!(who < says);
        assert_eq!(nodes[who].role, Role::Slot);
        assert_eq!(nodes[says].role, Role::Slot);
    }
    let mut works = 0_u32;
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::WORK) {
            assert_eq!(node.role, Role::Slot);
            assert!(named_position(subtree(&tree, at), Role::Paragraph, None).is_some());
            works = works.checked_add(1).expect("work count");
        }
    }
    assert_eq!(works, 8);
}
#[test]
fn run_and_omission_markers_keep_the_turn_region() {
    let tree = tree(&structured_thread());
    let mut markers = 0_u32;
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&crate::look::MARKER) {
            assert_eq!(node.role, Role::Region);
            assert!(node.styles.as_slice().contains(&crate::look::TURN));
            markers = markers.checked_add(1).expect("marker count");
        }
    }
    assert_eq!(markers, 3);
}

fn chat_event(
    root: &mut temper_client_domain::Domain,
    event: temper_client_domain::Event,
    out: &mut skein_lib::Queue<temper_client_domain::Request>,
) {
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: fixtures::domain_limits() },
        event,
        out,
    );
}
fn chat_with_history() -> temper_client_domain::Domain {
    use temper_client_domain::{Address, Event, Place, Request};
    let (mut root, _, _) = fixtures::root_at(
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        &fixtures::domain_limits(),
    );
    let mut out = skein_lib::Queue::with_capacity(temper_client_domain::max_out(&fixtures::domain_limits()));
    chat_event(&mut root, Event::Went { place: Place::Address(Address::Chat { task: TASK }) }, &mut out);
    let mut watch = None;
    let mut activity = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            jig_client::Down::Open {
                                watch: token,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { .. }),
                            },
                    },
            } => watch = Some(token),
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                    },
            } => activity = Some(read),
            Request::Client { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => {}
        }
    }
    let template = messaging_root();
    let mut task = template.pages().chat().expect("chat").task(template.client()).expect("task").clone();
    task.activity = 4;
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut rows = List::with_capacity(1);
    rows.push(c::ActivityLine {
        position: 4,
        at: Wall::EPOCH,
        event: c::Activity::Words {
            by: c::Party {
                number: PARTY,
                kind: c::PartyKind::Person,
                name: c::Text::new(Box::from(*b"Pat"), TEXT).expect("name"),
            },
            message: 1,
            words: c::Text::new(Box::from(*b"Hello"), TEXT).expect("words"),
        },
    })
    .expect("line");
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read: activity.expect("activity"),
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                        c::ActivityPage {
                            rows,
                            before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).expect("cursor")),
                            after: None,
                            cut: false,
                        },
                    )))),
                },
            },
        },
        &mut out,
    );
    root
}
#[test]
fn older_precedes_the_log_in_the_thread() {
    let view = fixtures::page_view(&chat_with_history());
    let nodes = subtree(view.tree(), styled(view.tree(), crate::look::THREAD));
    assert_eq!(nodes[0].role, Role::Button);
    assert_eq!(nodes[0].name.as_deref(), Some(b"Older".as_slice()));
    assert_eq!(nodes[1].role, Role::Log);
}
#[test]
fn pending_progress_and_the_answer_stay_before_the_composer() {
    use temper_client_domain::{Action, Event, Request, pages::Act};
    let mut root = chat_with_history();
    let mut out = skein_lib::Queue::with_capacity(temper_client_domain::max_out(&fixtures::domain_limits()));
    for act in [Act::MessageWords { words: Box::from(*b"Hello") }, Act::Send] {
        chat_event(&mut root, Event::Act { action: Action::Pages(act) }, &mut out);
    }
    assert_eq!(root.pages().chat().expect("chat").pending.len(), 1);
    let pending = fixtures::page_view(&root);
    let thread = styled(pending.tree(), crate::look::THREAD);
    let area = styled(pending.tree(), crate::look::COMPOSER_AREA);
    let log = named_position(pending.tree().nodes(), Role::Log, None).expect("log");
    let end = log.checked_add(usize::try_from(pending.tree().nodes()[log].size).expect("log size")).expect("log end");
    assert!(named_position(&pending.tree().nodes()[end..area], Role::Status, None).is_some());
    assert!(thread < log && end < area);
    chat_event(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(*token);
        }
    }
    let attempt = attempt.expect("attempt");
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Changed))),
                    why: None,
                },
            },
        },
        &mut out,
    );
    let answered = fixtures::page_view(&root);
    let applied = fixtures::words().say_card(crate::cards::P::Applied, &[]);
    let status = named_position(answered.tree().nodes(), Role::Status, Some(&applied)).expect("applied outcome");
    assert!(status < styled(answered.tree(), crate::look::COMPOSER_AREA));
}
