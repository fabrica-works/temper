use super::fixtures;
use crate::{End, Stop};
use skein_lib::{Time, Token, Wall};

#[test]
fn the_worker_dials_workers_and_its_hello_reaches_the_engine() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    let hello = fixtures::hello(&stand.service).unwrap();
    assert_eq!(hello.0, Token::new(1));
    assert_eq!(hello.1.slots, 1);
    assert_eq!(hello.1.hosting.as_ref(), &[]);
    assert_eq!(stand.kernel.spawned, 1);
}
#[test]
fn the_worker_iterates_only_once_the_engine_listens() {
    let mut stand = fixtures::one_process();
    stand.service.engine.store.opened = false;
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().started, "store still closed");
    stand.service.engine.store.opened = true;
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().started, "listener still opening");
    stand.turn();
    assert!(stand.service.engine.workers.opened());
    assert!(stand.service.worker.as_ref().unwrap().started);
}
#[test]
fn a_signal_drains_the_engine_then_ends_the_worker_at_once() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    stand.service.shutdown();
    assert!(stand.service.worker.as_ref().unwrap().aborted.is_none());
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    assert_eq!(stand.service.end(), Some(&End::Drained));
    assert!(stand.service.is_empty(), "the worker ended with the engine");
    assert!(stand.kernel.root_closed);
}
#[test]
fn a_second_signal_aborts_both_layers() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    stand.service.shutdown();
    stand.service.shutdown();
    assert_eq!(stand.service.end(), Some(&End::Aborted));
    assert!(stand.service.worker.as_ref().unwrap().aborted.is_some());
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    assert!(stand.service.is_empty());
}
#[test]
fn io_events_and_completions_reach_the_layer_whose_tokens_they_carry() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    assert!(stand.service.worker.as_ref().unwrap().git.ready, "git completions reached the worker");
    assert_eq!(stand.kernel.files, 6);
    stand.service.told.push(skein_io::Event::Closed { owner: Token::new(crate::tokens::CLIENTS) });
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert_eq!(stand.service.end(), Some(&End::Stopped(Stop::Listener)), "clients event reached engine");
    assert!(stand.service.worker.as_ref().unwrap().aborted.is_some());
}
#[test]
fn an_unusable_git_stops_the_service_naming_its_program() {
    let mut stand = fixtures::one_process();
    stand.kernel.unusable = true;
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    let Some(End::Stopped(Stop::Start(problems))) = stand.service.end() else { panic!("git refuses the start") };
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        problems.problems.get(0).unwrap().path,
        jig_protocol_config::Path::root()
            .member(b"application")
            .member(b"workers")
            .child(jig_protocol_config::Segment::Index(0))
            .member(b"git")
    );
    assert!(stand.service.is_empty());
}
