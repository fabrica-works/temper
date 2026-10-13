//! Both roots and Smith's domain share every durable and process crossing.
use jig_domain_tasks as tasks;
use temper_engine_domain::{Key, Record};
use temper_system_world::composed::{World, report_script};

#[test]
fn a_committed_chat_runs_through_the_worker_and_smith_and_returns_a_report() {
    let mut world = World::chat(910, report_script(false));
    world.run();
    let observed = &world.observed;
    assert_eq!(observed.assignments, 1);
    assert_eq!(observed.spawns, 1);
    assert_eq!(observed.answers, 1);
    assert_eq!(observed.gone, 1);
    assert_eq!(observed.turns, observed.turn_acks);
    assert!(observed.turns > 0);
    let task = world.assignment().task;
    assert!(matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))))
        if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { .. })))));
}

#[test]
fn smiths_host_call_and_its_durable_reply_traverse_the_worker_process_channel() {
    let mut world = World::chat(911, report_script(true));
    world.run();
    assert_eq!(world.observed.relays, 1);
    assert_eq!(world.observed.replies, 1);
    assert_eq!(world.agent().host_submissions().len(), 1);
    assert_eq!(world.agent().host_terminals().len(), 1);
    assert_eq!(world.observed.turns, world.observed.turn_acks);
}

#[test]
fn a_persons_words_reach_smith_through_the_live_worker_channel() {
    let mut world = World::chat(912, report_script(true));
    world.until_call();
    world.act(
        jig_fake_parties::Ask::Say {
            task: jig_fake_parties::Task::Last,
            words: Box::from(&b"continue with these words"[..]),
        },
        [42; 16],
    );
    world.run();
    assert!(world.observed.messages.iter().any(|(_, words)| words.as_ref() == b"continue with these words"));
    assert!(
        world
            .agent()
            .messages_seen()
            .iter()
            .any(|(_, seen)| matches!(seen, smith_agent_world::messages_referee::Seen::Input { .. }))
    );
}

#[test]
fn a_persons_stop_crosses_the_worker_and_cancels_smith_before_the_task_answers() {
    let mut world = World::chat(913, report_script(true));
    world.until_call();
    world.act(jig_fake_parties::Ask::Stop { task: jig_fake_parties::Task::Last }, [43; 16]);
    world.run();
    assert_eq!(world.observed.cancels, 1);
    assert!(matches!(
        world.agent().answer(),
        smith_domain_run::Answer::Failed { failure: smith_domain_run::Failure::Cancelled, .. }
    ));
    assert_eq!(world.observed.answers, 1);
    assert_eq!(world.observed.gone, 1);
}

#[test]
fn an_adopted_repository_is_prepared_by_the_worker_and_mounted_in_smith() {
    let mut world = World::with_repository(914, report_script(false));
    world.run();
    assert!(world.observed.checkouts >= 4);
    assert_eq!(world.assignment().workspace.repositories.len(), 1);
    let repository = &world.assignment().workspace.repositories[0];
    assert!(repository.push.is_none(), "inspection keeps the mount read only");
    assert_eq!(
        world.agent().disk().files().get(b"repo/src/lib.rs".as_slice()),
        Some(&b"pub fn answer() -> u32 { 42 }\n".as_slice())
    );
    assert_eq!(world.observed.answers, 1);
}

#[test]
fn a_parked_smith_run_resumes_from_the_prefix_restored_through_both_roots() {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let mut world = World::chat(
        915,
        Script {
            cue: Box::from(&b"@system"[..]),
            turns: Box::new([
                Turn {
                    lines: Box::new([Line::Call { name: Box::from(&b"wait"[..]), arguments: Box::from(&b"{}"[..]) }]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
                Turn {
                    lines: Box::new([Line::Text { text: Box::from(&b"Waiting for the person."[..]) }]),
                    finish: Finish::Stop,
                    tokens: 20,
                },
                Turn {
                    lines: Box::new([Line::Call { name: Box::from(&b"wait"[..]), arguments: Box::from(&b"{}"[..]) }]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
            ]),
        },
    );
    world.run();
    assert!(matches!(world.agent().answer(), smith_domain_run::Answer::Parked { .. }));
    let previous = world.agent().turns().last().expect("committed conversation").sequence;
    let attempt = world.assignment().attempt;
    world.resume(b"continue", report_script(false));
    world.run();
    assert!(world.assignment().attempt > attempt);
    assert!(world.agent().turns().first().expect("resumed output").sequence > previous);
    assert_eq!(world.observed.assignments, 2);
    assert_eq!(world.observed.spawns, 2);
    assert_eq!(world.observed.answers, 2);
}

#[test]
fn a_withheld_store_commit_holds_the_party_reply_and_worker_assignment() {
    let mut world = World::chat(916, report_script(false));
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    for _ in 0..200 {
        world.tick();
    }
    assert!(world.store.applied > 0);
    assert!(world.party.replies.is_empty(), "the sign-in reply waits for durability acknowledgement");
    assert_eq!(world.observed.assignments, 0);
    assert_eq!(world.observed.spawns, 0);
    world.release_commits();
    world.run();
    assert_eq!(world.observed.answers, 1);
}
