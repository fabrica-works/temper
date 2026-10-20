use super::fixtures;
use crate::{
    setup::{Mode, Setup},
    steps,
};
use alloc::boxed::Box;
fn names(setup: &Setup, expected: &[(&[u8], bool)]) {
    let steps = steps(setup, &flags());
    assert_eq!(steps.len(), expected.len());
    for (step, (name, long)) in steps.iter().zip(expected) {
        assert_eq!(step.name(), *name);
        assert_eq!(step.is_long(), *long);
    }
}
#[test]
fn a_managed_and_an_owned_forge_run_the_six_steps_of_the_design_tree() {
    names(
        &fixtures::setup(),
        &[
            (b"prepare 2", false),
            (b"forgejo 2", true),
            (b"runner 2", true),
            (b"reconcile 1", false),
            (b"reconcile 2", false),
            (b"serve", true),
        ],
    );
    let commands = steps(&fixtures::setup(), &flags());
    let expected: &[&[&[u8]]] = &[
        &[b"forge", b"prepare", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2"],
        &[b"forge", b"run", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2", b"forgejo"],
        &[b"forge", b"run", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2", b"runner"],
        &[b"forge", b"reconcile", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"1"],
        &[b"forge", b"reconcile", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2"],
        &[b"serve", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache"],
    ];
    for (step, expected) in commands.iter().zip(expected) {
        assert_eq!(step.arguments().len(), expected.len());
        for (arg, expected) in step.arguments().iter().zip(*expected) {
            assert_eq!(arg.as_ref(), *expected);
        }
    }
}
#[test]
fn an_owned_forge_without_a_runner_has_no_runner_step() {
    let mut setup = fixtures::setup();
    match &mut setup.entries[1].mode {
        Mode::Owned(owned) => owned.runner = None,
        Mode::Managed { .. } => panic!("owned"),
    }
    names(
        &setup,
        &[
            (b"prepare 2", false),
            (b"forgejo 2", true),
            (b"reconcile 1", false),
            (b"reconcile 2", false),
            (b"serve", true),
        ],
    );
}
#[test]
fn a_managed_forge_alone_runs_reconcile_then_serve() {
    let setup = Setup { entries: Box::new([fixtures::setup().entries[0].clone()]) };
    names(&setup, &[(b"reconcile 1", false), (b"serve", true)]);
}
#[test]
fn an_empty_setup_runs_serve_alone() {
    names(&Setup { entries: Box::new([]) }, &[(b"serve", true)]);
}

fn flags() -> Box<[Box<[u8]>]> {
    Box::new([
        Box::from(b"--config".as_slice()),
        Box::from(b"/config".as_slice()),
        Box::from(b"--data".as_slice()),
        Box::from(b"/data".as_slice()),
        Box::from(b"--cache".as_slice()),
        Box::from(b"/cache".as_slice()),
    ])
}
