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
    support::live::notice(&format!(
        "live completions={completions}; wall={}s; state={}",
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000,
        scratch.path().display()
    ));
    let seen = result.unwrap_or_else(|failed| std::panic::resume_unwind(failed));
    support::live::notice(&format!("live completions={completions}; wall={}s", seen.wall_seconds));
    assert_eq!((seen.seen.signed_in, seen.seen.finished), (1, true), "{}", String::from_utf8_lossy(&seen.seen.stderr));
    assert_eq!(forgejo.branch_head(&seen.branch), Some(seen.pushed), "the branch is at the run's commit");
    assert!((1..=25).contains(&completions), "at most 25 completions and at least one: {completions}");
    forgejo.stop();
    scratch.remove();
}

/// domain/testing.md, 2.7: init's shipped chat chooses change on real Responses.
#[test]
#[ignore = "live: a person's sitting, run by scripts/proof-live.sh forge-land"]
fn a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{self, Face, LivePlan};
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let deadline = started.saturating_add(Duration::from_secs(2_400));
    let store = std::path::PathBuf::from(
        std::env::var_os("TEMPER_LIVE_STORE").expect("run live through scripts/proof-live.sh forge-land"),
    );
    forge_land::reserve_live_store(&store).unwrap_or_else(|message| {
        support::live::notice(&message);
        panic!("{message}")
    });
    let scratch = support::Scratch::live();
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    forge_land::live_deployment(&paths);
    let (cached, fingerprints) = forge_land::cache_forgejo_binaries(&paths[2]);
    let minute = std::process::Command::new("/usr/bin/date")
        .args(["-u", "+%Y-%m-%dT%H:%MZ"])
        .env_clear()
        .output()
        .expect("UTC ISO minute");
    assert!(minute.status.success());
    let tag = format!("live-{}-{}", String::from_utf8(minute.stdout).expect("date UTF-8").trim(), std::process::id());
    let line = format!("Hello from temper, {tag}.");
    let words = format!("Please add the line \"{line}\" to the end of README.md in work/hello and land it.");
    support::live::notice(&format!("TAG {tag}"));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut bot = None;
        let mut previous = None;
        for first in [true, false] {
            let run = if first { "first" } else { "second" };
            let client = store.join(run);
            fs::create_dir(&client).expect("fresh client root");
            fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client root");
            let remaining = deadline.saturating_since(clock.now().now);
            assert!(remaining > Duration::from_secs(10), "40 minute total story budget");
            let outcome = forge_land::run_live(
                LivePlan { paths: &paths, client: &client, first, words: &words, line: &line },
                remaining,
                &Face::Live,
            )
            .unwrap_or_else(|message| panic!("{message}"));
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            assert_eq!(outcome.exit, Some(skein_io::kernel::Exit::Code(0)), "{run}: {stderr}");
            assert_eq!(outcome.left, Some(vec![]), "no surviving children: {stderr}");
            bot = Some(forge_land::assert_forgejo_setup(
                run,
                &outcome.calls[..if first { 7 } else { 4 }],
                &stderr,
                bot,
                &paths[1],
            ));
            let evidence = outcome.checked.expect("landing and reply checked before stopping");
            let identity = forge_land::landed_identity(&evidence.task, &evidence.reads.page);
            assert!(identity.0.contains(&evidence.forge.merge), "temper reads the forge's merge hash");
            assert!(identity.1.starts_with(&format!("PR #{} ·", evidence.forge.number)), "temper reads the same pull");
            if first {
                assert!(evidence.reads.page.contains("CI at its head\nPassed"), "CI passed on temper's change page");
                previous = Some((identity, evidence.forge, evidence.replies, evidence.delegates));
            } else {
                let (before, forge, replies, delegates) = previous.as_ref().expect("first landing");
                assert_eq!(&identity, before, "same change identity after restart");
                assert_eq!(&evidence.forge, forge, "no second push or merge: same branches and pulls");
                assert_eq!(&evidence.delegates, delegates, "same change and producer tasks");
                for (position, turn) in replies {
                    assert_eq!(evidence.replies.get(position), Some(turn), "committed chat reply survives restart");
                }
            }
            for (path, fingerprint) in cached.iter().zip(&fingerprints) {
                assert_eq!(&forge_land::cached_binary_fingerprint(path), fingerprint, "pinned binaries reused");
            }
        }
        let (_, forge, _, _) = previous.expect("landed identity");
        support::live::notice(&format!("TAG {tag}\nPULL {} #{}\nMERGE {}", forge.url, forge.number, forge.merge));
    }));
    let completions = support::live::deployment_completions(&paths);
    support::live::notice(&format!(
        "TAG {tag}; live completions={completions}; wall={}s",
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000
    ));
    result.unwrap_or_else(|failed| {
        let message = failed
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| failed.downcast_ref::<&str>().copied())
            .unwrap_or("live story failed");
        support::live::notice(&format!("FAILURE {message}"));
        std::panic::resume_unwind(failed)
    });
    assert!(clock.now().now <= deadline, "40 minute total story budget");
    assert!((1..=25).contains(&completions), "live completion budget: {completions}");
    scratch.remove();
}
