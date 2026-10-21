use super::fixtures;
use crate::{
    setup::{Mode, Setup},
    steps,
};
use alloc::boxed::Box;
fn names(setup: &Setup, expected: &[(&[u8], bool)]) {
    let steps = steps(setup, &flags()).always;
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
    let commands = steps(&fixtures::setup(), &flags()).always;
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
fn a_fake_forge_keeps_its_reconcile_without_process_steps() {
    names(
        &fixtures::fake_setup(),
        &[
            (b"prepare 2", false),
            (b"forgejo 2", true),
            (b"runner 2", true),
            (b"reconcile 1", false),
            (b"reconcile 2", false),
            (b"reconcile 3", false),
            (b"serve", true),
        ],
    );
    assert!(steps(&fixtures::fake_setup(), &flags()).asked.is_empty());
}
#[test]
fn an_owned_forge_without_a_runner_has_no_runner_step() {
    let mut setup = fixtures::setup();
    match &mut setup.entries[1].mode {
        Mode::Owned(owned) => owned.runner = None,
        Mode::Managed { .. } | Mode::Fake(_) => panic!("owned"),
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

#[test]
fn asked_groups_keep_their_place_and_stages_without_reconcile() {
    for runner in [false, true] {
        let mut setup = fixtures::setup();
        setup.entries[1].asked = true;
        if !runner {
            let Mode::Owned(owned) = &mut setup.entries[1].mode else { panic!("owned") };
            owned.runner = None;
        }
        let commands = steps(&setup, &flags());
        assert_eq!(commands.always.len(), 2);
        for (step, name) in commands.always.iter().zip([b"reconcile 1".as_slice(), b"serve"]) {
            assert_eq!(step.name(), name);
        }
        assert_eq!(commands.asked.len(), 1);
        let group = &commands.asked[0];
        assert_eq!((group.forge, group.before), (2, 0));
        assert_eq!(group.steps.len(), if runner { 3 } else { 2 });
        assert_eq!(group.steps[0].0, crate::Stage::Prepare);
        for (stage, step) in group.steps.iter().skip(1) {
            assert_eq!(*stage, crate::Stage::Run);
            assert!(step.is_long());
        }
    }
    for (stage, word) in
        [(crate::Stage::Fetch, b"fetch".as_slice()), (crate::Stage::Prepare, b"prepare"), (crate::Stage::Run, b"run")]
    {
        assert_eq!(stage.word(), word);
    }
    let mut setup = fixtures::setup();
    let mut other = setup.entries[1].clone();
    other.forge = 3;
    other.asked = true;
    setup.entries = Box::new([setup.entries[1].clone(), other]);
    assert_eq!(steps(&setup, &flags()).asked[0].before, 3);
}
