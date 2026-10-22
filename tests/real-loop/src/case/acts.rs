//! The person's acts, with documents identified by the moment they were read.
use super::{A, DESIGN, DIRECTOR, PERSON, PROJECT};
use crate::{
    argv,
    calls::{self, D1, call, join, number, sign_in, start},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Read {
    DirectorEnded,
    AEnded,
    DesignEnded,
    AWaiting,
    Refused,
    PaceAsked,
    Ranked,
    SurveyEnded,
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
        // Arm the director's watch while it is live, then watch each delegate
        // before its requester ends: small's historical join budget cannot
        // hydrate the design through two ended ancestors (M1's finding).
        start(D1, argv!["wait", "task", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(DIRECTOR), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(A), "--until", "replied"]),
        call(D1, argv!["wait", "task", number(DESIGN), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(A), "--until", "ended", "--json"]),
        join(DIRECTOR_ENDED),
    ]
}

const REFUSED_DIRECTOR_ENDED: usize = 7;
const A_WAITING: usize = 4;
const REFUSED: usize = 5;
const REFUSED_A_ENDED: usize = 3;
pub const READS_4_5_REFUSED: &[(Read, usize)] = &[
    (Read::DirectorEnded, REFUSED_DIRECTOR_ENDED),
    (Read::AWaiting, A_WAITING),
    (Read::Refused, REFUSED),
    (Read::AEnded, REFUSED_A_ENDED),
];

#[must_use]
pub fn step_4_5_refused() -> Vec<calls::Step> {
    use super::{FIRST_ACCOUNT, REFUSED_A, REFUSED_PERSON, SECOND_KEY_FILE};
    vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "refused", REFUSED_PERSON]),
        call(D1, argv!["wait", "task", number(DIRECTOR), "--until", "replied"]),
        // Arm A before replacement: after its result the requester can end
        // before a new watch has hydrated A through that historical parent.
        start(D1, argv!["wait", "task", number(REFUSED_A), "--until", "ended", "--json"]),
        calls::until(D1, argv!["task", number(REFUSED_A), "--json"], b"\"why\":\"refused\"", b"\"run_wait\":null"),
        call(D1, argv!["accounts", "--json"]),
        call(D1, argv!["replace-key", number(FIRST_ACCOUNT), calls::file(SECOND_KEY_FILE)]),
        join(REFUSED_A_ENDED),
        calls::until(D1, argv!["task", number(DIRECTOR), "--json"], b"\"phase\":\"ended\"", b"\"phase\":\"running\""),
    ]
}

const PACE_DIRECTOR_ENDED: usize = 3;
const PACE_ASKED: usize = 5;
const RANKED: usize = 7;
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
    use super::{PACE_A, PACE_ANSWER, PACE_PERSON, PACE_QUESTION_MESSAGE, SECOND_ACCOUNT, SECOND_KEY_FILE, SURVEY};
    vec![
        sign_in(D1),
        call(D1, argv!["add-key", "1", calls::file(SECOND_KEY_FILE), "--deployment", "--key", "second-llm"]),
        call(D1, argv!["chat", "--project", number(PROJECT), "--key", "pace", PACE_PERSON]),
        start(D1, argv!["wait", "task", number(DIRECTOR), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "inbox", "--until", "waiting"]),
        call(D1, argv!["inbox", "--kind", "questions", "--json"]),
        call(D1, argv!["rank", number(SECOND_ACCOUNT), "1"]),
        call(D1, argv!["accounts", "--json"]),
        calls::until(D1, argv!["chat", number(DIRECTOR)], super::PACE_PENDING.as_bytes(), b"A delegated."),
        call(D1, argv!["answer", number(DIRECTOR), number(PACE_QUESTION_MESSAGE), PACE_ANSWER]),
        call(D1, argv!["wait", "task", number(SURVEY), "--until", "ended", "--json"]),
        call(D1, argv!["wait", "task", number(PACE_A), "--until", "ended", "--json"]),
        join(PACE_DIRECTOR_ENDED),
    ]
}
