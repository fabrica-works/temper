use super::fixtures;
use jig_client::contract::{self as c, ActKind, Admission, ObjectKey, Read, ReadValue};
use skein_lib::{List, Wall};
use skein_ui::Role;
use temper_client_domain::{Action, Event, address::Filter, pages::Act};
const PARTY: u64 = 73;
const TEXT_BYTES: u32 = 32;
const SINGLE_OFFER: u32 = 1;
const FUNDING_PERIOD: u64 = 1;
const FUNDING_NEEDED: u64 = 5;
const FUNDING_LEFT: u64 = 20;
const FUNDING_AFTER: u64 = 15;
const EMPTY_BATCH: u32 = 0;
const PROPOSAL_INDEX: u32 = 1;
const PROPOSAL: c::ProposalKey =
    c::ProposalKey { proposer: c::Proposer::Task(fixtures::INBOX_TASK), number: fixtures::INBOX_QUESTION };
fn text(words: &[u8]) -> c::Text {
    c::Text::new(Box::from(words), TEXT_BYTES).expect("fixture text")
}
fn root(act: ActKind, admission: Admission) -> temper_client_domain::Domain {
    let mut acts = List::with_capacity(SINGLE_OFFER);
    acts.push(c::Offered { act, admission, effect: None }).expect("one act");
    let mut funding = List::with_capacity(SINGLE_OFFER);
    funding
        .push(c::OwnerFundingOffer {
            source: c::Funder::Pool { project: fixtures::PROJECT, person: PARTY, period: FUNDING_PERIOD },
            needed: FUNDING_NEEDED,
            left: Some(FUNDING_LEFT),
            after: Some(FUNDING_AFTER),
        })
        .expect("one funding source");
    let page = c::OwnerProposal {
        key: PROPOSAL,
        project: fixtures::PROJECT,
        reason: c::DocumentText::complete(text(b"Make a goal")),
        action: c::OwnerProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
        as_holder: true,
        waiting: c::OwnerProposalWaiting { holder: c::OwnerHistoryHolder::Person(PARTY), since: Wall::EPOCH },
        funding,
        acts,
    };
    fixtures::inbox_with_document(
        Filter::ALL,
        Some((Read::Proposal { proposal: PROPOSAL }, ReadValue::OwnerProposal(Box::new(page)))),
    )
}
fn press(root: &mut temper_client_domain::Domain, act: Act) {
    fixtures::root_event(root, Event::Act { action: Action::Pages(act) });
}
fn open(root: &mut temper_client_domain::Domain, act: ActKind) {
    press(root, Act::Decide { object: ObjectKey::Proposal(PROPOSAL), revision: None, act });
}
const ACCEPT_WORDS: [&[u8]; 4] = [b"Proposed tasks", b"Funding needed: 5", b"Funding left: 20", b"Funding after: 15"];
#[test]
fn the_accept_confirmation_says_what_it_grants_costs_and_leaves_in_the_pool() {
    let mut root = root(ActKind::Accept, Admission::Allowed);
    open(&mut root, ActKind::Accept);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Accept").is_some());
    let said = fixtures::text(tree);
    for expected in ACCEPT_WORDS {
        assert!(contains(&said, expected), "{expected:?}");
    }
}
#[test]
fn rejecting_says_so_with_its_reason_before_it_is_sent() {
    let mut root = root(ActKind::Reject, Admission::Allowed);
    press(&mut root, Act::CardWords { object: ObjectKey::Proposal(PROPOSAL), words: Box::from(*b"Too costly") });
    open(&mut root, ActKind::Reject);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject with reason").is_some());
    assert!(contains(&fixtures::text(tree), b"Too costly"));
}
#[test]
fn an_act_only_proposable_says_so_and_to_whom() {
    let root = root(ActKind::Accept, Admission::Proposed { to: c::Recipient::Party(PARTY) });
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Button, b"Accept").is_none());
    assert!(contains(&fixtures::text(tree), b"This needs a proposal to"));
}

fn contains(text: &[u8], expected: &[u8]) -> bool {
    for words in text.windows(expected.len()) {
        if words == expected {
            return true;
        }
    }
    false
}
const OUTCOMES: [(jig_client::Outcome, &[u8]); 3] = [
    (jig_client::Outcome::NotKept, b"The request was not kept."),
    (jig_client::Outcome::Unknown, b"The engine no longer knows this request."),
    (
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotOffered,
            at: None,
        })),
        b"This act is not offered",
    ),
];
#[test]
fn the_card_reports_client_outcomes_in_jigs_words() {
    let root = root(ActKind::Accept, Admission::Allowed);
    let card = root.pages().inbox().expect("inbox").cards.get(PROPOSAL_INDEX).expect("proposal");
    let limits = fixtures::tree_limits();
    let words = fixtures::words();
    for (outcome, expected) in OUTCOMES {
        let mut card = card.clone();
        card.outcome = Some(outcome);
        let mut builder = skein_ui::Builder::new(&limits);
        let mut bindings = crate::frame::Bindings::new(limits.bindings);
        crate::confirm::card(
            &mut builder,
            &root,
            &card,
            &words,
            &mut bindings,
            fixtures::domain_limits().client.domain.rows,
        );
        assert!(contains(&fixtures::text(&builder.finish()), expected), "{expected:?}");
    }
}
