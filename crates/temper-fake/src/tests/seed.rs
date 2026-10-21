//! The same six acts for fresh, already seeded and refused deployments.
use crate::seed::{Act, Ended, Next, Plan};
use serde_json::json;

const SIGN_IN: &[&str] = &["sign-in", "-"];
const ACCOUNTS: &[&str] = &["accounts", "--json"];
const ADD: &[&str] = &["add-key", "3", "/data/demo/llm.key", "--deployment", "--key", "demo-llm"];
const WAIT: &[&str] = &["wait", "accounts", "--until", "usable"];
const SIGN_OUT: &[&str] = &["sign-out"];
const LINES: &[&str] = &[
    "temper fake seed: act 1: signed in\n",
    "temper fake seed: act 2: the deployment's key was added\n",
    "temper fake seed: act 3: accounts are usable\ntemper fake seed: act 4: the demo's repository is not adopted and the forges are not waited on; both wait on temper-14\n",
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
        (SIGN_OUT, false, Some(0), vec![], vec![], Some(LINES[3])),
        (SIGN_IN, true, Some(1), vec![], b"temper ctl: the sign-in link was refused\n".to_vec(), Some(LINES[4])),
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
    for (index, act) in
        [(0, Act::SignIn), (1, Act::Key), (2, Act::Key), (3, Act::Usable), (4, Act::SignOut), (5, Act::Spent)]
    {
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
            Act::Adopt => unreachable!(),
        };
        let why = if act == Act::Spent { "unexpected spent-link refusal" } else { "call failed" };
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
        ("accepted spent", 5, Some(0), vec![], vec![], Act::Spent, "temper fake seed: act 6: the spent link was accepted again (status 0, stderr \"\")\n".to_owned()),
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
        let demo = super::demo::read(&super::demo::fixture()).unwrap().unwrap();
        let mut plan = Plan::new(&demo);
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
            match plan.ended(&Ended { status: step.status, stdout: &step.stdout, stderr: &step.stderr }) {
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
                Next::Call(_) => panic!("{}: expected Done", row.name),
            }
        }
    }
}
