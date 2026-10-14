//! The forge-backed direct driver's limits and adoption form values.
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client::api;

#[must_use]
pub fn forge_limits() -> engine::Limits {
    let mut limits = crate::walking::limits();
    limits.people.people = 4;
    limits.people.holdings = 4;
    limits.journal.writes +=
        (jig_domain_people::max_out(&limits.people) - jig_domain_people::max_out(&crate::walking::limits().people)) * 4;
    limits
}

#[must_use]
pub fn adoption(forge: u16) -> forge::Adoption {
    forge::Adoption {
        project: 1,
        home: true,
        provider: api::Repository { forge, repository: 2 },
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: forge::Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    }
}
