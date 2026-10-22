pub mod support;

// The process configuration reader admits a MiB, 64K tokens and 32 levels.
const DOCUMENT: jig_protocol_config::DocumentLimits =
    jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
// client/README.md 5.4 admits twelve entries independently of document rows.
const AUTHORITY_ENTRIES: u32 = 12;

fn process_engine() -> jig_protocol_config::Engine {
    let scratch = support::Scratch::startup("native-document-limits");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bytes = std::fs::read(scratch.path().join("temper.json")).expect("process configuration");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    // Forge::Unreached omits Forgejo's overrides; include its whole-authority bounds here.
    value["engine"]["limits"]["overrides"]["authority.grants"] = 12.into();
    value["engine"]["limits"]["overrides"]["tasks.authority_grants"] = 12.into();
    value["engine"]["limits"]["overrides"]["notes.scopes"] = 15.into();
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    // Parsing a fixture without a deployment; only its limits are used.
    let engine = jig_protocol_config::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("engine configuration");
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
    // Parsing a fixture without a deployment; only its limits are used.
    let engine = jig_protocol_config::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("engine configuration");
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

#[test]
fn the_clusters_documents_check_as_their_shapes() {
    use temper_service::startup::{self, Shape};
    let scratch = support::Scratch::startup("cluster-documents");
    let cluster = support::cluster::Cluster::documents(
        scratch.path(),
        &[("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    );
    let engine: serde_json::Value = serde_json::from_slice(
        &std::fs::read(cluster.dir("engine").join("config/temper.json")).expect("lab/cluster/engine.json"),
    )
    .expect("engine JSON");
    let engine_limits = &engine["engine"]["limits"]["overrides"];
    assert_eq!(
        engine_limits["fleet.grace_ms"],
        support::cluster::FLEET_GRACE_MS,
        "lab/cluster/engine.json: engine.limits.overrides.fleet.grace_ms"
    );
    for (process, expected) in [("engine", Shape::EngineAlone), ("w1", Shape::WorkerAlone), ("w2", Shape::WorkerAlone)]
    {
        let root = cluster.dir(process);
        let directories = temper_real_loop_world::cluster::directories_in([
            &root.join("config"),
            &root.join("data"),
            &root.join("cache"),
        ]);
        let bytes = std::fs::read(root.join("config/temper.json")).expect("cluster document");
        let shape = startup::shape(&bytes, &directories.jig())
            .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        assert_eq!(shape, expected, "{process}");
        match shape {
            Shape::EngineAlone => {
                startup::named(&bytes, &directories.jig()).expect("checked engine alone");
            }
            Shape::WorkerAlone => {
                let part = startup::worker_part(&bytes, &directories.jig()).expect("checked worker alone");
                let worker = &part.worker.jig;
                let stop = jig_protocol_config::stop_bound(
                    &worker.graces,
                    worker.limits.link.ping,
                    worker.limits.link.silence,
                )
                .unwrap_or_else(|| panic!("lab/cluster/{process}.json: workers[0].graces stop_bound overflows"));
                assert!(
                    stop < skein_lib::Duration::from_millis(support::cluster::FLEET_GRACE_MS),
                    "lab/cluster/{process}.json: workers[0].graces stop_bound must be below engine.json: engine.limits.overrides.fleet.grace_ms"
                );
                let value: serde_json::Value = serde_json::from_slice(&bytes).expect("worker JSON");
                for key in ["link.ping_ms", "link.silence_ms"] {
                    assert_eq!(
                        value["workers"][0]["limits"]["overrides"][key], engine_limits[key],
                        "lab/cluster/{process}.json: workers[0].limits.overrides.{key} must equal engine.json: engine.limits.overrides.{key}"
                    );
                }
            }
            Shape::Standalone => panic!("cluster members are separate"),
        }
    }
    scratch.remove();
}
