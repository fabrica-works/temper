//! The real component on io's ring against an isolated Forgejo (forge.md, 10).
pub(crate) mod hosted;
mod reads;
mod writes;
use crate::{
    Result,
    facts::{self, Fact},
    fixture::Fixture,
    write,
};
use hosted::Hosted;
use std::{fs, path::Path};
use temper_throwaway_forgejo::Forgejo;
/// # Errors
/// Returns a binary, fixture, io, exchange or secret-scan failure.
pub fn record(scratch: &Path, out: &Path) -> Result<()> {
    let forgejo = Forgejo::from_env().map_err(|e| format!("{e:?}"))?;
    let facts = run(&forgejo, scratch)?;
    let path = out.join("facts.txt");
    let before = fs::read_to_string(&path)?;
    fs::write(path, facts::replace_block(&before, "component-", &facts))?;
    Ok(())
}
/// # Errors
/// Returns provisioning, ring or secret-scan failures; each failed call is a fact.
pub fn run(forgejo: &Forgejo, scratch: &Path) -> Result<Vec<Fact>> {
    let f = Fixture::new(forgejo, scratch)?;
    let mut secrets = write::Secrets::from_instance(&f.instance);
    crate::runner::component(&f, &mut secrets)?;
    let writer = crate::setup::scoped(&f, &mut secrets, "writer", "temper-api", crate::tokens::API)?;
    let mut host = Hosted::new(&f, &writer)?;
    let mut facts = vec![host.started("component-start-ready", None)?];
    for (name, token, expected) in [
        (
            "component-start-other-user-stops",
            &f.stranger.token,
            temper_engine_protocol_forge::Stopped::OtherUser { user: f.stranger.id },
        ),
        (
            "component-start-wrong-token-stops",
            &temper_throwaway_forgejo::Token::new("0".repeat(40)),
            temper_engine_protocol_forge::Stopped::Refused {
                read: temper_engine_protocol_forge::StartRead::User,
                status: 401,
            },
        ),
    ] {
        let mut other = Hosted::new(&f, token)?;
        facts.push(other.started(name, Some(expected))?);
        other.finish();
    }
    reads::run(&f, &mut host, &mut facts)?;
    writes::run(&f, &mut host, &mut facts)?;
    host.finish();
    write::check("component-facts", b"", facts.iter().map(Fact::line).collect::<String>().as_bytes(), &secrets)?;
    f.instance.stop().map_err(|e| crate::failure(&e))?;
    Ok(facts)
}
fn fact(name: &'static str, holds: bool, seen: &impl std::fmt::Debug) -> Fact {
    Fact { name, outcome: if holds { facts::Outcome::Holds } else { facts::Outcome::Fails(format!("{seen:?}")) } }
}
impl Hosted {
    fn observe(
        &mut self,
        name: &'static str,
        repository: temper_engine_domain_forge_client::api::Repository,
        op: temper_engine_domain_forge_client::api::Op,
        check: impl FnOnce(&temper_engine_domain_forge_client::api::Answer) -> bool,
    ) -> Fact {
        let result = self.call(repository, op);
        fact(name, result.as_ref().is_ok_and(check), &result)
    }
}
trait Operation {
    fn operation(self) -> temper_engine_domain_forge_client::api::Op;
}
impl Operation for temper_engine_domain_forge_client::api::Read {
    fn operation(self) -> temper_engine_domain_forge_client::api::Op {
        temper_engine_domain_forge_client::api::Op::Read(self)
    }
}
impl Operation for temper_engine_domain_forge_client::api::Write {
    fn operation(self) -> temper_engine_domain_forge_client::api::Op {
        temper_engine_domain_forge_client::api::Op::Write(self)
    }
}
