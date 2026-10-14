//! Records format-faithful exchanges against an isolated Forgejo (forge.md, 10).
mod cases;
mod fixture;
pub mod mask;
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
    let secrets = write::Secrets::from_instance(&fixture.instance);
    cases::record(&fixture, out, &mut mask, &secrets)?;
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
