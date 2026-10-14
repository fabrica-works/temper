use super::fixtures;
use crate::Worker;
use jig_domain_host as host;
use jig_protocol_config::{Path, Problem, Segment, What};
use skein_io::{self as io, kernel};
use skein_lib::{Token, Wall};
use temper_worker_domain::{
    self as root,
    checkout::{Ending, Fact, git},
};
use temper_worker_protocol_git_world::io::Invocation;

#[test]
fn a_clone_reaches_git_with_its_identity_s_newest_value() {
    let mut stand = fixtures::started_worker();
    stand.worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 1, b"tok-old"));
    stand.worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 2, b"tok-new"));
    stand.worker.link(fixtures::NOW, fixtures::repository_assignment());
    let (_, clone) = stand.answer_until_spawn(b"clone");
    let newest = &b"GIT_CONFIG_VALUE_0=Authorization: Basic dGVtcGVyLWJvdDp0b2stbmV3"[..];
    assert!(clone.env.contains(&Box::from(newest)), "the newest generation reaches git");
    assert!(clone.env.contains(&Box::from(b"HOME=/worker/home".as_slice())));
    for argument in &clone.args {
        assert_eq!(skein_lib::bytes::find(argument, b"tok-"), None, "no token in an argument");
    }
}

#[test]
fn an_operation_whose_identity_has_no_value_fails_broken_without_git() {
    let mut stand = fixtures::started_worker();
    stand.worker.link(fixtures::NOW, fixtures::repository_assignment());
    for _ in 0_u16..1000 {
        if !stand.turn() {
            break;
        }
    }
    assert_eq!(stand.below.spawns, 1, "the version check alone spawned git");
    assert_eq!(ended(&mut stand.worker), Some(Ending::Failed { fault: git::Fault::Broken }));
}

#[test]
fn a_cancel_reaches_the_component_and_its_done_reaches_the_root() {
    let mut stand = fixtures::started_worker();
    stand.worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 1, b"tok-new"));
    stand.worker.link(fixtures::NOW, fixtures::repository_assignment());
    let (owner, spawn) = stand.answer_until_spawn(b"clone");
    stand.below.script.invocations.push_back(Invocation::captured("clone"));
    stand.below.out.io.push(io::Request::Spawn { owner, spawn });
    stand.below.drain();
    stand.input();
    stand.worker.link(
        fixtures::NOW,
        jig_protocol_worker_link::Request::Down {
            down: host::LinkDown::Cancel { run: Token::new(7), attempt: Token::new(2) },
            parts: Box::new([]),
        },
    );
    for _ in 0_u16..1000 {
        if !stand.turn() {
            break;
        }
    }
    assert_eq!(stand.below.signals, 1);
    assert_eq!(ended(&mut stand.worker), Some(Ending::Failed { fault: git::Fault::Cancelled }));
}

#[test]
fn the_link_dials_only_once_the_git_component_has_started() {
    let mut stand = fixtures::WorkerStand::new();
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    let Some(io::Request::Spawn { spawn, .. }) = stand.worker.io().iter().next() else { panic!("git starts first") };
    assert_eq!(spawn.args[26].as_ref(), b"version");
    for request in &*stand.worker.io() {
        if let io::Request::Connect { .. } = request {
            panic!("link cannot dial before git starts");
        }
    }
    let mut dials = 0_u32;
    for _ in 0_u16..1000 {
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        for _ in 0..stand.worker.io().len() {
            let request = stand.worker.io().pop().unwrap();
            if let io::Request::Connect { .. } = request {
                assert!(stand.worker.git.ready);
                dials += 1;
            } else {
                stand.below.out.io.push(request);
            }
        }
        stand.drain();
        if !stand.input() {
            break;
        }
    }
    assert_eq!(dials, 1);
}

#[test]
fn an_unusable_git_refuses_the_worker_naming_its_program() {
    let mut stand = fixtures::WorkerStand::new();
    stand.below.script.invocations.front_mut().unwrap().transcript.stdout = b"git version 2.40.0\n".as_slice().into();
    for _ in 0_u16..1000 {
        if !stand.turn() {
            break;
        }
    }
    assert_eq!(
        stand.worker.refused().unwrap().problems.get(0),
        Some(&Problem {
            path: Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"git"),
            what: What::Form,
        })
    );
    assert!(!stand.worker.git.ready);
}

#[test]
fn the_component_s_file_operations_leave_through_the_worker() {
    let mut stand = fixtures::WorkerStand::new();
    for _ in 0_u16..1000 {
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        if let Some(submit) = stand.worker.submitted().pop() {
            assert_eq!(crate::tokens::prefix(submit.op), 6);
            let kernel::Op::MakeDirectory { dir: root, name: path, mode } = submit.kind else {
                panic!("layout starts with a directory")
            };
            assert_eq!(root, kernel::Fd::new(9));
            assert_eq!(path.as_ref(), b"trash");
            assert_eq!(mode, 0o700);
            return;
        }
        stand.drain();
        stand.input();
    }
    panic!("version success starts the layout");
}

fn ended(worker: &mut Worker) -> Option<Ending> {
    let mut last = None;
    while let Some(fact) = worker.root.pop_fact() {
        if let root::Fact::Checkout { fact: Fact::Ended { done, .. } } = fact {
            last = Some(done);
        }
    }
    last
}
