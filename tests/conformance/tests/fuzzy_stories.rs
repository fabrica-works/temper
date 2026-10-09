use jig_conformance::Cut;
use temper_conformance_world::stories;

#[test]
fn every_temper_story_is_crashed_at_every_commit() {
    for story in stories::ALL {
        let baseline = stories::run(story, Cut::None, 101).expect("baseline");
        eprintln!("{story:?}: {} durable commit cuts, {} baseline iterations", baseline.commits, baseline.iterations);
        for cut in 1..=baseline.commits {
            stories::run(story, Cut::AfterCommit(cut), 101)
                .unwrap_or_else(|error| panic!("{story:?}, cut {cut}: {error:?}"));
        }
    }
}
