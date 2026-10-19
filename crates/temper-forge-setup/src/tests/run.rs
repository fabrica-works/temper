use super::fixtures;
use crate::{
    run::{self, Refusal},
    spec::Which,
};
#[test]
fn the_run_refuses_an_absent_preparation() {
    assert_eq!(run::admit_state(&fixtures::launch(Which::Forgejo), None), Err(Refusal::NotPrepared));
}
#[test]
fn the_run_refuses_an_unfinished_preparation() {
    let mut state = fixtures::prepared();
    state.prepared = false;
    assert_eq!(run::admit_state(&fixtures::launch(Which::Forgejo), Some(&state)), Err(Refusal::Unfinished));
}
#[test]
fn the_run_refuses_an_older_preparation() {
    let mut state = fixtures::prepared();
    state.forgejo = b"0.0.0".as_slice().into();
    assert_eq!(
        run::admit_state(&fixtures::launch(Which::Forgejo), Some(&state)),
        Err(Refusal::Versions {
            forgejo: b"0.0.0".as_slice().into(),
            runner: Some(skein_forge::forgejo::pin::RUNNER_VERSION.as_bytes().into()),
            course: crate::upgrade::Course::Older,
        })
    );
}
#[test]
fn the_run_refuses_a_newer_preparation() {
    let mut state = fixtures::prepared();
    state.forgejo = b"999.0.0".as_slice().into();
    assert_eq!(
        run::admit_state(&fixtures::launch(Which::Forgejo), Some(&state)),
        Err(Refusal::Versions {
            forgejo: b"999.0.0".as_slice().into(),
            runner: Some(skein_forge::forgejo::pin::RUNNER_VERSION.as_bytes().into()),
            course: crate::upgrade::Course::Newer,
        })
    );
}
#[test]
fn the_run_refuses_a_different_runner_preparation() {
    let mut state = fixtures::prepared();
    state.runner = None;
    assert_eq!(
        run::admit_state(&fixtures::launch(Which::Forgejo), Some(&state)),
        Err(Refusal::Runner { recorded: false })
    );
}
#[test]
fn the_run_refuses_a_runner_the_setup_lacks() {
    let mut owned = fixtures::owned();
    owned.runner = None;
    assert_eq!(run::launch(&owned, &fixtures::place(), Which::Runner), Err(Refusal::NoRunner));
}
#[test]
fn the_run_refuses_another_forgejo_binary() {
    assert_eq!(
        run::admit_binary(&fixtures::launch(Which::Forgejo), "other binary"),
        Err(Refusal::Binary { printed: "other binary".into() })
    );
}
#[test]
fn the_run_refuses_a_runner_version_with_an_extra_digit() {
    let line = fixtures::suffixed_line(Which::Runner, "0");
    assert_eq!(run::admit_binary(&fixtures::launch(Which::Runner), &line), Err(Refusal::Binary { printed: line }));
}
#[test]
fn the_run_accepts_a_preparation_at_the_pins() {
    for which in [Which::Forgejo, Which::Runner] {
        assert_eq!(run::admit_state(&fixtures::launch(which), Some(&fixtures::prepared())), Ok(()));
    }
}
#[test]
fn the_run_accepts_a_binary_at_the_pin() {
    for which in [Which::Forgejo, Which::Runner] {
        assert_eq!(run::admit_binary(&fixtures::launch(which), &fixtures::line(which)), Ok(()));
    }
}
#[test]
fn the_run_accepts_a_forgejo_build_suffix() {
    assert_eq!(
        run::admit_binary(&fixtures::launch(Which::Forgejo), &fixtures::suffixed_line(Which::Forgejo, "+build")),
        Ok(())
    );
}
#[test]
fn the_launch_names_the_selected_lock_and_state() {
    let forgejo = fixtures::launch(Which::Forgejo);
    let runner = fixtures::launch(Which::Runner);
    assert_eq!(forgejo.lock.as_ref(), b"/srv/forge-2/forgejo.lock");
    assert_eq!(runner.lock.as_ref(), b"/srv/forge-2/runner.lock");
    assert_eq!(forgejo.state.as_ref(), b"/srv/forge-2/state");
}

#[test]
fn the_run_names_prepare_for_a_runner_removed() {
    let mut launch = fixtures::launch(Which::Forgejo);
    launch.runner = false;
    assert_eq!(run::admit_state(&launch, Some(&fixtures::prepared())), Err(Refusal::Runner { recorded: true }));
}
