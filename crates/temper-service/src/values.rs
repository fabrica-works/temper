//! Lent account credentials stay in the protocol layer (protocol/credentials.md, 3).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_accounts::table::{self, Table};
use jig_protocol_engine_workers::{GrantValue, Scheme};
use skein_lib::{List, Map, Secret, Time};

/// A configured key can exceed the OAuth token bound. The worker has no
/// provider declarations and reserves the key ceiling unconditionally.
pub(crate) fn credential_bytes(engine: &jig_protocol_config::Engine) -> u32 {
    let mut bound = engine.limits.token;
    for provider in &engine.llm.providers {
        if provider.keys {
            bound = crate::worker_settings::credential_bytes(bound);
        }
    }
    bound
}
#[derive(Debug)]
pub struct Values {
    pub accounts: Table,
    /// Configured git identities, encoded once at generation one.
    pub identities: Map<u32, Secret>,
}
impl Values {
    /// Named source slots, bounded by the accounts driver.
    #[must_use]
    pub fn new(accounts: u32, token_bytes: u32) -> Self {
        Self { accounts: Table::new(accounts, token_bytes), identities: Map::with_capacity(0) }
    }
}

impl Values {
    /// One optional value per grant, preserving the root's order.
    #[must_use]
    pub fn grants(&self, grants: &[host::Grant], now: Time) -> Box<[Option<GrantValue>]> {
        let mut values = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
        for grant in grants {
            let value = match grant.form.table() {
                host::GrantTable::Accounts => {
                    account_grant(grant.form, self.accounts.value(grant.account, grant.generation, now))
                }
                host::GrantTable::Application if grant.generation == 1 => {
                    identity_grant(self.identities.get(&grant.account))
                }
                host::GrantTable::Application => None,
            };
            values.push(value).expect("one slot per grant");
        }
        values.into_boxed()
    }
}

fn account_grant(form: host::Form, value: Option<&table::Value>) -> Option<GrantValue> {
    let value = value?;
    let actual = match &value.scheme {
        table::Scheme::Bearer => host::Form::Credential(host::Scheme::Bearer),
        table::Scheme::Key => host::Form::Credential(host::Scheme::Key),
        table::Scheme::Withdrawn { .. } => host::Form::Withdrawn,
    };
    if form != actual {
        return None;
    }
    Some(match &value.scheme {
        table::Scheme::Bearer => GrantValue::Credential { scheme: Scheme::Bearer, token: value.token.clone() },
        table::Scheme::Key => GrantValue::Credential { scheme: Scheme::Key, token: value.token.clone() },
        table::Scheme::Withdrawn { id } => GrantValue::Withdrawn { id: id.clone(), bearer: value.token.clone() },
    })
}
fn identity_grant(value: Option<&Secret>) -> Option<GrantValue> {
    let value = value?;
    Some(GrantValue::Application { secret: value.clone() })
}
impl Values {
    pub(crate) fn fill(
        &mut self,
        identities: &[crate::settings::Identity],
        limits: &temper_protocol_worker_wire::v1::Limits,
    ) -> Result<(), jig_protocol_config::Problems> {
        use jig_protocol_config::What;
        self.identities = Map::with_capacity(u32::try_from(identities.len()).expect("identity count"));
        for identity in identities {
            let value = crate::translate::identity(identity, limits).or(Err(identity_problem(What::Bound)))?;
            let previous = self.identities.insert(identity.number, value).or(Err(identity_problem(What::Bound)))?;
            if previous.is_some() {
                return Err(identity_problem(What::Duplicate));
            }
        }
        Ok(())
    }
}
fn identity_problem(what: jig_protocol_config::What) -> jig_protocol_config::Problems {
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"engine").member(b"identities"), what);
    problems
}
