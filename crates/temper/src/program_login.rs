//! Terminal program login without deployment resources (protocol/credentials.md, 3).
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
mod tests {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    #[test]
    fn source_names_use_smiths_length_and_alphabet() {
        let max = smith_host_domain::SourceName::BYTES;
        for (bytes, valid) in [
            (vec![], false),
            (b"a".to_vec(), true),
            (vec![b'a'; max], true),
            (vec![b'a'; max + 1], false),
            (b"az09-".to_vec(), true),
            (b"A".to_vec(), false),
            (b"a_b".to_vec(), false),
            (b"../a".to_vec(), false),
            (vec![0xff], false),
            (vec![0], false),
        ] {
            assert_eq!(source_name(&OsString::from_vec(bytes.clone())).is_ok(), valid, "{bytes:?}");
        }
    }

    #[test]
    fn smiths_terminal_command_keeps_os_paths_and_filters_the_callers_environment() {
        let directory = std::env::temp_dir().join(format!("temper-login-command-{}", std::process::id()));
        std::fs::create_dir(&directory).expect("login command scratch");
        let executable = Path::new("/nonexistent/configured-claude");
        let programs = directory.join("programs");
        let source = programs.join("source");
        let (agent, deployment) = crate::tests::configuration::documents(&directory, "/fixture/temper");
        let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
        agent["programs"] = serde_json::json!({"claude_code":{"path":executable,"directory":programs}});
        std::fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("agent JSON")).expect("agent");
        std::fs::write(directory.join("root.der"), skein_tls_world::pki::ROOT).expect("public trust");
        let configuration = directory.join(OsString::from_vec(b"config-\xff.json".to_vec()));
        std::fs::write(&configuration, deployment).expect("non-UTF8 configuration path");
        let benign = OsString::from_vec(b"value-\xff".to_vec());
        let command = command(
            &configuration,
            OsStr::new("source"),
            [
                ("HOME".into(), "/home/person".into()),
                ("PATH".into(), "/bin".into()),
                ("TEMPER_BENIGN".into(), benign.clone()),
                ("CLAUDE_CONFIG_DIR".into(), "/wrong".into()),
                ("ANTHROPIC_API_KEY".into(), "private".into()),
                ("CLAUDE_CODE_OAUTH_TOKEN".into(), "private".into()),
                ("AWS_BEARER_TOKEN_BEDROCK".into(), "private".into()),
                ("CLAUDE_CODE_USE_VERTEX".into(), "private".into()),
            ],
        )
        .expect("checked terminal command");
        assert_eq!(command.get_program(), executable.as_os_str());
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
        let environment = command
            .get_envs()
            .map(|(name, value)| (name, value.expect("set")))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(environment.len(), 4);
        for (name, value) in [
            ("HOME", OsStr::new("/home/person")),
            ("PATH", OsStr::new("/bin")),
            ("TEMPER_BENIGN", benign.as_os_str()),
            ("CLAUDE_CONFIG_DIR", source.as_os_str()),
        ] {
            assert_eq!(environment[OsStr::new(name)], value);
        }
        std::fs::remove_dir_all(directory).expect("remove command scratch");
    }
}
