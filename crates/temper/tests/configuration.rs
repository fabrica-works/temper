pub mod support;

#[test]
fn the_process_configurations_documents_fit_the_native_client() {
    let scratch = support::Scratch::startup("native-document-limits");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bytes = std::fs::read(scratch.path().join("config.json")).expect("process configuration");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("engine document");
    let engine = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    let native = temper_shell_terminal::limits::native().client.jig;
    assert!(engine.limits.core.documents.rows <= native.rows, "the native client can receive every document row");
    assert!(
        engine.limits.core.documents.text_bytes <= native.text,
        "the native client can receive every document text"
    );
    scratch.remove();
}
