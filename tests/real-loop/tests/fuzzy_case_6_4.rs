//! Reference case 6.4: one deployment repair before the waiting change.
use temper_real_loop_world::{Exit, case, run_cluster_forge};

#[test]
fn a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4()
 {
    // A1 is delegated directly, tracked as a goal; A is omitted. One
    // repository stands for kit/app until 6.6. The person's red push stands
    // for app's slow tier, and a1/a2's landings are omitted. Adopting a4 is
    // design (temper-on-temper.md, 6.8): the deployment repairs separately.
    // Ninety minutes is represented by a far-future timer, read armed and
    // removed, never waited out. One repair restores CI; exhaustion of the
    // repairs bound belongs to the lower world. All waits follow events.
    // A ready change discovers the red base only after its blocking gate:
    // release a4's review after the push, then hold the repair's coder while
    // the director arms its timer and the paused change is read.
    // Exact-head freshness asks another a4 review after its clean update;
    // head cues hold only the first review. Coders write and finish in one
    // response. The armed activity prefix completes the ended journal page.
    // Owners/changes use 5.4's six grants and its change/hold helpers.
    // The pause's news to waiting goals is missing (forge.md, 9):
    // start_queue_repair notifies no goal. A1's actual a4 result is checked.
    // a4 writes its own file: its pre-push head keeps the green cue, and
    // updating onto the repaired main needs no conflict coder.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_6_4(),
        case::scripts::step_6_4,
        case::scripts::step_6_4_rules,
        case::acts::step_6_4,
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
    let observed = case::oracle::Observed::from_ran(&ran, case::acts::READS_6_4);
    case::oracle::assert_met(&case::oracle::step_6_4(&observed));
}
