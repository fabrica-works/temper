use crate::limits::{self, Refused, Unfit};
use skein_lib::Duration;
#[test]
fn the_small_profile_has_the_five_values_of_the_plan() {
    assert_eq!(
        limits::small(),
        limits::Limits {
            backoff: Duration::from_millis(500),
            backoff_max: Duration::from_secs(30),
            restarts: 5,
            window: Duration::from_secs(300),
            grace: Duration::from_secs(10)
        }
    );
    assert_eq!(limits::check(&limits::small()), Ok(()));
    assert!(limits::worst_case(&limits::small(), 6).expect("fits") > 0, "state is priced");
}
#[test]
fn a_backoff_above_its_maximum_names_both_limits() {
    let limits = limits::Limits { backoff: Duration::from_secs(31), ..limits::small() };
    assert_eq!(limits::check(&limits), Err(Unfit::Backoff));
    assert_eq!(Unfit::Backoff.names(), &[b"box.backoff_ms".as_slice(), b"box.backoff_max_ms"]);
}
#[test]
fn a_maximum_not_below_the_window_names_both_limits() {
    let limits = limits::Limits { window: Duration::from_secs(30), ..limits::small() };
    assert_eq!(limits::check(&limits), Err(Unfit::Window));
    assert_eq!(Unfit::Window.names(), &[b"box.backoff_max_ms".as_slice(), b"box.window_ms"]);
}
#[test]
fn restarts_above_sixty_four_name_their_limit() {
    let limits = limits::Limits { restarts: 65, ..limits::small() };
    assert_eq!(limits::check(&limits), Err(Unfit::Restarts));
    assert_eq!(Unfit::Restarts.names(), &[b"box.restarts".as_slice()]);
}
#[test]
fn a_name_that_is_no_box_limit_is_unknown() {
    for name in [b"backoff_ms".as_slice(), b"box.nope", b"setup.restarts"] {
        assert_eq!(limits::apply(&mut limits::small(), name, 1), Err(Refused::Unknown), "{name:?}");
    }
}
#[test]
fn a_value_past_u32_is_out_of_range() {
    for name in
        [b"box.backoff_ms".as_slice(), b"box.backoff_max_ms", b"box.restarts", b"box.window_ms", b"box.grace_ms"]
    {
        assert_eq!(limits::apply(&mut limits::small(), name, 4_294_967_296), Err(Refused::Range), "{name:?}");
        assert_eq!(limits::apply(&mut limits::small(), name, 0), Ok(()), "zero fits {name:?}");
    }
}
