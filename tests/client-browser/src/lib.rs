//! The browser tier (client/browser.md, 4.3): Chromium against `temper serve`, on a scratch deployment whose
//! forge and LLM endpoint are unreached, serving the bundle the gate builds. Its helpers are jig's
//! `tests/client-browser/src/lib.rs`'s, for temper's binary and bundle.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{
    Setup, Step,
    harness::{Names, Output, Printed},
};
use skein_ui::Role;
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
};
mod configuration;
mod forgery;

/// How long a story waits for what a page shows.
pub const WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(5);
/// How long a story watches for what a page must not show.
pub const ABSENT: skein_lib::Duration = skein_lib::Duration::from_secs(1);
/// The heading of the first chat a story starts.
pub const FIRST_CHAT: &[u8] = b"Chat 1";
/// Its address.
pub const FIRST_CHAT_ADDRESS: &[u8] = b"/chats/1";

/// The `temper` binary beside the test executable.
#[must_use]
pub fn temper() -> PathBuf {
    let exe = std::env::current_exe().expect("test executable");
    let binary = exe.parent().and_then(Path::parent).expect("target directory").join("temper");
    assert!(binary.is_file(), "build the browser server with heavy cargo build -p temper: {}", binary.display());
    binary
}
#[must_use]
pub fn chromium() -> PathBuf {
    if let Some(path) = std::env::var_os("SKEIN_TEST_CHROMIUM") {
        let path = PathBuf::from(path);
        assert!(path.is_file(), "SKEIN_TEST_CHROMIUM must name a file");
        return path.canonicalize().expect("configured Chromium");
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").expect("PATH")) {
        let path = directory.join("chromium");
        if path.is_file() {
            return path.canonicalize().expect("Chromium on PATH");
        }
    }
    panic!("Chromium is required: set SKEIN_TEST_CHROMIUM or put chromium on PATH");
}
/// The bundle the gate builds, named by `TEMPER_TEST_BUNDLE`.
#[must_use]
pub fn bundle() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("TEMPER_TEST_BUNDLE")
            .expect("build the bundle with scripts/bundle.sh target/temper-bundle and set TEMPER_TEST_BUNDLE"),
    );
    let path = if path.is_absolute() { path } else { Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(path) };
    assert!(
        path.join("index.html").is_file(),
        "build the bundle with scripts/bundle.sh {}: it holds no index.html",
        path.display()
    );
    path.canonicalize().expect("absolute bundle")
}
/// From a signed-in page, start the first chat with `words` and see it.
#[must_use]
pub fn start_chat(words: &[u8]) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"Start a chat");
    vec![
        see(live()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") }),
        see(composer()),
        Step::Person(PersonStep::Type { find: composer(), words: Box::from(words) }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Start") }),
        see(Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT)),
    ]
}
fn live() -> Find {
    Find::exactly_named(Role::Status, b"Live")
}

/// A scratch deployment for one story's `temper serve`.
pub struct Tier {
    scratch: skein_scratch::Scratch,
    forged: Box<[u8]>,
}
impl Tier {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::write(scratch.path(), &temper(), &bundle());
        Self { scratch, forged: forgery::request() }
    }
    /// Account offers with a static public key set, served by the same scratch server before Visit.
    #[must_use]
    pub fn accounts(name: &str) -> Self {
        let tier = Self::reserved(name);
        let path = tier.scratch.path().join("config.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        let private_bundle = tier.scratch.path().join("bundle");
        copy_bundle(&bundle(), &private_bundle);
        let digest = fs::read_dir(&private_bundle)
            .expect("bundle entries")
            .map(|entry| entry.expect("bundle entry"))
            .find(|entry| entry.file_type().expect("bundle entry type").is_dir())
            .expect("digest directory");
        // Public, synthetic 2048-bit RSA modulus; no callback or signature is exercised here.
        let keys = serde_json::json!({"keys":[{"kty":"RSA","n":format!("g{}", "A".repeat(341)),"e":"AQAB"}]});
        fs::write(digest.path().join("issuer.webmanifest"), serde_json::to_vec(&keys).expect("key set"))
            .expect("static issuer key set");
        let socket = value["application"]["clients"]["listener"]["socket"].as_str().expect("reserved socket");
        let keys =
            format!("http://{socket}/bundle/{}/issuer.webmanifest", digest.file_name().to_str().expect("digest"));
        value["application"]["clients"]["bundle"] = serde_json::json!(private_bundle);
        value["engine"]["limits"]["overrides"]["accounts.sign_in_deadline_ms"] = 2000.into();
        value["engine"]["llm"]["providers"] = serde_json::json!([{
            "name":"Test provider", "number":17, "models":["fake"],
            "sources":{"keys":true,"sign_in":{
                "issuer":"http://127.0.0.1:9", "client":"temper-test", "scopes":["openid","chat"],
                "authorization":"http://127.0.0.1:9/authorize", "token":"http://127.0.0.1:9/token",
                "keys":keys, "transport":"plaintext", "must_grant":["chat"],
                "redirect_path":"callback"
            }}
        }]);
        configuration::private(&path, &serde_json::to_vec(&value).expect("accounts deployment JSON"));
        tier
    }
    /// A tier whose server listens at a loopback port reserved beforehand, so that a restart opens the same
    /// address and a page reconnects to it. Jig's tier has the same hook.
    #[must_use]
    pub fn reserved(name: &str) -> Self {
        let tier = Self::new(name);
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve restart port");
        let address = listener.local_addr().expect("restart address");
        let path = tier.scratch.path().join("config.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        value["application"]["clients"]["listener"]["socket"] = serde_json::json!(address.to_string());
        configuration::private(&path, &serde_json::to_vec(&value).expect("reserved deployment JSON"));
        tier
    }
    #[must_use]
    pub fn setup(&self) -> Setup<'_> {
        let args = vec!["serve".into(), self.scratch.path().join("config.json").into_os_string()];
        Setup {
            server: temper(),
            args: args.clone(),
            restart: args,
            printed: Printed {
                sent: skein_fake_person_browser::harness::Sent::Page,
                prefix: b"http://127.0.0.1:",
                on: Output::Stdout,
                names: Names::Url,
            },
            chromium: chromium(),
            expected: vec![],
            origins: vec![],
        }
    }
    /// The banner's status node names the link's state; signed out, the page says how to sign in.
    #[must_use]
    pub fn states(&self) -> jig_client_fake_person_browser::States {
        jig_client_fake_person_browser::States {
            live: live(),
            offline: Find::exactly_named(Role::Status, b"Offline"),
            signed_out: Find::text(b"Signed out. To sign in, open a sign-in link temper serve printed."),
        }
    }
    /// A forged chat would show its words on the chats page if accepted.
    #[must_use]
    pub fn target(&self) -> jig_client_fake_person_browser::Target<'_> {
        jig_client_fake_person_browser::Target {
            prefix: b"/jig",
            page: b"/chats",
            request: &self.forged,
            done: Find::text(b"Forged"),
        }
    }
}

/// Keep the built bundle immutable while giving this one-serve story its static issuer document.
fn copy_bundle(from: &Path, to: &Path) {
    fs::create_dir(to).expect("private bundle directory");
    for entry in fs::read_dir(from).expect("built bundle") {
        let entry = entry.expect("bundle entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("bundle entry type").is_dir() {
            copy_bundle(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy bundle file");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::configuration::{UNREACHED, documents};
    use std::path::Path;
    fn tier_configuration() -> (serde_json::Value, serde_json::Value) {
        let (agent, value) =
            documents(Path::new("/private/test"), Path::new("/test/temper"), Path::new("/test/bundle"));
        (serde_json::from_slice(&agent).expect("agent JSON"), serde_json::from_slice(&value).expect("deployment JSON"))
    }
    #[test]
    fn the_tier_names_the_bundle_for_its_server() {
        assert_eq!(tier_configuration().1["application"]["clients"]["bundle"], "/test/bundle");
    }
    #[test]
    fn the_tier_listens_on_loopback_at_a_dynamic_port() {
        assert_eq!(tier_configuration().1["application"]["clients"]["listener"]["socket"], "127.0.0.1:0");
    }
    #[test]
    fn the_tiers_forge_is_unreached() {
        assert_eq!(tier_configuration().1["application"]["forges"][0]["api"]["socket"], UNREACHED);
    }
    #[test]
    fn the_tiers_llm_endpoint_is_unreached() {
        let (agent, value) = tier_configuration();
        assert_eq!(value["engine"]["llm"]["endpoints"][0]["address"]["socket"], UNREACHED);
        assert_eq!(agent["endpoints"][0]["address"], UNREACHED);
    }
    #[test]
    fn the_tiers_paths_are_absolute() {
        let value = tier_configuration().1;
        assert_eq!(value["application"]["forges"][0]["api_token"], "/private/test/secrets/forge-1.api-token");
        assert_eq!(value["application"]["workers"][0]["root"], "/private/test/worker-root");
    }
}
