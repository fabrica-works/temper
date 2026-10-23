//! The person's acts, with documents identified by the moment they were read.
use super::{A, DESIGN, DIRECTOR, PERSON, PROJECT};
use crate::{
    argv,
    calls::{self, D1, call, join, number, sign_in, start},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Read {
    BEndedEarlier,
    ImplementerRunning,
    H1Running,
    HelperEnded,
    BHeld,
    BStillHeld,
    ImplementerEnded,
    H1Ended,
    OwnerEnded,
    PauseEnded,
    DirectorEnded,
    AEnded,
    DesignEnded,
    AWaiting,
    Refused,
    Limited,
    PaceAsked,
    Ranked,
    SurveyEnded,
    CheckEnded,
    DirectorActivity,
    DirectorTimer,
    BenchmarkWaiting,
    BenchmarkEnded,
    BEnded,
    /// The retried director reports.
    HangEnded,
    /// The director commits its question read before stopping.
    HangQuestionRead,
    /// The director waits in its first attempt.
    HangWaiting,
    /// A's next phase reports.
    PhaseEnded,
    /// The newest director activity page.
    HangActivity,
    /// The middle director activity page.
    HangActivityOlder,
    /// The oldest director activity page.
    HangActivityFirst,
    RulesWaiting,
    RulesHeld,
    RulesQuestionWaiting,
    ScopeEnded,
    RulesEnded,
    ShapeEnded,
    MoverEnded,
    PlanAnswerPending,
    PlannerParked,
    PlannerEnded,
    PlannerEarlier,
    PlannerFirst,
    PlanEnded,
    SubownerEnded,
}
// Provisioning's key call does not count as a story act. The director's
// ended wait starts first; its delegates' ended documents are read before
// their requesters end, then the director's wait is joined.
const DIRECTOR_ENDED: usize = 3;
const A_ENDED: usize = 7;
const DESIGN_ENDED: usize = 6;
pub const READS_4_1: &[(Read, usize)] =
    &[(Read::DirectorEnded, DIRECTOR_ENDED), (Read::AEnded, A_ENDED), (Read::DesignEnded, DESIGN_ENDED)];

#[must_use]
pub fn step_4_1() -> Vec<calls::Step> {
    vec![
        sign_in(D1),
        // Leave out the goal's issue: recovery of its edit holds a closing goal
        // (M1's finding; forge_route.rs project_goal projects only in a home).
        call(
            D1,
            argv![
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "adopt-kit"
            ],
        ),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "schedule", PERSON]),
        // Keep these watches live: a fresh wait on an ended delegate exits
        // unknown. jig's crates/jig-domain-core/src/routing/views.rs:304 maps
        // documents/small.rs:23-28's missing/ended live row to that refusal.
        // 2071ea76 fixes historical reads only; it cannot admit these watches.
        start(D1, argv!["wait", "task", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(DIRECTOR), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(A), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(DESIGN), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(A), "--until", "ended", "--json"]),
        join(DIRECTOR_ENDED),
    ]
}

// The two held slices share their reads; the refused one adds the person's act.
const HELD_A_ENDED: usize = 3;
const A_WAITING: usize = 5;
const HELD_ACCOUNTS: usize = 6;
const REFUSED_DIRECTOR_ENDED: usize = 8;
const LIMITED_DIRECTOR_ENDED: usize = 7;
pub const READS_4_5_REFUSED: &[(Read, usize)] = &[
    (Read::DirectorEnded, REFUSED_DIRECTOR_ENDED),
    (Read::AWaiting, A_WAITING),
    (Read::Refused, HELD_ACCOUNTS),
    (Read::AEnded, HELD_A_ENDED),
];
pub const READS_4_5_LIMITED: &[(Read, usize)] = &[
    (Read::DirectorEnded, LIMITED_DIRECTOR_ENDED),
    (Read::AWaiting, A_WAITING),
    (Read::Limited, HELD_ACCOUNTS),
    (Read::AEnded, HELD_A_ENDED),
];

#[must_use]
pub fn step_4_5_refused() -> Vec<calls::Step> {
    use super::{FIRST_ACCOUNT, SECOND_KEY_FILE};
    held("refused", Some(call(D1, argv!["replace-key", number(FIRST_ACCOUNT), calls::file(SECOND_KEY_FILE)])))
}

#[must_use]
pub fn step_4_5_limited() -> Vec<calls::Step> {
    // Only reads and waits follow the chat: the engine lifts the limit.
    held("limited", None)
}

fn held(key: &'static str, act: Option<calls::Step>) -> Vec<calls::Step> {
    let mut steps = vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", key, super::HELD_PERSON]),
        call(D1, argv!["wait", "task", number(DIRECTOR), "--until", "replied"]),
        // A fresh wait after the requester ends exits unknown: jig's
        // routing/views.rs admission and documents/small.rs require live rows.
        // Keep A's watch across the act until jig supports historical waits.
        start(D1, argv!["wait", "task", number(A), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(A), "--until", "phase:due"]),
        call(D1, argv!["task", number(A), "--json"]),
        call(D1, argv!["accounts", "--json"]),
    ];
    steps.extend(act);
    steps.push(join(HELD_A_ENDED));
    // The director can already be ended too: a fresh wait exits unknown.
    steps.push(calls::until(
        D1,
        argv!["task", number(DIRECTOR), "--json"],
        b"\"phase\":\"ended\"",
        b"\"phase\":\"running\"",
    ));
    steps
}

const PACE_DIRECTOR_ENDED: usize = 3;
const PACE_ASKED: usize = 6;
const RANKED: usize = 8;
const SURVEY_ENDED: usize = 10;
const PACE_A_ENDED: usize = 11;
pub const READS_4_4: &[(Read, usize)] = &[
    (Read::DirectorEnded, PACE_DIRECTOR_ENDED),
    (Read::PaceAsked, PACE_ASKED),
    (Read::Ranked, RANKED),
    (Read::SurveyEnded, SURVEY_ENDED),
    (Read::AEnded, PACE_A_ENDED),
];

#[must_use]
pub fn step_4_4() -> Vec<calls::Step> {
    use super::{PACE_ANSWER, PACE_PERSON, PACE_QUESTION_MESSAGE, SECOND_ACCOUNT, SECOND_KEY_FILE, SURVEY};
    vec![
        sign_in(D1),
        call(D1, argv!["add-key", "1", calls::file(SECOND_KEY_FILE), "--deployment", "--key", "second-llm"]),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "pace", PACE_PERSON]),
        start(D1, argv!["wait", "task", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "inbox", "--until", "waiting"]),
        // A's question makes the director due. Wait for its acknowledgement
        // run to park before the person reads the question: it must outlive
        // that acknowledgement (jig's domain/tasks.md, 8.2). Polling chat
        // instead races its live run watch with that run's end and can retry.
        call(D1, argv!["wait", "task", number(DIRECTOR), "--until", "phase:idle"]),
        call(D1, argv!["inbox", "--kind", "questions", "--json"]),
        call(D1, argv!["rank", number(SECOND_ACCOUNT), "1"]),
        call(D1, argv!["accounts", "--json"]),
        call(D1, argv!["answer", number(DIRECTOR), number(PACE_QUESTION_MESSAGE), PACE_ANSWER, "--key", "pace-answer"]),
        // The director reports after A, which reports after the survey. Its
        // existing live watch is the completion barrier; read their historical
        // documents once afterwards, without polling or opening ended watches.
        join(PACE_DIRECTOR_ENDED),
        call(D1, argv!["task", number(SURVEY), "--json"]),
        call(D1, argv!["task", number(A), "--json"]),
    ]
}

// Call positions within the signed-in check suffix; non-call steps consume none.
const CHECK_DIRECTOR_ENDED: usize = 1;
const CHECK_BENCHMARK_WAITING: usize = 4;
const CHECK_BENCHMARK_ENDED: usize = 5;
const CHECK_B_ENDED: usize = 6;
const CHECK_DIRECTOR_ACTIVITY: usize = 7;
const CHECK_DIRECTOR_TIMER: usize = 8;
pub const READS_4_3: &[(Read, usize)] = &[
    (Read::CheckEnded, CHECK_DIRECTOR_ENDED),
    (Read::BenchmarkWaiting, CHECK_BENCHMARK_WAITING),
    (Read::BenchmarkEnded, CHECK_BENCHMARK_ENDED),
    (Read::BEnded, CHECK_B_ENDED),
    (Read::DirectorActivity, CHECK_DIRECTOR_ACTIVITY),
    (Read::DirectorTimer, CHECK_DIRECTOR_TIMER),
];

/// The check follows the opener's sign-in; `base` is its first recorded call.
#[must_use]
pub fn step_4_3(base: usize) -> Vec<calls::Step> {
    use super::{B, BENCHMARK, CHECK, CHECK_PERSON};
    vec![
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "check", CHECK_PERSON]),
        start(D1, argv!["wait", "task", number(CHECK), "--until", "ended", "--json"]),
        calls::watching(base + CHECK_DIRECTOR_ENDED),
        // Each replied wait follows a completed delegation, admitting the
        // next task's watch without racing a not-yet-created task number.
        call(D1, argv!["wait", "task", number(CHECK), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(B), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(BENCHMARK), "--until", "phase:due", "--json"]),
        calls::add_worker("s2"),
        join(base + CHECK_DIRECTOR_ENDED),
        call(D1, argv!["task", number(BENCHMARK), "--activity", "32", "--json"]),
        call(D1, argv!["task", number(B), "--activity", "32", "--json"]),
        call(D1, argv!["task", number(CHECK), "--activity", "32", "--json"]),
        // The latest page starts after the timer: follow its supplied cursor.
        call(
            D1,
            argv![
                "activity",
                number(CHECK),
                "--before",
                calls::activity_before(base + CHECK_DIRECTOR_ACTIVITY),
                "--json"
            ],
        ),
    ]
}

/// Preserve every opener act, then run the check in the signed-in directory.
#[must_use]
pub fn step_4_1_4_3() -> Vec<calls::Step> {
    let mut steps = step_4_1();
    let base = READS_4_1.iter().map(|(_, call)| call).max().expect("opener calls") + 1;
    steps.extend(step_4_3(base));
    steps
}

#[must_use]
pub fn reads_4_1_4_3() -> Vec<(Read, usize)> {
    let base = READS_4_1.iter().map(|(_, call)| call).max().expect("opener calls") + 1;
    READS_4_1.iter().copied().chain(READS_4_3.iter().map(|&(read, call)| (read, base + call))).collect()
}

const HANG_ENDED: usize = 2;
const HANG_QUESTION_READ: usize = 3;
const HANG_WAITING: usize = 4;
const HANG_A_ENDED: usize = 5;
const HANG_ACTIVITY: usize = 6;
const HANG_ACTIVITY_OLDER: usize = 7;
const HANG_ACTIVITY_FIRST: usize = 8;
/// Documents within the standalone restart story, relative to its first call.
pub const READS_4_6: &[(Read, usize)] = &[
    (Read::HangEnded, HANG_ENDED),
    (Read::HangQuestionRead, HANG_QUESTION_READ),
    (Read::HangWaiting, HANG_WAITING),
    (Read::PhaseEnded, HANG_A_ENDED),
    (Read::HangActivity, HANG_ACTIVITY),
    (Read::HangActivityOlder, HANG_ACTIVITY_OLDER),
    (Read::HangActivityFirst, HANG_ACTIVITY_FIRST),
];

/// Sign in and run the restart story; `base` names its first recorded call.
#[must_use]
pub fn step_4_6(base: usize) -> Vec<calls::Step> {
    use super::{HANG, HANG_PERSON, HANG_READ, PHASE};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "hang", HANG_PERSON]),
        start(D1, argv!["wait", "task", number(HANG), "--until", "ended", "--json"]),
        calls::watching(base + HANG_ENDED),
        // ctl's predicates distinguish phase/replied, but cannot select a
        // committed turn or its text. Activity contains only committed turns;
        // chat/run can also contain the live writing, so cannot be this barrier.
        calls::until(D1, argv!["activity", number(HANG), "--json"], HANG_READ.as_bytes(), b"run_began"),
        // Once the read is committed, await the following in-run wait.
        // Its withheld turn is sealed when the worker stops, as A's parked waits are.
        call(D1, argv!["wait", "task", number(HANG), "--until", "replied", "--json"]),
        calls::stop_worker("ws"),
        join(base + HANG_ENDED),
        call(D1, argv!["task", number(PHASE), "--activity", "32", "--json"]),
        call(D1, argv!["activity", number(HANG), "--json"]),
        call(D1, argv!["activity", number(HANG), "--before", calls::activity_before(base + HANG_ACTIVITY), "--json"]),
        call(
            D1,
            argv!["activity", number(HANG), "--before", calls::activity_before(base + HANG_ACTIVITY_OLDER), "--json"],
        ),
        // The ended chat is the cheapest event proving its retry cannot use ws.
        calls::start_worker("ws"),
    ]
}

// Act 2, step 5.1 (domain/testing.md, 5.3)
const RULES_ENDED: usize = 3;
pub const READS_5_1: &[(Read, usize)] = &[
    (Read::RulesWaiting, 5),
    (Read::RulesHeld, 7),
    (Read::RulesQuestionWaiting, 8),
    (Read::RulesEnded, RULES_ENDED),
    (Read::ScopeEnded, 11),
];

#[must_use]
pub fn step_5_1() -> Vec<calls::Step> {
    use super::{
        NEW_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, RULES_PERSON, SCOPE, SCOPE_QUESTION, SCOPE_QUESTION_HOLD,
    };
    vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--role",
                "context",
                "--no-home",
                "--key",
                "rules-kit"
            ],
        ),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "rules", RULES_PERSON]),
        // Keep the ended watch across Stop and Release instead of polling the
        // full task document through the director's answer and both reports.
        start(D1, argv!["wait", "task", number(RULES), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(RULES), "--until", "replied"]),
        calls::until(D1, argv!["task", number(RULES), "--json"], b"\"work\":\"waiting\"", b"\"phase\":\"running\""),
        call(D1, argv!["stop", number(RULES), "--key", "rules-stop"]),
        calls::until(D1, argv!["task", number(RULES), "--json"], b"\"stopped_by\"", b"\"phase\":\"running\""),
        calls::release_hold(SCOPE_QUESTION_HOLD),
        calls::until(
            D1,
            argv!["task", number(RULES), "--activity", "32", "--json"],
            SCOPE_QUESTION.as_bytes(),
            b"\"stopped_by\"",
        ),
        // A must park before the answer: its next run then fetches main's guide.
        call(D1, argv!["wait", "task", number(SCOPE), "--until", "phase:idle"]),
        calls::push(RULES_FILE.as_bytes(), NEW_RULES.as_bytes()),
        call(D1, argv!["release", number(RULES), "--hold", number(RULES_HOLD_REVISION), "--key", "rules-release"]),
        join(RULES_ENDED),
        call(D1, argv!["task", number(SCOPE), "--json"]),
    ]
}

// Act 2, step 5.3 (domain/testing.md, 5.3)
const SHAPE_ENDED: usize = 2;
pub const READS_5_3: &[(Read, usize)] = &[(Read::MoverEnded, 3), (Read::ShapeEnded, SHAPE_ENDED)];

#[must_use]
pub fn step_5_3() -> Vec<calls::Step> {
    use super::{MOVER, SHAPE, SHAPE_PERSON};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "shape", SHAPE_PERSON]),
        start(D1, argv!["wait", "task", number(SHAPE), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(MOVER), "--until", "ended", "--json"]),
        join(SHAPE_ENDED),
    ]
}

// Act 2, step 5.2 (domain/testing.md, 5.3)
const B_ENDED_5_2: usize = 16;
pub const READS_5_2: &[(Read, usize)] = &[
    (Read::ImplementerRunning, 5),
    (Read::H1Running, 7),
    (Read::HelperEnded, 8),
    (Read::BHeld, 11),
    (Read::ImplementerEnded, 12),
    (Read::H1Ended, 13),
    (Read::BStillHeld, 14),
    (Read::BEnded, B_ENDED_5_2),
    (Read::BEndedEarlier, 17),
    (Read::OwnerEnded, 18),
    (Read::PauseEnded, 19),
];

#[must_use]
pub fn step_5_2() -> Vec<calls::Step> {
    use super::{
        Agent, B_HOLD, BOWNER, H1, H1_REVIEWER, HELPER, IMPLEMENTER, IMPLEMENTER_HOLD, OWNER, PAUSE, PAUSE_PERSON,
        PROJECT, REVIEW_HOLD,
    };
    vec![
        sign_in(D1),
        call(
            D1,
            argv![
                "adopt",
                "--project",
                number(PROJECT),
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--no-home",
                "--key",
                "adopt-kit"
            ],
        ),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "pause", PAUSE_PERSON]),
        call(D1, argv!["wait", "task", number(PAUSE), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(BOWNER), "--until", "replied"]),
        // The documents establish the case's in-flight work before Stop.
        // The fake Pieces keep each run writing until its release below.
        calls::until(D1, argv!["task", number(IMPLEMENTER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        // h1 exists after B's reply; its review is created asynchronously.
        // Wait for its live delegate card before reading the review's document.
        calls::until(D1, argv!["task", number(H1), "--json"], Agent::Review.cue(), b"\"task\""),
        calls::until(D1, argv!["task", number(H1_REVIEWER), "--json"], b"\"work\":\"writing\"", b"\"task\""),
        calls::until(
            D1,
            argv!["task", number(HELPER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        // replied alone can still be writing; read the parked owner's phase.
        call(D1, argv!["wait", "task", number(BOWNER), "--until", "phase:idle"]),
        call(D1, argv!["stop", number(BOWNER), "--key", "stop-b"]),
        calls::until(
            D1,
            argv!["task", number(BOWNER), "--activity", "32", "--json"],
            b"\"phase\":\"held\"",
            b"\"task\"",
        ),
        calls::release_hold(IMPLEMENTER_HOLD),
        calls::release_hold(REVIEW_HOLD),
        calls::until(
            D1,
            argv!["task", number(IMPLEMENTER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        calls::until(D1, argv!["task", number(H1), "--activity", "32", "--json"], b"\"phase\":\"ended\"", b"\"task\""),
        call(D1, argv!["task", number(BOWNER), "--activity", "32", "--json"]),
        call(D1, argv!["release", number(BOWNER), "--hold", number(B_HOLD), "--key", "release-b"]),
        calls::until(
            D1,
            argv!["task", number(BOWNER), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
        // B's two attempts span two activity pages; retain the supplied cursor.
        call(D1, argv!["activity", number(BOWNER), "--before", calls::activity_before(B_ENDED_5_2), "--json"]),
        call(D1, argv!["task", number(OWNER), "--activity", "32", "--json"]),
        calls::until(
            D1,
            argv!["task", number(PAUSE), "--activity", "32", "--json"],
            b"\"phase\":\"ended\"",
            b"\"task\"",
        ),
    ]
}

// Act 2, step 5.5 (domain/testing.md, 5.3)
const PLAN_ENDED: usize = 2;
pub const READS_5_5: &[(Read, usize)] = &[
    (Read::PlanEnded, PLAN_ENDED),
    (Read::PlannerParked, 3),
    (Read::PlanAnswerPending, 4),
    (Read::PlannerEnded, 5),
    (Read::PlannerEarlier, 6),
    (Read::PlannerFirst, 7),
    (Read::SubownerEnded, 8),
];

#[must_use]
pub fn step_5_5() -> Vec<calls::Step> {
    use super::{PLAN_ANSWER_HOLD, PLAN_ANSWER_PENDING, PLAN_DIRECTOR, PLAN_PERSON, PLANNER, SUBOWNER};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "plan", PLAN_PERSON]),
        start(D1, argv!["wait", "task", number(PLAN_DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(PLANNER), "--until", "phase:idle", "--json"]),
        // Step::Held gates ctl writes. The LLM's emitted piece, read through
        // the live run document, proves its answer is at the fake hold.
        calls::until(
            D1,
            argv!["run", number(PLAN_DIRECTOR), "--json"],
            PLAN_ANSWER_PENDING.as_bytes(),
            b"\"writing\":",
        ),
        calls::release_hold(PLAN_ANSWER_HOLD),
        join(PLAN_ENDED),
        call(D1, argv!["task", number(PLANNER), "--activity", "32", "--json"]),
        call(D1, argv!["activity", number(PLANNER), "--before", calls::activity_before(5), "--json"]),
        call(D1, argv!["activity", number(PLANNER), "--before", calls::activity_before(6), "--json"]),
        call(D1, argv!["task", number(SUBOWNER), "--json"]),
    ]
}
