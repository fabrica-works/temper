use super::fixtures;
#[test]
fn jig_names_only_itself_except_the_shared_request_key_reexport() {
    for (path, text) in &fixtures::sources() {
        // The page and ctl share these exact key bytes (client/forge.md, 5.4).
        let text = if path == std::path::Path::new("src/jig/key.rs") {
            text.replace("pub use temper_client_domain::key::{Name, named};", "").into_boxed_str()
        } else {
            text.clone()
        };
        assert!(!text.to_ascii_lowercase().contains("temper"), "{path:?} names temper");
        for (at, _) in text.match_indices("crate::") {
            assert!(text[at..].starts_with("crate::jig"), "{path:?} names an item outside jig");
        }
    }
}
