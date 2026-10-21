//! The deployment member against facts from the real document readers.
use crate::{demo, llm};
use jig_protocol_config::{self as config, Path};
use serde_json::{Value, json};
use temper_forge_setup::{problem, setup::Setup};

type Row = (Vec<(&'static str, Option<Value>)>, &'static str);

fn fixture() -> Value {
    let mut d: Value = serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).unwrap();
    d["engine"]["llm"]["endpoints"].as_array_mut().unwrap().push(json!({
        "name":"demo", "number":3, "dialect":3, "provider":"demo",
        "address":{"socket":"127.0.0.1:8370"}, "transport":"plaintext", "headers":[],
        "connect_ms":10000, "handshake_ms":10000
    }));
    d["engine"]["llm"]["models"].as_array_mut().unwrap().push(json!({
        "name":"demo", "endpoint":"demo", "window":4096, "output":1024,
        "prices":{"input":1,"cached":1,"output":1,"unit":1}
    }));
    d["engine"]["llm"]["providers"].as_array_mut().unwrap().push(json!({
        "name":"demo", "number":3, "models":["demo"], "sources":{"keys":true}
    }));
    d["application"]["setup"][0].as_object_mut().unwrap().remove("orgs");
    d["application"]["setup"][0]["mode"] = json!({"fake":{
        "listen":{"socket":"127.0.0.1:3000"},"git":"demo/bin/git", "owner":"demo",
        "repositories":[{"name":"widget","landing":"main"}]
    }});
    d["application"]["demo"] = json!({"llm":{
        "listen":{"socket":"127.0.0.1:8370"}, "provider":3, "key":"demo/llm.key",
        "asked":[{"cue":"Add CSV export to widget","turns":["Export CSV",{
            "lines":[{"text":"Done"}],"finish":"stop","tokens":1
        }]}],"otherwise":["demo model"]}, "forge":1, "project":1, "client":"demo/client"});
    d
}
fn member(d: &skein_json::Document, at: u32, name: &[u8]) -> u32 {
    config::read::Object::new(d, at).unwrap().member(d, name).unwrap()
}
fn read(value: &Value) -> Result<Option<demo::Demo>, crate::Refusal> {
    let bytes = serde_json::to_vec(value).unwrap();
    let d = config::document(&bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 }).unwrap();
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let engine = config::engine(&d, member(&d, 0, b"engine"), &directories).unwrap();
    let app = member(&d, 0, b"application");
    let mut ps = problem::Problems::empty();
    let setup = Setup::read(&d, member(&d, app, b"setup"), &temper_forge_setup::limits::small(), &directories, &mut ps)
        .unwrap();
    assert!(ps.is_empty(), "{:?}", problem::lines(&ps));
    let mut ps = config::Problems::empty();
    let listeners: Vec<_> = [b"clients".as_slice(), b"webhooks"]
        .iter()
        .map(|name| {
            let at = member(&d, member(&d, app, name), b"listener");
            match config::address::read(&d, at, &Path::root(), &mut ps).unwrap() {
                config::Address::Addr(a) => a.socket().unwrap(),
                config::Address::Host { .. } => panic!("fixture socket"),
            }
        })
        .collect();
    assert!(ps.is_empty());
    let projects: Vec<_> = engine.projects.iter().map(|p| p.number).collect();
    let worker = directories
        .join(config::Directory::Data, value["application"]["workers"][0]["root"].as_str().unwrap().as_bytes())
        .unwrap();
    let mut taken = vec![engine.deployment.directory.as_ref(), worker.as_ref()];
    taken.extend(setup.entries.iter().map(|e| e.directory.as_ref()));
    demo::read(
        &d,
        member(&d, app, b"demo"),
        &demo::Facts {
            llm: &engine.llm,
            projects: &projects,
            setup: &setup,
            listeners: &listeners,
            directories: &directories,
            taken: &taken,
        },
    )
}
#[test]
#[expect(clippy::too_many_lines, reason = "one refusal table covers the member and its document relationships")]
fn demo_refusals() {
    let owned: Value = serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).unwrap();
    let sign_in = json!({"issuer":"https://issuer.example", "client":"client", "scopes":[],
        "authorization":"https://issuer.example/auth", "token":"https://issuer.example/token",
        "keys":"https://issuer.example/keys", "transport":"tls", "server_name":"issuer.example",
        "must_grant":[], "redirect_path":"callback"});
    let scripts = json!([{"cue":"x","turns":["x"]}]);
    // Each row edits a fresh document before either real parser sees it.
    let mut rows: Vec<Row> = vec![
        (vec![("/application/demo/extra", Some(json!(1)))], "application.demo.extra: unknown member\n"),
        (vec![("/application/demo/llm/extra", Some(json!(1)))], "application.demo.llm.extra: unknown member\n"),
        (vec![("/application/demo/client", None)], "application.demo.client: missing\n"),
        (vec![("/application/demo", Some(json!([])))], "application.demo: wrong value kind\n"),
        (
            vec![("/application/demo/llm/provider", Some(json!("3")))],
            "application.demo.llm.provider: wrong value kind\n",
        ),
        (
            vec![("/application/demo/llm/listen/socket", Some(json!("192.0.2.1:8370")))],
            "application.demo.llm.listen: must be a loopback socket address\n",
        ),
        (
            vec![("/application/demo/llm/listen/socket", Some(json!("127.0.0.1:7100")))],
            "application.demo.llm.listen: must be unique\n",
        ),
        (
            vec![("/application/demo/llm/listen/socket", Some(json!("127.0.0.1:7101")))],
            "application.demo.llm.listen: must be unique\n",
        ),
        (
            vec![("/application/demo/llm/listen/socket", Some(json!("127.0.0.1:3000")))],
            "application.demo.llm.listen: must be unique\n",
        ),
        (
            vec![
                ("/application/setup/0", Some(owned["application"]["setup"][0].clone())),
                ("/application/demo/llm/listen/socket", Some(json!("127.0.0.1:3000"))),
            ],
            "application.demo.llm.listen: must be unique\n",
        ),
        (
            vec![("/application/demo/llm/provider", Some(json!(9)))],
            "application.demo.llm.provider: names no LLM provider\n",
        ),
        (
            vec![
                ("/engine/llm/providers/1/sources/sign_in", Some(sign_in.clone())),
                ("/engine/llm/providers/1/signed_in_plan", Some(json!(false))),
            ],
            "application.demo.llm.provider: must offer a key alone\n",
        ),
        (
            vec![
                ("/engine/llm/providers/1/sources/programs", Some(json!(["claude_code"]))),
                ("/engine/llm/providers/1/manage_usage", Some(json!("https://example.test/usage"))),
            ],
            "application.demo.llm.provider: must offer a key alone\n",
        ),
        (
            vec![("/engine/llm/providers/1/sources/keys", Some(json!(false)))],
            "application.demo.llm.provider: must offer a key alone\n",
        ),
        (
            vec![
                ("/engine/llm/providers/1/models", Some(json!([]))),
                ("/engine/llm/models", Some(json!([owned["engine"]["llm"]["models"][0].clone()]))),
            ],
            "application.demo.llm.provider: names a provider with no models\n",
        ),
        (
            vec![("/engine/llm/endpoints/1/address/socket", Some(json!("127.0.0.1:8371")))],
            "application.demo.llm.provider: names an endpoint not at llm.listen\n",
        ),
        (
            vec![
                ("/engine/llm/endpoints/1/transport", Some(json!("tls"))),
                ("/engine/llm/endpoints/1/server_name", Some(json!("localhost"))),
            ],
            "application.demo.llm.provider: names an endpoint with TLS\n",
        ),
        (
            vec![("/engine/llm/endpoints/1/dialect", Some(json!(1)))],
            "application.demo.llm.provider: names an endpoint not in OpenAI's dialect\n",
        ),
        (vec![("/engine/llm/models/1/window", Some(json!(0)))], "application.demo.llm.provider: must be at least 1\n"),
        (vec![("/engine/llm/models/1/output", Some(json!(0)))], "application.demo.llm.provider: must be at least 1\n"),
        (
            vec![("/engine/llm/models/1/output", Some(json!(4097)))],
            "application.demo.llm.provider: must not exceed window\n",
        ),
        (
            vec![("/application/demo/llm/key", Some(json!("")))],
            "application.demo.llm.key: relative path must be non-empty with no .. component\n",
        ),
        (
            vec![("/application/demo/llm/key", Some(json!("k".repeat(4097))))],
            "application.demo.llm.key: past its bound\n",
        ),
        (
            vec![("/application/demo/client", Some(json!("../client")))],
            "application.demo.client: relative path must be non-empty with no .. component\n",
        ),
        (
            vec![("/application/demo/llm/asked", Some(json!((0..9).map(|_| scripts[0].clone()).collect::<Vec<_>>())))],
            "application.demo.llm.asked: past its bound\n",
        ),
        (
            vec![("/application/demo/llm/asked", Some(json!([scripts[0].clone(), scripts[0].clone()])))],
            "application.demo.llm.asked[1].cue: must be unique\n",
        ),
        (
            vec![("/application/demo/llm/otherwise", Some(json!([])))],
            "application.demo.llm.otherwise: must be non-empty\n",
        ),
        (
            vec![(
                "/application/demo/llm/asked",
                Some(json!(
                    (0..8)
                        .map(
                            |i| json!({"cue":i.to_string(),"turns":(0..8).map(|_|"x".repeat(4024)).collect::<Vec<_>>()})
                        )
                        .collect::<Vec<_>>()
                )),
            )],
            "application.demo.llm.otherwise: scripts past their joint byte bound\n",
        ),
        (vec![("/application/demo/forge", Some(json!(2)))], "application.demo.forge: names no fake setup entry\n"),
        (
            vec![("/application/setup/0", Some(owned["application"]["setup"][0].clone()))],
            "application.demo.forge: names no fake setup entry\n",
        ),
        (vec![("/application/demo/project", Some(json!(9)))], "application.demo.project: names no project\n"),
        (vec![("/application/demo/client", Some(json!("forge-1")))], "application.demo.client: must be unique\n"),
        (vec![("/application/demo/client", Some(json!("workspaces")))], "application.demo.client: must be unique\n"),
    ];
    rows.push((
        vec![
            ("/engine/llm/providers/1/sources/sign_in", Some(sign_in)),
            ("/engine/llm/providers/1/signed_in_plan", Some(json!(true))),
            ("/engine/llm/providers/1/manage_usage", Some(json!("https://example.test/usage"))),
        ],
        "application.demo.llm.provider: must offer a key alone\n",
    ));
    rows.push((
        vec![("/application/demo/llm/key", Some(json!("demo/\u{0}")))],
        "application.demo.llm.key: must be an absolute path without NUL\n",
    ));
    let mut models = vec![owned["engine"]["llm"]["models"][0].clone()];
    models.extend((0..9).map(|i| {
        json!({"name":format!("demo-{i}"), "endpoint":"demo", "window":4096,"output":1024,
        "prices":{"input":1,"cached":1,"output":1,"unit":1}})
    }));
    rows.push((
        vec![
            ("/engine/llm/models", Some(json!(models))),
            ("/engine/llm/providers/1/models", Some(json!((0..9).map(|i| format!("demo-{i}")).collect::<Vec<_>>()))),
        ],
        "application.demo.llm.provider: past its bound\n",
    ));
    for (changes, expected) in rows {
        let mut d = fixture();
        for (path, value) in &changes {
            let (parent, field) = path.rsplit_once('/').unwrap();
            let parent = d.pointer_mut(parent).unwrap();
            match value {
                Some(v) => {
                    if let Some(o) = parent.as_object_mut() {
                        o.insert(field.into(), v.clone());
                    } else {
                        parent[field.parse::<usize>().unwrap()] = v.clone();
                    }
                }
                None => {
                    parent.as_object_mut().unwrap().remove(field);
                }
            }
        }
        assert_eq!(read(&d).unwrap_err().line().as_ref(), expected.as_bytes(), "{changes:?}");
    }
}
#[test]
fn demo_acceptance() {
    for (enabled, window, output) in [(false, 4096, 1024), (true, 4096, 1024), (true, 65536, 32768)] {
        let mut value = fixture();
        value["engine"]["llm"]["models"][1]["window"] = json!(window);
        value["engine"]["llm"]["models"][1]["output"] = json!(output);
        if !enabled {
            value["application"]["demo"] = Value::Null;
        }
        let demo = read(&value).unwrap();
        if !enabled {
            assert!(demo.is_none());
            continue;
        }
        let demo = demo.unwrap();
        let m = &value["application"]["demo"]["llm"];
        let standalone = json!({"listen":m["listen"], "key":"/data/demo/llm.key", "tls":null,
            "models":[{"name":"demo","window":window,"output":output}], "cued":[],
            "asked":m["asked"], "otherwise":m["otherwise"]});
        assert_eq!(demo.llm(), &llm::Document::decode(&serde_json::to_vec(&standalone).unwrap()).unwrap());
        assert_eq!(demo.provider(), 3);
        assert_eq!(demo.forge(), 1);
        assert_eq!(demo.project(), 1);
        assert_eq!(demo.client(), b"/data/demo/client");
    }
}
