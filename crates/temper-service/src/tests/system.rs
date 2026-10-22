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
    stand.service.engine.as_mut().expect("engine").store.opened = false;
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().started, "store still closed");
    stand.service.engine.as_mut().expect("engine").store.opened = true;
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().started, "listener still opening");
    for _ in 0_u8..4 {
        stand.turn();
    }
    assert!(stand.service.engine.as_ref().expect("engine").workers.opened());
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
    stand.service.told.push(skein_io::Event::Closed { owner: Token::new(crate::tokens::SERVER) });
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

#[test]
fn a_socket_workers_bind_announces_its_bound_address_once() {
    let mut service = fixtures::started();
    service.notes().push(crate::Note::ForgeUnconfigured { number: 1 });
    assert_eq!(service.notes().room(), 0, "forge diagnostics occupy their whole queue");
    let addr = ([127, 0, 0, 1], 41733).into();
    service.told.push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::WORKER_LISTENERS),
        listener: Token::new(77),
        addr: skein_io::Addr::Ip(addr),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.listening().pop(), Some(crate::Note::Listening { listener: crate::Listening::Workers, addr }));
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
    assert_eq!(service.notes().pop(), Some(crate::Note::ForgeUnconfigured { number: 1 }));
}

#[test]
fn an_in_process_workers_bind_has_no_listening_note() {
    let mut service = fixtures::started();
    service.told.push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::WORKER_LISTENERS),
        listener: Token::new(77),
        addr: skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap()),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
}

#[test]
fn a_clients_bind_announces_the_servers_bound_address_once() {
    let mut service = fixtures::started();
    let addr = ([127, 0, 0, 1], 41734).into();
    service.told.push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::SERVER),
        listener: Token::new(77),
        addr: skein_io::Addr::Ip(addr),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.listening().pop(), Some(crate::Note::Listening { listener: crate::Listening::Clients, addr }));
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
}

#[test]
fn standalone_announces_clients_without_announcing_in_process_workers() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    let addr = fixtures::engine().clients.listener;
    let jig_protocol_config::Address::Addr(addr) = addr else { panic!("clients socket") };
    let addr = addr.socket().unwrap();
    assert_eq!(
        stand.service.listening().pop(),
        Some(crate::Note::Listening { listener: crate::Listening::Clients, addr })
    );
    assert!(stand.service.listening().is_empty());
    stand.turn();
    assert!(stand.service.listening().is_empty());
}
