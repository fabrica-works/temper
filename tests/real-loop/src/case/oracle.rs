//! Reference-case.md, 4.1: effects over observations any world can supply.
use super::{
    A, A_REPORT, Agent, DESIGN, DESIGN_DOC, DESIGN_REPORT, DESIGN_WORDS, DIRECTOR, DIRECTOR_REPORT, DRAFT, DRAFT_WORDS,
    GREEN_LIGHT, OWNER_EFFORT, OWNER_MODEL, PROJECT, QUESTION, acts::Read, draft_commit, kit_tree,
};
use crate::{Exit, Ran};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Clone)]
pub struct Observed {
    /// ctl's JSON `task` object, by the act that read it; absent if that act failed.
    pub documents: BTreeMap<Read, Value>,
    /// Each LLM query in order, by its cue: its messages' texts and tool outputs.
    pub told: Vec<(Agent, Box<[u8]>)>,
    /// kit's main at its last merged head.
    pub kit: Kit,
}
#[derive(Clone)]
pub struct Kit {
    pub main: BTreeMap<Vec<u8>, Vec<u8>>,
    pub merged_green: bool,
}
impl Observed {
    #[must_use]
    pub fn from_ran(ran: &Ran, reads: &[(Read, usize)]) -> Self {
        let mut documents = BTreeMap::new();
        for (read, position) in reads {
            let Some(call) = ran.calls().get(*position) else { continue };
            if call.exit != Some(Exit::Code(0)) {
                continue;
            }
            if let Ok(mut document) = serde_json::from_slice::<Value>(&call.stdout)
                && let Some(task) = document.get_mut("task")
            {
                documents.insert(*read, task.take());
            }
        }
        let mut told = Vec::new();
        for observation in ran.llm() {
            let skein_fake_peers::llm::Observation::Query { query, .. } = observation else { continue };
            let Some(agent) = cued(&query.system) else { continue };
            let mut words = Vec::new();
            for message in &query.messages {
                for part in &message.parts {
                    use skein_fake_llm_domain::api::Part;
                    match part {
                        Part::Text { text } | Part::ToolOutput { output: text, .. } => {
                            words.extend_from_slice(text);
                            words.push(b'\n');
                        }
                        Part::Opaque { .. } | Part::ToolCall { .. } => {}
                    }
                }
            }
            told.push((agent, words.into_boxed_slice()));
        }
        Self {
            documents,
            told,
            kit: Kit { main: ran.forge().main().clone(), merged_green: ran.forge().merged_green() },
        }
    }
}
fn cued(system: &[u8]) -> Option<Agent> {
    let mut first_at = usize::MAX;
    let mut first = None;
    for agent in [Agent::Director, Agent::A, Agent::Design, Agent::Review] {
        for (at, word) in system.windows(agent.cue().len()).enumerate() {
            if word == agent.cue() {
                if at < first_at {
                    first_at = at;
                    first = Some(agent);
                }
                break;
            }
        }
    }
    first
}

pub struct Miss {
    pub step: &'static str,
    pub promise: Promise,
    pub what: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Promise {
    Must(&'static str),
    Add(&'static str),
}
impl fmt::Display for Miss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, words) = match self.promise {
            Promise::Must(words) => ("Must", words),
            Promise::Add(words) => ("Add", words),
        };
        write!(f, "{} {kind} \"{words}\": {}", self.step, self.what)
    }
}
const DELEGATE: Promise =
    Promise::Must("delegate a goal with instructions, input, priority, host kind, pool and models (exact)");
const AUTHORITY: Promise = Promise::Add("authority bounds A's subtree");
const SPEND: Promise = Promise::Add("spend rolls up to the goal");
const QUESTION_ANSWER: Promise = Promise::Must("a question and its answer (exact)");
const MARKDOWN: Promise = Promise::Must("the fast path for Markdown alone (exact)");
fn check(misses: &mut Vec<Miss>, met: bool, promise: Promise, what: String) {
    if !met {
        misses.push(Miss { step: "4.1", promise, what });
    }
}
fn text(value: &Value, path: &str) -> String {
    value.pointer(path).and_then(Value::as_str).unwrap_or_default().into()
}
fn number(value: &Value, path: &str) -> Option<u64> {
    value.pointer(path).and_then(Value::as_u64)
}
fn contains(words: &[u8], needle: &str) -> bool {
    crate::calls::occurrences(words, needle.as_bytes()) != 0
}

#[must_use]
pub fn step_4_1(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    let mut docs = Vec::new();
    for (read, expected) in [(Read::DirectorEnded, DIRECTOR), (Read::AEnded, A), (Read::DesignEnded, DESIGN)] {
        let document = observed.documents.get(&read).unwrap_or(&Value::Null);
        check(
            &mut misses,
            number(document, "/chip/task") == Some(expected),
            DELEGATE,
            format!("{read:?}: task number must be {expected}"),
        );
        check(
            &mut misses,
            number(document, "/project") == Some(PROJECT),
            DELEGATE,
            format!("{read:?}: project must be {PROJECT}"),
        );
        docs.push(document);
    }
    let [director, a, design] = docs.as_slice() else { unreachable!("three reads") };
    check(&mut misses, number(a, "/priority") == Some(1), DELEGATE, "A.priority must be 1".into());
    check(
        &mut misses,
        text(a, "/llm/main/model") == OWNER_MODEL,
        DELEGATE,
        format!("A.llm.main.model must be {OWNER_MODEL}"),
    );
    check(
        &mut misses,
        text(a, "/llm/main/effort") == OWNER_EFFORT,
        DELEGATE,
        format!("A.llm.main.effort must be {OWNER_EFFORT}"),
    );
    let spec = text(a, "/spec/specification/words/bytes");
    check(
        &mut misses,
        spec.contains(DRAFT) && spec.contains(&draft_commit()),
        DELEGATE,
        "A.spec must name the draft and its first commit".into(),
    );
    for path in ["/parent", "/requester/task"] {
        if let Some(value) = a.pointer(path) {
            check(&mut misses, value.as_u64() == Some(DIRECTOR), DELEGATE, format!("A{path} must be the director"));
        }
    }
    authority(&mut misses, director, a, "A");
    authority(&mut misses, a, design, "Design");
    rolled_up(&mut misses, a, design, "A", true);
    rolled_up(&mut misses, director, a, "Director", false);

    question_answer(&mut misses, observed, a, director);
    markdown(&mut misses, observed);
    misses
}

fn question_answer(misses: &mut Vec<Miss>, observed: &Observed, a: &Value, director: &Value) {
    let mut report_at = None;
    let mut acknowledged = false;
    let mut delivered = false;
    let sent = format!("Message {QUESTION} sent");
    for (position, (agent, words)) in observed.told.iter().enumerate() {
        if *agent == Agent::Director && contains(words, DESIGN_REPORT) && report_at.is_none() {
            report_at = Some(position);
        }
        if *agent == Agent::A {
            acknowledged |= contains(words, &sent);
            delivered |= contains(words, GREEN_LIGHT) && matches!(report_at, Some(at) if at < position);
        }
    }
    check(misses, report_at.is_some(), QUESTION_ANSWER, "Director was not told A's design report".into());
    check(misses, acknowledged, QUESTION_ANSWER, format!("A was not told {sent}"));
    check(
        misses,
        delivered,
        QUESTION_ANSWER,
        "A was not told the green light after the director heard its question".into(),
    );
    for (document, expected, label) in [(a, A_REPORT, "A"), (director, DIRECTOR_REPORT, "Director")] {
        check(
            misses,
            text(document, "/result/value/report/bytes") == expected,
            QUESTION_ANSWER,
            format!("{label}.result must be {expected:?}"),
        );
    }
}

fn markdown(misses: &mut Vec<Miss>, observed: &Observed) {
    // The fast path itself belongs to the forge's CI and is left out here
    // (domain/testing.md, 5.3's 4.1 row). We check the merge and its Markdown.
    check(misses, observed.kit.merged_green, MARKDOWN, "kit.main must have merged with green CI at its head".into());
    check(
        misses,
        observed.kit.main.get(DESIGN_DOC.as_bytes()).map(Vec::as_slice) == Some(DESIGN_WORDS.as_bytes()),
        MARKDOWN,
        "kit.main must hold A's design words".into(),
    );
    check(
        misses,
        observed.kit.main.get(DRAFT.as_bytes()).map(Vec::as_slice) == Some(DRAFT_WORDS.as_bytes()),
        MARKDOWN,
        "kit's draft must be unchanged".into(),
    );
    let mut before = BTreeMap::new();
    for file in kit_tree() {
        before.insert(file.path.to_vec(), file.content.to_vec());
    }
    let mut paths = BTreeSet::new();
    paths.extend(before.keys());
    paths.extend(observed.kit.main.keys());
    for path in paths {
        if before.get(path) != observed.kit.main.get(path) {
            check(
                misses,
                path.ends_with(b".md"),
                MARKDOWN,
                format!("changed path {:?} is not Markdown", String::from_utf8_lossy(path)),
            );
        }
    }
}

fn rolled_up(misses: &mut Vec<Miss>, parent: &Value, child: &Value, label: &str, positive: bool) {
    let below = number(parent, "/budget/numbers/spent_below");
    let spent = number(child, "/budget/numbers/spent");
    let child_below = number(child, "/budget/numbers/spent_below");
    let sum = match (spent, child_below) {
        (Some(spent), Some(below)) => spent.checked_add(below),
        _ => None,
    };
    check(
        misses,
        at_most(sum, below),
        SPEND,
        format!("{label}.budget.spent_below must cover its child's spent + spent_below"),
    );
    if positive {
        check(
            misses,
            matches!(below, Some(below) if below > 0),
            SPEND,
            format!("{label}.budget.spent_below must be positive"),
        );
    }
}
fn authority(misses: &mut Vec<Miss>, parent: &Value, child: &Value, label: &str) {
    let parent = &parent["authority"]["authority"];
    let child = &child["authority"]["authority"];
    for path in ["/spend", "/delegation/tasks", "/delegation/depth"] {
        check(
            misses,
            at_most(number(child, path), number(parent, path)),
            AUTHORITY,
            format!("{label}.authority{path} exceeds its parent or is absent"),
        );
    }
    check(
        misses,
        matches!((number(child, "/tools"), number(parent, "/tools")), (Some(child), Some(parent)) if child & !parent == 0),
        AUTHORITY,
        format!("{label}.authority.tools must be a subset"),
    );
    match (parent["delegation"]["kinds"].as_array(), child["delegation"]["kinds"].as_array()) {
        (Some(parents), Some(children)) => {
            for kind in children {
                check(
                    misses,
                    parents.contains(kind),
                    AUTHORITY,
                    format!("{label}.authority.delegation.kinds: {kind} is not covered"),
                );
            }
        }
        _ => check(misses, false, AUTHORITY, format!("{label}.authority.delegation.kinds is absent")),
    }
    match (parent["grants"].as_array(), child["grants"].as_array()) {
        (Some(parents), Some(children)) => {
            for grant in children {
                let mut covered = false;
                for parent in parents {
                    covered |= covers(parent, grant);
                }
                check(misses, covered, AUTHORITY, format!("{label}.authority.grants: {grant} is not covered"));
            }
        }
        _ => check(misses, false, AUTHORITY, format!("{label}.authority.grants is absent")),
    }
}
fn covers(parent: &Value, child: &Value) -> bool {
    if parent["connector"].as_u64().is_none()
        || parent["connector"] != child["connector"]
        || parent["kind"].as_u64().is_none()
        || parent["kind"] != child["kind"]
    {
        return false;
    }
    let (Some(parents), Some(children)) =
        (parent["pattern"]["segments"].as_array(), child["pattern"]["segments"].as_array())
    else {
        return false;
    };
    if !children.starts_with(parents) {
        return false;
    }
    let terminal = &parent["pattern"]["last"];
    if let Some(exact) = terminal.get("exact").and_then(Value::as_str) {
        return parents == children && child["pattern"]["last"]["exact"].as_str() == Some(exact);
    }
    let Some(prefix) = terminal.get("open").and_then(Value::as_str) else { return false };
    // Segments and terminal bytes are lower-case hex in ctl's JSON. Hex
    // preserves byte prefixes, so compare at whole-byte boundaries.
    let candidate = if let Some(segment) = children.get(parents.len()) {
        segment.as_str()
    } else {
        match child["pattern"]["last"].get("exact") {
            Some(exact) => exact.as_str(),
            None => child["pattern"]["last"]["open"].as_str(),
        }
    };
    matches!(candidate, Some(candidate) if prefix.len().is_multiple_of(2) && candidate.starts_with(prefix))
}
fn at_most(child: Option<u64>, parent: Option<u64>) -> bool {
    matches!((child, parent), (Some(child), Some(parent)) if child <= parent)
}

pub fn assert_met(misses: &[Miss]) {
    let mut report = String::new();
    for miss in misses {
        use fmt::Write;
        writeln!(report, "{miss}").expect("miss report");
    }
    assert!(misses.is_empty(), "{report}");
}

#[cfg(test)]
mod tests {
    use super::{
        A, A_REPORT, Agent, BTreeMap, DELEGATE, DESIGN, DESIGN_DOC, DESIGN_REPORT, DESIGN_WORDS, DIRECTOR,
        DIRECTOR_REPORT, GREEN_LIGHT, Kit, MARKDOWN, OWNER_EFFORT, OWNER_MODEL, Observed, PROJECT, QUESTION,
        QUESTION_ANSWER, Read, Value, assert_met, contains, draft_commit, kit_tree, step_4_1,
    };
    use crate::case::{A_SPEC_PREFIX, A_SPEC_SUFFIX};
    use serde_json::json;

    fn met() -> Observed {
        let mut documents = BTreeMap::new();
        for (read, task, tools, spend, tasks, depth, spent, below, report) in [
            (Read::DirectorEnded, DIRECTOR, 3, 40, 7, 4, 2, 5, DIRECTOR_REPORT),
            (Read::AEnded, A, 3, 20, 6, 3, 2, 3, A_REPORT),
            (Read::DesignEnded, DESIGN, 1, 10, 4, 2, 1, 2, ""),
        ] {
            documents.insert(read, json!({
                "chip":{"task":task}, "project":PROJECT, "requester":{"task":DIRECTOR},
                "priority":1, "llm":{"main":{"model":OWNER_MODEL,"effort":OWNER_EFFORT}},
                "spec":{"specification":{"words":{"bytes":format!("{A_SPEC_PREFIX}{}{A_SPEC_SUFFIX}", draft_commit())}}},
                "authority":{"authority":{"tools":tools,"spend":spend,
                    "grants":[{"connector":1,"kind":2,"pattern":{"segments":["666f726765"],"last":{"open":"61636d65"}}}],
                    "delegation":{"kinds":[{"charter":1},{"procedure":2}],"tasks":tasks,"depth":depth}}},
                "budget":{"numbers":{"spent":spent,"spent_below":below}},
                "result":{"value":{"report":{"bytes":report}}}
            }));
        }
        let mut main = BTreeMap::new();
        for file in kit_tree() {
            main.insert(file.path.to_vec(), file.content.to_vec());
        }
        main.insert(DESIGN_DOC.as_bytes().to_vec(), DESIGN_WORDS.as_bytes().to_vec());
        Observed {
            documents,
            told: vec![
                (Agent::A, format!("Message {QUESTION} sent").into_bytes().into_boxed_slice()),
                (Agent::Director, DESIGN_REPORT.as_bytes().into()),
                (Agent::A, GREEN_LIGHT.as_bytes().into()),
            ],
            kit: Kit { main, merged_green: true },
        }
    }

    #[test]
    fn a_met_checkpoint_has_no_misses() {
        assert_met(&step_4_1(&met()));
    }

    #[test]
    fn the_checkpoint_reports_every_independent_miss_together() {
        // Three independent corruptions of otherwise sufficient observations.
        const MISSES: usize = 3;
        const GREEN_LIGHT_QUERIES: usize = 1;
        let mut broken = met().clone();
        broken.documents.get_mut(&Read::AEnded).expect("A document")["priority"] = Value::Null;
        broken.kit.main.insert(b"src/export.rs".to_vec(), b"fn export() {}\n".to_vec());
        let mut removed = 0;
        for (agent, words) in &mut broken.told {
            if *agent == Agent::A && contains(words, GREEN_LIGHT) {
                *words = Box::new([]);
                removed += 1;
            }
        }
        assert_eq!(removed, GREEN_LIGHT_QUERIES);
        let misses = step_4_1(&broken);
        assert_eq!(misses.len(), MISSES);
        let [priority, answer, markdown] = misses.as_slice() else { panic!("three misses") };
        assert_eq!(priority.promise, DELEGATE);
        assert!(priority.what.contains("A.priority"));
        assert_eq!(answer.promise, QUESTION_ANSWER);
        assert!(answer.what.contains("green light"));
        assert_eq!(markdown.promise, MARKDOWN);
        assert!(markdown.what.contains("src/export.rs"));
    }
}
