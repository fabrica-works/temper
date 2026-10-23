//! Reference case 5.2: Stop holds B alone; its delegates continue and news waits.
use temper_real_loop_world::{
    Exit,
    case::{
        self,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

#[test]
fn a_stopped_owners_delegates_go_on_and_its_news_waits_for_its_release_without_a_park_or_a_retired_charter() {
    // Leaves out the director parking B (there is no agent Stop), retiring the
    // helper's charter (no runtime withdrawal; live tasks refuse restoration
    // without it), and a second helper. A held review stands for h1 in flight,
    // not already ready in kit's queue: this fixture's CI passes at once.
    // Nothing acts on the results until B's one release; the person's tour of
    // every owner's work is left out (domain/testing.md, 5.3's 5.2 row).
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_5_2(),
        case::scripts::step_5_2,
        case::scripts::step_5_2_rules,
        case::acts::step_5_2,
    );
    for (at, called) in ran.calls().iter().enumerate() {
        assert_eq!(
            called.exit,
            Some(Exit::Code(0)),
            "call {at}: {}\n{}",
            String::from_utf8_lossy(&called.stderr),
            String::from_utf8_lossy(&called.stdout)
        );
    }
    let observed = Observed::from_ran(&ran, case::acts::READS_5_2);
    oracle::assert_met(&oracle::step_5_2(&observed));
}
