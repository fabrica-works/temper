use super::fixtures;
use crate::family;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{List, Reader, ReplyTo, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
const REPLY: ReplyTo = ReplyTo::new(Token::new(19));
const REQUEST_KEY: [u8; 16] = [5; 16];
const WATCHER: Token = Token::new(7);
fn configured(git: &[u8]) -> Box<[crate::settings::Forge]> {
    let mut forges = fixtures::engine().forges;
    forges.get_mut(0).unwrap().git = git.into();
    forges
}
fn adopt() -> w::Adopt {
    let mut checks = List::with_capacity(2);
    checks.push(2).unwrap();
    checks.push(3).unwrap();
    w::Adopt::new(
        &w::CEILINGS,
        w::AdoptParts {
            project: 7,
            home: true,
            forge: 1,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            prefix: b"repo".as_slice().into(),
            role: w::Role::Adopted,
            landing: b"main".as_slice().into(),
            ci: true,
            checks,
        },
    )
    .unwrap()
}
fn exchange(family: u16, version: u32, record: Box<[u8]>) -> kit::ToApplication {
    kit::ToApplication::Request {
        reply_to: REPLY,
        sign_in: 3,
        key: REQUEST_KEY,
        family,
        version,
        record,
        digest: [7; 32],
        carried: None,
        scope: people::ApplicationScope::Project(7),
    }
}
#[test]
fn an_adopt_reaches_the_root_with_its_forges_host() {
    let forges = configured(b"https://forge.example:3000/git");
    let event =
        family::to_root(exchange(wire::FORGE, wire::CLIENT_VERSION, wire::adopt(&adopt())), &forges, &w::CEILINGS)
            .unwrap();
    let root::Event::Party(root::PartyInput::Application { reply_to, sign_in, key, digest, request, .. }) = event
    else {
        panic!("application request")
    };
    assert_eq!((reply_to, sign_in, key, digest), (REPLY, 3, [5; 16], [7; 32]));
    assert_eq!(
        request,
        forge::Adoption {
            project: 7,
            home: true,
            forge: 1,
            host: b"forge.example:3000".as_slice().into(),
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            prefix: b"repo".as_slice().into(),
            role: forge::Role::Adopted,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([2, 3]),
        }
    );
}
#[test]
fn an_adopt_naming_no_configured_forge_reaches_the_root_with_no_host() {
    let event =
        family::to_root(exchange(wire::FORGE, wire::CLIENT_VERSION, wire::adopt(&adopt())), &[], &w::CEILINGS).unwrap();
    let root::Event::Party(root::PartyInput::Application { request, .. }) = event else { panic!("adoption") };
    assert!(request.host.is_empty());
    assert_eq!(request.forge, 1);
}
#[test]
fn an_adopt_of_another_version_is_answered_undecodable() {
    for (row, family, version, record, expected) in [
        ("family", 2, 1, wire::adopt(&adopt()), REPLY),
        ("version", 1, 2, wire::adopt(&adopt()), REPLY),
        ("malformed", 1, 1, Box::new([]), REPLY),
        ("trailing", 1, 1, trailing_adopt(), REPLY),
    ] {
        let Err(answer) = family::to_root(exchange(family, version, record), &[], &w::CEILINGS) else {
            panic!("{row}: undecodable")
        };
        let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *answer else {
            panic!("{row}: undecodable reply")
        };
        assert_eq!(to, expected, "{row}");
    }
}
fn trailing_adopt() -> Box<[u8]> {
    let record = wire::adopt(&adopt());
    let mut bytes = skein_lib::Writer::new(record.len().checked_add(1).unwrap());
    bytes.put(&record).unwrap();
    bytes.put(&[0]).unwrap();
    bytes.finish()
}
#[test]
fn a_forge_family_read_is_unknown() {
    for (row, family, version, expected) in
        [("forge", wire::FORGE, 1, (REPLY, wire::FORGE, None)), ("other", 2, 3, (REPLY, 2, None))]
    {
        let input = kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family,
            version,
            record: Box::new([]),
            scope: people::ApplicationScope::Project(7),
            window: jig_domain_core::documents::Window { rows: 2, after: None },
        };
        let Err(answer) = family::to_root(input, &[], &w::CEILINGS) else { panic!("{row}: unknown read") };
        let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
            panic!("{row}: read")
        };
        assert_eq!((reply, family, record), expected, "{row}");
    }
}
#[test]
fn a_forge_family_watch_is_refused_unknown() {
    for (row, family, version, expected) in [
        ("forge", wire::FORGE, 1, (Token::new(7), jig_domain_core::WatchRefusal::Unknown)),
        ("other", 2, 3, (Token::new(7), jig_domain_core::WatchRefusal::Unknown)),
    ] {
        let input = kit::ToApplication::Watch {
            watcher: WATCHER,
            sign_in: 3,
            family,
            version,
            record: Box::new([]),
            scope: people::ApplicationScope::Project(7),
            window: jig_domain_core::documents::Window { rows: 2, after: None },
        };
        let Err(answer) = family::to_root(input, &[], &w::CEILINGS) else { panic!("{row}: refused watch") };
        let kit::Event::WatchRefused { watcher, why } = *answer else { panic!("{row}: refusal") };
        assert_eq!((watcher, why), expected, "{row}");
    }
}
#[test]
fn a_git_base_with_a_port_keeps_it_in_the_host() {
    for (git, expected) in [
        (b"https://forge.example/git".as_slice(), b"forge.example".as_slice()),
        (b"http://127.0.0.1:3000/".as_slice(), b"127.0.0.1:3000".as_slice()),
        (b"https://[::1]:3000/git".as_slice(), b"[::1]:3000".as_slice()),
        (b"https://forge.example".as_slice(), b"forge.example".as_slice()),
        (b"https://user@forge.example:3000/git".as_slice(), b"forge.example:3000".as_slice()),
    ] {
        assert_eq!(family::host(git).as_ref(), expected, "{git:?}");
    }
}
fn encoded(outcome: Option<forge::ApplicationOutcome>) -> w::Outcome {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let bytes = family::answer(outcome, &w::CEILINGS, &jig).unwrap();
    let mut reader = Reader::new(&bytes);
    let answer = w::AdoptAnswer::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    answer.into_parts().outcome
}
#[test]
fn each_outcome_is_encoded_as_the_forge_familys_answer() {
    for (row, input, expected) in [
        ("no forge", forge::AdoptionRefusal::NoSuchForge, w::ForgeReason::NoSuchForge),
        ("missing", forge::AdoptionRefusal::NotFound, w::ForgeReason::NotFound),
        ("access", forge::AdoptionRefusal::NoAccess, w::ForgeReason::NoAccess),
        ("refused", forge::AdoptionRefusal::Refused, w::ForgeReason::Refused),
        ("in flight", forge::AdoptionRefusal::InFlight, w::ForgeReason::InFlight),
        ("too large", forge::AdoptionRefusal::TooLarge, w::ForgeReason::TooLarge),
        ("unreachable", forge::AdoptionRefusal::Unreachable, w::ForgeReason::Unreachable),
        ("invalid", forge::AdoptionRefusal::InvalidAnswer, w::ForgeReason::InvalidAnswer),
    ] {
        let w::Outcome::ForgeRefused(answer) = encoded(Some(forge::ApplicationOutcome::ForgeRefused(input))) else {
            panic!("{row}: forge refusal")
        };
        assert_eq!(answer.into_parts().reason, expected, "{row}");
    }
}
#[test]
fn an_outcome_past_its_retention_is_encoded_unknown() {
    assert_eq!(encoded(None), w::Outcome::Unknown);
}
#[test]
fn a_core_refusal_uses_jigs_refusal_record() {
    use jig_client_protocol_wire::v10 as j;
    let limits = fixtures::engine().jig.limits.clients.wire;
    for (row, input, expected) in [
        ("unknown", people::Refusal::Unknown, j::Refusal::Unknown),
        ("not offered", people::Refusal::NotOffered, j::Refusal::NotOffered),
        (
            "taken",
            people::Refusal::Taken { by: 7 },
            j::Refusal::Taken(j::RefusalTaken::new(&limits, j::RefusalTakenParts { by: 7 }).unwrap()),
        ),
        (
            "authority",
            people::Refusal::Authority { lacking: people::Lack::Writer, proposable: Some(people::Accepter::Role(1)) },
            j::Refusal::Authority(
                j::RefusalAuthority::new(
                    &limits,
                    j::RefusalAuthorityParts { lacking: j::Lacking::Writer, proposable: Some(true) },
                )
                .unwrap(),
            ),
        ),
    ] {
        let w::Outcome::CoreRefused(record) = encoded(Some(forge::ApplicationOutcome::CoreRefused(input))) else {
            panic!("{row}: core refusal")
        };
        let mut reader = Reader::new(record.refusal());
        assert_eq!(j::Refusal::decode(&limits, &mut reader).unwrap(), expected, "{row}");
        assert!(reader.is_empty(), "{row}");
    }
}
fn adopted(protection: forge::Protection) -> forge::ApplicationOutcome {
    forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
        repository: skein_forge::api::Repository { forge: 1, repository: 42 },
        owner: b"acme".as_slice().into(),
        name: b"shop".as_slice().into(),
        role: forge::Role::Fork,
        home: true,
        kinds: forge::Kinds {
            read: true,
            push: false,
            open: true,
            land: false,
            review: true,
            status: false,
            comment: true,
            issue: false,
            branch: true,
        },
        protection,
    })
}
#[test]
fn an_adopted_repository_keeps_its_facts_in_the_answer() {
    let w::Outcome::Adopted(record) = encoded(Some(adopted(forge::Protection::Absent))) else { panic!("adopted") };
    let a = record.into_parts();
    assert_eq!((a.forge, a.repository, a.home, a.role), (1, 42, true, w::Role::Fork));
    assert_eq!((a.owner.as_ref(), a.name.as_ref()), (b"acme".as_slice(), b"shop".as_slice()));
    let k = a.kinds;
    assert_eq!(
        [k.read(), k.push(), k.open(), k.land(), k.review(), k.status(), k.comment(), k.issue(), k.branch()],
        [true, false, true, false, true, false, true, false, true]
    );
    assert_eq!(a.protection, w::Protection::Absent);
}
#[test]
fn an_adopted_repository_keeps_its_protection_rule() {
    let outcome = adopted(forge::Protection::Rule(skein_forge::api::Protection {
        queue: false,
        signatures: false,
        linear: false,
        code_owners: false,
        bypass: skein_forge::api::Bypass::Never,

        branch: b"main".as_slice().into(),
        contexts: Box::new([b"ci/build".as_slice().into()]),
        approvals: 2,
        dismiss_stale: true,
    }));
    let w::Outcome::Adopted(record) = encoded(Some(outcome)) else { panic!("adopted") };
    let w::Protection::Rule(rule) = record.into_parts().protection else { panic!("rule") };
    let rule = rule.into_parts();
    assert_eq!(rule.branch.as_ref(), b"main");
    assert_eq!(rule.contexts.as_slice(), &[b"ci/build".as_slice().into()]);
    assert_eq!((rule.approvals, rule.dismiss_stale), (2, true));
}

#[test]
fn a_request_under_an_owner_without_records_is_answered_undecodable() {
    for owner in [wire::Owner::Pages, wire::Owner::Runner] {
        let number = wire::number(owner);
        let Err(answer) =
            family::to_root(exchange(number, wire::CLIENT_VERSION, wire::adopt(&adopt())), &[], &w::CEILINGS)
        else {
            panic!("owner is not served: {owner:?}")
        };
        let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *answer else {
            panic!("undecodable answer: {owner:?}")
        };
        assert_eq!(to, REPLY, "{owner:?}");
    }
}

const NOT_OFFERED_RECORD: &[u8] = &[10];
const NO_ESCALATION_RECORD: &[u8] = &[12];
const PROPOSABLE_WRITER_RECORD: &[u8] = &[5, 13, 1, 1];
const MALFORMED_POSITION: u32 = 3;
const MALFORMED_RECORD: &[u8] = &[0, 0, 0, 0, 3, 3];
fn refusal_bytes(refusal: people::Refusal) -> Box<[u8]> {
    let w::Outcome::CoreRefused(record) = encoded(Some(forge::ApplicationOutcome::CoreRefused(refusal))) else {
        panic!("core refusal")
    };
    record.into_parts().refusal
}
#[test]
fn a_core_refusal_is_carried_as_jigs_refusal_record() {
    assert_eq!(refusal_bytes(people::Refusal::NotOffered).as_ref(), NOT_OFFERED_RECORD);
}
#[test]
fn a_refusal_naming_no_one_to_escalate_to_is_carried_with_its_facts() {
    assert_eq!(refusal_bytes(people::Refusal::NoEscalationHolder).as_ref(), NO_ESCALATION_RECORD);
}
#[test]
fn an_authority_refusal_keeps_whom_a_proposal_could_go_to() {
    let refusal =
        people::Refusal::Authority { lacking: people::Lack::Writer, proposable: Some(people::Accepter::Role(1)) };
    assert_eq!(refusal_bytes(refusal).as_ref(), PROPOSABLE_WRITER_RECORD);
}
#[test]
fn a_malformed_refusal_encodes_its_fields() {
    let refusal = people::Refusal::NotWellFormed { at: MALFORMED_POSITION, why: people::Malformed::Cycle };
    assert_eq!(refusal_bytes(refusal).as_ref(), MALFORMED_RECORD);
}
#[test]
fn a_family_request_keeps_its_supplied_admission_scope() {
    let mut input = exchange(wire::FORGE, wire::CLIENT_VERSION, wire::adopt(&adopt()));
    let kit::ToApplication::Request { scope, .. } = &mut input else { panic!("request") };
    *scope = people::ApplicationScope::Deployment;
    let event = family::to_root(input, &[], &w::CEILINGS).unwrap();
    let root::Event::Party(root::PartyInput::Application { scope, .. }) = event else { panic!("application") };
    assert_eq!(scope, people::ApplicationScope::Deployment);
}
