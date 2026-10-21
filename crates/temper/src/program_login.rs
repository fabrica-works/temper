//! Terminal program login without deployment resources (protocol/credentials.md, 3).
//! A pre-exec failure adds one `temper program-login:` line. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use std::{
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, process::CommandExt},
};

fn source_name(name: &OsStr) -> Result<&str, String> {
    smith_host_domain::SourceName::new(name.as_bytes())
        .ok_or_else(|| smith_agent_shell::program::Refusal::Name.to_string())?;
    Ok(name.to_str().expect("validated ASCII source name"))
}

fn command(
    directories: &super::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<std::process::Command, String> {
    let name = source_name(name)?;
    let bytes = directories.read()?;
    let super::setup::Deployment { named, .. } = super::setup::check(&bytes, &directories.jig())?;
    let agent = super::agent::preflight(&named)?;
    let program = agent
        .programs
        .iter()
        .find(|program| program.adapter == temper_service::settings::Adapter::ClaudeCode)
        .ok_or("workers[0].agent.configuration: programs.claude_code: not configured")?;
    let source = smith_agent_shell::program::prepare(super::path(&program.directory), name)
        .map_err(|why| super::agent::program_directory_refusal(&why))?;
    Ok(smith_agent_shell::program::sign_in(super::path(&program.path), &source, environment))
}

pub(crate) fn run(directories: &super::Directories, name: &OsStr) -> Result<u8, String> {
    let error = command(directories, name, std::env::vars_os())?.exec();
    Err(format!("exec: {error}"))
}

#[cfg(test)]
mod tests;
