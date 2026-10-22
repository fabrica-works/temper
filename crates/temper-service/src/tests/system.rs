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

#[test]
fn the_box_pipes_carry_no_service_data_and_close_on_drain_or_abort() {
    use skein_io::kernel::Fd;
    for abort in [false, true] {
        let mut stand = fixtures::one_process();
        stand.service.adopt_box(Fd::new(3), Fd::new(4), 128).unwrap();
        assert_eq!(stand.service.adopt_box(Fd::new(5), Fd::new(6), 128), Err(Fd::new(5)));
        fixtures::until_hosting(&mut stand);
        let mut pending_reads = 0_u32;
        for submit in &stand.service.submissions {
            if let skein_io::kernel::Op::PipeRead { fd, .. } = &submit.kind
                && *fd == Fd::new(4)
            {
                pending_reads = pending_reads.checked_add(1).unwrap();
            }
        }
        assert_eq!(pending_reads, 1, "io's read is still pending; nothing reached the service");
        assert!(!stand.kernel.box_read_cancelled);
        assert_eq!(stand.kernel.box_writes, 0);
        assert!(!stand.kernel.closed.as_slice().contains(&Fd::new(3)));
        assert!(!stand.kernel.closed.as_slice().contains(&Fd::new(4)));
        stand.service.shutdown();
        if abort {
            stand.service.shutdown();
        }
        for _ in 0_u16..256 {
            stand.turn();
            if stand.service.is_empty() {
                break;
            }
        }
        assert_eq!(stand.service.end(), Some(if abort { &End::Aborted } else { &End::Drained }));
        assert!(stand.service.is_empty());
        assert!(stand.kernel.box_read_cancelled, "close cancelled io's pending read");
        for note in stand.service.notes().iter() {
            if let crate::Note::BoxLink { line } = note {
                panic!("our close is not the box closing: {line:?}");
            }
        }
        assert_eq!(stand.kernel.box_writes, 0);
        for fd in [Fd::new(3), Fd::new(4)] {
            let mut count = 0_u32;
            for closed in &stand.kernel.closed {
                if *closed == fd {
                    count = count.checked_add(1).unwrap();
                }
            }
            assert_eq!(count, 1);
        }
    }
}

#[test]
fn a_scripted_box_answers_the_engines_asks() {
    use skein_io::kernel::Fd;
    use temper_engine_domain as root;
    use temper_engine_domain_forge::{Connection, Event, SetUpRefusal, SetUpStep};
    const OWNED: u16 = 3;
    const OTHER: u16 = 4;
    // Allow the root continuations and memory-store handoffs, then output credit and IO.
    fn turns(stand: &mut fixtures::system::SystemStand) {
        const TURNS: u8 = 64;
        for _ in 0..TURNS {
            stand.turn();
        }
    }
    const LINE_BYTES: u32 = fixtures::system::BOX_BYTES;
    let mut stand = fixtures::one_process();
    stand.service.adopt_box(Fd::new(3), Fd::new(4), LINE_BYTES).unwrap();
    fixtures::until_hosting(&mut stand);
    // SystemStand has fixed membership and an in-memory store. Startup's
    // absent-file state is tested separately; inject its root inputs here.
    for forge in [OWNED, OTHER] {
        crate::setup::lifecycle::state(stand.service.engine.as_mut().unwrap(), forge, Connection::NotSetUp);
    }
    turns(&mut stand);
    let ask = root::Event::Forge(Box::new(Event::SetUp { forge: OWNED }));
    stand.service.engine.as_mut().unwrap().events.push(ask);
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );

    for byte in b"step 3 prepare\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SettingUp { step: Some(SetUpStep::Prepare) }
    );
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\n");

    for byte in b"failed 3 prepare 3 exited 1\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SetUpFailed { why: b"prepare 3 exited 1".as_slice().into() }
    );
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\n");

    stand.service.engine.as_mut().unwrap().events.push(root::Event::Forge(Box::new(Event::SetUp { forge: OWNED })));
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\nstart 3\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );

    stand.service.engine.as_mut().unwrap().events.push(root::Event::Forge(Box::new(Event::SetUp { forge: OTHER })));
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\nstart 3\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );
    assert_eq!(stand.service.engine.as_ref().unwrap().domain.forge_connection(OTHER), Connection::NotSetUp);
    assert_eq!(stand.service.notes().pop(), Some(crate::Note::SetUpRefused { number: OTHER, why: SetUpRefusal::Busy }));
    assert!(stand.service.notes().is_empty(), "one refusal note");

    for byte in b"started 9\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 3\nstart 3\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain.forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );
    assert_eq!(
        stand.service.notes().pop(),
        Some(crate::Note::BoxLink { line: b"box link: dropped started for forge 9, not asked".as_slice().into() })
    );
    assert!(stand.service.notes().is_empty());
}
