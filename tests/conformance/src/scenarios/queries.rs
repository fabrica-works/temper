//! Query deliveries and ended pages checked against the durable index.
use super::{World, ask, check};
use crate::{Peers, observations::Store};
use jig_conformance::{referee::Violation, scenarios::Action};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;

const WATCH: u64 = 7090;
const READ: u64 = 7091;
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    next: Option<reads::Cursor>,
    found: Vec<u64>,
    checks: u32,
    deliveries: u32,
}
fn query() -> documents::TaskQuery {
    jig_core_world::fixtures::OWN_TASKS
}
fn indexed(store: &Store) -> Vec<u64> {
    store
        .rows
        .keys()
        .filter_map(|key| match key {
            root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { party: 1, task, .. })) => Some(*task),
            root::Key::Core(_) | root::Key::Forge { .. } => None,
        })
        .collect()
}
fn read(world: &mut World, from: Option<reads::Cursor>) {
    crate::send(
        world,
        root::Event::Party(root::PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(READ)),
            sign_in: world.peers.sign_in.expect("signed in"),
            read: reads::Read::Tasks { query: query(), from },
        }),
    );
}
pub(super) fn advance(world: &mut World, action: Action, phase: &mut u8) -> Result<bool, Violation> {
    match action {
        Action::WatchQuery => {
            if *phase < 2 && world.peers.children.is_empty() {
                ask(world, 110 + *phase, people::Ask::StartChat { project: 1, words: b"query task".as_slice().into() });
                *phase += 1;
                return Ok(true);
            }
            if world.peers.children.is_empty() {
                world.peers.children = world
                    .store
                    .rows
                    .values()
                    .filter_map(|record| match record {
                        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some(row.number),
                        root::Record::Core(_) | root::Record::Forge { .. } => None,
                    })
                    .collect();
            }
            crate::send(
                world,
                root::Event::Party(root::PartyInput::Watch {
                    watcher: Token::new(WATCH),
                    sign_in: world.peers.sign_in.expect("signed in"),
                    subject: documents::Watched::Query { query: query() },
                    window: documents::Window { rows: 4, after: None },
                }),
            );
        }
        Action::EndTask => {
            let task = *world.peers.children.iter().find(|task| !super::ended(world, **task)).expect("live query task");
            if *phase == 0 && !world.peers.assignments.iter().any(|a| a.task == task) {
                world.advance(6_000_000_000);
                *phase = 1;
                return Ok(true);
            }
            if *phase == 1 {
                world.advance(1_000_000_000);
                *phase = 2;
                return Ok(true);
            }
            super::message(world, task);
        }
        Action::ReadEnded => {
            if *phase == 0 {
                world.peers.query.found.clear();
                *phase = 1;
                read(world, Some(reads::Cursor::Store { scope: reads::Scope::Tasks { query: query() }, after: None }));
                return Ok(true);
            }
            if let Some(next) = world.peers.query.next.take() {
                read(world, Some(next));
                return Ok(true);
            }
            super::check_lazy(world.peers.query.found == indexed(&world.store), || {
                format!(
                    "ended query differs from its durable party index: found={:?}, index={:?}",
                    world.peers.query.found,
                    indexed(&world.store)
                )
            })?;
            world.peers.query.checks += 1;
        }
        Action::Compound
        | Action::CompoundAgain
        | Action::CancelWithGrace
        | Action::Finish
        | Action::Ready
        | Action::Late
        | Action::Effect(_)
        | Action::DeliverLate
        | Action::OtherHand
        | Action::HoldWrites
        | Action::ReleaseWrites
        | Action::CheckDeadline
        | Action::ChangeJudge
        | Action::FillPool
        | Action::ShrinkPool
        | Action::FinishHolder(_)
        | Action::Propose
        | Action::NarrowPolicy
        | Action::AcceptProposal
        | Action::StartStanding
        | Action::Period(_)
        | Action::Resume
        | Action::PartyAmend
        | Action::RunAmend
        | Action::Conversation { .. }
        | Action::Turn
        | Action::MarkTurns
        | Action::EndConversations
        | Action::MarkResults => unreachable!("query action"),
    }
    Ok(false)
}
fn line(store: &Store, line: &documents::TaskLine) {
    let task = line.chip.number;
    let record = store
        .rows
        .get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task))))
        .or_else(|| store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(owner) | tasks::Stored::Ended(owner)))) =
        record
    else {
        panic!("delivered query line has no durable owner")
    };
    assert_eq!((line.project, line.requester), (owner.project, owner.requester));
}
pub(crate) fn delivered(peers: &mut Peers, store: &Store, delivery: &root::Delivery) {
    if let root::Delivery::Document { watcher, document } = delivery
        && *watcher == Token::new(WATCH)
    {
        let documents::Document::Lines(lines) = document.as_ref() else { panic!("watched query") };
        for value in &lines.lines {
            line(store, value);
        }
        peers.query.deliveries += 1;
    }
    if let root::Delivery::Read { reply, answer } = delivery
        && *reply == ReplyTo::new(Token::new(READ))
    {
        let reads::Answer::Page(page) = answer else { panic!("ended query page: {answer:?}") };
        for row in &page.rows {
            let reads::PageRow::Line(value) = row else { panic!("ended line") };
            line(store, value);
            peers.query.found.push(value.chip.number);
        }
        peers.query.next.clone_from(&page.next);
    }
}

pub(super) fn check_ending(world: &World) -> Result<(), Violation> {
    check(
        world.peers.children.len() == 3
            && indexed(&world.store).len() == 3
            && world.peers.query.checks == 3
            && world.peers.query.deliveries >= 4,
        "query was not watched and read across all three task ends and restarts",
    )
}
