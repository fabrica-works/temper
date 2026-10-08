use jig_conformance::{
    Cut,
    scenarios::{self, Scenario},
};
use temper_conformance_world::Temper;

#[test]
fn every_shared_scenario_is_crashed_at_every_commit() {
    for scenario in scenarios::ALL {
        let Some(baseline) = scenarios::run::<Temper>(scenario, Cut::None, 101).expect("baseline") else {
            assert_eq!(scenario, Scenario::ShrinkingPool);
            continue;
        };
        eprintln!("{scenario:?}: {} durable commit cuts", baseline.commits);
        for cut in 1..=baseline.commits {
            scenarios::run::<Temper>(scenario, Cut::AfterCommit(cut), 101)
                .unwrap_or_else(|error| panic!("{scenario:?}, cut {cut}: {error:?}"));
        }
    }
}
