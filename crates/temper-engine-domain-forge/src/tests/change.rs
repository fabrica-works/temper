use super::*;

// env() starts at 100 s; the change policy stalls for another 30 s.
const WAIT_UNTIL: Wall = Wall::from_nanos(130_000_000_000);

#[test]
fn an_overtaking_open_outcome_restarts_the_fresh_chain() {
    use skein_forge::api::{Answer, Op, Read};
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::opened_during_chain();
    fixtures::step_change(&mut d);
    let overtaken = fixtures::finish_missing(&mut d);
    let (branch, branch_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, branch, Ok(Answer::Commit(fixtures::HEAD)));
    let (base, base_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, base, Ok(Answer::Commit(fixtures::BASE)));
    let (pull, pull_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, pull, Ok(Answer::Pull(fixtures::open_pull())));
    fixtures::finish_missing(&mut d);
    let finished = fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false }));
    assert_eq!(overtaken.as_ref(), &[]);
    assert_eq!(
        [branch_read, base_read, pull_read],
        [
            Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }),
            Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }),
            Op::Read(Read::PullFor { head: Box::from(b"temper/51".as_slice()), base: Box::from(b"main".as_slice()) }),
        ]
    );
    let [Request::Save { .. }, Request::ChangeDecision { task, decision, entry, evidence }] = finished.as_ref() else {
        panic!("one saved decision")
    };
    assert_eq!((*task, decision, entry, evidence), (fixtures::CHANGE, &change::Decision::Ready, &None, &None));
    assert_eq!(ready(&mut d).as_ref(), &[]);
}

#[test]
fn an_unwoken_chain_finishes_with_its_single_decision() {
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::opened_during_chain();
    let finished = fixtures::finish_missing(&mut d);
    let [Request::Save { .. }, decision] = finished.as_ref() else { panic!("one saved decision") };
    assert_eq!(
        decision,
        &Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Wait { until: Some(WAIT_UNTIL) },
            entry: None,
            evidence: None,
        }
    );
    assert_eq!(ready(&mut d).as_ref(), &[]);
}
