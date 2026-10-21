use super::fixtures::{self, DIRECTORIES, cases, file, parse, text};
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
fn the_default_is_the_owned_example_so_edited() {
    let owned = parse(fixtures::OWNED);
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid facts");
        let actual = parse(&file(&answered, b"temper.json").bytes);
        let mut expected = owned.clone();
        assert_eq!(replace_org_patterns(&mut expected), 40, "{row}");
        for (path, value) in [
            (
                "/application/forges",
                json!([{"number": 2_u16, "kind": "github", "api": null, "transport": null, "git": null, "web": null}]),
            ),
            ("/application/identities", json!([{"number": 1_u32, "forge": 2_u16, "login": null, "author": null}])),
            ("/application/setup", parse(b"[]")),
            ("/application/webhooks", Value::Null),
            (
                "/application/clients/listener/socket",
                json!(match facts.port.get() {
                    8367 => "127.0.0.1:8367",
                    41023 => "127.0.0.1:41023",
                    _ => unreachable!("the table's ports"),
                }),
            ),
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

#[test]
fn the_agent_configuration_is_the_owned_examples_but_its_programs() {
    let owned = parse(fixtures::AGENT);
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid facts");
        let mut expected = owned.clone();
        let programs = match facts.claude {
            Some(claude) => json!({"claude_code": {"path": text(claude), "directory": "/d/temper/programs"}}),
            None => json!({}),
        };
        set(&mut expected, "/programs", programs);
        assert_eq!(parse(&file(&answered, b"agent.json").bytes), expected, "{row}");
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
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid text paths");
        let document = &file(&answered, b"temper.json").bytes;
        assert!(bytes::find(document, escaped).is_some(), "{row}: JSON escaped the path");
        let parsed = parse(document);
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
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid facts");
        for name in [b"temper.json".as_slice(), b"agent.json".as_slice()] {
            assert_eq!(bytes::find(&file(&answered, name).bytes, b"\"$"), None, "{row}: {name:?}");
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
            ]
            .as_slice(),
        ),
        ("agent", template::AGENT, [(b"\"$programs\"".as_slice(), 1_u32)].as_slice()),
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
