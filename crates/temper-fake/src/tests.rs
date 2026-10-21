//! One table for each document and each independent behaviour.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests edit JSON values with serde, leaving the step crate under the root lints"
)]
#![expect(clippy::default_numeric_fallback, reason = "JSON table integers have no Rust arithmetic unit")]
use super::{
    credential::{self, credential},
    forge, llm, turn,
};
use alloc::vec::Vec;
use jig_protocol_config::Path;
use serde_json::{Value, json};
use skein_fake_llm_domain::api;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use skein_shell::Host as _;

fn llm_document() -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"key":"/tmp/key","tls":null,
        "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],"asked":[],"otherwise":["fallback"]})
}
fn forge_document() -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":"/tmp/forge/admin_token","owner":"fake",
        "repositories":[{"name":"widget","landing":"main","ci":null}]})
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table names every document bound and refusal path")]
fn llm_document_checks() {
    let scripts = json!([{"cue":"hello","turns":["answer"]}]);
    let models = json!([{"name":"qa","window":4096,"output":1024}]);
    let object = json!({"lines":[{"text":"hello"}],"finish":"stop","tokens":1});
    let rows = [
        ("/listen", json!({"socket":"192.0.2.1:0"}), "listen: must be a loopback socket address\n"),
        ("/listen", json!({"host":"localhost","port":1}), "listen: must be a loopback socket address\n"),
        ("/key", json!("relative"), "key: must be an absolute path without NUL\n"),
        ("/key", json!("/tmp/\u{0}"), "key: must be an absolute path without NUL\n"),
        ("/tls", json!({"root":"relative"}), "tls.root: must be an absolute path without NUL\n"),
        ("/tls", json!({"root":"/tmp/root","extra":1}), "tls.extra: unknown member\n"),
        ("/tls", json!({}), "tls.root: missing\n"),
        ("/models", json!([]), "models: must be non-empty\n"),
        ("/models", Value::Array((0..9).map(|_| models[0].clone()).collect()), "models: past its bound\n"),
        ("/models/0/name", json!(""), "models[0].name: must be non-empty\n"),
        ("/models/0/name", json!("x".repeat(4097)), "models[0].name: past its bound\n"),
        ("/models/0/window", json!(0), "models[0].window: must be at least 1\n"),
        ("/models/0/window", json!(4_294_967_296_u64), "models[0].window: past its bound\n"),
        ("/models/0/output", json!(0), "models[0].output: must be at least 1\n"),
        ("/models/0/output", json!(4097), "models[0].output: must not exceed window\n"),
        ("/models/0", json!({"name":"qa","window":1,"output":1,"extra":0}), "models[0].extra: unknown member\n"),
        ("/models", Value::Array((0..2).map(|_| models[0].clone()).collect()), "models[1].name: must be unique\n"),
        ("/cued", Value::Array((0..9).map(|_| scripts[0].clone()).collect()), "cued: past its bound\n"),
        ("/asked", Value::Array((0..9).map(|_| scripts[0].clone()).collect()), "asked: past its bound\n"),
        ("/asked", json!([{"cue":"","turns":["x"]}]), "asked[0].cue: must be non-empty\n"),
        ("/cued", json!([{"cue":"x".repeat(257),"turns":["x"]}]), "cued[0].cue: past its bound\n"),
        ("/asked", Value::Array((0..2).map(|_| scripts[0].clone()).collect()), "asked[1].cue: must be unique\n"),
        ("/cued", json!([{"cue":"x","turns":["x"],"extra":1}]), "cued[0].extra: unknown member\n"),
        ("/asked", json!([{"cue":"x","turns":[]}]), "asked[0].turns: must be non-empty\n"),
        ("/otherwise", json!([]), "otherwise: must be non-empty\n"),
        ("/otherwise", json!(["x", "x", "x", "x", "x", "x", "x", "x", "x"]), "otherwise: past its bound\n"),
        ("/otherwise", json!([{"lines":[],"finish":"stop","tokens":1}]), "otherwise[0].lines: must be non-empty\n"),
        (
            "/otherwise",
            json!([{"lines":[{"text":""}],"finish":"stop","tokens":1}]),
            "otherwise[0].lines[0].text: must be non-empty\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"call":"bad.name","arguments":{}}],"finish":"tool_calls","tokens":1}]),
            "otherwise[0].lines[0].call: expected ASCII letters, digits, _ or -\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"call":"x","arguments":[]}],"finish":"tool_calls","tokens":1}]),
            "otherwise[0].lines[0].arguments: expected a JSON object\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x","hold":"x"}],"finish":"stop","tokens":1}]),
            "otherwise[0].lines[0].hold: unknown member\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x"}],"finish":"no","tokens":1}]),
            "otherwise[0].finish: expected stop, tool_calls, length or content_filter\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x"}],"finish":"stop","tokens":0}]),
            "otherwise[0].tokens: must be at least 1\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x"}],"finish":"stop","tokens":1,"extra":0}]),
            "otherwise[0].extra: unknown member\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"fail":"x"}],"finish":"stop","tokens":1}]),
            "otherwise[0].lines[0].fail: unknown member\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"opaque":"x"}],"finish":"stop","tokens":1}]),
            "otherwise[0].lines[0].opaque: unknown member\n",
        ),
        ("/key", json!("/".to_owned() + &"x".repeat(4096)), "key: past its bound\n"),
        ("/tls", json!({"root":"/".to_owned()+&"x".repeat(4096)}), "tls.root: past its bound\n"),
        ("/key", Value::Null, "key: wrong value kind\n"),
        ("/models/0/output", json!(4_294_967_296_u64), "models[0].output: past its bound\n"),
        ("/models/0/window", json!(-1), "models[0].window: past its bound\n"),
        ("/asked", json!([{"cue":"x".repeat(257),"turns":["x"]}]), "asked[0].cue: past its bound\n"),
        ("/cued", json!([{"cue":"","turns":["x"]}]), "cued[0].cue: must be non-empty\n"),
        ("/cued", Value::Array((0..2).map(|_| scripts[0].clone()).collect()), "cued[1].cue: must be unique\n"),
        ("/cued", json!([{"cue":"x","turns":[]}]), "cued[0].turns: must be non-empty\n"),
        (
            "/asked",
            json!([{"cue":"x","turns":["a","b","c","d","e","f","g","h","i"]}]),
            "asked[0].turns: past its bound\n",
        ),
        (
            "/otherwise",
            json!([{"lines":(0..9).map(|_|json!({"text":"x"})).collect::<Vec<_>>(),"finish":"stop","tokens":1}]),
            "otherwise[0].lines: past its bound\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"call":"","arguments":{}}],"finish":"tool_calls","tokens":1}]),
            "otherwise[0].lines[0].call: must be non-empty\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"call":"x".repeat(65),"arguments":{}}],"finish":"tool_calls","tokens":1}]),
            "otherwise[0].lines[0].call: past its bound\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"call":"x"}],"finish":"tool_calls","tokens":1}]),
            "otherwise[0].lines[0].arguments: missing\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x","call":"x","arguments":{}}],"finish":"stop","tokens":1}]),
            "otherwise[0].lines[0]: expected text alone, or call and arguments\n",
        ),
        (
            "/otherwise",
            json!([{"lines":[{"text":"x"}],"finish":"stop","tokens":-1}]),
            "otherwise[0].tokens: past its bound\n",
        ),
        ("/otherwise", json!(["x".repeat(65_536)]), "otherwise[0]: answer past its byte bound\n"),
        (
            "/cued",
            Value::Array(
                (0..8)
                    .map(|i| json!({"cue":i.to_string(),"turns":(0..8).map(|_|"x".repeat(4_024)).collect::<Vec<_>>()}))
                    .collect(),
            ),
            "otherwise: scripts past their joint byte bound\n",
        ),
    ];
    for (pointer, value, expected) in rows {
        let mut doc = llm_document();
        *doc.pointer_mut(pointer).unwrap() = value;
        let refused = llm::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap_err();
        assert_eq!(&*refused.line(), expected.as_bytes(), "{pointer}");
    }
    for tls in [Value::Null, json!({"root":"/tmp/root"})] {
        let mut doc = llm_document();
        doc["tls"] = tls;
        doc["otherwise"] = json!([object]);
        llm::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    }
    let mut doc = llm_document();
    doc["extra"] = json!(1);
    assert_eq!(
        &*llm::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap_err().line(),
        b"extra: unknown member\n"
    );
    for (bytes, expected) in [
        (b"[]".to_vec(), b"document: wrong value kind\n".as_slice()),
        (b"{".to_vec(), b"document: invalid JSON\n"),
        (b"{}".to_vec(), b"listen: missing\n"),
        (alloc::vec![b' '; usize::try_from(super::DOCUMENT_BYTES).unwrap()+1], b"document: invalid JSON\n"),
        (br#"{"listen":{"socket":"127.0.0.1:0"},"key":"/tmp/first","key":"/tmp/second","tls":null,"models":[],"cued":[],"asked":[],"otherwise":["x"]}"#.to_vec(), b"key: duplicate\n"),
    ] { assert_eq!(llm::Document::decode(&bytes).unwrap_err().line().as_ref(), expected); }
    let mut doc = llm_document();
    let calls = json!({"lines":(0..8).map(|_|json!({"call":"x".repeat(64),"arguments":{}})).collect::<Vec<_>>(),"finish":"tool_calls","tokens":u64::MAX});
    let scripts = |prefix: &str| {
        (0..8).map(|i|json!({"cue":prefix.to_owned()+&i.to_string()+&"x".repeat(254),"turns":(0..8).map(|_|calls.clone()).collect::<Vec<_>>()})).collect::<Vec<_>>()
    };
    doc["models"] = json!(
        (0..8)
            .map(|i| json!({"name":"x".repeat(4095)+&i.to_string(),"window":u32::MAX,"output":u32::MAX}))
            .collect::<Vec<_>>()
    );
    doc["cued"] = json!(scripts("s"));
    doc["asked"] = json!(scripts("u"));
    doc["key"] = json!("/".to_owned() + &"x".repeat(4095));
    doc["tls"] = json!({"root":"/".to_owned()+&"x".repeat(4095)});
    llm::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table names every document bound and refusal path")]
fn forge_document_checks() {
    let rows = [
        ("/listen", json!({"socket":"0.0.0.0:0"}), "listen: must be a loopback socket address\n"),
        ("/admin_token", json!("relative"), "admin_token: must be an absolute path without NUL\n"),
        ("/owner", json!("temper-admin"), "owner: expected a Forgejo login other than temper-admin\n"),
        ("/owner", json!("bad/name"), "owner: expected a Forgejo login other than temper-admin\n"),
        ("/owner", json!("x".repeat(41)), "owner: past its bound\n"),
        ("/repositories", json!([]), "repositories: must be non-empty\n"),
        ("/repositories/0/name", json!("bad/name"), "repositories[0].name: expected a Forgejo repository name\n"),
        ("/repositories/0/name", json!("x".repeat(101)), "repositories[0].name: past its bound\n"),
        ("/repositories/0/landing", json!("bad..branch"), "repositories[0].landing: expected a branch name\n"),
        ("/repositories/0/landing", json!("x".repeat(257)), "repositories[0].landing: past its bound\n"),
        (
            "/repositories/0/ci",
            json!({"path":"../outside","green":"ok"}),
            "repositories[0].ci.path: expected a relative file path with no . or .. component\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"/outside","green":"ok"}),
            "repositories[0].ci.path: expected a relative file path with no . or .. component\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"README.md","green":"ok"}),
            "repositories[0].ci.path: must not name a seed file\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"src/main.rs","green":"ok"}),
            "repositories[0].ci.path: must not name a seed file\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":".forgejo/workflows/check.yml","green":"ok"}),
            "repositories[0].ci.path: must not name a seed file\n",
        ),
        ("/repositories/0/ci", json!({"path":"result","green":""}), "repositories[0].ci.green: must be non-empty\n"),
        (
            "/repositories/0/ci",
            json!({"path":"x".repeat(257),"green":"ok"}),
            "repositories[0].ci.path: past its bound\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"result","green":"x".repeat(8193)}),
            "repositories[0].ci.green: past its bound\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"result","green":"ok","extra":1}),
            "repositories[0].ci.extra: unknown member\n",
        ),
        (
            "/repositories/0",
            json!({"name":"widget","landing":"main","ci":null,"extra":1}),
            "repositories[0].extra: unknown member\n",
        ),
        ("/owner", json!(""), "owner: expected a Forgejo login other than temper-admin\n"),
        ("/admin_token", json!("/".to_owned() + &"x".repeat(4096)), "admin_token: past its bound\n"),
        ("/repositories/0/name", json!(""), "repositories[0].name: expected a Forgejo repository name\n"),
        ("/repositories/0/landing", json!(""), "repositories[0].landing: expected a branch name\n"),
        (
            "/repositories/0/ci",
            json!({"path":"","green":"ok"}),
            "repositories[0].ci.path: expected a relative file path with no . or .. component\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"ci/./result","green":"ok"}),
            "repositories[0].ci.path: expected a relative file path with no . or .. component\n",
        ),
        (
            "/repositories/0/ci",
            json!({"path":"ci/result","green":"ok","unknown":0}),
            "repositories[0].ci.unknown: unknown member\n",
        ),
    ];
    for (pointer, value, expected) in rows {
        let mut doc = forge_document();
        *doc.pointer_mut(pointer).unwrap() = value;
        assert_eq!(
            &*forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap_err().line(),
            expected.as_bytes(),
            "{pointer}"
        );
    }
    let repository = json!({"name":"widget","landing":"main","ci":null});
    for (count, expected) in
        [(2_usize, b"repositories[1].name: must be unique\n".as_slice()), (9, b"repositories: past its bound\n")]
    {
        let mut doc = forge_document();
        doc["repositories"] = Value::Array((0..count).map(|_| repository.clone()).collect());
        assert_eq!(&*forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap_err().line(), expected);
    }
    let mut doc = forge_document();
    doc["extra"] = json!(1);
    assert_eq!(
        &*forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap_err().line(),
        b"extra: unknown member\n"
    );
    forge::Document::decode(&serde_json::to_vec(&forge_document()).unwrap()).unwrap();
    for (bytes, expected) in [
        (b"[]".as_slice(), b"document: wrong value kind\n".as_slice()),
        (b"{", b"document: invalid JSON\n"),
        (b"{}", b"listen: missing\n"),
    ] {
        assert_eq!(forge::Document::decode(bytes).unwrap_err().line().as_ref(), expected);
    }
    let mut doc = forge_document();
    doc["owner"] = json!("x".repeat(40));
    doc["admin_token"] = json!("/".to_owned() + &"x".repeat(4095));
    doc["repositories"]=json!((0..8).map(|i|json!({"name":"x".repeat(99)+&i.to_string(),"landing":"x".repeat(256),"ci":{"path":"x".repeat(256),"green":"x".repeat(8192)}})).collect::<Vec<_>>());
    forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
}
#[test]
fn turns_spellings() {
    for (text, tokens) in [("", 0_u64), ("a", 1), ("abcd", 1), ("abcde", 2)] {
        let bytes = serde_json::to_vec(&text).unwrap();
        let d = super::read::document(&bytes).unwrap();
        let turn = turn::into_skein(turn::Turn::read(&d, 0, &Path::root()).unwrap());
        assert_eq!(
            turn,
            api::Turn {
                lines: Box::new([api::Line::Text { text: text.as_bytes().into() }]),
                finish: api::Finish::Stop,
                tokens
            }
        );
    }
    for (spelling, finish) in [
        ("stop", api::Finish::Stop),
        ("tool_calls", api::Finish::ToolCalls),
        ("length", api::Finish::Length),
        ("content_filter", api::Finish::ContentFilter),
    ] {
        let bytes = serde_json::to_vec(&json!({"lines":[{"text":"hello"},{"call":"tool_1-X","arguments":{"x":[1,true,null]}}],"finish":spelling,"tokens":99})).unwrap();
        let d = super::read::document(&bytes).unwrap();
        let turn = turn::into_skein(turn::Turn::read(&d, 0, &Path::root()).unwrap());
        assert_eq!(turn.finish, finish);
        assert_eq!(turn.tokens, 99);
        assert_eq!(
            turn.lines.as_ref(),
            &[
                api::Line::Text { text: b"hello".as_slice().into() },
                api::Line::Call {
                    name: b"tool_1-X".as_slice().into(),
                    arguments: br#"{"x":[1,true,null]}"#.as_slice().into()
                }
            ]
        );
    }
}
fn query(system: &[u8], request: &[u8]) -> api::Query {
    api::Query {
        credential: Some(0),
        model: b"qa".as_slice().into(),
        system: system.into(),
        messages: Box::new([api::Message {
            role: api::Role::User,
            parts: Box::new([api::Part::Text { text: request.into() }]),
        }]),
        tools: Box::new([]),
        choice: api::ToolChoice::None,
        max_tokens: 100,
        caching: api::Caching::Unscoped,
    }
}
fn answer(doc: &llm::Document, seed: u64, query: api::Query) -> api::Answer {
    let mut domain = doc.domain(seed);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: llm::LIMITS };
    let mut out = Queue::with_capacity(1);
    skein_fake_llm_domain::step(
        &mut domain,
        &env,
        skein_fake_llm_domain::Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: Token::new(2), query },
        &mut out,
    );
    if out.is_empty() {
        skein_fake_llm_domain::fire(&mut domain, &env, &mut out);
    }
    match out.pop().unwrap() {
        skein_fake_llm_domain::Request::Reply { result, .. } => result.unwrap(),
        skein_fake_llm_domain::Request::Piece { .. } | skein_fake_llm_domain::Request::Hold { .. } => {
            panic!("whole answer")
        }
    }
}
#[test]
fn llm_answers() {
    let mut doc = llm_document();
    doc["cued"] = json!([{"cue":"system cue","turns":["system answer"]}]);
    doc["asked"] = json!([{"cue":"asked cue","turns":["asked answer"]}]);
    let doc = llm::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    for (system, request, text) in
        [("system cue", "asked cue", "system answer"), ("", "asked cue", "asked answer"), ("", "other", "fallback")]
    {
        let answer = answer(&doc, 23, query(system.as_bytes(), request.as_bytes()));
        assert_eq!(answer.parts.as_ref(), &[api::Part::Text { text: text.as_bytes().into() }]);
    }
}
#[test]
fn seeded_credentials() {
    let doc = llm::Document::decode(&serde_json::to_vec(&llm_document()).unwrap()).unwrap();
    let mut previous = alloc::collections::BTreeSet::new();
    for seed in [0_u64, 23] {
        let before = answer(&doc, seed, query(b"", b"other"));
        for stream in [credential::KEY, credential::ADMIN_TOKEN] {
            let key = credential(seed, stream);
            assert_eq!(key, credential(seed, stream));
            assert!(key.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
            assert!(previous.insert(key), "streams and seeds differ");
        }
        assert_eq!(before, answer(&doc, seed, query(b"", b"other")));
    }
}
#[test]
fn forge_seed() {
    let mut doc = forge_document();
    doc["repositories"] =
        json!([{"name":"widget","landing":"main","ci":null},{"name":"second","landing":"trunk","ci":null}]);
    let doc = forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    for seed in [0_u64, 23, 0, 23] {
        let (mut forge, config) = doc.seed(seed);
        assert_eq!(config.users.as_slice(), &[(forge::OWNER, b"fake".as_slice().into())]);
        assert!(config.tokens.is_empty());
        for (i, repository) in config.repositories.iter().enumerate() {
            assert_eq!(repository.id, u32::try_from(i).unwrap() + 1);
            let landing = &doc.repositories()[i].landing;
            let head = forge.domain.branch(&repository.full, landing).unwrap();
            assert_eq!(head, u64::from(repository.id));
            let object = forge.domain.object(head).unwrap();
            assert_eq!(object.message.as_ref(), forge::MESSAGE);
            assert!(object.parent.is_none());
            assert!(object.merge_parent.is_none());
            for (path, content) in forge::FILES {
                assert_eq!(object.tree.get(path).unwrap().as_ref(), content);
            }
            assert_eq!(
                forge.domain.permission(&repository.full, 1).unwrap(),
                skein_fake_forge_domain::api::Permission::None
            );
            assert_eq!(
                forge.domain.permission(&repository.full, forge::OWNER).unwrap(),
                skein_fake_forge_domain::api::Permission::None
            );
        }
        assert!(forge.domain.pop_observation().is_none());
        assert_eq!(forge.domain.observations_lost(), 0);
    }
}
#[test]
fn ci_cue() {
    use skein_fake_forge_domain::{self as domain, api as a};
    for (cue, content, expected) in [
        (true, b"green".as_slice(), a::Check::Passed),
        (true, b"red", a::Check::Failed),
        (false, b"green", a::Check::Passed),
        (false, b"red", a::Check::Passed),
    ] {
        let mut doc = forge_document();
        if cue {
            doc["repositories"][0]["ci"] = json!({"path":"ci/result","green":"green"});
        }
        let doc = forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
        let (mut forge, _config) = doc.seed(23);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::LIMITS };
        let head =
            domain::advance(&mut forge.domain, &env, b"fake/widget", b"main", b"ci/result", content, forge::OWNER)
                .unwrap();
        let mut out = Queue::with_capacity(128);
        for _ in 0..128_u32 {
            if !forge.domain.is_due(Time::ZERO) {
                break;
            }
            domain::fire(&mut forge.domain, &env, &mut out);
            while out.pop().is_some() {}
            forge.domain.reclaim();
        }
        let statuses =
            forge.domain.inspect(&forge::LIMITS, b"fake/widget", &a::Read::Statuses { commit: head }).unwrap();
        let a::Answer::Statuses(statuses) = statuses else {
            panic!("statuses");
        };
        assert_eq!(statuses.len(), 1);
        assert_eq!(&*statuses[0].context, forge::CONTEXT);
        assert_eq!(statuses[0].state, expected);
    }
}

#[test]
#[expect(clippy::print_stderr, reason = "report the checked peers byte bounds for protocol/setup.md, 8.4")]
fn constants_fit() {
    let models = (0..8).map(|i| json!({"name":format!("qa-{i}"),"window":4096,"output":1024})).collect::<Vec<_>>();
    let scripts = |prefix: &str| {
        (0..8)
            .map(|i| json!({"cue":format!("{prefix}-{i}"),"turns":["a","b","c","d","e","f","g","h"]}))
            .collect::<Vec<_>>()
    };
    let bytes=serde_json::to_vec(&json!({"listen":{"socket":"127.0.0.1:0"},"key":"/tmp/key","tls":null,
            "models":models,"cued":scripts("system"),"asked":scripts("request"),"otherwise":["a","b","c","d","e","f","g","h"]})).expect("valid fixture");
    let document = llm::Document::decode(&bytes).expect("valid fixture");
    let key = credential(0, credential::KEY);
    let plain = crate::wire::llm(llm::configuration(&document, &key, 0)).expect("valid fixture");
    assert!(skein_fake_llm_domain::worst_case(&llm::LIMITS).is_some());
    let plain_bytes = plain.worst_case();
    let mut tls: Value = serde_json::from_slice(&bytes).expect("valid fixture");
    tls["tls"] = json!({"root":"/tmp/root"});
    let document = llm::Document::decode(&serde_json::to_vec(&tls).expect("valid fixture")).expect("valid fixture");
    let tls = crate::wire::llm(llm::configuration(&document, &key, 0)).expect("valid fixture");
    let document=forge::Document::decode(&serde_json::to_vec(&json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":"/tmp/admin_token","owner":"fake",
            "repositories":(0..8).map(|i|json!({"name":format!("repo-{i}"),"landing":"main","ci":null})).collect::<Vec<_>>()})).expect("valid fixture")).expect("valid fixture");
    let token = credential(0, credential::ADMIN_TOKEN);
    let (mut peer, _shared) = forge::peer(&document, &token, 0).expect("valid fixture");
    let setup = peer.setup_mut();
    let admin = setup.find(b"temper-admin").expect("valid fixture");
    assert!(admin.admin);
    assert_ne!(admin.id, forge::OWNER);
    assert_eq!(setup.issued().len(), 1);
    let issued = &setup.issued()[0];
    assert_eq!(issued.user, admin.id);
    assert_eq!(issued.secret.expose(), &token);
    assert_eq!(issued.scopes.as_ref(), &[Box::<[u8]>::from(b"all".as_slice())]);
    assert!(skein_fake_forge_protocol::git::fits(&crate::wire::forge_limits(), &forge::LIMITS.limits));
    assert!(skein_fake_forge_domain::worst_case(&forge::LIMITS.limits).is_some());
    eprintln!(
        "worst case: llm plaintext {plain_bytes}, llm TLS {}, forge {}, observation buffer {} bytes; operations llm {}, forge {}",
        tls.worst_case(),
        peer.worst_case(),
        crate::limits::OBSERVATION_BYTES,
        plain.operations(),
        peer.operations()
    );
    assert!(plain.observations().is_empty());
    assert!(peer.observations().is_empty());
}
