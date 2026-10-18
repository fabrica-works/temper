pub mod support;

// The process configuration reader admits a MiB, 64K tokens and 32 levels.
const DOCUMENT: jig_protocol_config::DocumentLimits =
    jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
// client.md 5.4 admits twelve entries independently of document rows.
const AUTHORITY_ENTRIES: u32 = 12;

#[test]
fn the_process_configurations_documents_fit_the_native_client() {
    let scratch = support::Scratch::startup("native-document-limits");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bytes = std::fs::read(scratch.path().join("config.json")).expect("process configuration");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    let engine = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    let native = temper_client_shell_terminal::limits::native().client.jig;
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
    scratch.remove();
}

#[test]
fn the_owned_example_fits_the_native_client() {
    let value: serde_json::Value = serde_json::from_slice(support::owned::EXAMPLE).expect("owned example JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    let engine = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    let native = temper_client_shell_terminal::limits::native().client.jig;
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
