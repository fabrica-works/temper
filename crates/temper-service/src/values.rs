//! Lent account credentials stay in the protocol layer (protocol/credentials.md, 3).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_accounts::table::Table;
use jig_protocol_engine_workers::GrantValue;
use skein_lib::{List, Map, Secret, Time};
#[derive(Debug)]
pub struct Values {
    pub accounts: Table,
    /// Configured git identities, encoded once at generation one.
    pub identities: Map<u32, Secret>,
}
impl Values {
    /// Numbered account slots, including the unused zero slot.
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
            let value = match grant.table {
                host::GrantTable::Accounts => account_grant(self.accounts.value(grant.account, grant.generation, now)),
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

fn account_grant(value: Option<&jig_protocol_accounts::table::Value>) -> Option<GrantValue> {
    let value = value?;
    Some(GrantValue { value: value.token.clone(), account_id: value.account_id.clone() })
}
fn identity_grant(value: Option<&Secret>) -> Option<GrantValue> {
    let value = value?;
    Some(GrantValue { value: value.clone(), account_id: Secret::new(Box::new([])) })
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
