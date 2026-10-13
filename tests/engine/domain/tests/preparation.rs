//! A run's preparation and its answer at awkward moments: a store read that
//! lands after the preparation it served was dropped, and an answer whose
//! parts exceed their bounds.
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Time, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain::{CoreRange, Range};
use temper_engine_domain_world::direct::{Driver, say_ask};
use temper_engine_domain_world::protocol;

/// Step until the root has asked for `task`'s transcript, leaving the
/// store's answer queued; the answer's owner.
fn transcript_in_flight(driver: &mut Driver, task: u64) -> Token {
    for _ in 0..50 {
        driver.advance(true);
        let asked = driver.loads.iter().find_map(|(owner, range)| match range {
            Range::Core(CoreRange::TaskTranscript { task: read, .. }) if *read == task => Some(*owner),
            Range::Core(_) | Range::Census(_) | Range::Forge => None,
        });
        if let Some(owner) = asked {
            return owner;
        }
    }
    panic!("the parked chat's preparation read its transcript")
}

// A message can end a preparation while its store read is in flight; the
// read's answer then belongs to none (jig's `examples/ops/domain/src/restart.rs`).
#[test]
fn a_transcript_read_landing_after_a_message_ended_its_preparation_is_dropped() {
    let (mut driver, first) = Driver::chat();
    driver.send(protocol::turn(Token::new(7), first.task, first.attempt, 1, 1, None, b"first"));
    driver.settle();
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.send(say_ask(driver.session(), first.task, 50));
    transcript_in_flight(&mut driver, first.task);
    driver.send(say_ask(driver.session(), first.task, 51));
    driver.settle();
    // The message ended that preparation; the task backs off and prepares again.
    driver.env.now = Time::from_nanos(Duration::from_secs(1).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(1).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    driver.settle();
    let next = driver.last_assignment();
    assert_eq!((next.task, next.attempt), (first.task, 2));
    assert_eq!(next.transcript.len(), 1);
    assert_eq!(next.messages.iter().map(|message| message.words.as_ref()).collect::<Vec<_>>(), [[50], [51]]);
}

// Parts beyond their bounds make the answer invalid; the run fails as invalid.
#[test]
fn an_answer_pushing_more_heads_than_a_task_may_hold_fails_its_run_as_invalid() {
    let (mut driver, run) = Driver::chat();
    let pushed = (0..5).map(|tag| temper_engine_domain_forge::Pushed { tag, commit: [7; 32] }).collect();
    let parts = engine::Parts { saved: None, pushed };
    let end = tasks::End::Finished {
        result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
        cancel_delegates: false,
    };
    driver.answer(Token::new(7), run.task, run.attempt, 1, end, parts);
    driver.settle();
    let task = driver.task(run.task).expect("the chat lives on");
    assert_eq!(task.tries.invalid, 1);
}
