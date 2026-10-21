use super::fixtures::{self, DIRECTORIES, cases, content, parse, text};
use crate::{files, template};
use serde_json::{Value, json};
use skein_lib::{Stack, bytes};

fn set(value: &mut Value, path: &str, replacement: Value) {
    *value.pointer_mut(path).expect("the member exists in the owned example") = replacement;
}

fn github() -> Value {
    json!({"segments": ["forge", "github.com"], "last": "open", "value": ""})
}

fn replace_org_patterns(document: &mut Value) -> u32 {
    let org = json!({"segments": ["forge", "127.0.0.1:3000"], "last": "open", "value": "acme"});
    let mut pending = Stack::with_capacity(4096);
    pending.push(document).expect("the root fits");
    let mut replaced = 0_u32;
    for _ in 0..4096_u32 {
        let Some(value) = pending.pop() else { break };
        if *value == org {
            *value = github();
            replaced = replaced.checked_add(1).expect("the owned example's pattern count fits");
            continue;
        }
        match value {
            Value::Array(items) => {
                for item in items {
                    pending.push(item).expect("the owned example fits the traversal bound");
                }
            }
            Value::Object(members) => {
                for member in members.values_mut() {
                    pending.push(member).expect("the owned example fits the traversal bound");
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    assert!(pending.is_empty(), "every owned example member was visited");
    replaced
}

#[test]
fn the_owned_example_offers_openai_keys_at_the_public_api() {
    let owned = parse(fixtures::OWNED);
    assert_eq!(
        owned["engine"]["llm"]["providers"],
        json!([{
            "name": "openai", "number": 1_u32, "signed_in_plan": false,
            "models": ["gpt-6-luna"], "sources": {"keys": true}
        }])
    );
    let endpoint = &owned["engine"]["llm"]["endpoints"][0];
    assert_eq!(endpoint["name"], "openai");
    assert_eq!(endpoint["provider"], "openai");
    assert_eq!(endpoint["dialect"], 3_u32);
    assert_eq!(endpoint["address"], json!({"host": "api.openai.com", "port": 443_u16}));
    assert_eq!(endpoint["transport"], "tls");
    assert_eq!(endpoint["server_name"], "api.openai.com");
    assert_eq!(endpoint["headers"], parse(b"[]"));
    assert_eq!(owned["engine"]["llm"]["models"][0]["endpoint"], "openai");
    let agent = parse(fixtures::AGENT);
    let endpoint = &agent["endpoints"][0];
    assert_eq!(endpoint["name"], "openai");
    assert_eq!(endpoint["provider"], "openai");
    assert_eq!(endpoint["dialect"], 3_u32);
    assert_eq!(endpoint["address"], "api.openai.com:443");
    assert_eq!(endpoint["transport"], "tls");
    assert_eq!(endpoint["server_name"], "api.openai.com");
    assert_eq!(endpoint["headers"], parse(b"[]"));
}

#[test]
fn the_default_is_the_owned_example_so_edited() {
    let owned = parse(fixtures::OWNED);
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        let actual = parse(content(&answered, b"temper.json"));
        let mut expected = owned.clone();
        assert_eq!(replace_org_patterns(&mut expected), 40, "{row}");
        for (path, value) in [
            (
                "/engine/llm/providers",
                json!([
                    owned["engine"]["llm"]["providers"][0],
                    {"name": "demo", "number": 3_u64, "signed_in_plan": false,
                     "models": ["demo"], "sources": {"keys": true}}
                ]),
            ),
            (
                "/engine/llm/endpoints",
                json!([
                    owned["engine"]["llm"]["endpoints"][0],
                    {"name": "demo", "number": 3_u64, "dialect": 3_u64, "provider": "demo",
                     "address": {"socket": format!("127.0.0.1:{}", facts.ports.llm)},
                     "transport": "plaintext", "headers": parse(b"[]"), "connect_ms": 10_000_u64, "handshake_ms": 10_000_u64}
                ]),
            ),
            ("/engine/llm/models", json!([owned["engine"]["llm"]["models"][0], demo_model()])),
            ("/engine/charters/0/alternatives", json!(["demo"])),
            // Written by startup's owned_memory_figures_for_init drift check, at bounded strings.
            ("/engine/memory_bytes", json!(85_510_306_965_947_u64)),
            ("/workers/0/memory_bytes", json!(3_266_333_819_083_u64)),
            (
                "/application/forges",
                json!([{"number": 2_u16, "kind": "github", "api": null, "transport": null, "git": null, "web": null}]),
            ),
            ("/application/identities", json!([{"number": 1_u32, "forge": 2_u16, "login": null, "author": null}])),
            ("/application/setup", parse(b"[]")),
            ("/application/webhooks", Value::Null),
            ("/application/clients/listener/socket", json!(format!("127.0.0.1:{}", facts.ports.clients))),
            (
                "/application/clients/bundle",
                match facts.bundle {
                    Some(bundle) => json!(text(bundle)),
                    None => Value::Null,
                },
            ),
            (
                "/application/workers/0/git",
                match facts.git {
                    Some(git) => json!(text(git)),
                    None => Value::Null,
                },
            ),
            ("/application/landing/projects/0/rules/0/pattern", github()),
            ("/application/landing/projects/0/rules/0/gates", parse(b"[]")),
            ("/engine/charters/0/instructions", json!("charters/chat.txt")),
            ("/engine/charters/1/instructions", json!("charters/coding.txt")),
            ("/engine/hosts/workers/0/digest", json!("worker.digest")),
            ("/engine/deployment/directory", json!("deployment")),
            ("/workers/0/secret", json!("worker.secret")),
            ("/workers/0/agent/program", json!(text(facts.binary))),
            ("/workers/0/agent/configuration", json!("/c/temper/agent.json")),
            ("/workers/0/agent/arguments/1", json!("/c/temper/agent.json")),
        ] {
            set(&mut expected, path, value);
        }
        assert_eq!(actual, expected, "{row}");
    }
}

fn demo_model() -> Value {
    json!({"name": "demo", "endpoint": "demo", "window": 131_072_u64, "output": 8_192_u64,
           "prices": {"input": 1_u64, "cached": 1_u64, "output": 1_u64, "unit": 1_000_000_u64}})
}

fn demo_agent(port: u16) -> Value {
    json!({"name": "demo", "number": 3_u64, "dialect": 3_u64, "provider": "openai",
           "address": format!("127.0.0.1:{port}"), "transport": "plaintext",
           "identity": "plain", "connect": 10_000_u64, "handshake": 10_000_u64, "headers": parse(b"[]"),
           "models": [{"name": "demo", "window": 131_072_u64, "output": 8_192_u64,
                       "prices": {"input": 1_u64, "cached": 1_u64, "output": 1_u64, "unit": 1_000_000_u64},
                       "reasoning_item": 2_048_u64, "head": 60_000_u64, "idle": 30_000_u64}]})
}

#[test]
fn the_agent_configuration_is_the_owned_examples_but_its_programs() {
    let owned = parse(fixtures::AGENT);
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        let mut expected = owned.clone();
        let programs = match facts.claude {
            Some(claude) => json!({"claude_code": {"path": text(claude), "directory": "/d/temper/programs"}}),
            None => json!({}),
        };
        set(&mut expected, "/programs", programs);
        set(&mut expected, "/endpoints", json!([owned["endpoints"][0], demo_agent(facts.ports.llm.get())]));
        assert_eq!(parse(content(&answered, b"agent.json")), expected, "{row}");
    }
}

#[test]
fn facts_are_written_as_json_text() {
    for (row, binary, escaped) in [
        (
            "quote and backslash",
            b"/opt/tem\"per\\bin/temper".as_slice(),
            b"\"/opt/tem\\\"per\\\\bin/temper\"".as_slice(),
        ),
        (
            "control bytes and UTF-8",
            b"/opt/\n\t\x01/\xc3\xa9/temper".as_slice(),
            b"\"/opt/\\n\\t\\u0001/\xc3\xa9/temper\"".as_slice(),
        ),
    ] {
        let facts = crate::Facts { binary, ..fixtures::facts() };
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid text paths");
        let document = content(&answered, b"temper.json");
        assert!(bytes::find(document, escaped).is_some(), "{row}: JSON escaped the path");
        let parsed = parse(document);
        assert_eq!(parsed["engine"]["llm"]["endpoints"][1]["address"], json!({"socket": "127.0.0.1:8370"}), "{row}");
        assert_eq!(parse(content(&answered, b"agent.json"))["endpoints"][1], demo_agent(8370), "{row}");
        assert_eq!(
            parsed
                .pointer("/workers/0/agent/program")
                .expect("the worker has an agent program")
                .as_str()
                .expect("a JSON path is text")
                .as_bytes(),
            binary,
            "{row}"
        );
    }
}

#[test]
fn no_placeholder_is_left() {
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        for name in [b"temper.json".as_slice(), b"agent.json".as_slice()] {
            assert_eq!(bytes::find(content(&answered, name), b"\"$"), None, "{row}: {name:?}");
        }
    }
    for (row, source, expected) in [
        (
            "temper",
            template::TEMPER,
            [
                (b"\"$binary\"".as_slice(), 1_u32),
                (b"\"$agent\"".as_slice(), 2),
                (b"\"$git\"".as_slice(), 1),
                (b"\"$listener\"".as_slice(), 1),
                (b"\"$bundle\"".as_slice(), 1),
                (b"\"$llm\"".as_slice(), 1),
            ]
            .as_slice(),
        ),
        ("agent", template::AGENT, [(b"\"$programs\"".as_slice(), 1_u32), (b"\"$llm\"".as_slice(), 1)].as_slice()),
    ] {
        // Exact token counts catch missing, extra and misspelled placeholders.
        let mut count = 0_u32;
        for (token, wanted) in expected {
            assert_eq!(bytes::count(source, token, u32::MAX), *wanted, "{row}: {token:?}");
            count = count.checked_add(*wanted).expect("the table's token count fits");
        }
        assert_eq!(bytes::count(source, b"\"$", u32::MAX), count, "{row}: only known tokens");
        parse(source);
    }
}
