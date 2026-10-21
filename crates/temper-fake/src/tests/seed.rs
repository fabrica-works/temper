//! The same six acts for fresh, already seeded and refused deployments.
use crate::seed::{Act, Ended, Next, Plan};
use serde_json::json;
use skein_lib::{Duration, Time};
use temper_forge_setup::setup::{Fake, Repository};

const SIGN_IN: &[&str] = &["sign-in", "-"];
const ACCOUNTS: &[&str] = &["accounts", "--json"];
const ADD: &[&str] = &["add-key", "3", "/data/demo/llm.key", "--deployment", "--key", "demo-llm"];
const WAIT: &[&str] = &["wait", "accounts", "--until", "usable"];
const ADOPT: &[&str] = &[
    "adopt",
    "--project",
    "1",
    "--owner",
    "demo",
    "--name",
    "widget",
    "--landing",
    "main",
    "--key",
    "demo-adopt",
    "--json",
];
const SIGN_OUT: &[&str] = &["sign-out"];
const LINES: &[&str] = &[
    "temper fake seed: act 1: signed in\n",
    "temper fake seed: act 2: the deployment's key was added\n",
    "temper fake seed: act 3: accounts are usable\n",
    "temper fake seed: act 4: the demo's repository was adopted\n",
    "temper fake seed: act 5: signed out\n",
    "temper fake seed: act 6: the spent link was refused\n",
];
struct Step {
    words: &'static [&'static str],
    link: bool,
    status: Option<u8>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    line: Option<&'static str>,
}
fn fresh(accounts: serde_json::Value) -> Vec<Step> {
    [
        (SIGN_IN, true, Some(0), vec![], vec![], Some(LINES[0])),
        (ACCOUNTS, false, Some(0), serde_json::to_vec(&accounts).unwrap(), vec![], None),
        (ADD, false, Some(0), vec![], vec![], Some(LINES[1])),
        (WAIT, false, Some(0), vec![], vec![], Some(LINES[2])),
        (ADOPT, false, Some(0), br#"{"adopted":{"repository":1}}"#.to_vec(), vec![], Some(LINES[3])),
        (SIGN_OUT, false, Some(0), vec![], vec![], Some(LINES[4])),
        (SIGN_IN, true, Some(1), vec![], b"temper ctl: the sign-in link was refused\n".to_vec(), Some(LINES[5])),
    ]
    .into_iter()
    .map(|(words, link, status, stdout, stderr, line)| Step { words, link, status, stdout, stderr, line })
    .collect()
}
struct Row {
    name: String,
    steps: Vec<Step>,
    failed: Option<(Act, Vec<u8>)>,
}
#[test]
fn seed_calls_read_before_acting_and_present_spent_last() {
    let empty = json!({"accounts":{"party":[],"deployment":[]}});
    let mut rows = Vec::new();
    for (name, party, deployment, found) in [
        ("fresh", json!([]), json!([]), false),
        ("deployment", json!([]), json!([{"provider":3}]), true),
        ("party", json!([{"provider":3}]), json!([]), false),
        ("another provider", json!([]), json!([{"provider":4}]), false),
    ] {
        let mut steps = fresh(json!({"accounts":{"party":party,"deployment":deployment}}));
        if found {
            steps.remove(2);
            steps[1].line = Some("temper fake seed: act 2: the deployment's key was found\n");
        }
        rows.push(Row { name: name.into(), steps, failed: None });
    }
    for (index, act) in [
        (0, Act::SignIn),
        (1, Act::Key),
        (2, Act::Key),
        (3, Act::Usable),
        (4, Act::Adopt),
        (5, Act::SignOut),
        (6, Act::Spent),
    ] {
        let mut steps = fresh(empty.clone());
        steps.truncate(index + 1);
        steps[index].status = Some(1);
        steps[index].stderr = b"refused\nprivate second line".to_vec();
        let number = match act {
            Act::SignIn => 1,
            Act::Key => 2,
            Act::Usable => 3,
            Act::SignOut => 5,
            Act::Spent => 6,
            Act::Adopt => 4,
        };
        let why = match act {
            Act::Spent => "unexpected spent-link refusal",
            Act::Adopt => "unexpected adoption answer",
            Act::SignIn | Act::Key | Act::Usable | Act::SignOut => "call failed",
        };
        rows.push(Row {
            name: format!("call {index} failed"),
            steps,
            failed: Some((
                act,
                format!("temper fake seed: act {number}: {why} (status 1, stderr \"refused\")\n").into_bytes(),
            )),
        });
    }
    for (name, index, status, stdout, stderr, act, why) in [
        ("signal", 0, None, vec![], vec![], Act::SignIn, "temper fake seed: act 1: call failed (status signal, stderr \"\")\n".to_owned()),
        ("malformed", 1, Some(0), b"{".to_vec(), vec![], Act::Key, "temper fake seed: act 2: accounts document lacks accounts.deployment or a provider number, or is invalid JSON (status 0, stderr \"\")\n".to_owned()),
        ("missing deployment", 1, Some(0), b"{}".to_vec(), vec![], Act::Key, "temper fake seed: act 2: accounts document lacks accounts.deployment or a provider number, or is invalid JSON (status 0, stderr \"\")\n".to_owned()),
        ("missing provider", 1, Some(0), br#"{"accounts":{"deployment":[{}]}}"#.to_vec(), vec![], Act::Key, "temper fake seed: act 2: accounts document lacks accounts.deployment or a provider number, or is invalid JSON (status 0, stderr \"\")\n".to_owned()),
        ("wait not met", 3, Some(3), vec![], vec![], Act::Usable, "temper fake seed: act 3: call failed (status 3, stderr \"\")\n".to_owned()),
        ("accepted spent", 6, Some(0), vec![], vec![], Act::Spent, "temper fake seed: act 6: the spent link was accepted again (status 0, stderr \"\")\n".to_owned()),
        ("quote bound", 0, Some(1), vec![], vec![b'x'; 256], Act::SignIn, format!("temper fake seed: act 1: call failed (status 1, stderr \"{}\")\n", "x".repeat(256))),
        ("quote past bound", 0, Some(1), vec![], vec![b'x'; 257], Act::SignIn, format!("temper fake seed: act 1: call failed (status 1, stderr \"{}\")\n", "x".repeat(256))),
    ] {
        let mut steps = fresh(empty.clone());
        steps.truncate(index + 1);
        steps[index].status = status;
        steps[index].stdout = stdout;
        steps[index].stderr = stderr;
        rows.push(Row { name: name.into(), steps, failed: Some((act, why.into_bytes())) });
    }
    for row in rows {
        let mut plan = plan();
        let mut failure = None;
        for step in row.steps {
            let Next::Call(call) = plan.next() else { panic!("{}: call expected", row.name) };
            assert_eq!(
                call.words.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                step.words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
                "{}",
                row.name
            );
            assert_eq!(call.link, step.link, "{}", row.name);
            match plan.ended(&Ended { status: step.status, stdout: &step.stdout, stderr: &step.stderr }, Time::ZERO) {
                Ok(line) => assert_eq!(line.as_deref(), step.line.map(str::as_bytes), "{}", row.name),
                Err(why) => {
                    failure = Some((why.act, why.why.to_vec()));
                    break;
                }
            }
        }
        assert_eq!(failure, row.failed, "{}", row.name);
        if failure.is_none() {
            match plan.next() {
                Next::Done => {}
                Next::Call(_) | Next::Wait => panic!("{}: expected Done", row.name),
            }
        }
    }
}

fn plan() -> Plan {
    let demo = super::demo::read(&super::demo::fixture()).unwrap().unwrap();
    let fake = Fake {
        listen: "127.0.0.1:3000".parse().unwrap(),
        git: b"/data/demo/bin/git".as_slice().into(),
        owner: b"demo".as_slice().into(),
        repositories: Box::new([
            Repository { name: b"widget".as_slice().into(), landing: b"main".as_slice().into() },
            Repository { name: b"second".as_slice().into(), landing: b"trunk".as_slice().into() },
        ]),
    };
    Plan::new(&demo, &fake, Duration::from_secs(1))
}
fn adopting() -> Plan {
    let mut plan = plan();
    for stdout in [b"".as_slice(), br#"{"accounts":{"deployment":[{"provider":3}]}}"#, b""] {
        plan.ended(&Ended { status: Some(0), stdout, stderr: &[] }, Time::ZERO).unwrap();
    }
    assert_eq!(plan.act(), Act::Adopt);
    plan
}
fn key(plan: &Plan, expected: &str) {
    let Next::Call(call) = plan.next() else { panic!("adoption call") };
    let mut words: Vec<_> = ADOPT.iter().map(|word| word.as_bytes()).collect();
    words[10] = expected.as_bytes();
    assert_eq!(call.words.iter().map(AsRef::as_ref).collect::<Vec<_>>(), words);
    assert!(!call.link);
}
#[test]
fn adoption_walks_refused_keys_and_replays_to_the_adopting_key() {
    let refused = br#"{"forge_refused":{"reason":"unreachable"}}"#;
    let adopted = br#"{"adopted":{"forge":1,"repository":1}}"#;
    let mut saved = alloc::collections::BTreeMap::new();
    for replay in [false, true] {
        let mut plan = adopting();
        for (index, name, status, answer) in
            [(0, "demo-adopt", 2, refused.as_slice()), (1, "demo-adopt-2", 0, adopted.as_slice())]
        {
            key(&plan, name);
            let answer = if replay {
                let (saved_status, answer) = saved.get(name).unwrap();
                assert_eq!(*saved_status, status);
                serde_json::to_vec(&json!({"operation":"adopt","outcome":answer})).unwrap()
            } else {
                let answer: serde_json::Value = serde_json::from_slice(answer).unwrap();
                assert!(saved.insert(name, (status, answer.clone())).is_none());
                serde_json::to_vec(&answer).unwrap()
            };
            let now = Time::ZERO.saturating_add(Duration::from_millis(index * 250));
            let (stdout, stderr) =
                if status == 0 { (answer.as_slice(), b"".as_slice()) } else { (b"".as_slice(), answer.as_slice()) };
            let line = plan.ended(&Ended { status: Some(status), stdout, stderr }, now).unwrap();
            if status == 2 {
                assert!(line.is_none());
                match plan.next() {
                    Next::Wait => {}
                    Next::Call(_) | Next::Done => panic!("wait before the next key"),
                }
                assert_eq!(plan.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_millis(250))));
                plan.tick(Time::ZERO.saturating_add(Duration::from_millis(249))).unwrap();
                match plan.next() {
                    Next::Wait => {}
                    Next::Call(_) | Next::Done => panic!("wait before the next key"),
                }
                plan.tick(Time::ZERO.saturating_add(Duration::from_millis(250))).unwrap();
            } else {
                assert_eq!(line.as_deref(), Some(crate::seed::ADOPT_LINE));
                assert_eq!(plan.act(), Act::SignOut);
                assert_eq!(plan.next_deadline(), None);
            }
        }
        assert_eq!(saved.len(), 2);
    }
}
#[test]
fn adoption_rejects_every_other_answer() {
    for (status, stdout, stderr) in [
        (0, b"{}".as_slice(), b"".as_slice()),
        (0, b"{".as_slice(), b"".as_slice()),
        (0, br#"{"adopted":null}"#, b"".as_slice()),
        (0, br#"{"operation":"other","outcome":{"adopted":{}}}"#, b"".as_slice()),
        (2, b"".as_slice(), br#"{"forge_refused":{"reason":"no_access"}}"#),
        (2, b"".as_slice(), br#"{"core_refused":{}}"#),
        (2, br#"{"forge_refused":{"reason":"unreachable"}}"#, b"".as_slice()),
        (1, b"".as_slice(), b"refused\nprivate second line".as_slice()),
    ] {
        let failed = adopting().ended(&Ended { status: Some(status), stdout, stderr }, Time::ZERO).unwrap_err();
        assert_eq!(failed.act, Act::Adopt);
        assert!(failed.why.starts_with(b"temper fake seed: act 4: unexpected adoption answer"));
        assert!(!failed.why.windows(b"private".len()).any(|word| word == b"private"));
    }
}
#[test]
fn adoption_deadline_quotes_the_first_refusal_and_bounds_a_running_call() {
    for refusals in [false, true] {
        let mut plan = adopting();
        if refusals {
            for (attempt, answer) in [
                ("demo-adopt", b"{\"forge_refused\":{\"reason\":\"unreachable\"}}\n".as_slice()),
                ("demo-adopt-2", b"{ \"forge_refused\":{\"reason\":\"unreachable\"}}\n".as_slice()),
            ] {
                key(&plan, attempt);
                plan.ended(&Ended { status: Some(2), stdout: &[], stderr: answer }, Time::ZERO).unwrap();
                plan.tick(Time::ZERO.saturating_add(Duration::from_millis(250))).unwrap();
            }
            key(&plan, "demo-adopt-3");
        }
        plan.tick(Time::ZERO.saturating_add(Duration::from_millis(999))).unwrap();
        let failed = plan.tick(Time::ZERO.saturating_add(Duration::from_secs(1))).unwrap_err();
        assert_eq!(failed.act, Act::Adopt);
        let expected = if refusals {
            "temper fake seed: act 4: setup deadline elapsed (status 2, stderr \"{\"forge_refused\":{\"reason\":\"unreachable\"}}\")\n"
        } else {
            "temper fake seed: act 4: setup deadline elapsed (status signal, stderr \"\")\n"
        };
        assert_eq!(failed.why.as_ref(), expected.as_bytes());
        assert_eq!(plan.next_deadline(), None);
    }
}
