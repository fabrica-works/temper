//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod settings;
pub mod store;
#[cfg(test)]
mod tests;
pub mod tokens;

pub mod accounts;
pub mod values;
pub use accounts::AccountsRuntime;
mod engine;
mod route;
mod stop;
/// How the service ended (protocol.md, 6).
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
/// Why the service stopped; failed startup names its configuration paths.
#[derive(PartialEq, Eq, Debug)]
pub enum Stop {
    CommitFailed { number: u64 },
    Start(Problems),
    Listener,
}
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_io::kernel::Fd;
use skein_lib::{List, Time, Wall};
/// The engine root and its protocol children; the service I/O pass follows it.
#[expect(missing_debug_implementations, reason = "the service owns account token records")]
pub struct Service {
    engine: engine::Engine,
}
impl Service {
    /// Construct the root, its deployment store and its configured accounts.
    pub fn engine(
        mut settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
    ) -> Result<Self, Problems> {
        let limits =
            limits::engine(&settings.jig.limits, &settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        if temper_engine_domain::worst_case(&limits).is_none() {
            return Err(start_problem(b"limits", What::Bound));
        }
        for charter in &mut settings.jig.charters {
            charter.policy.instructions = settings
                .files
                .contents(&charter.policy.instructions)
                .ok_or(start_problem(b"charters", What::Unreadable))?
                .into();
        }
        let store_limits = store::limits(&limits, &settings.jig.limits.core, &limits.forge, &settings.jig.limits.store)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let max = temper_engine_domain::max_out(&limits);
        let accounts = if settings.jig.llm.accounts.is_empty() {
            None
        } else {
            Some(accounts::Accounts::new(
                &settings.jig,
                &settings.files,
                seeds,
                accounts,
                max.checked_mul(6).expect("account routing room"),
            )?)
        };
        let values = values::Values::new(
            limits.accounts.accounts.checked_add(1).expect("numbered accounts"),
            settings.jig.limits.driver.client.document.token_bytes,
        );
        let count = u32::try_from(settings.forges.len()).expect("configured forges");
        let mut writers = List::with_capacity(count);
        let mut identities = List::with_capacity(count);
        for forge in &settings.forges {
            writers
                .push(temper_engine_domain_forge_client::Writer { forge: forge.number, author: forge.user })
                .expect("forge writers");
            identities
                .push(temper_engine_domain_forge::GitIdentity { forge: forge.number, credential: forge.identity })
                .expect("forge identities");
        }
        let mut first_periods = List::with_capacity(limits.authority.projects);
        for project in &settings.jig.projects {
            first_periods
                .push((project.number, project.policy.period_spend))
                .or(Err(start_problem(b"projects", What::Bound)))?;
        }
        let config = temper_engine_domain::Config {
            core: jig_protocol_config::core(&settings.jig, seeds.deployment, seeds.seed),
            landing: settings.landing,
            forge: temper_engine_domain_forge_client::Config {
                namespace: Box::new(seeds.deployment),
                writers: writers.into_boxed(),
            },
            identities: identities.into_boxed(),
            forge_charter: settings.charter,
        };
        Ok(Self {
            engine: engine::Engine::new(
                config,
                &limits,
                store::Store::new(&limits, &store_limits),
                accounts,
                values,
                first_periods,
            ),
        })
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        self.engine.store.adopt(root, effective_user)
    }
    /// Earliest deadline of the root and its children.
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        self.engine.next_deadline()
    }
    /// Whether an iteration has work on the supplied clock.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.engine.work_pending(now)
    }
    /// A first call drains; another aborts.
    pub fn shutdown(&mut self) {
        stop::shutdown(&mut self.engine);
    }
    /// How the engine ended, once it has.
    #[must_use]
    pub const fn end(&self) -> Option<&End> {
        self.engine.end.as_ref()
    }
}
fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// Drive store terminals, account events, one root input and released outputs.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    engine::iterate(&mut service.engine, now, wall);
}
