pub mod support;

// The process configuration reader admits a MiB, 64K tokens and 32 levels.
const DOCUMENT: jig_protocol_config::DocumentLimits =
    jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
// client.md 5.4 admits twelve entries independently of document rows.
const AUTHORITY_ENTRIES: u32 = 12;

fn process_engine() -> jig_protocol_config::Engine {
    let scratch = support::Scratch::startup("native-document-limits");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bytes = std::fs::read(scratch.path().join("config.json")).expect("process configuration");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    // Forge::Unreached omits Forgejo's overrides; include its whole-authority bounds here.
    value["engine"]["limits"]["overrides"]["authority.grants"] = 12.into();
    value["engine"]["limits"]["overrides"]["tasks.authority_grants"] = 12.into();
    value["engine"]["limits"]["overrides"]["notes.scopes"] = 15.into();
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    let engine = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    scratch.remove();
    engine
}

#[test]
fn the_process_configurations_documents_fit_the_native_client() {
    let engine = process_engine();
    let client = temper_client_shell_terminal::limits::native().client;
    let native = client.client.domain;
    assert!(engine.limits.core.documents.rows <= native.rows, "the native client can receive every document row");
    assert!(
        engine.limits.core.documents.lines <= native.rows,
        "the native client can receive every document's short list"
    );
    let paths = engine
        .limits
        .core
        .authority
        .segments
        .max(engine.limits.core.tasks.hold_segments)
        .max(engine.limits.core.tasks.authority_segments);
    assert!(paths <= native.segments, "the native client can receive every document path");
    assert!(
        engine.limits.core.documents.text_bytes <= native.text,
        "the native client can receive every document text"
    );
    assert!(engine.limits.core.documents.grants <= client.client.domain.authority, "every grant fits");
    assert!(engine.limits.core.documents.executors <= client.client.domain.authority, "every executor fits");
    assert!(engine.limits.core.documents.implications <= client.client.domain.authority, "every implication fits");
    assert!(temper_engine_domain::WATCHES_PER_STREAM >= native.watches, "the stream fits the client's watches");
    assert!(temper_engine_domain::VIEWS.watchers_per_party >= native.watches, "the party fits the client's watches");
    assert!(temper_engine_domain::VIEWS.watchers >= native.watches, "the engine fits the client's watches");
}

#[test]
fn the_owned_example_fits_the_native_client() {
    let value: serde_json::Value = serde_json::from_slice(support::owned::EXAMPLE).expect("owned example JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    let engine = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    let native = temper_client_shell_terminal::limits::native().client.client.domain;
    assert!(engine.limits.core.documents.rows <= native.rows);
    assert!(engine.limits.core.documents.lines <= native.rows);
    let paths = engine
        .limits
        .core
        .authority
        .segments
        .max(engine.limits.core.tasks.hold_segments)
        .max(engine.limits.core.tasks.authority_segments);
    assert!(paths <= native.segments);
    assert!(engine.limits.core.documents.text_bytes <= native.text);
    assert!(engine.limits.core.authority.grants <= AUTHORITY_ENTRIES);
    assert!(engine.limits.core.tasks.authority_grants <= AUTHORITY_ENTRIES);
    assert_owned_limits(&value["engine"]["limits"]);
}
fn assert_owned_limits(limits: &serde_json::Value) {
    assert_eq!(
        limits,
        &serde_json::json!({"profile":"small","overrides":{
            "authority.segments":8,"tasks.hold_segments":8,"tasks.authority_segments":8,
            "documents.rows":2,"documents.lines":2,"documents.text_bytes":128,
            "views.watchers":16,"views.watchers_per_party":16,
            "authority.grants":12,"tasks.authority_grants":12,"notes.scopes":15,
            "tasks.authority_bytes":512,"tasks.batch":1
        }})
    );
}
