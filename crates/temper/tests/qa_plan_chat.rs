//! A `ChatGPT` plan's allowance, park, wake and Close path (protocol/configuration.md, 5; domain/testing.md, 2.6); a QA pass run by `--profile qa`.
pub mod support;

#[test]
fn signed_in_plan_reads_the_allowance_turn_and_closes_once() {
    let scratch = support::Scratch::startup("qa-plan-chat-lasting");
    let issuer = support::sign_in::configuration(scratch.path());
    support::sign_in::answered(scratch.path(), issuer);
    scratch.remove();
}

#[test]
fn signed_in_plan_holds_one_past_allowance_and_release_repeats_once() {
    let scratch = support::Scratch::startup("qa-plan-chat-exceeded");
    let issuer = support::sign_in::configuration(scratch.path());
    support::sign_in::exceeded(scratch.path(), issuer);
    scratch.remove();
}
