//! The setup owner checks its document before commands read any files.
use jig_protocol_config as config;
use temper_forge_setup::{
    check::{self, Checked, Members},
    problem,
};
use temper_service::startup::{Named, Transport, User};

pub(crate) fn check(bytes: &[u8]) -> Result<(Named, Checked), String> {
    let named = temper_service::named(bytes)
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?;
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .expect("named checked document");
    let mut forges = Vec::with_capacity(named.forges.len());
    for forge in &named.forges {
        let identity =
            named.identities.iter().find(|identity| identity.number == forge.identity).expect("named checked identity");
        forges.push(check::Forge {
            number: forge.number,
            api: match &forge.api {
                config::Address::Addr(a) => a.socket(),
                config::Address::Host { .. } => None,
            },
            plaintext: matches!(forge.transport, Transport::Plaintext),
            git: forge.git.clone(),
            user: match &forge.user {
                User::Id(value) => check::User::Id(*value),
                User::File(path) => check::User::File(path.clone()),
            },
            api_token: forge.api_token.clone(),
            identity_token: identity.token.clone(),
            identity_login: identity.login.clone(),
        });
    }
    let listeners = match &named.clients {
        config::Address::Addr(a) => a.socket().into_iter().collect::<Vec<_>>(),
        config::Address::Host { .. } => Vec::new(),
    };
    let members = Members { forges: forges.into_boxed_slice(), listeners: listeners.into_boxed_slice() };
    let application = super::member(&document, 0, b"application");
    let at = super::member(&document, application, b"setup");
    let checked = check::checked(&document, at, &members, &named.overrides)
        .map_err(|why| String::from_utf8_lossy(&problem::lines(&why)).into_owned())?;
    Ok((named, checked))
}
