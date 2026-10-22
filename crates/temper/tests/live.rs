//! A person signs in with `ChatGPT`, then a real model answers and pushes a change.
pub mod support;
use skein_lib::Duration;
#[test]
#[ignore = "live: a person's sitting, run by scripts/proof-live.sh"]
fn a_real_model_answers_a_chat_and_pushes_a_small_change() {
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let scratch = support::Scratch::live();
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    support::live::configure(scratch.path(), &forgejo);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        support::live::run(scratch.path(), &forgejo, Duration::from_secs(2_580))
    }));
    let completions = support::live::completions(scratch.path());
    eprintln!(
        "live completions={completions}; wall={}s; state={}",
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000,
        scratch.path().display()
    );
    let seen = result.unwrap_or_else(|failed| std::panic::resume_unwind(failed));
    eprintln!("live completions={completions}; wall={}s", seen.wall_seconds);
    assert_eq!((seen.seen.signed_in, seen.seen.finished), (1, true), "{}", String::from_utf8_lossy(&seen.seen.stderr));
    assert_eq!(forgejo.branch_head(&seen.branch), Some(seen.pushed), "the branch is at the run's commit");
    assert!((1..=25).contains(&completions), "at most 25 completions and at least one: {completions}");
    forgejo.stop();
    scratch.remove();
}
