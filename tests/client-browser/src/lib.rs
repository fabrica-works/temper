//! The browser tier (client/browser.md, 4.3): Chromium against `temper serve`, on a scratch deployment whose
//! forge and LLM endpoint are unreached, serving the bundle the gate builds. Its helpers are jig's
//! `tests/client-browser/src/lib.rs`'s, for temper's binary and bundle.
use skein_fake_person::Find;
use skein_fake_person_browser::{
    Setup,
    harness::{Names, Output, Printed},
};
use skein_ui::Role;
use std::path::{Path, PathBuf};
mod configuration;

/// How long a story waits for what a page shows.
pub const WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(5);
/// How long a story watches for what a page must not show.
pub const ABSENT: skein_lib::Duration = skein_lib::Duration::from_secs(1);

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

/// A scratch deployment for one story's `temper serve`.
pub struct Tier {
    scratch: skein_scratch::Scratch,
}
impl Tier {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::write(scratch.path(), &temper(), &bundle());
        Self { scratch }
    }
    #[must_use]
    pub fn setup(&self) -> Setup<'_> {
        let args = vec!["serve".into(), self.scratch.path().join("config.json").into_os_string()];
        Setup {
            server: temper(),
            args: args.clone(),
            restart: args,
            printed: Printed { prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url },
            chromium: chromium(),
            expected: vec![],
            origins: vec![],
        }
    }
    /// The banner's status node names the link's state; signed out, the page says how to sign in.
    #[must_use]
    pub fn states(&self) -> jig_client_fake_person_browser::States {
        jig_client_fake_person_browser::States {
            live: Find::exactly_named(Role::Status, b"Live"),
            offline: Find::exactly_named(Role::Status, b"Offline"),
            signed_out: Find::text(b"Signed out. To sign in, open a sign-in link temper serve printed."),
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
