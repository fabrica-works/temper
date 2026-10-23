//! Reference case 5.4: new direction and deferred landing news; overlap wake is a finding.
use temper_real_loop_world::{
    Exit,
    case::{
        self,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

#[test]
fn a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_without_an_overlap_wake_or_a_closing_comment()
 {
    // Leaves out the overlap wake (finding): crates/temper-engine-domain/src/route/forge_route.rs:1026 filters
    // goal_members by task.root == goal, so h1 (root 1, goal 3) is omitted
    // from B's paths; temper-engine-domain-forge/src/domain.rs:3918 classifies the landing Kept. The news
    // still waits for B's release and is told once. The proposed repair is
    // to collect the goal's subtree, rather than compare tree roots.
    // Leaves out the kept branch: domain/forge.md 8.6 deletes it; its closed
    // pull keeps the commits. Leaves out the closing comment, a finding in
    // the director's backlog: crates/temper-engine-domain-forge/src/domain.rs:3049
    // writes Close, then DeleteBranch at :3088; tests/engine/forge/tests/simulation.rs:1295
    // checks those two writes alone. Leaves out x3 past its cap: a running
    // task's deadline never fires (jig's crates/jig-domain-tasks/src/domain.rs:698
    // and crates/jig-domain-core/src/run.rs:121; case_5_3.rs).
    // Leaves out enforced grants (draft/temper-on-temper.md 6.7): A's grant is
    // its instructions' convention; the footprint detects overlap at landing.
    // Leaves out the hunks (news names commits), B's rebase (its planned h1
    // is cancelled to go again from new main), and the twenty minutes.
    // Three one-slot workers: x3's review and h1's coder can remain held
    // together, with one slot for A/a1 or B. Waiting owner runs park.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_5_4(),
        case::scripts::step_5_4,
        || Box::new([]),
        case::acts::step_5_4,
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
    let observed = Observed::from_ran(&ran, case::acts::READS_5_4);
    oracle::assert_met(&oracle::step_5_4(&observed));
}
