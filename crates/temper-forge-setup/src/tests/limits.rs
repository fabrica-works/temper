use super::fixtures;
use crate::{check, limits, problem};
#[test]
fn the_small_profile_passes_the_setup_limit_checks() {
    assert_eq!(limits::check(&limits::small()), Ok(()));
    assert_eq!(
        check::checked(&fixtures::document(fixtures::TWO), 0, &fixtures::members(), &[]).unwrap().setup,
        fixtures::setup()
    );
}
#[test]
fn a_backoff_past_its_maximum_names_both_limits() {
    for (name, value, expected) in [
        (b"setup.backoff_ms".as_slice(), 6000, b"application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n".as_slice()),
        (b"setup.backoff_max_ms", 120_000, b"application.limits.overrides.setup.backoff_max_ms: must agree with application.limits.overrides.setup.deadline_ms\n"),
    ] {
        let problems = check::checked(&fixtures::document(fixtures::TWO), 0, &fixtures::members(), &[(name.into(), value)]).unwrap_err();
        assert_eq!(&*problem::lines(&problems), expected);
    }
}
#[test]
fn an_unknown_zero_or_overflowing_limit_is_refused() {
    for (name, value, expected) in [
        (b"box.restarts".as_slice(), 1, limits::Refused::Unknown),
        (b"setup.orgs", 0, limits::Refused::Range),
        (b"setup.orgs", u64::MAX, limits::Refused::Range),
        (b"setup.deadline_ms", u64::MAX, limits::Refused::Range),
    ] {
        assert_eq!(limits::apply(&mut limits::small(), name, value), Err(expected));
    }
}
