//! Records format-faithful exchanges against an isolated Forgejo (forge.md, 10).
mod cases;
pub mod facts;
mod fixture;
pub mod mask;
mod runner;
pub mod write;
use std::{error::Error, path::Path};
use temper_throwaway_forgejo::{Exchange, Forgejo};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn failure(error: &temper_throwaway_forgejo::Failure) -> Box<dyn Error> {
    format!("{error:?}").into()
}
/// # Errors
/// Returns a refused binary, provisioning, exchange or secret-scan failure.
pub fn run(scratch: &Path, out: &Path) -> Result<()> {
    let forgejo = Forgejo::from_env().map_err(|e| format!("{e:?}"))?;
    let fixture = fixture::Fixture::new(&forgejo, scratch)?;
    let mut mask = mask::Mask::new(fixture.instance.port());
    let mut secrets = write::Secrets::from_instance(&fixture.instance);
    cases::record(&fixture, out, &mut mask, &secrets)?;
    let mut facts = facts::evaluate(&forgejo, &fixture, out)?;
    let (runner, actions) = runner::run(&fixture, out, &mut mask, &mut secrets)?;
    facts.extend(actions);
    facts.push(facts::Fact {
        name: "webhook-deliveries",
        outcome: facts::Outcome::Unchecked("webhooks are built after standalone".into()),
    });
    let lines = facts.iter().map(facts::Fact::line).collect::<String>();
    write::check("facts.txt", b"", lines.as_bytes(), &secrets)?;
    facts::write(out, &facts)?;
    std::fs::write(
        out.join("facts.txt"),
        format!("# forgejo {} sha256 {}\n# runner {runner}\n{lines}", forgejo.version(), forgejo.sha256()),
    )?;
    std::fs::write(
        out.join("transcripts/binary.txt"),
        format!("version {}\nsha256 {}\n", forgejo.version(), forgejo.sha256()),
    )?;
    fixture.instance.stop().map_err(|e| failure(&e))
}
fn json(exchange: &Exchange) -> Result<serde_json::Value> {
    Ok(serde_json::from_slice(&exchange.body)?)
}
#[cfg(test)]
mod tests;
