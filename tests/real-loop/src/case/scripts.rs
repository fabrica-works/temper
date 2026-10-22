//! Every judgement of reference-case.md, 4.1, played word for word.
use super::{
    A, A_REPORT, A_SPEC_PREFIX, A_SPEC_SUFFIX, ANSWERED, ASKED, Agent, DESIGN_DELEGATED, DESIGN_DOC, DESIGN_LINE,
    DESIGN_REPORT, DESIGN_SPEC, DESIGN_TITLE, DESIGN_WORDS, DIRECTOR, DIRECTOR_REPORT, GREEN_LIGHT,
    OWNER_EFFORT_NUMBER, OWNER_MODEL_NUMBER, QUESTION, REVIEWED, SCHEDULED, draft_commit,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use std::net::SocketAddr;

fn call(name: &[u8], value: &Value) -> Turn {
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(value).expect("tool JSON").into_boxed_slice(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}
fn wait(words: &str) -> Turn {
    Turn {
        lines: Box::new([
            Line::Text { text: words.as_bytes().into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}
fn report(words: &str) -> Turn {
    call(b"finish", &json!({"form":"report","text":words,"fields":{}}))
}
fn grants(forge: SocketAddr, kinds: &[u8]) -> Value {
    let mut grants = Vec::new();
    for kind in kinds {
        grants.push(
            json!({"connector":1,"kind":kind,"segments":["forge",forge.to_string()],"terminal":"open","last":"acme"}),
        );
    }
    Value::Array(grants)
}

#[must_use]
pub fn step_4_1(forge: SocketAddr) -> Box<[Script]> {
    let owner = json!({"batch":[{
        "executor":{"kind":"agent","charter":1},"tracked":{"priority":1},
        "spec":{"words":format!("{A_SPEC_PREFIX}{}{A_SPEC_SUFFIX}", draft_commit()),"parameters":[]},
        "contract":{"kind":"report","words":128},
        "llm":{"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}},
        "authority":{"tools":3,"grants":grants(forge, &[2,3,4,7,8,10,11,12,13]),
            "delegation":{"kinds":[{"kind":"procedure","number":2},{"kind":"agent","number":1}],"tasks":6,"depth":3},
            "budget":{"spend":20},"notes":0}
    }]});
    let design = json!({"batch":[{
        "executor":{"kind":"procedure","connector":1,"code":2},
        "spec":{"words":DESIGN_SPEC,"parameters":[
            {"name":1,"kind":"resource","connector":1,"resource":crate::forge::ID},
            {"name":2,"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":1,"change_kind":1,"words":128},
        "authority":{"tools":1,"grants":grants(forge, &[2,3,4,10,11,12]),
            "delegation":{"kinds":[{"kind":"agent","number":1}],"tasks":4,"depth":2},
            "budget":{"spend":10},"notes":0}
    }]});
    Box::new([
        Script {
            cue: Agent::Director.cue().into(),
            turns: Box::new([
                call(b"delegate", &owner),
                wait(SCHEDULED),
                call(b"message", &json!({"target":A,"form":"answer","question":QUESTION,"words":GREEN_LIGHT})),
                wait(ANSWERED),
                report(DIRECTOR_REPORT),
            ]),
        },
        Script {
            cue: Agent::A.cue().into(),
            turns: Box::new([
                call(b"delegate", &design),
                wait(DESIGN_DELEGATED),
                call(b"message", &json!({"target":DIRECTOR,"form":"question","words":DESIGN_REPORT})),
                wait(ASKED),
                report(A_REPORT),
            ]),
        },
        Script {
            cue: Agent::Design.cue().into(),
            turns: Box::new([
                call(b"write", &json!({"path":DESIGN_DOC,"content":DESIGN_WORDS})),
                call(b"finish", &json!({"form":"change","fields":{"title":DESIGN_TITLE,"body":DESIGN_LINE}})),
            ]),
        },
        Script {
            cue: Agent::Review.cue().into(),
            turns: Box::new([call(
                b"finish",
                &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}),
            )]),
        },
    ])
}
