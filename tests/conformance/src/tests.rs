use super::*;

const TASK: u64 = 1;
const ATTEMPT: u64 = 2;
const HEAD: skein_forge::api::Commit = [7; 32];
const RESTART: Clock = Clock { now: 1 };

#[test]
fn a_worker_reports_its_unchanged_push_head_after_an_engine_restart() {
    let config = Config::new(71, Some(jig_conformance::scenarios::Scenario::Grace));
    let mut peers = Temper::peers(&config, 71);
    peers.reported.insert(TASK, HEAD);
    Temper::cold(&mut peers, &Store::new(), RESTART);
    let event = link::worker_event(
        &mut peers,
        workers::Up::Answer {
            channel: link::CHANNEL,
            task: TASK,
            attempt: ATTEMPT,
            spent: 0,
            end: workers::End::Parked,
            read: None,
            refused: Box::new([]),
        },
    );
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Up { parts, .. }, .. }) = event else {
        panic!("worker answer reaches the engine");
    };
    assert_eq!(parts.pushed, Box::new([forge::Pushed { tag: 2, commit: HEAD }]) as Box<[forge::Pushed]>);
}
