use super::fixtures;
use jig_client::contract::*;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
const TEXT: u32 = 32;
const SPENT: u64 = 7;
const BELOW: u64 = 2;
const BUDGET: u64 = 100;
const RESERVED: u64 = 0;
const LEFT: u64 = 91;
const LABEL: u32 = 1;
const DELEGATES: u32 = 3;
const EMPTY: u32 = 0;
const ACTIVITY: u64 = 1;
fn line() -> TaskLine {
    let budget = OwnerBudget {
        numbers: OwnerNumbers { budget: BUDGET, spent: SPENT, spent_below: BELOW, reserved: RESERVED },
        left: LEFT,
        funder: Funder::Task(TASK),
        deadline: None,
    };
    let details = OwnerChipDetails {
        project: fixtures::PROJECT,
        executor: OwnerExecutor::Person(PersonAddress::Party(73)),
        phase_since: Wall::EPOCH,
        run_since: None,
        attempt: None,
        retry: None,
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        person_wait: None,
        hold: None,
        budget: budget.clone(),
        activity: ACTIVITY,
        resources: List::with_capacity(EMPTY),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: DELEGATES,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    };
    TaskLine {
        chip: Chip {
            task: TASK,
            title: DocumentText::complete(Text::new(Box::from(*b"Hello"), TEXT).expect("title")),
            executor: Executor::Party { party: 73 },
            phase: Phase::Idle,
            details: Box::new(ChipPhase::Owner {
                phase: OwnerChipPhase::Active(RunStage::Idle),
                details: Box::new(details),
            }),
        },
        project: fixtures::PROJECT,
        requester: OwnerRequester::Person(73),
        tracked: None,
        label: LABEL,
        flags: TaskFlags { conversation: true, listed: false, retention: Retention::Kept },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: WaitsOn::Nothing,
        phases: OwnerPhaseCounts { running: DELEGATES, ..OwnerPhaseCounts::ZERO },
        held_below: EMPTY,
        conversation: None,
    }
}
fn tree(line: &TaskLine) -> (skein_ui::Tree, crate::frame::Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chats::row(
        &mut b,
        &fixtures::root_in_project(Some(fixtures::PROJECT)),
        line,
        &fixtures::words(),
        &mut bindings,
    );
    (b.finish(), bindings)
}
#[test]
fn a_chats_row_says_its_status_from_jigs_facts() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}
#[test]
fn a_chats_row_says_its_spend_and_delegates() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Spent 7; delegates spent 2").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Live delegates 3").is_some());
}
#[test]
fn a_chats_row_links_to_its_chat_by_number() {
    let (tree, bindings) = tree(&line());
    let mut bound = None;
    for node in tree.nodes() {
        if node.role == Role::Link && node.name.as_deref() == Some(b"Chat 9") {
            bound = node.binding;
        }
    }
    let binding = bound.expect("bound link");
    assert_eq!(
        bindings.decode(binding.token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Chat { task: TASK }
        }))
    );
}
#[test]
fn the_chat_composer_waits_for_the_person_and_project() {
    let mut root = temper_client_domain::unlinked(&fixtures::domain_limits(), fixtures::SEED);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Chats),
        },
    );
    let mut view = fixtures::view();
    assert_eq!(fixtures::find(fixtures::build(&mut view, &root), Role::TextArea, b"Start a chat"), None);
}
