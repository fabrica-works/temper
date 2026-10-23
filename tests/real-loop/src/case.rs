//! The reference case's cast, accounts and first kit tree (domain/testing.md, 5.3).
use crate::{fixtures, forge, landing};
use serde_json::json;
use skein_fake_forge_domain::{self as domain, api};

pub mod acts;
pub mod oracle;
pub mod scripts;

// Fresh deployment numbers; the oracle checks them against the person's documents
// and A's question's tool output and the answer delivered to A.
pub const PROJECT: u64 = 1;
pub const DIRECTOR: u64 = 1;
pub const A: u64 = 2;
pub const DESIGN: u64 = 3;
pub const QUESTION: u64 = 6;
// After step_4_1's chat and A's delegation create tasks 1..=5,
// step_4_3's chat creates CHECK; its delegate creates B, which delegates BENCHMARK.
pub const CHECK: u64 = 6;
pub const B: u64 = 7;
pub const BENCHMARK: u64 = 8;
// The change procedure in step_4_1 creates its coder, then its reviewer.
pub const CODER: u64 = 4;
pub const REVIEW: u64 = 5;
pub const CHECK_PERSON: &str = "@case_check Schedule B's benchmark and run the two-hour check.";
pub const B_SPEC: &str = "@case_b Own the benchmark; delegate it and report its ending.";
pub const BENCHMARK_SPEC: &str = "@case_benchmark Run the benchmark and wait in place.";
pub const BENCHMARK_RUNNING: &str = "The benchmark is running.";
pub const CHECK_REASON: &str = "The two-hour check found the benchmark stalled; cancel it.";
pub const B_REPORT: &str = "The benchmark was cancelled; B has nothing left running.";
pub const CHECK_REPORT: &str = "The two-hour check cancelled the stalled benchmark and B reported.";
pub const FIRST_ACCOUNT: u64 = 1;
pub const SECOND_ACCOUNT: u64 = 3;
pub const SECOND_KEY_FILE: &str = "second.key";
pub const DEFAULT_MODEL: &str = "fake";
pub const DEFAULT_MODEL_NUMBER: u8 = 0;
pub const SURVEY: u64 = 3;
pub const PACE_QUESTION_MESSAGE: u64 = 1;
pub const PACE_PERSON: &str = "@case_director Ask about the pace and keep my answer.";
pub const PACE_QUESTION: &str = "Claude spends eight times what the week can sustain. I recommend moving what needs no taste to Codex and switching s2 to the second subscription. Keep the pace?";
pub const PACE_ANSWER: &str = "Keep the pace while the hardware allows; move what needs no taste.";
pub const PACE_NOTE: &str = "Pace";
pub const PACE_PENDING: &str = "The pace question is waiting for your answer.";
pub const PACE_A_SPEC: &str =
    "@case_a Ask the director whether to keep the pace; on its direction, have a survey delegated and report.";
// Distinct from the initial question's cue after smith recaps on a model change.
pub const PACE_A_RESUMED_SPEC: &str = "@case_a resumed Have the survey delegated and report on the pace direction.";
pub const HELD_PERSON: &str = "@case_director Delegate A's report and wait for it.";
pub const HELD_A_SPEC: &str = "@case_a Report when your account allows the work to continue.";
pub const SURVEY_SPEC: &str = "@case_survey Survey the work that needs no taste and report.";
pub const RESUMED_REPORT: &str = "The work continued on its account.";
pub const PACE_REPORT: &str = "The pace is kept and the survey is done.";
pub const SURVEY_REPORT: &str = "Work needing no taste can move.";
// The fake forge's one repository stands as kit, since forge.rs names it,
// until the fixture holds two.
pub const KIT: &str = "acme/app";
pub const OWNER_MODEL: &str = "owner";
pub const OWNER_EFFORT: &str = "xhigh";
// Model and effort numbers are their configuration order, after the default model.
pub const OWNER_MODEL_NUMBER: u8 = 1;
pub const OWNER_EFFORT_NUMBER: u8 = 0;
pub const DRAFT: &str = "docs/drafts/export.md";
pub const DESIGN_DOC: &str = "docs/design/export.md";
pub const OWNERS_DOC: &str = "docs/process/owners.md";
pub const DRAFT_WORDS: &str = "# Export draft\n\nExport the application's data.\n";
pub const OWNERS_WORDS: &str = "# Owners\n\nAn owner designs its draft, reports, then implements it.\n";
pub const DESIGN_WORDS: &str = "# Export\n\nDesigned by A from the draft at its commit.\n";
pub const DESIGN_LINE: &str = "Designed by A from the draft at its commit.";
pub const PERSON: &str = "@case_director Export's draft has landed; schedule it, priority 1.";
pub const A_SPEC_PREFIX: &str = "@case_a Own the draft Export: docs/drafts/export.md at kit's commit ";
pub const A_SPEC_SUFFIX: &str = ". Design it as a change of Markdown alone, send me your design report as a question, and finish on my green light.";
pub const SCHEDULED: &str = "Scheduled: A owns Export at priority 1.";
pub const GREEN_LIGHT: &str = "Green light: go on.";
pub const ANSWERED: &str = "Answered A.";
pub const DIRECTOR_REPORT: &str = "A designed Export and has its green light.";
pub const DESIGN_SPEC: &str = "@case_design Write Export's design at docs/design/export.md, Markdown alone.";
pub const DESIGN_DELEGATED: &str = "Design delegated.";
pub const DESIGN_REPORT: &str =
    "Export's design has landed on kit's main at docs/design/export.md: one Markdown file. Green light to go on?";
pub const ASKED: &str = "Asked the director.";
pub const A_REPORT: &str = "Export's design landed, and the director gave its green light.";
pub const DESIGN_TITLE: &str = "Export's design";
pub const REVIEWED: &str = "Checked";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Director,
    A,
    Design,
    Review,
    Survey,
    Check,
    B,
    Benchmark,
    /// The director whose run crosses a worker restart.
    Hang,
    /// A's next phase, asking that director for its direction.
    Phase,
    Rules,
    Scope,
    Pause,
    Owner,
    Bowner,
    Helper,
    Implementer,
    H1Coder,
    Shape,
    Mover,
    Plan,
    Planner,
    Subowner,
}
impl Agent {
    #[must_use]
    pub fn cue(self) -> &'static [u8] {
        match self {
            Self::Plan => b"@case_plan_director",
            Self::Planner => b"@case_planner",
            Self::Subowner => b"@case_subowner",
            Self::Shape => b"@case_shape",
            Self::Mover => b"@case_mover",
            Self::Rules => b"@case_rules",
            Self::Scope => b"@case_scope",
            Self::H1Coder => b"@case_h1",
            Self::Pause => b"@case_pause",
            Self::Owner => b"@case_owner",
            Self::Bowner => b"@case_bowner",
            Self::Helper => b"@case_helper",
            Self::Implementer => b"@case_implementer",
            Self::Hang => b"@case_hang",
            Self::Phase => b"@case_phase",
            Self::Check => b"@case_check",
            Self::B => b"@case_b ",
            Self::Benchmark => b"@case_benchmark",
            Self::Director => b"@case_director",
            Self::A => b"@case_a ",
            Self::Design => b"@case_design",
            Self::Survey => b"@case_survey",
            Self::Review => b"Review this change at the named head",
        }
    }
}

#[must_use]
pub fn kit_tree() -> Box<[api::File]> {
    let mut tree = (forge::fixture().setup)().tree.into_vec();
    for (path, words) in [(DRAFT, DRAFT_WORDS), (OWNERS_DOC, OWNERS_WORDS)] {
        tree.push(api::File { path: path.as_bytes().into(), content: words.as_bytes().into() });
    }
    tree.into_boxed_slice()
}

#[must_use]
pub fn draft_commit() -> String {
    // State::new uses this configuration and seed. repository stores the first
    // tree without a timestamp; real-loop wall skew cannot change its identity.
    let config = temper_fake_git_hosted::fixtures::CONFIG;
    let mut domain = domain::Domain::new(&config, fixtures::SEED);
    let first = domain::repository(&mut domain, &config, (fixture().setup)());
    let object = skein_fake_forge_protocol::routes::object(first);
    let mut words = String::new();
    for byte in object.bytes {
        use std::fmt::Write;
        write!(words, "{byte:02x}").expect("commit hex");
    }
    words
}

#[must_use]
pub fn fixture() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (landing::fixture().setup)();
            setup.tree = kit_tree();
            setup.checks.cue =
                Some(api::Cue::File { path: DESIGN_DOC.as_bytes().into(), green: DESIGN_LINE.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (landing::fixture().configure)(value, agent, forge);
            value["engine"]["limits"]["overrides"]["people.projects"] = 2.into();
            value["engine"]["limits"]["overrides"]["authority.projects"] = 2.into();
            value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
            // A runs on the director's explicit choice, distinct from the default.
            let mut model = value["engine"]["llm"]["models"][0].clone();
            model["name"] = OWNER_MODEL.into();
            model["efforts"] = json!([OWNER_EFFORT]);
            value["engine"]["llm"]["models"].as_array_mut().expect("models").push(model);
            value["engine"]["llm"]["providers"][0]["models"] = json!(["fake", OWNER_MODEL]);
            value["engine"]["charters"][0]["alternatives"] = json!([OWNER_MODEL]);
            let mut model = agent["endpoints"][0]["models"][0].clone();
            model["name"] = OWNER_MODEL.into();
            model["efforts"] = json!([OWNER_EFFORT]);
            agent["endpoints"][0]["models"].as_array_mut().expect("agent models").push(model);
            // The director delegates, messages, amends and keeps project notes.
            value["engine"]["rules"]["ceiling"]["tools"] = 391.into();
            value["engine"]["rules"]["ceiling"]["notes"] = 2.into();
            // The extra owner adds one delegation level before the change procedure.
            value["engine"]["rules"]["ceiling"]["delegation"]["depth"] = 5.into();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["tools"] = 391.into();
                project["policy"]["ceiling"]["notes"] = 2.into();
                project["policy"]["roles"][0]["authority"]["notes"] = 2.into();
                project["policy"]["ceiling"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["tools"] = 391.into();
            }
            // A tracked owner reserves core news, landing news, an answer and its
            // delegate's result together; the landing fixture's inbox holds 4 KiB.
            value["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = 16384.into();
            // The director's prescribed final report exceeds the chat's 32 bytes.
            value["engine"]["templates"][0]["contract"]["report"]["words"] = 128.into();
            // The brief includes the full draft commit and its instructions.
            value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 512.into();
            landing::admit(value);
        },
    }
}

/// Keep only the director on its worker; delegates retain the usual parking waits.
#[must_use]
pub fn fixture_placed() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            // The chat and A both use charter 1 in the base fixture. A waiting
            // run keeps its one slot until a message resumes it, so give only
            // the director a charter whose wait exceeds PATIENCE (10 s).
            let mut director = value["engine"]["charters"][0].clone();
            director["number"] = 2.into();
            director["waiting_ms"] = 60000.into();
            value["engine"]["charters"].as_array_mut().expect("charters").push(director);
            value["engine"]["limits"]["overrides"]["tasks.charters"] = 2.into();
            value["engine"]["templates"][0]["executor"]["charter"] = 2.into();
            // The person must also be offered the new executor for that chat.
            let mut kinds = value["engine"]["rules"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("executor kinds").push(json!({"kind":"charter","number":2}));
            value["engine"]["rules"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
            }
            landing::admit(value);
        },
        ..fixture()
    }
}

#[must_use]
pub fn fixture_4_4() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            // The survey's delegate and wait share one completion. Its
            // resumed script then cannot collide with the model-change recap.
            agent["profile"]["declared"]["calls_per_response"] = 2.into();
            value["workers"][0]["limits"]["overrides"]["hub.run_calls"] = 2.into();
        },
        ..fixture()
    }
}

/// The check keeps all three runs in place, with timer and cancellation authority.
#[must_use]
pub fn fixture_check() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            // One adopted repository contains one pull. Its initial scan and
            // the scripted forge hooks supply the listing changes; repeating
            // the listing every 10 ms adds traffic throughout the timer story.
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value["application"]["limits"]["overrides"][format!("forge.client.{name}")] = (story_ms + 1).into();
            }
            // Branch watches keep their 100 ms slow reconciliation: at most
            // 2 * (PATIENCE / 100 ms + 1) steady reads for the two branches.
            // Subscribe is family 32; cancel shares control (4), already in 391.
            let tools = 0x0187 | 0x0020;
            value["engine"]["rules"]["ceiling"]["tools"] = tools.into();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["tools"] = tools.into();
                project["policy"]["roles"][0]["authority"]["tools"] = tools.into();
            }
            // Announce placement and wait in one completion, with serial tools.
            agent["profile"]["declared"]["calls_per_response"] = 2.into();
            agent["profile"]["policy"]["parallel_tools"] = 1.into();
            value["workers"][0]["limits"]["overrides"]["hub.run_calls"] = 2.into();
            // The opener's checkout needs an LLM grant and an application git
            // grant. One refuses its coder; two fit the three-worker ring.
            value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 2.into();
            landing::admit(value);
        },
        ..fixture_placed()
    }
}

/// The third-chat scenario starts a fresh deployment; its director is first.
pub const HANG: u64 = 1;
/// A's next phase is this deployment's first delegate.
pub const PHASE: u64 = 2;
/// The person requests the restart scenario.
pub const HANG_PERSON: &str = "@case_hang Delegate A's next phase and answer it once after the workstation goes away.";
/// The brief for A's next phase.
pub const PHASE_SPEC: &str = "@case_phase Own Export's next phase and ask the director for its green light.";
/// A's question retained across the director's retry.
pub const PHASE_QUESTION: &str = "May Export's next phase begin?";
/// The director's single answer.
pub const PHASE_ANSWER: &str = "Begin Export's next phase.";
/// A's completed next-phase report.
pub const PHASE_REPORT: &str = "Export's next phase has its direction.";
/// The committed read barrier before the workstation stops.
pub const HANG_READ: &str = "A's next-phase question is read; its answer is still waiting.";
/// The restarted director's final report.
pub const HANG_REPORT: &str = "The restarted director answered A and its next phase reported.";
/// The first question in this fresh deployment, checked in the read and answer.
pub const PHASE_QUESTION_NUMBER: u64 = 1;

/// The base case's one-slot workers, owner model, notes and placed director.
/// No repository adoption, landing policy, protected tree or check scripts are needed.
#[must_use]
pub fn fixture_4_6() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            value["application"]["landing"]["projects"] = json!([]);
            // The retry path is the same, only its wait is short; the backoff is not what this story tests.
            value["engine"]["limits"]["overrides"]["tasks.retries.agent.base_ms"] = 1.into();
            landing::admit(value);
        },
        setup: forge::fixture().setup,
    }
}

// Act 2, step 5.1 (domain/testing.md, 5.3)
pub const RULES: u64 = 1;
pub const SCOPE: u64 = 2;
pub const SCOPE_QUESTION_NUMBER: u64 = 2;
pub const RULES_HOLD_REVISION: u64 = 1;
pub const RULES_PERSON: &str = "@case_rules Delegate A to ask about x3's scope and wait for its report.";
pub const SCOPE_SPEC: &str = "@case_scope Own x3's scope; ask the director and report on its answer.";
pub const SCOPE_QUESTION: &str = "What is x3's scope in the next milestone?";
pub const SCOPE_ANSWER: &str = "x3 owns the next milestone's design and its defined end.";
pub const RULES_DELEGATED: &str = "A owns the scope question.";
pub const RULES_ANSWERED: &str = "A has its scope answer.";
pub const SCOPE_ASKED: &str = "The scope question awaits the director.";
pub const SCOPE_REPORT: &str = "x3's scope has its direction.";
pub const RULES_REPORT: &str = "A reported x3's scope under the new process.";
pub const SCOPE_QUESTION_HOLD: &[u8] = b"case-scope-question";
pub const RULES_FILE: &str = "AGENTS.md";
pub const DROPPED_RULE: &str = "Owners check shared version numbers before they land.";
pub const OLD_RULES: &str = "# Process\n\nOwners check shared version numbers before they land.\n";
pub const NEW_RULES: &str = "# Process\n\nAn owner takes one mission, at most four milestones, then stops.\nMilestones take 2-3 hours under a 4-hour cap, with a sub-owner each.\nThe plan comes first. Start a fresh session at each milestone's end.\nRun a QA pass beside the next milestone.\n";

#[must_use]
pub fn fixture_5_1() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (forge::fixture().setup)();
            let mut tree = setup.tree.into_vec();
            tree.push(api::File { path: RULES_FILE.as_bytes().into(), content: OLD_RULES.as_bytes().into() });
            setup.tree = tree.into_boxed_slice();
            setup
        },
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            value["engine"]["charters"][1]["resume"] = false.into();
            value["application"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
    }
}

// Act 2, step 5.3 (domain/testing.md, 5.3)
pub const SHAPE: u64 = 1;
pub const MOVER: u64 = 2;
pub const MOVER_QUESTION_NUMBER: u64 = 1;
pub const MOVER_AMENDMENT_REVISION: u64 = 1;
pub const SHAPE_PERSON: &str = "@case_shape Help A move Export to the new shape; amend its instructions when it asks.";
pub const MOVER_SPEC: &str = "@case_mover Own Export; ask the director how to go on under the new shape and report.";
pub const OLD_BRIEF: &str = "Implement Export in one long session.";
pub const MOVER_QUESTION: &str = "How do I go on under the new shape?";
pub const NEW_BRIEF: &str = "x3 finishes the old way, outside any milestone, under the person's two-hour cap; if it doesn't finish, we evaluate together. Then make the milestone plan first.";
pub const SHAPE_REASON: &str = "Continue x3 under the cap, then plan the milestones.";
pub const SHAPE_DELEGATED: &str = "A owns Export under its old brief.";
pub const SHAPE_AMENDED: &str = "A's instructions now answer its question.";
pub const MOVER_ASKED: &str = "The new-shape question awaits the director.";
pub const MOVER_REPORT: &str = "x3 will finish under the cap; the milestone plan comes next.";
pub const SHAPE_REPORT: &str = "A reported under its amended instructions.";

#[must_use]
pub fn fixture_5_3() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            let mut owner = value["engine"]["charters"][0].clone();
            owner["number"] = 2.into();
            // Configuration names an instruction file. Keep it in this fixture's
            // private data tree, also when the cluster separates config and data.
            let directory =
                std::path::Path::new(value["workers"][0]["agent"]["configuration"].as_str().expect("agent path"))
                    .parent()
                    .expect("fixture data");
            let instructions = directory.join("charters/owner.txt");
            fixtures::private(&instructions, OLD_BRIEF.as_bytes()).expect("owner instructions");
            owner["instructions"] = instructions.to_str().expect("instructions path").into();
            value["engine"]["charters"].as_array_mut().expect("charters").push(owner);
            value["engine"]["limits"]["overrides"]["tasks.charters"] = 2.into();
            let mut kinds = value["engine"]["rules"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("executor kinds").push(json!({"kind":"charter","number":2}));
            value["engine"]["rules"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
            }
            // Neither task mounts a repository or delegates a change.
            value["application"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
        ..fixture()
    }
}

// Act 2, step 5.2 (domain/testing.md, 5.3)
// The chat delegates A, the helper, then B; only B creates descendants.
// B delegates its implementer, then h1, which creates its coder and reviewer.
pub const PAUSE_WORKERS: &[&str] = &["ws", "s1", "s2"];
pub const PAUSE: u64 = 1;
pub const OWNER: u64 = 2;
pub const HELPER: u64 = 3;
pub const HELPER_CHARTER: u64 = 3;
pub const BOWNER: u64 = 4;
pub const IMPLEMENTER: u64 = 5;
pub const H1: u64 = 6;
pub const H1_CODER: u64 = 7;
pub const H1_REVIEWER: u64 = 8;
pub const B_HOLD: u64 = 1;
pub const PAUSE_PERSON: &str = "@case_pause Pause every owner but A; I'll go through each one's work and say what resumes. Stop the old shape's leftovers.";
pub const OWNER_SPEC: &str = "@case_owner A remains free to report.";
pub const BOWNER_SPEC: &str = "@case_bowner Delegate your implementer and h1, then await their endings.";
pub const HELPER_SPEC: &str = "@case_helper Wait on the old shape's charter.";
pub const IMPLEMENTER_SPEC: &str = "@case_implementer Finish your running work when released.";
pub const H1_SPEC: &str = "@case_h1 Write docs/design/h1.md, Markdown alone.";
pub const PAUSE_PROPOSAL: &str = "I propose stopping B; the person must stop it.";
pub const HELPER_REASON: &str = "The person has stopped the old shape's leftovers; cancel this helper.";
pub const IMPLEMENTER_RESULT: &str = "B's implementer finished its running work.";
pub const OWNER_REPORT: &str = "A remained free and finished.";
pub const BOWNER_REPORT: &str = "B received its implementer's result and h1's landing after release.";
pub const PAUSE_REPORT: &str = "B reported after release; A finished and the old helper was cancelled.";
pub const H1_DOC: &str = "docs/design/h1.md";
pub const H1_WORDS: &str = "# h1\n\nB's change went on while B was stopped.\n";
pub const H1_LINE: &str = "B's change went on while B was stopped.";
pub const IMPLEMENTER_HOLD: &[u8] = b"case-5-2-implementer";
pub const REVIEW_HOLD: &[u8] = b"case-5-2-review";

#[must_use]
pub fn fixture_5_2() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (fixture().setup)();
            setup.checks.cue =
                Some(api::Cue::File { path: H1_DOC.as_bytes().into(), green: H1_LINE.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            // One host call for each of the three one-slot workers.
            value["engine"]["limits"]["overrides"]["fleet.calls"] = PAUSE_WORKERS.len().into();
            // Initial adoption and forge hooks supply the listing changes,
            // as in fixture_check; repeating the listing adds no evidence.
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value["application"]["limits"]["overrides"][format!("forge.client.{name}")] = (story_ms + 1).into();
            }
            let mut helper = value["engine"]["charters"][0].clone();
            helper["number"] = HELPER_CHARTER.into();
            value["engine"]["charters"].as_array_mut().expect("charters").push(helper);
            value["engine"]["limits"]["overrides"]["tasks.charters"] = 3.into();
            let mut kinds = value["engine"]["rules"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("kinds").push(json!({"kind":"charter","number":HELPER_CHARTER}));
            value["engine"]["rules"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
                // The chat's run, A, the helper and B reserve 10 + 5 + 5 + 25.
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 50.into();
            }
            // The coder's checkout needs both its LLM and application git grants,
            // as in fixture_check. The cluster supplies one slot per worker.
            value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 2.into();
            landing::admit(value);
        },
    }
}

// Act 2, step 5.5 (domain/testing.md, 5.3)
pub const PLAN_DIRECTOR: u64 = 1;
pub const PLANNER: u64 = 2;
pub const SUBOWNER: u64 = 3;
pub const PLAN_QUESTION_NUMBER: u64 = 2;
pub const PLAN_ANSWER_NUMBER: u64 = 3;
pub const PLAN_NOTE_NUMBER: u64 = 1;
pub const A_DEADLINE: u64 = 4_102_444_800_000_000_000;
pub const A1_DEADLINE: u64 = A_DEADLINE - 14_400_000_000_000;
pub const PLAN_ANSWER_PENDING: &str = "Wait for A's first run to park before answering.";
pub const PLAN_ANSWER_HOLD: &[u8] = b"case-5-5-answer";
pub const PLAN_PERSON: &str = "@case_plan_director Give A the mission's deadline and answer its plan.";
pub const PLANNER_SPEC: &str =
    "@case_planner Plan the nearer mission, ask the director, and delegate M1's sub-owner after its green light.";
pub const SUBOWNER_SPEC: &str =
    "@case_subowner Own M1 under its deadline; try to delegate a helper and report the refusal.";
pub const PLAN_NOTE: &str = "Export's nearer mission";
pub const PLAN_WORDS: &str = "A nearer end: this mission ends at the end-to-end story; the rest of Export is a later mission. M1: the path through app and kit's fake. M2: the rest of the story. Each takes 2 to 3 hours. May I use B's files?";
pub const PLAN_ANSWER: &str =
    "Green light: the nearer end, M1 and M2 approved; every ask granted, including B's files.";
pub const SUBOWNER_REFUSAL: &str = "Authority needs proposal: authority ceiling; ";
pub const SUBOWNER_REPORT: &str = "M1's sub-owner could not delegate its own helper.";
pub const PLANNER_REPORT: &str = "The recalled plan started M1's sub-owner under its deadline; it could not delegate.";
pub const PLAN_REPORT: &str = "A's answered plan and M1's sub-owner have reported.";

#[must_use]
pub fn fixture_5_5() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            let mut owner = value["engine"]["charters"][0].clone();
            owner["number"] = 2.into();
            owner["resume"] = false.into();
            value["engine"]["charters"].as_array_mut().expect("charters").push(owner);
            value["engine"]["limits"]["overrides"]["tasks.charters"] = 2.into();
            let mut kinds = value["engine"]["rules"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("kinds").push(json!({"kind":"charter","number":2}));
            value["engine"]["rules"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
            }
            value["application"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
        ..fixture()
    }
}
