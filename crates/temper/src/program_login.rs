//! Terminal program login without deployment resources (protocol/credentials.md, 3).
//! A pre-exec failure adds one `temper program-login:` line. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use std::{
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::Path,
};

fn source_name(name: &OsStr) -> Result<&str, String> {
    smith_host_domain::SourceName::new(name.as_bytes())
        .ok_or_else(|| smith_agent_shell::program::Refusal::Name.to_string())?;
    Ok(name.to_str().expect("validated ASCII source name"))
}

fn command(
    configuration: &Path,
    name: &OsStr,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<std::process::Command, String> {
    let name = source_name(name)?;
    let bytes = skein_shell::read_file(configuration, super::CONFIG_BYTES)
        .map_err(|why| format!("configuration {}: {}", configuration.display(), super::unread(why)))?;
    let (named, _, _) = super::setup::check(&bytes)?;
    let directory = std::env::current_dir().map_err(|why| why.to_string())?;
    let agent = super::agent::preflight(&directory, &named)?;
    let program = agent
        .programs
        .iter()
        .find(|program| program.adapter == temper_service::settings::Adapter::ClaudeCode)
        .ok_or("workers[0].agent.configuration: programs.claude_code: not configured")?;
    let source = smith_agent_shell::program::prepare(super::path(&program.directory), name)
        .map_err(|why| super::agent::program_directory_refusal(&why))?;
    Ok(smith_agent_shell::program::sign_in(super::path(&program.path), &source, environment))
}

pub(crate) fn run(configuration: &Path, name: &OsStr) -> Result<u8, String> {
    let error = command(configuration, name, std::env::vars_os())?.exec();
    Err(format!("exec: {error}"))
}

#[cfg(test)]
mod tests;
