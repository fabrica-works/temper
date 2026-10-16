use super::fixtures;
use crate::route::{self, Down, Notice, Up, family};
use jig_client_domain::contract;
use jig_client_protocol_link as link;
use temper_client_domain::{self as client, forge};
use temper_client_protocol_wire::{self as wire, CLIENT_VERSION, FORGE, v1 as w};

#[test]
fn an_adoption_round_trips_through_the_family_record() {
    let limits = fixtures::wire_limits();
    for role in [forge::Role::Owned, forge::Role::Adopted, forge::Role::Fork, forge::Role::Context] {
        let mut adoption = fixtures::adoption();
        adoption.role = role;
        let ask = forge::Ask::Adopt(adoption);
        let record = family::request(&ask, &limits).expect("fixture encodes");
        assert_eq!((record.family, record.version), (FORGE, CLIENT_VERSION), "{role:?}: owner");
        assert_eq!(family::restored(&record, &limits), Some(ask), "{role:?}: every field");
    }
}
fn adopted(protection: w::Protection) -> w::Outcome {
    let limits = fixtures::wire_limits();
    w::Outcome::Adopted(
        w::Adopted::new(
            &limits,
            w::AdoptedParts {
                forge: 1,
                repository: 7,
                owner: Box::from(*b"acme"),
                name: Box::from(*b"shop"),
                role: w::Role::Owned,
                home: true,
                kinds: w::Kinds::new(
                    &limits,
                    w::KindsParts { skein_bools: [true, false, true, false, true, false, true, false, true] },
                )
                .expect("kinds fit"),
                protection,
            },
        )
        .expect("adopted fits"),
    )
}
fn protection_rule() -> w::Protection {
    let limits = fixtures::wire_limits();
    let mut contexts = skein_lib::List::with_capacity(1);
    contexts.push(Box::from(*b"check")).expect("one context");
    w::Protection::Rule(
        w::Rule::new(
            &limits,
            w::RuleParts { branch: Box::from(*b"main"), contexts, approvals: 2, dismiss_stale: true },
        )
        .expect("rule fits"),
    )
}
fn answer(outcome: w::Outcome) -> jig_client_domain::Family {
    let record = w::AdoptAnswer::new(&fixtures::wire_limits(), w::AdoptAnswerParts { outcome }).expect("outcome fits");
    jig_client_domain::Family { family: FORGE, version: CLIENT_VERSION, bytes: wire::answer(&record) }
}
fn refused(reason: w::ForgeReason) -> w::Outcome {
    w::Outcome::ForgeRefused(
        w::ForgeRefused::new(&fixtures::wire_limits(), w::ForgeRefusedParts { reason }).expect("reason fits"),
    )
}
fn outcome_rows() -> [(&'static str, w::Outcome, forge::Answer); 13] {
    let core = w::Outcome::CoreRefused(
        w::CoreRefused::new(&fixtures::wire_limits(), w::CoreRefusedParts { refusal: fixtures::core_refusal() })
            .expect("core refusal fits"),
    );
    [
        (
            "unknown protection",
            adopted(w::Protection::Unknown),
            forge::Answer::Adopted(fixtures::adopted(forge::Protection::Unknown)),
        ),
        (
            "absent protection",
            adopted(w::Protection::Absent),
            forge::Answer::Adopted(fixtures::adopted(forge::Protection::Absent)),
        ),
        (
            "protection rule",
            adopted(protection_rule()),
            forge::Answer::Adopted(fixtures::adopted(forge::Protection::Rule(forge::Rule {
                branch: Box::from(*b"main"),
                contexts: Box::from([Box::from(&b"check"[..])]),
                approvals: 2,
                dismiss_stale: true,
            }))),
        ),
        ("core refused", core, forge::Answer::CoreRefused(contract::Refusal::Role { held: None })),
        (
            "no such forge",
            refused(w::ForgeReason::NoSuchForge),
            forge::Answer::ForgeRefused(forge::Reason::NoSuchForge),
        ),
        ("not found", refused(w::ForgeReason::NotFound), forge::Answer::ForgeRefused(forge::Reason::NotFound)),
        ("no access", refused(w::ForgeReason::NoAccess), forge::Answer::ForgeRefused(forge::Reason::NoAccess)),
        ("refused", refused(w::ForgeReason::Refused), forge::Answer::ForgeRefused(forge::Reason::Refused)),
        ("in flight", refused(w::ForgeReason::InFlight), forge::Answer::ForgeRefused(forge::Reason::InFlight)),
        ("too large", refused(w::ForgeReason::TooLarge), forge::Answer::ForgeRefused(forge::Reason::TooLarge)),
        ("unreachable", refused(w::ForgeReason::Unreachable), forge::Answer::ForgeRefused(forge::Reason::Unreachable)),
        (
            "invalid answer",
            refused(w::ForgeReason::InvalidAnswer),
            forge::Answer::ForgeRefused(forge::Reason::InvalidAnswer),
        ),
        ("unknown", w::Outcome::Unknown, forge::Answer::Unknown),
    ]
}
#[test]
fn each_outcome_decodes_to_the_forge_sides_answer() {
    let family = fixtures::family();
    for (name, outcome, expected) in outcome_rows() {
        assert_eq!(family::answer(&answer(outcome), &family.wire, &family.link), Some(expected), "{name}");
    }
}
#[test]
fn jigs_refusal_at_admission_of_a_forge_attempt_is_entrance_refused() {
    let mut family = fixtures::family();
    let sent = family::down(fixtures::send_adoption(fixtures::ATTEMPT), &mut family);
    let Down::Link(link::Event::Send { ask: jig_client_domain::Ask::Family(record), .. }) = sent else {
        panic!("the adoption leaves as the family's record");
    };
    assert_eq!((record.family, record.version), (FORGE, CLIENT_VERSION));
    let refused = jig_client_domain::Answered::Jig(contract::Answer::Refused(fixtures::role_refusal()));
    let up = family::up(fixtures::final_answer(fixtures::ATTEMPT, refused), &mut family);
    let Up::Root(client::Event::Answer { answer: Some(client::Answered::Forge(answer)), .. }) = up else {
        panic!("jig's refusal reaches the forge's side");
    };
    assert_eq!(answer, forge::Answer::EntranceRefused(fixtures::entrance(fixtures::role_refusal())));
    assert!(!family.attempts.answered(fixtures::ATTEMPT, false), "final removes the attempt");
}
#[test]
fn a_restored_adoption_of_another_version_is_dropped() {
    let mut family = fixtures::family();
    let record = client::store::Record::Table {
        version: client::store::VERSION,
        record: skein_lib::RequestRecord {
            key: [9; 16],
            scope: 1,
            first_sent: None,
            request: client::store::Held {
                key: jig_client_domain::Key {
                    asker: jig_client_domain::Asker(u32::from(FORGE)),
                    number: fixtures::ATTEMPT.raw(),
                },
                request: client::Ask::Forge(forge::Ask::Adopt(fixtures::adoption())),
            },
        },
    };
    let saved = route::saved(record.clone(), &family).expect("bounded record");
    assert_eq!(route::kept(saved.clone(), &family), Some(record), "current version is restored");
    let link::Saved::Table { version, mut record } = saved else {
        panic!("table saved");
    };
    let jig_client_domain::Ask::Family(mut ask) = record.request.request else {
        panic!("forge saved");
    };
    ask.version = 2;
    record.request.request = jig_client_domain::Ask::Family(ask);
    let up = route::up(link::Request::Kept { record: link::Saved::Table { version, record } }, &mut family);
    assert!(
        matches!(up, Up::Notice(Notice::Dropped { keeper: link::Keeper::Family(FORGE) })),
        "another version dropped"
    );
}
#[test]
fn each_attempt_terminal_frees_room_for_its_retry() {
    use skein_lib::RequestEnvelope;
    let mut family = fixtures::family();
    family.attempts = family::Attempts::new(1);
    for (name, envelope) in
        [("lost", RequestEnvelope::Lost), ("busy", RequestEnvelope::Again), ("signed out", RequestEnvelope::SignedOut)]
    {
        assert!(
            matches!(route::down(fixtures::send_adoption(fixtures::ATTEMPT), &mut family), Down::Link(_)),
            "{name}: retry admitted"
        );
        assert!(family.attempts.sent(skein_lib::Token::new(999)).is_err(), "{name}: bounded");
        let up = route::up(link::Request::Answer { attempt: fixtures::ATTEMPT, envelope, answer: None }, &mut family);
        assert!(matches!(up, Up::Root(client::Event::Answer { .. })), "{name}: terminal routed");
        assert!(!family.attempts.answered(fixtures::ATTEMPT, false), "{name}: attempt retired");
    }
}
#[test]
fn an_undecodable_family_answer_is_lost_and_keeps_the_keyed_request() {
    let mut family = fixtures::family();
    let record = jig_client_domain::Family { family: FORGE, version: CLIENT_VERSION, bytes: Box::from(*b"invalid") };
    let up =
        route::up(fixtures::final_answer(fixtures::ATTEMPT, jig_client_domain::Answered::Family(record)), &mut family);
    assert!(
        matches!(up, Up::Root(client::Event::Answer { envelope: skein_lib::RequestEnvelope::Lost, answer: None, .. })),
        "an undecodable outcome cannot retire the table's request"
    );
}
#[test]
fn the_forge_familys_hello_reaches_its_side() {
    let mut family = fixtures::family();
    let versions = contract::VersionRange { first: 2, last: 2 };
    let up = route::up(link::Request::Hello { family: Some(FORGE), versions }, &mut family);
    let Up::Root(client::Event::Forge(forge::Event::Served { versions: found })) = up else {
        panic!("hello reaches the side");
    };
    assert_eq!(found, versions);
    assert_eq!(family::served(found), forge::Served::OutOfDate);
}
