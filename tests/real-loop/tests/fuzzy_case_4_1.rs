//! Reference case 4.1 hosts a director, owner, coder and reviewer through git
//! and landing, well past the real-loop story's 1 s guide; hence a fuzzy binary.
//! Its time is the change procedure's landing path, about 2.7 s of CPU; it moves
//! to the default suite once that path's cost is cut (the engine's empty Resume,
//! jig's empty completions).
use temper_real_loop_world::{
    case::{
        self,
        oracle::{self, Observed},
    },
    run_forge,
};

#[test]
fn a_director_delegates_a_design_and_gives_its_green_light() {
    // Leaves out A's survey before its design; A's instructions and pool;
    // which workers have room; a host kind (one worker); Markdown's fast
    // path (the fake CI passes at once): domain/testing.md, 5.3's 4.1 row;
    // and the goal's issue, whose edit's recovery holds a closing goal
    // (M1's finding; forge_route.rs project_goal projects only in a home).
    let ran = run_forge(case::fixture(), case::scripts::step_4_1, case::acts::step_4_1);
    oracle::assert_met(&oracle::step_4_1(&Observed::from_ran(&ran, case::acts::READS_4_1)));
}
