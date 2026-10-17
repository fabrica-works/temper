//! Configuration and credentials shared by the world's stories.
use skein_lib::{List, Secret};
use temper_fake_forge_protocol::{Config, Faults};
use temper_forge_setup::{Setup, limits, problem::Problems, setup::Entry};
pub const ADMIN: &[u8] = b"admin-token-for-the-setup-world";
pub const API: &[u8] = b"api-token-issued-to-the-bot";
pub const GIT: &[u8] = b"git-token-issued-to-the-bot";
const MANAGED: &[u8] = br#"[{"forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["acme","work"],"mode":{"managed":{"admin_token":"admin_token"}}}]"#;
const OWNED: &[u8] = br#"[{"forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["acme","work"],"mode":{"owned":{"forgejo":"/opt/forgejo","runner":null,"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"ada","email":"ada@example.invalid"}}}}]"#;
#[must_use]
pub fn entry(owned: bool) -> Entry {
    let document = jig_protocol_config::document(
        if owned { OWNED } else { MANAGED },
        &jig_protocol_config::DocumentLimits { bytes: 4096, tokens: 256, depth: 16 },
    )
    .expect("world setup document");
    Setup::read(&document, 0, &limits::small(), &mut Problems::empty())
        .expect("world setup")
        .entry(1)
        .expect("forge 1")
        .clone()
}
#[must_use]
pub fn config() -> Config {
    Config {
        hooks: None,
        base: b"http://127.0.0.1:3000".as_slice().into(),
        tokens: List::with_capacity(0),
        users: List::with_capacity(0),
        repositories: List::with_capacity(0),
        // Three teams require a second page; all other fake limits are shipped.
        max_response_items: 2,
        default_paging_num: 30,
        faults: Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
#[must_use]
pub fn secret(bytes: &[u8]) -> Secret {
    Secret::new(bytes.into())
}
/// A bot with both valid token files; changing one file isolates its repair.
pub fn bot_tokens(world: &mut crate::World) -> u64 {
    use temper_forge_setup::place::File;
    let id = world.store_mut().account(b"temper", false, b"temper:bot");
    world.store_mut().token(id, b"temper-api", &[b"read:user", b"write:issue", b"write:repository"], secret(API));
    world.store_mut().token(id, b"temper-git", &[b"write:repository", b"read:user"], secret(GIT));
    world.put(File::ApiToken, secret(API));
    world.put(File::GitToken, secret(GIT));
    id
}
#[must_use]
pub fn team(name: &[u8]) -> temper_forge_forgejo::document::TeamSpec {
    use temper_forge_forgejo::document::{Access, TeamSpec, Units};
    TeamSpec {
        name: name.into(),
        permission: Access::Write,
        includes_all_repositories: true,
        units: Units { code: Access::Write, issues: Access::Write, pulls: Access::Write, actions: Access::Write },
    }
}
#[must_use]
pub fn listener() -> std::net::SocketAddr {
    "127.0.0.1:9898".parse().expect("listener")
}
pub const URL: &[u8] = b"http://127.0.0.1:9898/hooks/1";
pub const WEBHOOK: &[u8] = b"webhook-secret-present-before-reconcile";
