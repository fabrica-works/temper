use super::fixtures;
use jig_client::{
    contract as c,
    words::{DecisionWord, Word},
};
use skein_ui::{Builder, phrases::Hole};
const FUNDS: u64 = 40;
const LEFT: u64 = 120;
const AFTER: u64 = 80;
#[test]
fn proposal_funding_uses_the_mockups_pool_sentence() {
    let words = fixtures::words();
    assert_eq!(
        words
            .say_word(
                Word::Decision(DecisionWord::Funds),
                &[Hole::Amount(FUNDS), Hole::Amount(LEFT), Hole::Amount(AFTER)]
            )
            .as_ref(),
        b"Funds 40 from your pool \xc2\xb7 120 left, 80 after",
    );
}
#[test]
fn a_result_shows_its_committed_words_in_temper_nodes() {
    let mut builder = Builder::new(&fixtures::tree_limits());
    let result = c::ResultValue::Report(c::DocumentText::complete(
        c::Text::new(Box::from(*b"Ready to review"), fixtures::domain_limits().client.domain.text)
            .expect("report fits"),
    ));
    crate::cards::value(&mut builder, &result, &fixtures::words());
    assert_eq!(fixtures::text(&builder.finish()).as_ref(), b"Ready to review");
}
const OTHER_TASK: u64 = 12;
const ENTRY: u64 = 11;
const POOL: u64 = 7;
const HOLDS: [(c::HoldCategory, &[u8]); 3] = [
    (c::HoldCategory::Dependency { task: OTHER_TASK }, b"Dependency task 12"),
    (c::HoldCategory::Uncertain { entry: ENTRY }, b"Uncertain outbox entry 11"),
    (c::HoldCategory::PoolLost { pool: POOL }, b"Allocation lost from pool 7"),
];
#[test]
fn numbered_hold_reasons_keep_the_documents_supplied_identity() {
    for (reason, expected) in HOLDS {
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::cards::hold(&mut builder, &reason, &fixtures::words());
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected);
    }
}
