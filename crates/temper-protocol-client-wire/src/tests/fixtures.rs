use crate::{Bounds, v1};
use alloc::boxed::Box;
use skein_lib::List;

pub const BOUNDS: Bounds = Bounds { field_bytes: 16, checks: 2, contexts: 2, refusal_bytes: 64 };

pub fn limits() -> v1::Limits {
    crate::limits(&BOUNDS).expect("test bounds within ceilings")
}

pub fn adopt() -> v1::Adopt {
    v1::Adopt::new(
        &limits(),
        v1::AdoptParts {
            project: 1,
            home: true,
            forge: crate::FORGE,
            owner: Box::from(&b"acme"[..]),
            name: Box::from(&b"shop"[..]),
            prefix: Box::from(&b"temper/"[..]),
            role: v1::Role::Owned,
            landing: Box::from(&b"main"[..]),
            ci: true,
            checks: List::with_capacity(2),
        },
    )
    .expect("default adoption")
}

pub fn adopted() -> v1::Adopted {
    let limits = limits();
    let kinds = v1::Kinds::new(&limits, v1::KindsParts { skein_bools: [true; 9] }).expect("observed capabilities");
    v1::Adopted::new(
        &limits,
        v1::AdoptedParts {
            forge: crate::FORGE,
            repository: 7,
            owner: Box::from(&b"acme"[..]),
            name: Box::from(&b"shop"[..]),
            role: v1::Role::Owned,
            home: true,
            kinds,
            protection: v1::Protection::Unknown,
        },
    )
    .expect("adopted repository")
}

pub fn outcome_rows() -> [v1::Outcome; 13] {
    let limits = limits();
    let original = adopted().into_parts();
    let mut contexts = List::with_capacity(1);
    contexts.push(Box::from(&b"build"[..])).expect("one context");
    let rule = v1::Rule::new(
        &limits,
        v1::RuleParts { branch: Box::from(&b"main"[..]), contexts, approvals: 1, dismiss_stale: true },
    )
    .expect("protection rule");
    let absent = v1::Adopted::new(&limits, v1::AdoptedParts { protection: v1::Protection::Absent, ..original.clone() })
        .expect("no protection");
    let protected = v1::Adopted::new(&limits, v1::AdoptedParts { protection: v1::Protection::Rule(rule), ..original })
        .expect("protection observed");
    let refusal = v1::CoreRefused::new(&limits, v1::CoreRefusedParts { refusal: Box::from(&b"core refusal"[..]) })
        .expect("core refusal");
    [
        v1::Outcome::Adopted(adopted()),
        v1::Outcome::Adopted(absent),
        v1::Outcome::Adopted(protected),
        v1::Outcome::CoreRefused(refusal),
        forge_refused(v1::ForgeReason::NoSuchForge),
        forge_refused(v1::ForgeReason::NotFound),
        forge_refused(v1::ForgeReason::NoAccess),
        forge_refused(v1::ForgeReason::Refused),
        forge_refused(v1::ForgeReason::InFlight),
        forge_refused(v1::ForgeReason::TooLarge),
        forge_refused(v1::ForgeReason::Unreachable),
        forge_refused(v1::ForgeReason::InvalidAnswer),
        v1::Outcome::Unknown,
    ]
}

fn forge_refused(reason: v1::ForgeReason) -> v1::Outcome {
    v1::Outcome::ForgeRefused(
        v1::ForgeRefused::new(&limits(), v1::ForgeRefusedParts { reason }).expect("forge refusal"),
    )
}
