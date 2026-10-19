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
