//! A chat and a pushed change, with the same referee for both providers.
pub mod support;
use skein_lib::Duration;
#[test]
fn a_real_model_answers_a_chat_and_pushes_a_small_change() {
    let mode = support::live::mode();
    let scratch = support::Scratch::new("live");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::live::configure(scratch.path(), &mode, &forgejo);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        support::live::run(scratch.path(), &forgejo, peer, Duration::from_secs(840))
    }));
    let completions = support::live::completions(scratch.path());
    eprintln!("live completions={completions}; mode={mode:?}; state={}", scratch.path().display());
    let seen = result.unwrap_or_else(|failed| std::panic::resume_unwind(failed));
    eprintln!("live completions={completions}; wall={}s; mode={mode:?}", seen.wall_seconds);
    assert_eq!((seen.seen.signed_in, seen.seen.finished), (1, true), "{}", String::from_utf8_lossy(&seen.seen.stderr));
    assert_eq!(forgejo.branch_head(&seen.branch), Some(seen.pushed), "the branch is at the run's commit");
    assert!((1..=25).contains(&completions), "at most 25 completions and at least one: {completions}");
    forgejo.stop();
    scratch.remove();
}
