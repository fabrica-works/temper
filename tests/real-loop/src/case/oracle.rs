//! Reference-case checkpoints: effects over observations any world can supply.
use super::{
    A, A_REPORT, Agent, DESIGN, DESIGN_DOC, DESIGN_REPORT, DESIGN_WORDS, DIRECTOR, DIRECTOR_REPORT, DRAFT, DRAFT_WORDS,
    GREEN_LIGHT, OWNER_EFFORT, OWNER_MODEL, PROJECT, QUESTION, acts::Read, draft_commit, kit_tree,
};
use crate::{Exit, Ran, calls};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Clone)]
pub struct Observed {
    /// ctl's whole JSON document, by the act that read it; absent if that act failed.
    pub documents: BTreeMap<Read, Value>,
    /// Each LLM query in order, by its cue: its messages' texts and tool outputs.
    pub told: Vec<Told>,
    /// Command verbs of the submitted ctl argv (reads and waits included).
    pub commands: Vec<String>,
    /// kit's main after its last observed merge or the person's push.
    pub kit: Kit,
}
#[derive(Clone)]
pub struct Told {
    /// Ordered query messages, preserving roles and tool-call identities across a retry.
    pub history: Box<[skein_fake_llm_domain::api::Message]>,
    pub agent: Agent,
    /// Repository conventions and the brief, separate from conversation messages.
    pub system: Box<[u8]>,
    pub words: Box<[u8]>,
    pub model: Box<[u8]>,
    pub credential: Option<u32>,
}
#[derive(Clone)]
pub struct Kit {
    pub main: BTreeMap<Vec<u8>, Vec<u8>>,
    pub merged_green: bool,
}
impl Told {
    /// The oracle's per-query view, shared with process-tier observation readers.
    #[must_use]
    pub fn new(
        system: &[u8],
        model: &[u8],
        credential: Option<u32>,
        messages: &[skein_fake_llm_domain::api::Message],
    ) -> Option<Self> {
        let agent = cued(system)?;
        let mut words = system.to_vec();
        words.push(b'\n');
        for message in messages {
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
        Some(Self {
            system: system.into(),
            history: messages.into(),
            agent,
            words: words.into_boxed_slice(),
            model: model.into(),
            credential,
        })
    }
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
            if let Ok(document) = serde_json::from_slice::<Value>(&call.stdout) {
                documents.insert(*read, document);
            }
        }
        let mut told = Vec::new();
        for observation in ran.llm() {
            let skein_fake_peers::llm::Observation::Query { query, .. } = observation else { continue };
            if let Some(query) = Told::new(&query.system, &query.model, query.credential, &query.messages) {
                told.push(query);
            }
        }
        Self {
            documents,
            commands: Vec::new(),
            told,
            kit: Kit { main: ran.forge().main().clone(), merged_green: ran.forge().merged_green() },
        }
    }
}
impl Observed {
    /// The command verbs of the story's ctl argv, in order, for checks on which acts it made.
    #[must_use]
    pub fn with_commands(mut self, steps: &[calls::Step]) -> Self {
        for step in steps {
            if let calls::Step::Call { argv, .. } | calls::Step::Start { argv, .. } | calls::Step::Until { argv, .. } =
                step
                && let Some(calls::Arg::Lit(command)) = argv.0.first()
            {
                self.commands.push((*command).into());
            }
        }
        self
    }
}
fn cued(system: &[u8]) -> Option<Agent> {
    let mut first_at = usize::MAX;
    let mut first = None;
    for agent in [
        Agent::Director,
        Agent::A,
        Agent::Design,
        Agent::Review,
        Agent::Survey,
        Agent::Check,
        Agent::B,
        Agent::Benchmark,
        Agent::Hang,
        Agent::Phase,
        Agent::Rules,
        Agent::Scope,
        Agent::Pause,
        Agent::Owner,
        Agent::Bowner,
        Agent::Helper,
        Agent::Implementer,
        Agent::H1Coder,
        Agent::Shape,
        Agent::Mover,
        Agent::Plan,
        Agent::Planner,
        Agent::Subowner,
    ] {
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
    Step(&'static str),
}
impl fmt::Display for Miss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, words) = match self.promise {
            Promise::Must(words) => ("Must", words),
            Promise::Add(words) => ("Add", words),
            Promise::Step(words) => ("Step", words),
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
        let document = &observed.documents.get(&read).unwrap_or(&Value::Null)["task"];
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
    for (position, Told { agent, words, .. }) in observed.told.iter().enumerate() {
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
        QUESTION_ANSWER, Read, Told, Value, assert_met, contains, draft_commit, kit_tree, step_4_1,
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
            documents.insert(read, json!({"task":{
                "chip":{"task":task}, "project":PROJECT, "requester":{"task":DIRECTOR},
                "priority":1, "llm":{"main":{"model":OWNER_MODEL,"effort":OWNER_EFFORT}},
                "spec":{"specification":{"words":{"bytes":format!("{A_SPEC_PREFIX}{}{A_SPEC_SUFFIX}", draft_commit())}}},
                "authority":{"authority":{"tools":tools,"spend":spend,
                    "grants":[{"connector":1,"kind":2,"pattern":{"segments":["666f726765"],"last":{"open":"61636d65"}}}],
                    "delegation":{"kinds":[{"charter":1},{"procedure":2}],"tasks":tasks,"depth":depth}}},
                "budget":{"numbers":{"spent":spent,"spent_below":below}},
                "result":{"value":{"report":{"bytes":report}}}
            }}));
        }
        let mut main = BTreeMap::new();
        for file in kit_tree() {
            main.insert(file.path.to_vec(), file.content.to_vec());
        }
        main.insert(DESIGN_DOC.as_bytes().to_vec(), DESIGN_WORDS.as_bytes().to_vec());
        Observed {
            documents,
            commands: Vec::new(),
            told: vec![
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::A,
                    words: format!("Message {QUESTION} sent").into_bytes().into_boxed_slice(),
                    model: Box::new([]),
                    credential: Some(0),
                },
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::Director,
                    words: DESIGN_REPORT.as_bytes().into(),
                    model: Box::new([]),
                    credential: Some(0),
                },
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::A,
                    words: GREEN_LIGHT.as_bytes().into(),
                    model: Box::new([]),
                    credential: Some(0),
                },
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
        broken.documents.get_mut(&Read::AEnded).expect("A document")["task"]["priority"] = Value::Null;
        broken.kit.main.insert(b"src/export.rs".to_vec(), b"fn export() {}\n".to_vec());
        let mut removed = 0;
        for Told { agent, words, .. } in &mut broken.told {
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

const ACCOUNT_WAIT: Promise =
    Promise::Must("an account's states hold the work they pay for, and name the act they wait for (exact)");
const SURVIVES: Promise = Promise::Add("no session dies, so nothing has to be recreated afterwards");

fn step_check(misses: &mut Vec<Miss>, step: &'static str, met: bool, promise: Promise, what: &str) {
    if !met {
        misses.push(Miss { step, promise, what: what.into() });
    }
}
fn document(observed: &Observed, read: Read) -> &Value {
    observed.documents.get(&read).unwrap_or(&Value::Null)
}

fn task_document(observed: &Observed, read: Read) -> &Value {
    let value = document(observed, read);
    value.get("task").filter(|task| task.is_object()).unwrap_or(value)
}

#[must_use]
pub fn step_4_5_refused(observed: &Observed) -> Vec<Miss> {
    use super::{FIRST_ACCOUNT, HELD_A_SPEC};
    let mut misses = Vec::new();
    let waiting = document(observed, Read::AWaiting);
    let accounts = document(observed, Read::Refused);
    for (met, promise, what) in [
        (
            number(waiting, "/chip/task") == Some(A)
                && number(waiting, "/run_wait/account/account") == Some(FIRST_ACCOUNT)
                && text(waiting, "/run_wait/account/why") == "refused",
            ACCOUNT_WAIT,
            "A's waiting task must name its account and refused",
        ),
        (
            accounts.pointer("/accounts/deployment").and_then(Value::as_array).is_some_and(|rows| {
                rows.iter().any(|row| {
                    number(row, "/number") == Some(FIRST_ACCOUNT) && text(row, "/state/kind") == "sign_in_again"
                })
            }),
            ACCOUNT_WAIT,
            "the refused account must need sign_in_again",
        ),
    ] {
        step_check(&mut misses, "4.5", met, promise, what);
    }
    account_survives(&mut misses, observed);
    let resumed = observed.told.iter().position(|told| told.agent == Agent::A && told.credential == Some(1));
    step_check(
        &mut misses,
        "4.5",
        resumed.is_some_and(|at| {
            let told = &observed.told[at];
            contains(&told.words, HELD_A_SPEC)
                && observed.told[..at].iter().all(|told| told.credential == Some(0))
                && observed.told[..at].iter().any(|told| told.agent == Agent::A && contains(&told.words, HELD_A_SPEC))
        }),
        SURVIVES,
        "A's resumed query must retain its spec on credential 1, after queries on credential 0",
    );
    misses
}

fn account_survives(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{FIRST_ACCOUNT, RESUMED_REPORT};
    let waiting = document(observed, Read::AWaiting);
    let a = &document(observed, Read::AEnded)["task"];
    let director = task_document(observed, Read::DirectorEnded);
    for (met, promise, what) in [
        (
            number(a, "/chip/task") == Some(A)
                && number(a, "/spec_revision/value") == Some(0)
                && number(a, "/kept_account") == Some(FIRST_ACCOUNT)
                && a.pointer("/result/ending/task/done").is_some()
                && text(a, "/result/value/report/bytes") == RESUMED_REPORT,
            SURVIVES,
            "A must finish done with its original task, revision and kept account",
        ),
        (
            number(waiting, "/chip/task") == number(a, "/chip/task") && number(a, "/chip/task").is_some(),
            SURVIVES,
            "the waiting and ended task numbers must be the same",
        ),
        (
            number(director, "/chip/task") == Some(DIRECTOR)
                && text(director, "/result/value/report/bytes") == RESUMED_REPORT,
            SURVIVES,
            "the original director must finish its report",
        ),
        (a["delegates"].as_array().is_some_and(Vec::is_empty), SURVIVES, "A must have no delegate"),
        (
            observed.told.iter().all(|told| matches!(told.agent, Agent::A | Agent::Director)),
            SURVIVES,
            "no agent beyond the director and A may run",
        ),
    ] {
        step_check(misses, "4.5", met, promise, what);
    }
}

#[must_use]
pub fn step_4_5_limited(observed: &Observed) -> Vec<Miss> {
    use super::{FIRST_ACCOUNT, HELD_A_SPEC};
    let mut misses = Vec::new();
    let waiting = document(observed, Read::AWaiting);
    let until = number(waiting, "/run_wait/account/why/limited/until");
    for (met, promise, what) in [
        (
            number(waiting, "/chip/task") == Some(A)
                && text(waiting, "/chip/phase") == "due"
                && number(waiting, "/run_wait/account/account") == Some(FIRST_ACCOUNT)
                && number(waiting, "/run_wait/account/source") == Some(0)
                && until.is_some(),
            ACCOUNT_WAIT,
            "A's waiting task must name its account, source and limited until while due",
        ),
        (
            until.is_some()
                && document(observed, Read::Limited)
                    .pointer("/accounts/deployment")
                    .and_then(Value::as_array)
                    .is_some_and(|rows| {
                        rows.iter().any(|row| {
                            number(row, "/number") == Some(FIRST_ACCOUNT)
                                && later(row, until)
                                && row["sources"].as_array().is_some_and(|sources| {
                                    sources
                                        .iter()
                                        .any(|source| number(source, "/number") == Some(0) && later(source, until))
                                })
                        })
                    }),
            ACCOUNT_WAIT,
            "the account and its source must be unusable and later until the same reset",
        ),
        (
            until
                .zip(number(task_document(observed, Read::AEnded), "/result/at"))
                .is_some_and(|(reset, ended)| ended > reset),
            SURVIVES,
            "A's result timestamp must be past the reset",
        ),
        (
            observed.commands.iter().position(|command| command == "chat").is_some_and(|chat| {
                observed.commands[chat + 1..]
                    .iter()
                    .all(|command| matches!(command.as_str(), "wait" | "task" | "accounts"))
            }),
            ACCOUNT_WAIT,
            "no person's act may follow the chat, including retry or replace-key",
        ),
    ] {
        step_check(&mut misses, "4.5", met, promise, what);
    }
    account_survives(&mut misses, observed);
    // The failed call committed no assistant turn: a fresh attempt must still
    // receive the spec that was in the first query, on the original credential.
    let mut queries = observed.told.iter().filter(|told| told.agent == Agent::A);
    let first = queries.next();
    let resumed = queries.next_back();
    step_check(
        &mut misses,
        "4.5",
        first.zip(resumed).is_some_and(|(first, resumed)| {
            first.credential == Some(0)
                && resumed.credential == Some(0)
                && contains(&first.words, HELD_A_SPEC)
                && contains(&resumed.words, HELD_A_SPEC)
        }),
        SURVIVES,
        "A's resumed query must retain its spec on credential 0 after its failed query",
    );
    misses
}

// Each document projects the engine's one monotonic deadline to wall time as
// it is read (jig crates/jig-domain-core/src/documents/deployment.rs:163), so
// two reads differ by their reads' gap. Allow 1 ms, in nanoseconds.
const PROJECTION: u64 = 1_000_000;
fn later(value: &Value, until: Option<u64>) -> bool {
    text(value, "/state/kind") == "later"
        && number(value, "/state/until").zip(until).is_some_and(|(state, until)| state.abs_diff(until) <= PROJECTION)
        && value["usable"] == false
}

#[cfg(test)]
mod account_tests {
    use super::{
        ACCOUNT_WAIT, Agent, BTreeMap, DIRECTOR, Kit, Observed, PROJECTION, Read, SURVIVES, Told, Value, assert_met,
        later, step_4_5_limited, step_4_5_refused,
    };
    use crate::case::{A, FIRST_ACCOUNT, HELD_A_SPEC, RESUMED_REPORT};
    use serde_json::json;

    fn told(agent: Agent, words: &str, credential: u32) -> Told {
        Told {
            system: Box::new([]),
            history: Box::new([]),
            agent,
            words: words.as_bytes().into(),
            model: Box::new([]),
            credential: Some(credential),
        }
    }
    fn refused_met() -> Observed {
        Observed {
            documents: BTreeMap::from([
                (
                    Read::AWaiting,
                    json!({"chip":{"task":A},
                    "run_wait":{"account":{"account":FIRST_ACCOUNT,"why":"refused"}}}),
                ),
                (
                    Read::Refused,
                    json!({"accounts":{"deployment":[{"number":FIRST_ACCOUNT,
                    "state":{"kind":"sign_in_again"}}]}}),
                ),
                (
                    Read::AEnded,
                    json!({"task":{"chip":{"task":A},"spec_revision":{"value":0},
                    "kept_account":FIRST_ACCOUNT,"delegates":[],
                    "result":{"ending":{"task":{"done":{}}},"value":{"report":{"bytes":RESUMED_REPORT}}}}}),
                ),
                (
                    Read::DirectorEnded,
                    json!({"task":{"chip":{"task":DIRECTOR},
                    "result":{"ending":{"task":{"done":{}}},"value":{"report":{"bytes":RESUMED_REPORT}}}}}),
                ),
            ]),
            commands: Vec::new(),
            told: vec![told(Agent::A, HELD_A_SPEC, 0), told(Agent::A, HELD_A_SPEC, 1)],
            kit: Kit { main: BTreeMap::new(), merged_green: false },
        }
    }
    #[test]
    fn refused_checkpoint_met() {
        assert_met(&step_4_5_refused(&refused_met()));
    }
    #[test]
    fn refused_checkpoint_reports_independent_misses_together() {
        const MISSES: usize = 3;
        let mut observed = refused_met();
        observed.documents.get_mut(&Read::AWaiting).expect("waiting")["run_wait"] = Value::Null;
        observed.documents.remove(&Read::Refused);
        observed.told.last_mut().expect("resumed").words = Box::new([]);
        let misses = step_4_5_refused(&observed);
        assert_eq!(misses.len(), MISSES);
        assert!(misses.iter().all(|miss| miss.step == "4.5"));
        assert!(misses.iter().any(|miss| miss.promise == ACCOUNT_WAIT && miss.what.contains("waiting task")));
        assert!(misses.iter().any(|miss| miss.promise == ACCOUNT_WAIT && miss.what.contains("sign_in_again")));
        assert!(misses.iter().any(|miss| miss.promise == SURVIVES && miss.what.contains("resumed query")));
    }

    const RESET: u64 = 10_000_000;
    fn limited_met() -> Observed {
        let mut observed = refused_met();
        observed.documents.insert(
            Read::AWaiting,
            json!({"chip":{"task":A,"phase":"due"},
                "run_wait":{"account":{"account":FIRST_ACCOUNT,"source":0,
                    "why":{"limited":{"until":RESET}}}}}),
        );
        observed.documents.remove(&Read::Refused);
        observed.documents.insert(
            Read::Limited,
            json!({"accounts":{"deployment":[{"number":FIRST_ACCOUNT,"usable":false,
                "state":{"kind":"later","until":RESET},
                "sources":[{"number":0,"usable":false,"state":{"kind":"later","until":RESET}}]}]}}),
        );
        observed.documents.get_mut(&Read::AEnded).expect("ended")["task"]["result"]["at"] = (RESET + 1).into();
        observed.commands = ["chat", "wait", "wait", "task", "accounts", "task"].map(String::from).into();
        observed.told = vec![told(Agent::A, HELD_A_SPEC, 0), told(Agent::A, HELD_A_SPEC, 0)];
        observed
    }
    #[test]
    fn limited_checkpoint_met() {
        assert_met(&step_4_5_limited(&limited_met()));
    }
    #[test]
    fn reset_projection_accepts_the_tolerance_in_both_directions() {
        for until in [RESET - PROJECTION, RESET + PROJECTION] {
            let row = json!({"usable":false,"state":{"kind":"later","until":until}});
            assert!(later(&row, Some(RESET)));
        }
    }
    #[test]
    fn reset_projection_rejects_one_past_the_tolerance_in_both_directions() {
        for until in [RESET - PROJECTION - 1, RESET + PROJECTION + 1] {
            let row = json!({"usable":false,"state":{"kind":"later","until":until}});
            assert!(!later(&row, Some(RESET)));
        }
    }
    #[test]
    fn limited_checkpoint_checks_both_account_and_source() {
        const MISSES: usize = 1;
        for path in ["/state/until", "/sources/0/state/until", "/sources/0/number", "/usable", "/sources/0/usable"] {
            let mut observed = limited_met();
            *observed.documents.get_mut(&Read::Limited).expect("limited")["accounts"]["deployment"][0]
                .pointer_mut(path)
                .expect("account field") = Value::Null;
            let misses = step_4_5_limited(&observed);
            assert_eq!(misses.len(), MISSES, "{path}");
            assert!(misses.iter().all(|miss| miss.promise == ACCOUNT_WAIT && miss.what.contains("unusable")));
        }
    }
    #[test]
    fn limited_checkpoint_rejects_completion_at_the_reset() {
        let mut observed = limited_met();
        observed.documents.get_mut(&Read::AEnded).expect("ended")["task"]["result"]["at"] = RESET.into();
        let misses = step_4_5_limited(&observed);
        assert!(misses.iter().any(|miss| miss.promise == SURVIVES && miss.what.contains("timestamp")));
    }
    #[test]
    fn limited_checkpoint_reports_independent_misses_together() {
        const MISSES: usize = 6;
        let mut observed = limited_met();
        observed.documents.get_mut(&Read::AWaiting).expect("waiting")["chip"]["phase"] = "running".into();
        observed.documents.remove(&Read::Limited);
        observed.documents.get_mut(&Read::AEnded).expect("ended")["task"]["result"]["at"] = RESET.into();
        observed.documents.get_mut(&Read::AEnded).expect("ended")["task"]["kept_account"] = Value::Null;
        observed.commands.push("retry".into());
        observed.told.last_mut().expect("resumed").words = Box::new([]);
        let misses = step_4_5_limited(&observed);
        assert_eq!(misses.len(), MISSES);
        assert!(misses.iter().all(|miss| miss.step == "4.5"));
        for what in ["waiting task", "unusable", "timestamp", "person's act", "original task", "resumed query"] {
            assert!(misses.iter().any(|miss| miss.what.contains(what)), "{what}");
        }
    }
}

const PACE_INBOX: Promise = Promise::Must("a question in the person's inbox (exact)");
const ONE_QUESTION: Promise = Promise::Add("the question waits in one inbox, not in two checks' reports");
const PACE_KEPT: Promise = Promise::Step("the answer goes into the director's notes");
const MODELS: Promise = Promise::Must("models amended (exact)");
const ORDER: Promise = Promise::Must("the holder's order of sources (exact)");

#[must_use]
pub fn step_4_4(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    pace_inbox(&mut misses, observed);
    pace_note(&mut misses, observed);
    pace_model(&mut misses, observed);
    pace_accounts(&mut misses, observed);
    misses
}
fn pace_inbox(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{PACE_ANSWER, PACE_QUESTION, PACE_QUESTION_MESSAGE};
    const ONE: usize = 1;
    let rows = document(observed, Read::PaceAsked).pointer("/needs/rows").and_then(Value::as_array);
    step_check(
        misses,
        "4.4",
        rows.is_some_and(|rows| {
            rows.iter().any(|row| {
                number(row, "/task/live/task") == Some(DIRECTOR)
                    && number(row, "/project") == Some(PROJECT)
                    && number(row, "/object/question/message") == Some(PACE_QUESTION_MESSAGE)
            })
        }),
        PACE_INBOX,
        "the inbox must hold A's question on the director in project 1",
    );
    step_check(
        misses,
        "4.4",
        rows.is_some_and(|rows| rows.len() == ONE),
        ONE_QUESTION,
        "the person's questions inbox must have exactly one row",
    );
    let asked =
        observed.told.iter().position(|told| told.agent == Agent::Director && contains(&told.words, PACE_QUESTION));
    let answered = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Director && contains(&told.words, &format!("1: {PACE_ANSWER}")));
    step_check(
        misses,
        "4.4",
        matches!((asked, answered), (Some(asked), Some(answered)) if asked < answered),
        PACE_INBOX,
        "the director must hear A's question, then the person's answer",
    );
    let sent = format!("Message {PACE_QUESTION_MESSAGE} sent");
    step_check(
        misses,
        "4.4",
        observed.told.iter().any(|told| told.agent == Agent::A && contains(&told.words, &sent)),
        PACE_INBOX,
        "A must be told the question's named message number",
    );
}
fn pace_note(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{PACE_ANSWER, PACE_NOTE};
    let mut written = None;
    for (at, told) in observed.told.iter().enumerate() {
        if told.agent != Agent::Director {
            continue;
        }
        let words = String::from_utf8_lossy(&told.words);
        for suffix in words.split("Note ").skip(1) {
            let Some((digits, _)) = suffix.split_once(" revision 1 written") else { continue };
            if let Ok(note) = digits.parse::<u64>()
                && note > 0
            {
                written = Some((at, note));
                break;
            }
        }
        if written.is_some() {
            break;
        }
    }
    step_check(misses, "4.4", written.is_some(), PACE_KEPT, "the director must be told Note N revision 1 written");
    step_check(
        misses,
        "4.4",
        written.is_some_and(|(at, note)| {
            let recalled = format!("Note {note} revision 1: {PACE_NOTE}\n{PACE_ANSWER}\n");
            observed
                .told
                .iter()
                .skip(at + 1)
                .any(|told| told.agent == Agent::Director && contains(&told.words, &recalled))
        }),
        PACE_KEPT,
        "after writing the note the director must recall its answer",
    );
}
fn pace_model(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DEFAULT_MODEL, PACE_ANSWER};
    let a = task_document(observed, Read::AEnded);
    step_check(
        misses,
        "4.4",
        number(a, "/chip/task") == Some(A)
            && number(a, "/project") == Some(PROJECT)
            && number(a, "/spec_revision/value") == Some(1)
            && text(a, "/llm/main/model") == DEFAULT_MODEL
            && a.pointer("/llm/main/effort") == Some(&Value::Null),
        MODELS,
        "A must have the default model without effort at spec revision 1",
    );
    let queries: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::A).collect();
    step_check(
        misses,
        "4.4",
        queries.first().is_some_and(|told| told.model.as_ref() == OWNER_MODEL.as_bytes())
            && queries.last().is_some_and(|told| {
                told.model.as_ref() == DEFAULT_MODEL.as_bytes() && contains(&told.words, PACE_ANSWER)
            }),
        MODELS,
        "A must first query owner, then use fake with the person's relayed answer",
    );
}
fn pace_accounts(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{FIRST_ACCOUNT, SECOND_ACCOUNT, SURVEY, SURVEY_REPORT};
    let rows = document(observed, Read::Ranked).pointer("/accounts/deployment").and_then(Value::as_array);
    let ranked = rows.is_some_and(|rows| {
        let numbers: Vec<_> = rows.iter().map(|row| number(row, "/number")).collect();
        let places: Vec<_> = rows.iter().map(|row| number(row, "/place")).collect();
        numbers == [Some(SECOND_ACCOUNT), Some(FIRST_ACCOUNT)] && places == [Some(0), Some(1)]
    });
    step_check(misses, "4.4", ranked, ORDER, "deployment accounts must put the second account first");
    let a = task_document(observed, Read::AEnded);
    step_check(
        misses,
        "4.4",
        number(a, "/chip/task") == Some(A) && number(a, "/kept_account") == Some(FIRST_ACCOUNT),
        ORDER,
        "A must keep its original account",
    );
    let survey = task_document(observed, Read::SurveyEnded);
    step_check(
        misses,
        "4.4",
        number(survey, "/chip/task") == Some(SURVEY)
            && number(survey, "/project") == Some(PROJECT)
            && number(survey, "/kept_account") == Some(SECOND_ACCOUNT)
            && text(survey, "/result/value/report/bytes") == SURVEY_REPORT,
        ORDER,
        "the survey must finish on the newly first account",
    );
    let queries: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Survey).collect();
    step_check(
        misses,
        "4.4",
        !queries.is_empty() && queries.iter().all(|told| told.credential == Some(1)),
        ORDER,
        "every survey query must use credential 1",
    );
}

#[cfg(test)]
mod pace_tests {
    use super::{
        Agent, BTreeMap, DIRECTOR, Kit, MODELS, ORDER, OWNER_MODEL, Observed, PACE_INBOX, PROJECT, Read, Told, Value,
        assert_met, step_4_4,
    };
    use crate::case::{
        A, DEFAULT_MODEL, FIRST_ACCOUNT, PACE_ANSWER, PACE_NOTE, PACE_QUESTION, PACE_QUESTION_MESSAGE, SECOND_ACCOUNT,
        SURVEY, SURVEY_REPORT,
    };
    use serde_json::json;

    fn told(agent: Agent, words: &str, model: &str, credential: u32) -> Told {
        Told {
            system: Box::new([]),
            history: Box::new([]),
            agent,
            words: words.as_bytes().into(),
            model: model.as_bytes().into(),
            credential: Some(credential),
        }
    }
    fn met() -> Observed {
        Observed {
            documents: BTreeMap::from([
                (
                    Read::PaceAsked,
                    json!({"needs":{"rows":[{"task":{"live":{"task":DIRECTOR}},
                    "project":PROJECT,"object":{"question":{"message":PACE_QUESTION_MESSAGE}}}]}}),
                ),
                (
                    Read::Ranked,
                    json!({"accounts":{"deployment":[
                    {"number":SECOND_ACCOUNT,"place":0},{"number":FIRST_ACCOUNT,"place":1}]}}),
                ),
                (
                    Read::AEnded,
                    json!({"task":{"chip":{"task":A},"project":PROJECT,
                    "spec_revision":{"value":1},"llm":{"main":{"model":DEFAULT_MODEL,"effort":null}},
                    "kept_account":FIRST_ACCOUNT}}),
                ),
                (
                    Read::SurveyEnded,
                    json!({"task":{"chip":{"task":SURVEY},"project":PROJECT,
                    "kept_account":SECOND_ACCOUNT,"result":{"value":{"report":{"bytes":SURVEY_REPORT}}}}}),
                ),
            ]),
            commands: Vec::new(),
            told: vec![
                told(Agent::A, &format!("Message {PACE_QUESTION_MESSAGE} sent"), OWNER_MODEL, 0),
                told(Agent::Director, PACE_QUESTION, DEFAULT_MODEL, 0),
                told(Agent::Director, &format!("1: {PACE_ANSWER}\nNote 1 revision 1 written"), DEFAULT_MODEL, 0),
                told(Agent::A, PACE_ANSWER, DEFAULT_MODEL, 0),
                told(Agent::Survey, "Begin", DEFAULT_MODEL, 1),
                told(Agent::Director, &format!("Note 1 revision 1: {PACE_NOTE}\n{PACE_ANSWER}\n"), DEFAULT_MODEL, 0),
            ],
            kit: Kit { main: BTreeMap::new(), merged_green: false },
        }
    }
    #[test]
    fn pace_checkpoint_met() {
        assert_met(&step_4_4(&met()));
    }
    #[test]
    fn pace_checkpoint_reports_independent_misses_together() {
        const MISSES: usize = 3;
        const QUESTION_ROW: usize = 0;
        let mut observed = met();
        observed.documents.get_mut(&Read::PaceAsked).expect("inbox")["needs"]["rows"][QUESTION_ROW]["project"] =
            Value::Null;
        observed.documents.get_mut(&Read::AEnded).expect("A")["task"]["spec_revision"] = Value::Null;
        observed.documents.get_mut(&Read::Ranked).expect("ranked")["accounts"]["deployment"]
            .as_array_mut()
            .expect("accounts")
            .reverse();
        let misses = step_4_4(&observed);
        assert_eq!(misses.len(), MISSES);
        assert!(misses.iter().all(|miss| miss.step == "4.4"));
        for promise in [PACE_INBOX, MODELS, ORDER] {
            assert!(misses.iter().any(|miss| miss.promise == promise));
        }
    }
}

// ctl wraps the source journal lines in its decoded activity rows.
fn check_activity(observed: &Observed, read: Read) -> Vec<&Value> {
    let document = document(observed, read);
    document
        .get("activity")
        .filter(|value| value.is_object())
        .unwrap_or(document)
        .pointer("/rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.pointer("/event/owner/source"))
        .collect()
}

/// Reference case 4.3's supported timer, cancel and empty-worker checkpoint.
#[must_use]
pub fn step_4_3(observed: &Observed) -> Vec<Miss> {
    use super::{B, B_REPORT, BENCHMARK, CHECK, CHECK_REASON, CHECK_REPORT};
    const TIMER: Promise = Promise::Must("a timer (exact)");
    const CANCEL: Promise = Promise::Must("cancel (exact)");
    const EMPTY: Promise = Promise::Add("a moved owner leaves nothing running behind");
    let mut misses = Vec::new();
    let director = task_document(observed, Read::CheckEnded);
    let benchmark = task_document(observed, Read::BenchmarkEnded);
    let b = task_document(observed, Read::BEnded);
    let director_activity = check_activity(observed, Read::DirectorTimer);
    let timer = director_activity
        .iter()
        .find_map(|line| number(line, "/line/heard/kind/timer/subscription").zip(number(line, "/line/heard/number")));
    step_check(
        &mut misses,
        "4.3",
        timer.is_some(),
        TIMER,
        "the director's activity must record the timer's subscription and wake",
    );
    let queries: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Check).collect();
    step_check(
        &mut misses,
        "4.3",
        timer.is_some_and(|(subscription, _message)| {
            let result = format!("Subscription {subscription} added");
            let wake = format!("\n{CHECK}: \n");
            queries.iter().position(|told| contains(&told.words, &result)).is_some_and(|at| {
                !contains(&queries[at].words, &wake)
                    && queries[at + 1..].iter().any(|told| {
                        contains(&told.words, &wake)
                            && contains(&told.words, "Messages arrived. Read them before continuing.")
                    })
            })
        }),
        TIMER,
        "after the subscription result, a later director query must contain the timer's empty wake and live wait result",
    );
    step_check(
        &mut misses,
        "4.3",
        number(benchmark, "/chip/task") == Some(BENCHMARK)
            && text(benchmark, "/chip/phase") == "ended"
            && number(benchmark, "/result/ending/task/cancelled/by/task") == Some(CHECK)
            && text(benchmark, "/result/ending/task/cancelled/reason/bytes") == CHECK_REASON,
        CANCEL,
        "the ended benchmark must record cancellation by the director with its reason",
    );
    let activity = check_activity(observed, Read::BenchmarkEnded);
    let attempt = number(benchmark, "/chip/details/owner/details/attempt");
    let cancelled = activity.iter().any(|line| {
        text(line, "/line/run_ended/end") == "cancelled"
            && number(line, "/line/run_ended/attempt").zip(attempt).is_some_and(|(ended, current)| ended == current)
    });
    step_check(&mut misses, "4.3", cancelled, CANCEL, "the benchmark's activity must record a cancelled run ending");
    let began: BTreeSet<_> = activity.iter().filter_map(|line| number(line, "/line/run_began/attempt")).collect();
    let ended: BTreeSet<_> = activity.iter().filter_map(|line| number(line, "/line/run_ended/attempt")).collect();
    step_check(
        &mut misses,
        "4.3",
        !began.is_empty()
            && began.is_subset(&ended)
            && document(observed, Read::BenchmarkEnded).pointer("/activity/cut") == Some(&Value::Bool(false))
            && document(observed, Read::BenchmarkEnded).pointer("/activity/before") == Some(&Value::Null)
            && document(observed, Read::BenchmarkEnded).pointer("/activity/after") == Some(&Value::Null)
            && number(benchmark, "/chip/details/owner/details/ended_at").is_some(),
        EMPTY,
        "the benchmark's complete activity must end every begun run, with the task ended",
    );
    // B's journal orders its lines, newest first (../client/tools.md, 11.3): the
    // cancel heard is older than the run that finished with B's report. The
    // journal's order, not two wall times, so a clock step can't reorder them.
    let b_activity = check_activity(observed, Read::BEnded);
    let reported = b_activity.iter().position(|line| text(line, "/line/run_ended/end") == "finished");
    let heard = b_activity.iter().position(|line| {
        text(line, "/line/heard/kind/result") == "cancelled"
            && number(line, "/line/heard/from/task") == Some(BENCHMARK)
            && text(line, "/line/heard/words") == CHECK_REASON
    });
    let heard_cancel = reported.zip(heard).is_some_and(|(reported, heard)| reported < heard);
    step_check(
        &mut misses,
        "4.3",
        number(b, "/chip/task") == Some(B)
            && text(b, "/result/value/report/bytes") == B_REPORT
            && heard_cancel
            && observed.told.iter().any(|told| {
                told.agent == Agent::B
                    && contains(&told.words, CHECK_REASON)
                    && contains(&told.words, "Messages arrived. Read them before continuing.")
            }),
        EMPTY,
        "B must hear its delegate's cancelled ending before reporting",
    );
    step_check(
        &mut misses,
        "4.3",
        number(director, "/chip/task") == Some(CHECK)
            && text(director, "/result/value/report/bytes") == CHECK_REPORT
            && observed.told.iter().any(|told| told.agent == Agent::Check && contains(&told.words, B_REPORT)),
        EMPTY,
        "the check director must hear B's result and end with its check report",
    );
    misses
}

const REPLACED: Promise = Promise::Must("the chat's run placed again after its worker's grace (exact)");
const FENCED: Promise = Promise::Must("one run per task, the lost attempt fenced before the next (gone)");
const QUESTIONS_WAIT: Promise = Promise::Step("A's questions wait, since the director lives there");
const ONCE: Promise = Promise::Add("nothing resumes twice");

/// Reference case 4.6's supported retry, retained question and single answer checkpoint.
#[must_use]
pub fn step_4_6(observed: &Observed) -> Vec<Miss> {
    use super::{HANG, HANG_REPORT, PHASE, PHASE_QUESTION_NUMBER, PHASE_REPORT};
    let mut misses = Vec::new();
    let reads = [Read::HangActivity, Read::HangActivityOlder, Read::HangActivityFirst];
    let rows: Vec<_> =
        reads.iter().flat_map(|read| document(observed, *read)["rows"].as_array().into_iter().flatten()).collect();
    let complete = reads.iter().all(|read| document(observed, *read)["cut"] == false)
        && document(observed, Read::HangActivityFirst)["before"].is_null();
    let ordered = rows
        .windows(2)
        .all(|pair| number(pair[0], "/position").zip(number(pair[1], "/position")).is_some_and(|(new, old)| new > old));
    let activity: Vec<_> = reads.iter().flat_map(|read| check_activity(observed, *read)).collect();
    let mut open = None;
    let mut sequenced = true;
    let mut attempts = Vec::new();
    let mut endings = Vec::new();
    for source in activity.iter().rev() {
        let line = &source["line"];
        if let Some(began) = line.get("run_began") {
            sequenced &= open.is_none();
            open = number(began, "/attempt");
            sequenced &= open.is_some();
            attempts.push(open);
        }
        if let Some(ended) = line.get("run_ended") {
            sequenced &= open.is_some() && open.take() == number(ended, "/attempt");
            endings.push(ended);
        }
    }
    sequenced &= open.is_none();
    step_check(
        &mut misses,
        "4.6",
        complete && ordered && sequenced,
        FENCED,
        "the complete journal must order every run beginning after the previous ending, with no attempt left open",
    );
    let director = task_document(observed, Read::HangEnded);
    let retried = matches!(attempts.as_slice(), [Some(first), Some(next)] if first != next)
        && matches!(endings.as_slice(), [first, next]
            if text(first, "/end/failed/class") == "agent" && text(next, "/end") == "finished");
    step_check(
        &mut misses,
        "4.6",
        retried
            && complete
            && ordered
            && sequenced
            && number(director, "/chip/task") == Some(HANG)
            && text(director, "/chip/phase") == "ended"
            && text(director, "/result/value/report/bytes") == HANG_REPORT,
        REPLACED,
        "the chat must have exactly two ordered attempts, failed as agent then finished, and end with its report",
    );
    let read = check_activity(observed, Read::HangQuestionRead)
        .iter()
        .any(|source| number(source, "/line/turn/read") == Some(PHASE_QUESTION_NUMBER));
    let waiting = task_document(observed, Read::HangWaiting);
    let (retained, answered_once) = restart_messages(observed);
    step_check(
        &mut misses,
        "4.6",
        read && number(waiting, "/chip/task") == Some(HANG)
            && text(waiting, "/chip/details/owner/details/observation/work") == "waiting"
            && retained,
        QUESTIONS_WAIT,
        "before the stop, the director must have committed A's question read and be waiting; its retry must retain one copy",
    );
    let phase = task_document(observed, Read::PhaseEnded);
    step_check(
        &mut misses,
        "4.6",
        answered_once
            && number(phase, "/chip/task") == Some(PHASE)
            && text(phase, "/chip/phase") == "ended"
            && text(phase, "/result/value/report/bytes") == PHASE_REPORT,
        ONCE,
        "the director must send one answer call, A must hear it once, and A's next phase must end with its report",
    );
    misses
}

fn user_words(told: &Told) -> impl Iterator<Item = &[u8]> {
    use skein_fake_llm_domain::api::{Part, Role};
    told.history.iter().filter(|message| message.role == Role::User).flat_map(|message| &message.parts).filter_map(
        |part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(text.as_ref()),
            Part::Opaque { .. } | Part::ToolCall { .. } => None,
        },
    )
}

fn restart_messages(observed: &Observed) -> (bool, bool) {
    use super::{PHASE, PHASE_ANSWER, PHASE_QUESTION, PHASE_QUESTION_NUMBER};
    use skein_fake_llm_domain::api::Part;
    let mut resumed = false;
    let mut retained = true;
    let mut answers = BTreeSet::new();
    let mut valid_answers = true;
    let mut received = 0;
    for told in &observed.told {
        if told.agent == Agent::Hang {
            let mut waits = 0;
            for part in told.history.iter().flat_map(|message| &message.parts) {
                if let Part::ToolCall { id, name, arguments } = part {
                    waits += usize::from(name.as_ref() == b"wait");
                    if name.as_ref() == b"message"
                        && let Ok(args) = serde_json::from_slice::<Value>(arguments)
                        && args["form"] == "answer"
                        && args["question"] == PHASE_QUESTION_NUMBER
                    {
                        valid_answers &= args["target"] == PHASE && args["words"] == PHASE_ANSWER;
                        answers.insert(id.as_ref());
                    }
                }
            }
            if waits >= 2 && user_words(told).any(|words| contains(words, "Begin the work your brief describes.")) {
                resumed = true;
                retained &=
                    user_words(told).map(|words| calls::occurrences(words, PHASE_QUESTION.as_bytes())).sum::<usize>()
                        == 1;
            }
        }
        if told.agent == Agent::Phase {
            received += user_words(told).map(|words| calls::occurrences(words, PHASE_ANSWER.as_bytes())).sum::<usize>();
        }
    }
    (resumed && retained, valid_answers && answers.len() == 1 && received == 1)
}

#[cfg(test)]
mod restart_tests {
    use super::*;
    use crate::case::{HANG, HANG_REPORT, PHASE, PHASE_ANSWER, PHASE_QUESTION, PHASE_QUESTION_NUMBER, PHASE_REPORT};
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn call(id: &str, name: &str, arguments: &Value) -> Part {
        Part::ToolCall {
            id: id.as_bytes().into(),
            name: name.as_bytes().into(),
            arguments: serde_json::to_vec(arguments).expect("tool arguments").into(),
        }
    }
    fn told(agent: Agent, history: Vec<Message>) -> Told {
        Told {
            system: Box::new([]),
            agent,
            history: history.into_boxed_slice(),
            words: Box::new([]),
            model: Box::new([]),
            credential: None,
        }
    }
    fn met() -> Observed {
        let events = [
            json!({"run_ended":{"attempt":8,"end":"finished"}}),
            json!({"run_began":{"attempt":8}}),
            json!({"run_ended":{"attempt":7,"end":{"failed":{"class":"agent"}}}}),
            json!({"run_began":{"attempt":7}}),
        ];
        let mut documents = BTreeMap::new();
        for (index, read) in
            [Read::HangActivity, Read::HangActivityOlder, Read::HangActivityFirst].into_iter().enumerate()
        {
            let selected = if index == 0 { &events[..2] } else { &events[index + 1..index + 2] };
            let newest = if index == 0 { 4 } else { 3 - index };
            let rows: Vec<_> = selected
                .iter()
                .enumerate()
                .map(|(offset, line)| json!({"position":newest-offset,"event":{"owner":{"source":{"line":line}}}}))
                .collect();
            documents.insert(read, json!({"cut":false,"before":if index == 2 { Value::Null } else { json!({"value":"older"}) },"rows":rows}));
        }
        for (read, task, report) in [(Read::HangEnded, HANG, HANG_REPORT), (Read::PhaseEnded, PHASE, PHASE_REPORT)] {
            documents.insert(
                read,
                json!({"task":{"chip":{"task":task,"phase":"ended"},"result":{"value":{"report":{"bytes":report}}}}}),
            );
        }
        documents.insert(
            Read::HangWaiting,
            json!({"task":{"chip":{"task":HANG,"details":{"owner":{"details":{"observation":{"work":"waiting"}}}}}}}),
        );
        documents.insert(
            Read::HangQuestionRead,
            json!({"rows":[{"event":{"owner":{"source":{"line":{"turn":{"read":PHASE_QUESTION_NUMBER}}}}}}]}),
        );
        let answer = call(
            "answer",
            "message",
            &json!({"target":PHASE,"form":"answer","question":PHASE_QUESTION_NUMBER,"words":PHASE_ANSWER}),
        );
        Observed {
            documents,
            commands: Vec::new(),
            kit: Kit { main: BTreeMap::new(), merged_green: false },
            told: vec![
                told(
                    Agent::Hang,
                    vec![
                        Message {
                            role: Role::Assistant,
                            parts: vec![
                                call("wait-one", "wait", &json!({})),
                                call("wait-two", "wait", &json!({})),
                                answer,
                            ]
                            .into_boxed_slice(),
                        },
                        Message {
                            role: Role::User,
                            parts: vec![Part::Text {
                                text: format!("Begin the work your brief describes. {PHASE_QUESTION}")
                                    .into_bytes()
                                    .into_boxed_slice(),
                            }]
                            .into_boxed_slice(),
                        },
                    ],
                ),
                told(
                    Agent::Phase,
                    vec![Message {
                        role: Role::User,
                        parts: vec![Part::ToolOutput {
                            id: b"wait".as_slice().into(),
                            output: PHASE_ANSWER.as_bytes().into(),
                            is_error: false,
                        }]
                        .into_boxed_slice(),
                    }],
                ),
            ],
        }
    }
    #[test]
    fn a_restarted_chat_meets_the_checkpoint() {
        assert_met(&step_4_6(&met()));
    }
    #[test]
    fn restart_promises_report_independent_failures_together() {
        const MISSES: usize = 4;
        let mut observed = met();
        observed.documents.get_mut(&Read::HangActivity).expect("activity")["rows"]
            .as_array_mut()
            .expect("rows")
            .reverse();
        observed.documents.get_mut(&Read::HangWaiting).expect("waiting")["task"]["chip"]["details"]["owner"]["details"]
            ["observation"]["work"] = json!("running");
        observed.documents.get_mut(&Read::PhaseEnded).expect("phase report")["task"]["result"]["value"]["report"]["bytes"] =
            json!("wrong report");
        let misses = step_4_6(&observed);
        assert_eq!(misses.len(), MISSES);
        assert!(misses.iter().all(|miss| miss.step == "4.6"));
        for promise in [REPLACED, FENCED, QUESTIONS_WAIT, ONCE] {
            assert!(misses.iter().any(|miss| miss.promise == promise));
        }
    }
    #[test]
    fn replayed_history_is_one_answer_but_a_new_call_is_another() {
        const DIRECTOR_QUERY: usize = 0;
        let mut observed = met();
        observed.told.push(observed.told[DIRECTOR_QUERY].clone());
        assert_met(&step_4_6(&observed));
        let duplicate = call(
            "second-answer",
            "message",
            &json!({"target":PHASE,"form":"answer","question":PHASE_QUESTION_NUMBER,"words":PHASE_ANSWER}),
        );
        observed.told.push(told(
            Agent::Hang,
            vec![Message { role: Role::Assistant, parts: vec![duplicate].into_boxed_slice() }],
        ));
        let misses = step_4_6(&observed);
        assert!(misses.iter().any(|miss| miss.promise == ONCE));
    }
    #[test]
    fn a_missing_question_or_a_duplicate_delivery_breaks_its_promise() {
        const DIRECTOR_QUERY: usize = 0;
        const USER_MESSAGE: usize = 1;
        const PHASE_QUERY: usize = 1;
        let mut observed = met();
        observed.told[DIRECTOR_QUERY].history[USER_MESSAGE].parts =
            vec![Part::Text { text: b"Begin the work your brief describes.".as_slice().into() }].into_boxed_slice();
        observed.told.push(observed.told[PHASE_QUERY].clone());
        let misses = step_4_6(&observed);
        assert!(misses.iter().any(|miss| miss.promise == QUESTIONS_WAIT));
        assert!(misses.iter().any(|miss| miss.promise == ONCE));
    }
}

// Act 2, step 5.1 (domain/testing.md, 5.3)
const RULES_CONTROL: Promise = Promise::Must("Stop and Release on the chat (exact)");
const RULES_INBOX: Promise = Promise::Must("questions wait in its inbox (exact)");
const RULES_CONVENTIONS: Promise = Promise::Must("the rules land as conventions that every later run reads (exact)");
const RULES_FRESH: Promise = Promise::Must("the director's next run starts fresh and reads them (exact)");
const RULES_NO_NOTICE: Promise = Promise::Add("no message telling anyone to re-read a rule");
const RULES_DROPPED: Promise = Promise::Add("a held owner needs no notice that a rule was dropped");

#[must_use]
pub fn step_5_1(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    rules_controls(&mut misses, observed);
    rules_director(&mut misses, observed);
    rules_owner(&mut misses, observed);
    rules_messages_and_endings(&mut misses, observed);
    misses
}

fn rules_controls(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{NEW_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, SCOPE, SCOPE_QUESTION, SCOPE_QUESTION_NUMBER};
    let waiting = task_document(observed, Read::RulesWaiting);
    let held = task_document(observed, Read::RulesHeld);
    let question = task_document(observed, Read::RulesQuestionWaiting);
    for (met, promise, what) in [
        (
            number(waiting, "/chip/task") == Some(RULES)
                && text(waiting, "/chip/details/owner/details/observation/work") == "waiting"
                && waiting["delegates"]
                    .as_array()
                    .is_some_and(|rows| rows.iter().any(|row| number(row, "/chip/task") == Some(SCOPE))),
            RULES_CONTROL,
            "the director must wait in its run with A delegated",
        ),
        (rules_held(held), RULES_CONTROL, "the chat must be held, stopped by the person at its hold revision"),
        (
            rules_held(question)
                && check_activity(observed, Read::RulesQuestionWaiting).iter().any(|row| {
                    number(row, "/task") == Some(RULES)
                        && number(row, "/line/words/message") == Some(SCOPE_QUESTION_NUMBER)
                        && number(row, "/line/words/by/task") == Some(SCOPE)
                        && text(row, "/line/words/kind") == "question"
                        && text(row, "/line/words/words") == SCOPE_QUESTION
                }),
            RULES_INBOX,
            "A's numbered question must wait at the still-held chat",
        ),
        (
            number(question, "/hold/owner/revision") == Some(RULES_HOLD_REVISION),
            RULES_INBOX,
            "the same hold must remain while the question waits",
        ),
        (
            observed.kit.main.get(RULES_FILE.as_bytes()).is_some_and(|words| words == NEW_RULES.as_bytes()),
            RULES_CONVENTIONS,
            "kit's main must hold the new guide",
        ),
    ] {
        step_check(misses, "5.1", met, promise, what);
    }
}

fn rules_director(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, OLD_RULES, SCOPE_QUESTION};
    use skein_fake_llm_domain::api::Role;
    let director: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Rules).collect();
    let resumed = director.iter().position(|told| {
        contains(&told.system, NEW_RULES) || user_words(told).any(|words| contains(words, SCOPE_QUESTION))
    });
    let before = resumed.map_or(&director[..], |at| &director[..at]);
    let after = resumed.map_or(&director[director.len()..], |at| &director[at..]);
    for (met, promise, what) in [
        (
            !before.is_empty()
                && before.iter().all(|told| contains(&told.system, OLD_RULES))
                && before.first().is_some_and(|first| before.iter().all(|told| told.system == first.system))
                && before.windows(2).all(|pair| pair[1].history.starts_with(&pair[0].history)),
            RULES_CONVENTIONS,
            "the director's opening run must read the old guide",
        ),
        (
            !after.is_empty()
                && after.iter().all(|told| contains(&told.system, NEW_RULES) && !contains(&told.system, DROPPED_RULE)),
            RULES_CONVENTIONS,
            "every director query told the question must read the new guide without the dropped rule",
        ),
        (
            after.first().is_some_and(|first| first.history.iter().all(|message| message.role != Role::Assistant)),
            RULES_FRESH,
            "the director's first query after release must contain none of its earlier assistant turns",
        ),
        (
            after.first().is_some_and(|first| {
                after.iter().all(|told| told.system == first.system)
                    && after.windows(2).all(|pair| pair[1].history.starts_with(&pair[0].history))
                    && after.iter().all(|told| {
                        user_words(told)
                            .map(|words| calls::occurrences(words, SCOPE_QUESTION.as_bytes()))
                            .sum::<usize>()
                            == 1
                    })
            }),
            RULES_INBOX,
            "the question must be delivered once in one director run's growing history",
        ),
    ] {
        step_check(misses, "5.1", met, promise, what);
    }
}

fn rules_owner(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, SCOPE_ANSWER};
    let a: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Scope).collect();
    step_check(
        misses,
        "5.1",
        a.first().is_some_and(|told| contains(&told.system, DROPPED_RULE)),
        RULES_DROPPED,
        "A's opening query must read the dropped rule while it is in force",
    );
    let answered: Vec<_> =
        a.iter().filter(|told| user_words(told).any(|words| contains(words, SCOPE_ANSWER))).collect();
    step_check(
        misses,
        "5.1",
        answered.len() == 1
            && answered.iter().all(|told| {
                contains(&told.system, NEW_RULES)
                    && !contains(&told.system, DROPPED_RULE)
                    && user_words(told).map(|words| calls::occurrences(words, SCOPE_ANSWER.as_bytes())).sum::<usize>()
                        == 1
            }),
        RULES_DROPPED,
        "A must hear the answer once in a run reading the new guide without the dropped rule",
    );
}

fn rules_messages_and_endings(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, RULES, RULES_REPORT, SCOPE, SCOPE_REPORT};
    step_check(
        misses,
        "5.1",
        observed.told.iter().all(|told| {
            message_words(told).all(|words| {
                NEW_RULES
                    .lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .all(|line| !contains(words, line))
                    && !contains(words, "re-read")
                    && !contains(words, "reread")
            })
        }),
        RULES_NO_NOTICE,
        "no query's messages may carry the new rules or a request to re-read them",
    );
    step_check(
        misses,
        "5.1",
        observed.told.iter().all(|told| message_words(told).all(|words| !contains(words, DROPPED_RULE))),
        RULES_DROPPED,
        "no message may tell an owner the dropped rule",
    );
    for (read, task, report) in [(Read::RulesEnded, RULES, RULES_REPORT), (Read::ScopeEnded, SCOPE, SCOPE_REPORT)] {
        let ended = task_document(observed, read);
        step_check(
            misses,
            "5.1",
            number(ended, "/chip/task") == Some(task)
                && text(ended, "/chip/phase") == "ended"
                && text(ended, "/result/value/report/bytes") == report,
            RULES_CONTROL,
            "the original director and A must end with their reports",
        );
    }
}

fn rules_held(task: &Value) -> bool {
    use super::{RULES, RULES_HOLD_REVISION};
    number(task, "/chip/task") == Some(RULES)
        && text(task, "/chip/phase") == "held"
        && number(task, "/hold/owner/reason/stopped_by/party") == Some(1)
        && number(task, "/hold/owner/revision") == Some(RULES_HOLD_REVISION)
}

fn message_words(told: &Told) -> impl Iterator<Item = &[u8]> {
    use skein_fake_llm_domain::api::Part;
    told.history.iter().flat_map(|message| &message.parts).filter_map(|part| match part {
        Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(text.as_ref()),
        Part::ToolCall { arguments, .. } => Some(arguments.as_ref()),
        Part::Opaque { .. } => None,
    })
}

#[cfg(test)]
mod rules_tests {
    use super::*;
    use crate::case::{
        DROPPED_RULE, NEW_RULES, OLD_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, RULES_REPORT, SCOPE, SCOPE_ANSWER,
        SCOPE_QUESTION, SCOPE_QUESTION_NUMBER, SCOPE_REPORT,
    };
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    const OPENING: usize = 0;
    const OPENING_FOLLOWUP: usize = 1;
    const FRESH: usize = 2;
    const LATER: usize = 3;
    const A_OPENING: usize = 4;
    const A_ANSWERED: usize = 5;

    fn user(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn assistant(words: &str) -> Message {
        Message { role: Role::Assistant, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn told(agent: Agent, guide: &str, history: &[Message]) -> Told {
        let mut system = agent.cue().to_vec();
        system.extend_from_slice(guide.as_bytes());
        Told::new(&system, b"fake", Some(0), history).expect("case cue")
    }
    fn held() -> Value {
        json!({"chip":{"task":RULES,"phase":"held"}, "hold":{"owner":{
            "reason":{"stopped_by":{"party":1}},"revision":RULES_HOLD_REVISION}}})
    }
    fn ended(task: u64, report: &str) -> Value {
        json!({"task":{"chip":{"task":task,"phase":"ended"},"result":{"value":{"report":{"bytes":report}}}}})
    }
    fn met() -> Observed {
        let fresh = vec![user(&format!("{SCOPE}: {SCOPE_QUESTION}"))];
        let mut later = fresh.clone();
        later.extend([assistant("Answer A."), user("Answer sent")]);
        Observed {
            documents: BTreeMap::from([
                (
                    Read::RulesWaiting,
                    json!({"chip":{"task":RULES,"details":{"owner":{"details":{
                    "observation":{"work":"waiting"}}}}},"delegates":[{"chip":{"task":SCOPE}}]}),
                ),
                (Read::RulesHeld, held()),
                (
                    Read::RulesQuestionWaiting,
                    json!({"task":held(),"activity":{"rows":[{"event":{"owner":{"source":{
                    "task":RULES,"line":{"words":{"message":SCOPE_QUESTION_NUMBER,"by":{"task":SCOPE},
                        "kind":"question","words":SCOPE_QUESTION}}}}}}]}}),
                ),
                (Read::RulesEnded, ended(RULES, RULES_REPORT)),
                (Read::ScopeEnded, ended(SCOPE, SCOPE_REPORT)),
            ]),
            told: vec![
                told(Agent::Rules, OLD_RULES, &[user("Begin the work your brief describes.")]),
                told(
                    Agent::Rules,
                    OLD_RULES,
                    &[user("Begin the work your brief describes."), assistant("Delegate A.")],
                ),
                told(Agent::Rules, NEW_RULES, &fresh),
                told(Agent::Rules, NEW_RULES, &later),
                told(Agent::Scope, OLD_RULES, &[user("Begin the work your brief describes.")]),
                told(Agent::Scope, NEW_RULES, &[user(SCOPE_ANSWER)]),
            ],
            commands: Vec::new(),
            kit: Kit {
                main: BTreeMap::from([(RULES_FILE.as_bytes().to_vec(), NEW_RULES.as_bytes().to_vec())]),
                merged_green: false,
            },
        }
    }
    fn rejects(observed: &Observed, promise: Promise, what: &str) {
        let misses = step_5_1(observed);
        assert!(misses.iter().all(|miss| miss.step == "5.1"));
        assert!(
            misses.iter().any(|miss| miss.promise == promise && miss.what.contains(what)),
            "expected {what}: {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        );
    }
    #[test]
    fn held_chat_and_fresh_rules_meet_the_checkpoint() {
        assert_met(&step_5_1(&met()));
    }
    #[test]
    fn the_wait_and_both_hold_reads_are_checked() {
        for (read, path, promise, what) in [
            (Read::RulesWaiting, "/chip/task", RULES_CONTROL, "wait in its run"),
            (Read::RulesWaiting, "/chip/details/owner/details/observation/work", RULES_CONTROL, "wait in its run"),
            (Read::RulesWaiting, "/delegates/0/chip/task", RULES_CONTROL, "wait in its run"),
            (Read::RulesHeld, "/chip/task", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/chip/phase", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/hold/owner/reason/stopped_by/party", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/hold/owner/revision", RULES_CONTROL, "stopped by the person"),
            (Read::RulesQuestionWaiting, "/task/chip/phase", RULES_INBOX, "still-held"),
            (Read::RulesQuestionWaiting, "/task/hold/owner/revision", RULES_INBOX, "same hold"),
        ] {
            let mut observed = met();
            *observed.documents.get_mut(&read).expect("document").pointer_mut(path).expect("field") = Value::Null;
            rejects(&observed, promise, what);
        }
    }
    #[test]
    fn the_waiting_question_has_its_recipient_sender_number_and_words() {
        for path in ["/task", "/line/words/message", "/line/words/by/task", "/line/words/kind", "/line/words/words"] {
            let mut observed = met();
            *observed.documents.get_mut(&Read::RulesQuestionWaiting).expect("question")["activity"]["rows"][0]
                ["event"]["owner"]["source"].pointer_mut(path).expect("question field") = Value::Null;
            rejects(&observed, RULES_INBOX, "numbered question");
        }
    }
    #[test]
    fn main_and_every_relevant_query_must_have_the_right_system_guide() {
        let mut observed = met();
        observed.kit.main.clear();
        rejects(&observed, RULES_CONVENTIONS, "kit's main");
        for at in [OPENING, FRESH, LATER, A_OPENING, A_ANSWERED] {
            let mut observed = met();
            observed.told[at].system = Box::new([]);
            let (promise, what) = match at {
                OPENING => (RULES_CONVENTIONS, "opening run"),
                A_OPENING => (RULES_DROPPED, "while it is in force"),
                A_ANSWERED => (RULES_DROPPED, "hear the answer once"),
                _ => (RULES_CONVENTIONS, "every director query"),
            };
            rejects(&observed, promise, what);
        }
        for at in [FRESH, LATER, A_ANSWERED] {
            let mut observed = met();
            let mut system = observed.told[at].system.to_vec();
            system.extend_from_slice(DROPPED_RULE.as_bytes());
            observed.told[at].system = system.into();
            rejects(
                &observed,
                if at == A_ANSWERED { RULES_DROPPED } else { RULES_CONVENTIONS },
                if at == A_ANSWERED { "hear the answer once" } else { "every director query" },
            );
        }
    }
    #[test]
    fn the_directors_history_must_be_fresh_then_grow_in_one_run() {
        let mut observed = met();
        let mut history = observed.told[FRESH].history.to_vec();
        history.push(assistant("Delegate A."));
        observed.told[FRESH].history = history.into();
        rejects(&observed, RULES_FRESH, "earlier assistant turns");
        let mut observed = met();
        observed.told[LATER].system = OLD_RULES.as_bytes().into();
        rejects(&observed, RULES_INBOX, "one director run");
        let mut observed = met();
        observed.told[LATER].history = Box::new([user("Another run"), user(SCOPE_QUESTION)]);
        rejects(&observed, RULES_INBOX, "one director run");
        for replacement in ["No question", &format!("{SCOPE_QUESTION} {SCOPE_QUESTION}")] {
            let mut observed = met();
            observed.told[FRESH].history = Box::new([user(replacement)]);
            rejects(&observed, RULES_INBOX, "delivered once");
        }
    }
    #[test]
    fn the_opening_queries_must_also_belong_to_one_run() {
        let mut observed = met();
        observed.told[OPENING_FOLLOWUP].system = format!("{OLD_RULES}another run").into_bytes().into();
        rejects(&observed, RULES_CONVENTIONS, "opening run");
        let mut observed = met();
        observed.told[OPENING_FOLLOWUP].history = Box::new([user("Another opening")]);
        rejects(&observed, RULES_CONVENTIONS, "opening run");
    }

    #[test]
    fn a_must_receive_the_answer_once() {
        let mut observed = met();
        observed.told[A_ANSWERED].history = Box::new([]);
        rejects(&observed, RULES_DROPPED, "hear the answer once");
        let mut observed = met();
        observed.told.push(observed.told[A_ANSWERED].clone());
        rejects(&observed, RULES_DROPPED, "hear the answer once");
        let mut observed = met();
        observed.told[A_ANSWERED].history = Box::new([user(&format!("{SCOPE_ANSWER} {SCOPE_ANSWER}"))]);
        rejects(&observed, RULES_DROPPED, "hear the answer once");
    }
    #[test]
    fn messages_cannot_supply_rules_or_reread_notices() {
        for words in NEW_RULES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .chain(["Please re-read the guide", "Please reread the guide"])
        {
            let mut observed = met();
            observed.told[A_ANSWERED].history = Box::new([user(words), user(SCOPE_ANSWER)]);
            rejects(&observed, RULES_NO_NOTICE, "no query's messages");
        }
        let mut observed = met();
        observed.told[A_ANSWERED].history = Box::new([user(DROPPED_RULE), user(SCOPE_ANSWER)]);
        rejects(&observed, RULES_DROPPED, "no message");
    }
    #[test]
    fn both_original_tasks_must_end_with_their_reports() {
        for read in [Read::RulesEnded, Read::ScopeEnded] {
            for path in ["/task/chip/task", "/task/chip/phase", "/task/result/value/report/bytes"] {
                let mut observed = met();
                *observed.documents.get_mut(&read).expect("ended").pointer_mut(path).expect("field") = Value::Null;
                rejects(&observed, RULES_CONTROL, "original director and A");
            }
        }
    }
}

// Act 2, step 5.3 (domain/testing.md, 5.3)
const SHAPE_INSTRUCTIONS: Promise = Promise::Must("instructions amended (exact)");
const SHAPE_HISTORY: Promise = Promise::Add("the amendment is in A's history, with what it replaced");
const SHAPE_ENDINGS: Promise = Promise::Step("the chat and A report under the new brief");

/// The observable before/after, without a deadline's hold (waiting on jig)
/// or a ctl read of the stored amendment history (a client gap).
#[must_use]
pub fn step_5_3(observed: &Observed) -> Vec<Miss> {
    use super::{
        MOVER, MOVER_AMENDMENT_REVISION, MOVER_QUESTION, MOVER_QUESTION_NUMBER, MOVER_REPORT, NEW_BRIEF, OLD_BRIEF,
        SHAPE, SHAPE_REPORT,
    };
    let mut misses = Vec::new();
    let ended = task_document(observed, Read::MoverEnded);
    step_check(
        &mut misses,
        "5.3",
        text(ended, "/instructions/bytes") == NEW_BRIEF
            && ended.pointer("/instructions/cut") == Some(&Value::Bool(false))
            && number(ended, "/spec_revision/value") == Some(MOVER_AMENDMENT_REVISION),
        SHAPE_HISTORY,
        "A's document must hold the whole new brief at the amendment's revision",
    );
    let a: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Mover).collect();
    step_check(
        &mut misses,
        "5.3",
        a.first().is_some_and(|told| contains(&told.system, OLD_BRIEF) && !contains(&told.system, NEW_BRIEF)),
        SHAPE_HISTORY,
        "A's first query must carry the old instructions alone in system text",
    );
    let amended = a
        .iter()
        .position(|told| contains(&told.system, NEW_BRIEF) || user_words(told).any(|words| contains(words, NEW_BRIEF)));
    let after = amended.map_or(&a[a.len()..], |at| &a[at..]);
    step_check(
        &mut misses,
        "5.3",
        amended.is_some_and(|at| at > 0)
            && !after.is_empty()
            && after.iter().all(|told| contains(&told.system, NEW_BRIEF) && !contains(&told.system, OLD_BRIEF)),
        SHAPE_INSTRUCTIONS,
        "every A query from the amendment onward must use the new instructions alone",
    );
    step_check(
        &mut misses,
        "5.3",
        !after.is_empty()
            && after.iter().all(|told| {
                user_words(told).map(|words| calls::occurrences(words, NEW_BRIEF.as_bytes())).sum::<usize>() == 1
            }),
        SHAPE_INSTRUCTIONS,
        "A must be told the amendment's whole message once in each later query's history",
    );
    let chat: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Shape).collect();
    step_check(
        &mut misses,
        "5.3",
        !chat.is_empty()
            && chat.iter().all(|told| !contains(&told.system, OLD_BRIEF) && !contains(&told.system, NEW_BRIEF)),
        SHAPE_INSTRUCTIONS,
        "the chat's charter must carry neither of A's instruction briefs",
    );
    let sent = format!("Message {MOVER_QUESTION_NUMBER} sent");
    let question_at = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Shape && user_words(told).any(|words| contains(words, MOVER_QUESTION)));
    let amendment_at = observed.told.iter().position(|told| {
        told.agent == Agent::Mover
            && (contains(&told.system, NEW_BRIEF) || user_words(told).any(|words| contains(words, NEW_BRIEF)))
    });
    step_check(
        &mut misses,
        "5.3",
        a.iter().any(|told| contains(&told.words, &sent))
            && question_at.zip(amendment_at).is_some_and(|(question, amendment)| question < amendment),
        SHAPE_INSTRUCTIONS,
        "A's numbered question must reach the chat before it amends A",
    );
    for (read, task, report) in [(Read::MoverEnded, MOVER, MOVER_REPORT), (Read::ShapeEnded, SHAPE, SHAPE_REPORT)] {
        let ended = task_document(observed, read);
        step_check(
            &mut misses,
            "5.3",
            number(ended, "/chip/task") == Some(task)
                && number(ended, "/project") == Some(PROJECT)
                && text(ended, "/chip/phase") == "ended"
                && text(ended, "/result/value/report/bytes") == report,
            SHAPE_ENDINGS,
            "the original chat and A must end with their reports in the project",
        );
    }
    misses
}

#[cfg(test)]
mod shape_tests {
    use super::*;
    use crate::case::{
        MOVER, MOVER_AMENDMENT_REVISION, MOVER_QUESTION, MOVER_QUESTION_NUMBER, MOVER_REPORT, NEW_BRIEF, OLD_BRIEF,
        SHAPE, SHAPE_REPORT,
    };
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn user(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn told(agent: Agent, brief: &str, messages: &[Message]) -> Told {
        Told::new(&[agent.cue(), b"\n", brief.as_bytes()].concat(), b"fake", Some(0), messages).expect("case cue")
    }
    fn met() -> Observed {
        let a = json!({"chip":{"task":MOVER,"phase":"ended"},"project":PROJECT,
            "instructions":{"bytes":NEW_BRIEF,"cut":false},"spec_revision":{"value":MOVER_AMENDMENT_REVISION},
            "result":{"value":{"report":{"bytes":MOVER_REPORT}}}});
        let shape = json!({"chip":{"task":SHAPE,"phase":"ended"},"project":PROJECT,
            "result":{"value":{"report":{"bytes":SHAPE_REPORT}}}});
        Observed {
            documents: BTreeMap::from([(Read::MoverEnded, a), (Read::ShapeEnded, shape)]),
            told: vec![
                told(Agent::Mover, OLD_BRIEF, &[]),
                told(
                    Agent::Mover,
                    OLD_BRIEF,
                    &[Message {
                        role: Role::User,
                        parts: Box::new([Part::ToolOutput {
                            id: b"question".as_slice().into(),
                            is_error: false,
                            output: format!("Message {MOVER_QUESTION_NUMBER} sent").into_bytes().into(),
                        }]),
                    }],
                ),
                told(Agent::Shape, "", &[user(MOVER_QUESTION)]),
                told(Agent::Mover, NEW_BRIEF, &[user(NEW_BRIEF)]),
                told(Agent::Mover, NEW_BRIEF, &[user(NEW_BRIEF), user("Continue")]),
            ],
            commands: Vec::new(),
            kit: Kit { main: BTreeMap::new(), merged_green: false },
        }
    }
    fn rejects(observed: &Observed, promise: Promise, what: &str) {
        let misses = step_5_3(observed);
        assert!(misses.iter().all(|miss| miss.step == "5.3"));
        assert!(
            misses.iter().any(|miss| miss.promise == promise && miss.what.contains(what)),
            "expected {what}: {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        );
    }
    #[test]
    fn amendment_and_its_observable_before_and_after_meet_the_checkpoint() {
        assert_met(&step_5_3(&met()));
    }
    #[test]
    fn the_current_instructions_and_their_revision_are_checked() {
        for path in ["/instructions/bytes", "/instructions/cut", "/spec_revision/value"] {
            let mut observed = met();
            *observed
                .documents
                .get_mut(&Read::MoverEnded)
                .expect("A's document")
                .pointer_mut(path)
                .expect("mutated field") = Value::Null;
            rejects(&observed, SHAPE_HISTORY, "A's document");
        }
    }
    #[test]
    fn the_opening_system_must_have_only_the_old_brief() {
        for brief in [String::new(), NEW_BRIEF.into(), format!("{OLD_BRIEF}\n{NEW_BRIEF}")] {
            let mut observed = met();
            observed.told[0].system = brief.into_bytes().into();
            rejects(&observed, SHAPE_HISTORY, "first query");
        }
    }
    #[test]
    fn every_system_after_the_amendment_must_have_only_the_new_brief() {
        for at in [3, 4] {
            for brief in [String::new(), OLD_BRIEF.into(), format!("{OLD_BRIEF}\n{NEW_BRIEF}")] {
                let mut observed = met();
                observed.told[at].system = brief.into_bytes().into();
                rejects(&observed, SHAPE_INSTRUCTIONS, "every A query");
            }
        }
    }
    #[test]
    fn the_amendment_message_must_arrive_once() {
        for at in [3, 4] {
            for messages in [vec![], vec![user(NEW_BRIEF), user(NEW_BRIEF)]] {
                let mut observed = met();
                observed.told[at].history = messages.into();
                rejects(&observed, SHAPE_INSTRUCTIONS, "whole message once");
            }
        }
        let mut observed = met();
        observed.told.truncate(3);
        rejects(&observed, SHAPE_INSTRUCTIONS, "every A query");
    }
    #[test]
    fn the_chat_never_inherits_either_brief() {
        for brief in [OLD_BRIEF, NEW_BRIEF] {
            let mut observed = met();
            observed.told[2].system = brief.as_bytes().into();
            rejects(&observed, SHAPE_INSTRUCTIONS, "chat's charter");
        }
        let mut observed = met();
        observed.told.retain(|told| told.agent != Agent::Shape);
        rejects(&observed, SHAPE_INSTRUCTIONS, "chat's charter");
    }
    #[test]
    fn the_numbered_question_and_its_delivery_are_checked() {
        let mut observed = met();
        observed.told[1] = told(Agent::Mover, OLD_BRIEF, &[]);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
        let mut observed = met();
        observed.told[2].history = Box::new([]);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
        let mut observed = met();
        observed.told.swap(2, 3);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
    }
    #[test]
    fn both_numbered_tasks_must_report_in_the_project() {
        for read in [Read::MoverEnded, Read::ShapeEnded] {
            for path in ["/chip/task", "/project", "/chip/phase", "/result/value/report/bytes"] {
                let mut observed = met();
                *observed.documents.get_mut(&read).expect("ended document").pointer_mut(path).expect("mutated field") =
                    Value::Null;
                rejects(&observed, SHAPE_ENDINGS, "original chat and A");
            }
            let mut observed = met();
            observed.documents.remove(&read);
            rejects(&observed, SHAPE_ENDINGS, "original chat and A");
        }
    }
}

// Act 2, step 5.2 (domain/testing.md, 5.3)
const PERSON_STOP: Promise = Promise::Must("Stop, which is the person's (exact)");
const DELEGATES_GO_ON: Promise =
    Promise::Must("delegates and queued changes going on under a stopped goal (to check, 6.8)");
const OLD_HELPER: Promise = Promise::Must("retiring a charter that still has tasks (to check, 6.8)");
const STOPPED_NEWS: Promise =
    Promise::Add("a stopped goal's news waits for its release, so nothing is relayed by hand");

#[must_use]
pub fn step_5_2(observed: &Observed) -> Vec<Miss> {
    use super::{BOWNER, H1, H1_REVIEWER, HELPER, IMPLEMENTER, OWNER, PAUSE, PROJECT};
    let mut misses = Vec::new();
    for (read, expected, parent) in [
        (Read::ImplementerRunning, IMPLEMENTER, BOWNER),
        (Read::H1Running, H1_REVIEWER, H1),
        (Read::BHeld, BOWNER, PAUSE),
        (Read::BStillHeld, BOWNER, PAUSE),
        (Read::BEnded, BOWNER, PAUSE),
        (Read::OwnerEnded, OWNER, PAUSE),
        (Read::HelperEnded, HELPER, PAUSE),
        (Read::ImplementerEnded, IMPLEMENTER, BOWNER),
        (Read::H1Ended, H1, BOWNER),
    ] {
        let task = task_document(observed, read);
        step_check(
            &mut misses,
            "5.2",
            number(task, "/chip/task") == Some(expected)
                && number(task, "/project") == Some(PROJECT)
                && number(task, "/requester/task") == Some(parent),
            DELEGATES_GO_ON,
            &format!("{read:?} must identify task {expected} under {parent} in project {PROJECT}"),
        );
    }
    for read in [Read::ImplementerRunning, Read::H1Running] {
        let task = task_document(observed, read);
        step_check(
            &mut misses,
            "5.2",
            text(task, "/chip/phase") == "running"
                && text(task, "/chip/details/owner/details/observation/work") == "writing"
                && number(task, "/chip/details/owner/phase/active/running/attempt")
                    .is_some_and(|attempt| number(task, "/chip/details/owner/details/attempt") == Some(attempt))
                && task["result"].is_null(),
            DELEGATES_GO_ON,
            &format!("{read:?} must show an open attempt writing before B's Stop"),
        );
    }
    pause_stop(observed, &mut misses);
    pause_children(observed, &mut misses);
    pause_news(observed, &mut misses);
    misses
}

fn pause_stop(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{B_HOLD, BOWNER, OWNER_REPORT};
    let held = task_document(observed, Read::BHeld);
    let still = task_document(observed, Read::BStillHeld);
    let b = task_document(observed, Read::BEnded);
    let a = task_document(observed, Read::OwnerEnded);
    for task in [held, still] {
        step_check(
            misses,
            "5.2",
            text(task, "/chip/phase") == "held"
                && number(task, "/hold/owner/revision") == Some(B_HOLD)
                && number(task, "/hold/owner/reason/stopped_by/party") == Some(1),
            PERSON_STOP,
            "B must stay held at the person's Stop revision",
        );
    }
    step_check(
        misses,
        "5.2",
        text(a, "/chip/phase") == "ended"
            && text(a, "/result/value/report/bytes") == OWNER_REPORT
            && a["hold"].is_null()
            && check_activity(observed, Read::OwnerEnded)
                .iter()
                .filter(|row| number(row, "/line/milestone/position").is_some())
                .count()
                == 2
            && check_activity(observed, Read::OwnerEnded)
                .iter()
                .filter(|row| number(row, "/line/run_began/attempt").is_some())
                .count()
                == 1
            && pause_activity(observed, Read::OwnerEnded)["cut"] == false
            && pause_activity(observed, Read::OwnerEnded)["before"].is_null()
            && !check_activity(observed, Read::OwnerEnded).iter().any(|row| row["line"].get("decided").is_some()),
        PERSON_STOP,
        "A must finish without a hold or control decision",
    );
    let before = check_activity(observed, Read::BHeld);
    let after: Vec<_> = check_activity(observed, Read::BEnded)
        .into_iter()
        .chain(check_activity(observed, Read::BEndedEarlier))
        .collect();
    let attempts =
        |rows: &[&Value]| rows.iter().filter_map(|row| number(row, "/line/run_began/attempt")).collect::<Vec<_>>();
    let previous = attempts(&before);
    let following = attempts(&after);
    step_check(
        misses,
        "5.2",
        number(held, "/chip/details/owner/details/attempt").is_some()
            && number(held, "/chip/details/owner/details/attempt")
                == number(still, "/chip/details/owner/details/attempt")
            && previous.first().copied() == number(held, "/chip/details/owner/details/attempt")
            && following.len() == previous.len() + 1
            && following.first().copied() == number(b, "/chip/details/owner/details/attempt")
            && following.get(1..) == Some(previous.as_slice())
            && following.first() > following.get(1)
            && after
                .iter()
                .find(|row| number(row, "/line/run_began/attempt").is_some())
                .and_then(|row| number(row, "/position"))
                .zip(number(still, "/activity"))
                .is_some_and(|(began, held_head)| began > held_head)
            && after.iter().all(|row| number(row, "/task") == Some(BOWNER))
            && after.windows(2).all(|pair| {
                number(pair[0], "/position").zip(number(pair[1], "/position")).is_some_and(|(new, old)| new > old)
            })
            && [Read::BHeld, Read::BEnded, Read::BEndedEarlier]
                .iter()
                .all(|read| pause_activity(observed, *read)["cut"] == false)
            && pause_activity(observed, Read::BHeld)["before"].is_null()
            && pause_activity(observed, Read::BEndedEarlier)["before"].is_null(),
        STOPPED_NEWS,
        "B's complete journal must add exactly one attempt after its unchanged held attempt",
    );
}

fn pause_children(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{H1, H1_CODER, H1_DOC, H1_REVIEWER, H1_WORDS, IMPLEMENTER, IMPLEMENTER_RESULT};
    let held = task_document(observed, Read::BHeld);
    let implementer = task_document(observed, Read::ImplementerEnded);
    let h1 = task_document(observed, Read::H1Ended);
    step_check(
        misses,
        "5.2",
        text(implementer, "/chip/phase") == "ended"
            && implementer.pointer("/result/ending/task/done").is_some_and(Value::is_object)
            && text(implementer, "/result/value/report/bytes") == IMPLEMENTER_RESULT,
        DELEGATES_GO_ON,
        "the implementer must finish its report under B",
    );
    step_check(
        misses,
        "5.2",
        text(h1, "/chip/phase") == "ended"
            // The change procedure returns Done(Change) only after landing;
            // ctl renders its commit as hex bytes, not the word "landed".
            && number(h1, "/result/ending/task/done/change/kind") == Some(1)
            && number(h1, "/result/value/change/part/connector") == Some(1)
            && text(h1, "/result/value/change/part/kind") == "result"
            && observed.kit.merged_green
            && observed.kit.main.get(H1_DOC.as_bytes()).is_some_and(|words| words == H1_WORDS.as_bytes()),
        DELEGATES_GO_ON,
        "h1 must end landed with its Markdown on kit's main, merged green",
    );
    let h1_activity = check_activity(observed, Read::H1Ended);
    step_check(
        misses,
        "5.2",
        h1_activity.iter().any(|row| {
            number(row, "/line/heard/from/task") == Some(H1_CODER)
                && row.pointer("/line/heard/kind/result/change").is_some_and(Value::is_object)
        }) && h1_activity.iter().any(|row| {
            number(row, "/line/heard/from/task") == Some(H1_REVIEWER)
                && number(row, "/line/heard/kind/result/verdict/code") == Some(1)
        }),
        DELEGATES_GO_ON,
        "h1 must receive its coder's change and its held reviewer's passing verdict",
    );
    let activity = check_activity(observed, Read::BStillHeld);
    let stopped = number(held, "/activity");
    // BHeld's activity head is read at Stop revision B_HOLD, before either
    // fake release. Each ending's journal position must be beyond that head;
    // BStillHeld follows both ended reads and precedes ctl Release.
    for child in [IMPLEMENTER, H1] {
        let heard = activity.iter().position(|row| {
            number(row, "/line/heard/from/task") == Some(child) && row.pointer("/line/heard/kind/result").is_some()
        });
        step_check(
            misses,
            "5.2",
            heard
                .and_then(|at| number(activity[at], "/position"))
                .zip(stopped)
                .is_some_and(|(heard, stop)| heard > stop && stop != 0),
            DELEGATES_GO_ON,
            &format!("B must retain task {child}'s ending after its Stop decision"),
        );
    }
}

fn pause_news(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{Agent, BOWNER_REPORT, H1, HELPER_CHARTER, HELPER_REASON, IMPLEMENTER_RESULT, PAUSE, PAUSE_REPORT};
    let b = task_document(observed, Read::BEnded);
    let helper = task_document(observed, Read::HelperEnded);
    let pause = task_document(observed, Read::PauseEnded);
    let queries: Vec<_> = observed.told.iter().filter(|query| query.agent == Agent::Bowner).collect();
    let resumed = queries.last();
    let label = format!("Result from task {H1}:\n");
    step_check(
        misses,
        "5.2",
        resumed.is_some_and(|query| contains(&query.words, IMPLEMENTER_RESULT) && contains(&query.words, &label))
            && queries
                .iter()
                .rev()
                .skip(1)
                .all(|query| !contains(&query.words, IMPLEMENTER_RESULT) && !contains(&query.words, &label))
            && resumed.is_some_and(|query| {
                calls::occurrences(&query.words, IMPLEMENTER_RESULT.as_bytes()) == 1
                    && calls::occurrences(&query.words, label.as_bytes()) == 1
            }),
        STOPPED_NEWS,
        "only B's query after Release must receive both endings once",
    );
    step_check(
        misses,
        "5.2",
        number(helper, "/chip/executor/agent/charter") == Some(HELPER_CHARTER)
            && number(helper, "/result/ending/task/cancelled/by/task") == Some(PAUSE)
            && text(helper, "/result/ending/task/cancelled/reason/bytes") == HELPER_REASON
            && text(helper, "/chip/phase") == "ended"
            && number(helper, "/chip/details/owner/details/live_delegates") == Some(0),
        OLD_HELPER,
        "the helper subtree must end cancelled by the director with its reason; charter retirement stays out",
    );
    for (task, expected, label) in [(b, BOWNER_REPORT, "B"), (pause, PAUSE_REPORT, "chat")] {
        step_check(
            misses,
            "5.2",
            text(task, "/chip/phase") == "ended" && text(task, "/result/value/report/bytes") == expected,
            STOPPED_NEWS,
            &format!("{label} must finish with its report"),
        );
    }
    step_check(
        misses,
        "5.2",
        number(pause, "/chip/task") == Some(PAUSE)
            && observed.told.iter().any(|query| query.agent == Agent::Pause && contains(&query.words, BOWNER_REPORT)),
        STOPPED_NEWS,
        "the chat must hear B's report before finishing",
    );
}

fn pause_activity(observed: &Observed, read: Read) -> &Value {
    let doc = document(observed, read);
    doc.get("activity").unwrap_or(doc)
}

#[cfg(test)]
mod pause_tests {
    use super::*;
    use crate::case::*;
    use serde_json::json;

    fn task(task: u64, parent: u64) -> Value {
        json!({"task":{"chip":{"task":task,"phase":"ended","details":{"owner":{"details":{"attempt":1,"live_delegates":0}}}},
            "project":PROJECT,"requester":{"task":parent},"hold":null,
            "result":{"ending":{"task":{"done":{}}}}},
            "activity":{"rows":[],"cut":false,"before":null,"after":null}})
    }
    fn documents() -> BTreeMap<Read, Value> {
        let mut documents = BTreeMap::from([
            (Read::ImplementerRunning, task(IMPLEMENTER, BOWNER)),
            (Read::H1Running, task(H1_REVIEWER, H1)),
            (Read::BHeld, task(BOWNER, PAUSE)),
            (Read::BStillHeld, task(BOWNER, PAUSE)),
            (Read::BEnded, task(BOWNER, PAUSE)),
            (
                Read::BEndedEarlier,
                json!({"activity":{"rows":[{"position":2,"line":{"run_began":{"attempt":1}}}],"before":null,"cut":false}}),
            ),
            (Read::OwnerEnded, task(OWNER, PAUSE)),
            (Read::HelperEnded, task(HELPER, PAUSE)),
            (Read::ImplementerEnded, task(IMPLEMENTER, BOWNER)),
            (Read::H1Ended, task(H1, BOWNER)),
            (Read::PauseEnded, task(PAUSE, 0)),
        ]);
        for read in [Read::ImplementerRunning, Read::H1Running] {
            let document = documents.get_mut(&read).expect("running");
            document["task"]["chip"]["phase"] = "running".into();
            document["task"]["chip"]["details"]["owner"]["phase"] = json!({"active":{"running":{"attempt":1}}});
            document["task"]["chip"]["details"]["owner"]["details"]["observation"] = json!({"work":"writing"});
            document["task"]["result"] = Value::Null;
        }
        for read in [Read::BHeld, Read::BStillHeld] {
            let document = documents.get_mut(&read).expect("held");
            document["task"]["chip"]["phase"] = "held".into();
            document["task"]["hold"] = json!({"owner":{"revision":B_HOLD,"reason":{"stopped_by":{"party":1}}}});
        }
        documents.get_mut(&Read::BHeld).expect("held")["task"]["activity"] = 7.into();
        documents.get_mut(&Read::BStillHeld).expect("still")["task"]["activity"] = 10.into();
        documents.get_mut(&Read::BHeld).expect("held")["activity"]["rows"] =
            json!([{"position":2,"line":{"run_began":{"attempt":1}}}]);
        documents.get_mut(&Read::BStillHeld).expect("still")["activity"]["rows"] = json!([
            {"position":10,"line":{"heard":{"from":{"task":H1},"kind":{"result":"done"}}}},
            {"position":8,"line":{"heard":{"from":{"task":IMPLEMENTER},"kind":{"result":"done"}}}},
            {"position":7,"line":{"milestone":{"position":7}}},
        ]);
        documents.get_mut(&Read::BEnded).expect("B")["activity"]["rows"] =
            json!([{"position":12,"line":{"run_began":{"attempt":2}}}]);
        documents.get_mut(&Read::BEnded).expect("B")["task"]["chip"]["details"]["owner"]["details"]["attempt"] =
            2.into();
        for (read, report) in [
            (Read::OwnerEnded, OWNER_REPORT),
            (Read::BEnded, BOWNER_REPORT),
            (Read::ImplementerEnded, IMPLEMENTER_RESULT),
            (Read::PauseEnded, PAUSE_REPORT),
        ] {
            documents.get_mut(&read).expect("report")["task"]["result"]["value"] = json!({"report":{"bytes":report}});
        }
        let change = json!({"kind":1,"part":{"connector":1,"kind":"result","key":{"number":1}}});
        documents.get_mut(&Read::H1Ended).expect("h1")["task"]["result"] =
            json!({"ending":{"task":{"done":{"change":change}}},"value":{"change":change}});
        documents.get_mut(&Read::HelperEnded).expect("helper")["task"]["result"]["ending"] =
            json!({"task":{"cancelled":{"by":{"task":PAUSE},"reason":{"bytes":HELPER_REASON}}}});
        documents.get_mut(&Read::OwnerEnded).expect("A")["activity"]["rows"] = json!([
            {"position":5,"line":{"milestone":{"position":5}}},
            {"position":2,"line":{"run_began":{"attempt":1}}},
            {"position":1,"line":{"milestone":{"position":1}}},
        ]);
        documents.get_mut(&Read::H1Ended).expect("h1")["activity"]["rows"] = json!([
            {"position":3,"line":{"heard":{"from":{"task":H1_REVIEWER},"kind":{"result":{"verdict":{"code":1}}}}}},
            {"position":2,"line":{"heard":{"from":{"task":H1_CODER},"kind":{"result":{"change":{}}}}}},
        ]);
        documents.get_mut(&Read::HelperEnded).expect("helper")["task"]["chip"]["executor"] =
            json!({"agent":{"charter":HELPER_CHARTER}});
        for document in documents.values_mut() {
            let task = number(document, "/task/chip/task").unwrap_or(BOWNER);
            for row in document["activity"]["rows"].as_array_mut().expect("rows") {
                *row = json!({"event":{"owner":{"source":{"task":task,"position":row["position"].clone(),"line":row["line"].clone()}}}});
            }
        }
        documents
    }
    fn met() -> Observed {
        let documents = documents();
        let told = |agent, words: String| Told {
            system: Box::new([]),
            agent,
            words: words.into_bytes().into(),
            history: Box::new([]),
            model: Box::new([]),
            credential: None,
        };
        Observed {
            documents,
            commands: Vec::new(),
            told: vec![
                told(Agent::Bowner, "Delegated".into()),
                told(Agent::Bowner, format!("{IMPLEMENTER_RESULT}\nResult from task {H1}:\nlanded")),
                told(Agent::Pause, BOWNER_REPORT.into()),
            ],
            kit: Kit {
                main: BTreeMap::from([(H1_DOC.as_bytes().to_vec(), H1_WORDS.as_bytes().to_vec())]),
                merged_green: true,
            },
        }
    }
    #[test]
    fn pause_checkpoint_met() {
        assert_met(&step_5_2(&met()));
    }

    #[test]
    fn every_required_document_field_rejects_a_mutation() {
        let required = [
            (Read::BHeld, "/task/chip/task"),
            (Read::BHeld, "/task/project"),
            (Read::BHeld, "/task/requester/task"),
            (Read::BHeld, "/task/chip/phase"),
            (Read::BHeld, "/task/hold/owner/revision"),
            (Read::BHeld, "/task/hold/owner/reason/stopped_by/party"),
            (Read::BStillHeld, "/task/chip/phase"),
            (Read::BStillHeld, "/task/hold/owner/revision"),
            (Read::BStillHeld, "/task/chip/details/owner/details/attempt"),
            (Read::BEnded, "/task/chip/details/owner/details/attempt"),
            (Read::BHeld, "/activity/cut"),
            (Read::BEndedEarlier, "/activity/cut"),
            (Read::BEndedEarlier, "/activity/rows/0/event/owner/source/line/run_began/attempt"),
            (Read::BEnded, "/activity/cut"),
            (Read::BEnded, "/task/result/value/report/bytes"),
            (Read::OwnerEnded, "/task/result/value/report/bytes"),
            (Read::OwnerEnded, "/task/chip/phase"),
            (Read::ImplementerEnded, "/task/result/value/report/bytes"),
            (Read::ImplementerEnded, "/task/result/ending/task/done"),
            (Read::H1Ended, "/task/result/ending/task/done/change/kind"),
            (Read::H1Ended, "/task/result/value/change/part/connector"),
            (Read::H1Ended, "/task/result/value/change/part/kind"),
            (Read::BStillHeld, "/activity/rows/0/event/owner/source/line/heard/from/task"),
            (Read::BStillHeld, "/activity/rows/1/event/owner/source/line/heard/from/task"),
            (Read::BHeld, "/task/activity"),
            (Read::BStillHeld, "/activity/rows/0/event/owner/source/position"),
            (Read::BStillHeld, "/activity/rows/1/event/owner/source/position"),
            (Read::H1Ended, "/activity/rows/0/event/owner/source/line/heard/from/task"),
            (Read::H1Ended, "/activity/rows/0/event/owner/source/line/heard/kind/result/verdict/code"),
            (Read::H1Ended, "/activity/rows/1/event/owner/source/line/heard/from/task"),
            (Read::OwnerEnded, "/activity/rows/0/event/owner/source/line/milestone/position"),
            (Read::OwnerEnded, "/activity/rows/1/event/owner/source/line/run_began/attempt"),
            (Read::HelperEnded, "/task/chip/executor/agent/charter"),
            (Read::HelperEnded, "/task/result/ending/task/cancelled/by/task"),
            (Read::HelperEnded, "/task/result/ending/task/cancelled/reason/bytes"),
            (Read::HelperEnded, "/task/chip/details/owner/details/live_delegates"),
            (Read::PauseEnded, "/task/result/value/report/bytes"),
            (Read::PauseEnded, "/task/chip/task"),
        ];
        for (read, path) in required {
            let mut observed = met();
            *observed.documents.get_mut(&read).expect("doc").pointer_mut(path).expect("field") = Value::Null;
            assert!(!step_5_2(&observed).is_empty(), "{read:?}{path}");
        }
        for read in [Read::ImplementerRunning, Read::H1Running] {
            for path in [
                "/task/chip/task",
                "/task/project",
                "/task/requester/task",
                "/task/chip/phase",
                "/task/chip/details/owner/phase/active/running/attempt",
                "/task/chip/details/owner/details/attempt",
                "/task/chip/details/owner/details/observation/work",
            ] {
                let mut observed = met();
                *observed.documents.get_mut(&read).expect("running").pointer_mut(path).expect("field") = Value::Null;
                assert!(!step_5_2(&observed).is_empty(), "{read:?}{path}");
            }
            let mut observed = met();
            observed.documents.get_mut(&read).expect("running")["task"]["result"] = json!({"done":{}});
            assert!(!step_5_2(&observed).is_empty(), "{read:?} already ended");
        }
        for &(read, _) in crate::case::acts::READS_5_2 {
            let mut observed = met();
            observed.documents.remove(&read);
            assert!(!step_5_2(&observed).is_empty(), "missing {read:?}");
        }
    }
    #[test]
    fn news_cannot_arrive_early_twice_or_without_both_endings() {
        let mut early = met();
        early.told[0].words = early.told[1].words.clone();
        let mut missing_h1 = met();
        missing_h1.told[1].words = IMPLEMENTER_RESULT.as_bytes().into();
        let mut twice = met();
        twice.told[1].words =
            format!("{} {IMPLEMENTER_RESULT}", String::from_utf8_lossy(&twice.told[1].words)).into_bytes().into();
        let mut twice_h1 = met();
        twice_h1.told[1].words =
            format!("{} Result from task {H1}:\n", String::from_utf8_lossy(&twice_h1.told[1].words))
                .into_bytes()
                .into();
        let mut no_chat_delivery = met();
        no_chat_delivery.told[2].words = Box::new([]);
        let mut no_queries = met();
        no_queries.told.clear();
        for (label, observed) in [
            ("early", early),
            ("missing h1", missing_h1),
            ("twice", twice),
            ("twice h1", twice_h1),
            ("chat", no_chat_delivery),
            ("no queries", no_queries),
        ] {
            assert!(!step_5_2(&observed).is_empty(), "{label}");
        }
    }
    #[test]
    fn stop_order_a_hold_and_the_merge_are_checked() {
        let mut red = met();
        red.kit.merged_green = false;
        let mut missing_file = met();
        missing_file.kit.main.clear();
        let mut early = met();
        early.documents.get_mut(&Read::BHeld).expect("held")["task"]["activity"] = 10.into();
        let mut a_held = met();
        a_held.documents.get_mut(&Read::OwnerEnded).expect("A")["task"]["hold"] = json!({"owner":{}});
        let mut a_released = met();
        a_released.documents.get_mut(&Read::OwnerEnded).expect("A")["activity"]["rows"]
            .as_array_mut()
            .expect("rows")
            .push(json!({"event":{"owner":{"source":{"line":{"milestone":{"position":3}}}}}}));
        let mut a_retried = met();
        a_retried.documents.get_mut(&Read::OwnerEnded).expect("A")["activity"]["rows"]
            .as_array_mut()
            .expect("rows")
            .push(json!({"event":{"owner":{"source":{"line":{"run_began":{"attempt":3}}}}}}));
        let mut incomplete = met();
        incomplete.documents.get_mut(&Read::BEndedEarlier).expect("earlier")["activity"]["before"] =
            json!({"value":"older"});
        let mut unordered = met();
        unordered.documents.get_mut(&Read::BEndedEarlier).expect("earlier")["activity"]["rows"][0]["event"]["owner"]
            ["source"]["position"] = 13.into();
        for (label, observed) in [
            ("red", red),
            ("missing file", missing_file),
            ("early", early),
            ("A held", a_held),
            ("A released", a_released),
            ("A retried", a_retried),
            ("incomplete", incomplete),
            ("unordered", unordered),
        ] {
            assert!(!step_5_2(&observed).is_empty(), "{label}");
        }
    }
}

// Act 2, step 5.5 (domain/testing.md, 5.3)
const PLAN_MESSAGES: Promise = Promise::Must("a question and its answer (exact)");
const PLAN_DEADLINES: Promise = Promise::Must("a mission and sub-owner carry their deadlines");
const PLAN_DEPTH: Promise = Promise::Must("a sub-owner cannot start one of its own (exact)");
const PLAN_FRESH: Promise = Promise::Must("the fresh owner recalls its plan");
const PLAN_ATTEMPTS: Promise = Promise::Add("one run per task, so there is no old session to stop");

/// Only the carried deadlines are checked; their effect and phase policy stay out.
#[must_use]
pub fn step_5_5(observed: &Observed) -> Vec<Miss> {
    use super::{
        PLAN_ANSWER, PLAN_DIRECTOR, PLAN_NOTE, PLAN_NOTE_NUMBER, PLAN_REPORT, PLAN_WORDS, PLANNER, PLANNER_REPORT,
        SUBOWNER, SUBOWNER_REPORT,
    };
    let mut misses = Vec::new();
    for (read, expected, parent, report) in [
        (Read::PlanEnded, PLAN_DIRECTOR, None, PLAN_REPORT),
        (Read::PlannerEnded, PLANNER, Some(PLAN_DIRECTOR), PLANNER_REPORT),
        (Read::SubownerEnded, SUBOWNER, Some(PLANNER), SUBOWNER_REPORT),
    ] {
        let task = task_document(observed, read);
        step_check(
            &mut misses,
            "5.5",
            number(task, "/chip/task") == Some(expected)
                && number(task, "/project") == Some(PROJECT)
                && parent.is_none_or(|parent| number(task, "/requester/task") == Some(parent))
                && text(task, "/chip/phase") == "ended"
                && text(task, "/result/value/report/bytes") == report,
            PLAN_MESSAGES,
            &format!("{read:?} must identify the original ended task and report"),
        );
    }
    plan_authority(observed, &mut misses);
    plan_messages(observed, &mut misses);
    let answer_at = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Planner && user_words(told).any(|words| contains(words, PLAN_ANSWER)));
    let pending = document(observed, Read::PlanAnswerPending);
    step_check(
        &mut misses,
        "5.5",
        number(pending, "/task") == Some(PLAN_DIRECTOR)
            && text(pending, "/writing/text") == super::PLAN_ANSWER_PENDING
            && pending.pointer("/writing/cut") == Some(&Value::Bool(false)),
        PLAN_FRESH,
        "the director's live answer piece must be held until A's first run has parked",
    );
    let parked = task_document(observed, Read::PlannerParked);
    step_check(
        &mut misses,
        "5.5",
        number(parked, "/chip/task") == Some(PLANNER)
            && text(parked, "/chip/phase") == "idle"
            && answer_at.is_some_and(|at| plan_fresh(&observed.told[at]))
            && plan_note_recalled(observed, Agent::Planner, PLAN_NOTE_NUMBER, PLAN_NOTE, PLAN_WORDS)
            && answer_at.is_some_and(|at| {
                plan_tool_results(observed, Agent::Planner, b"recall").iter().any(|(recalled, _, _)| *recalled > at)
            }),
        PLAN_FRESH,
        "A must park, start fresh on its answer, and recall its written plan in that run",
    );
    step_check(
        &mut misses,
        "5.5",
        plan_attempts(observed, &[Read::PlannerEnded, Read::PlannerEarlier, Read::PlannerFirst], PLANNER, 2),
        PLAN_ATTEMPTS,
        "A's complete ordered activity must end each attempt before the next begins",
    );
    misses
}

fn plan_authority(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{A_DEADLINE, A1_DEADLINE, PLANNER, SPEND_DEFICIT, SUBOWNER, SUBOWNER_REFUSAL};
    let a = task_document(observed, Read::PlannerEnded);
    let a1 = task_document(observed, Read::SubownerEnded);
    step_check(
        misses,
        "5.5",
        number(a, "/authority/authority/deadline") == Some(A_DEADLINE)
            && number(a1, "/authority/authority/deadline") == Some(A1_DEADLINE)
            && number(a1, "/authority/authority/deadline")
                .zip(number(a, "/authority/authority/deadline"))
                .is_some_and(|(child, owner)| child < owner)
            && number(a, "/priority") == Some(1)
            && number(a, "/goal") == Some(PLANNER),
        PLAN_DEADLINES,
        "A must be tracked at priority 1 and both delegated deadlines must be carried, A1's before A's",
    );
    let refused = plan_tool_results(observed, Agent::Subowner, b"delegate");
    step_check(
        misses,
        "5.5",
        number(a1, "/authority/authority/delegation/depth") == Some(0)
            && refused.len() == 1
            && refused
                .iter()
                .all(|(_, _, output)| contains(output, SUBOWNER_REFUSAL) && !contains(output, SPEND_DEFICIT))
            && number(a1, "/chip/details/owner/details/live_delegates") == Some(0)
            && a1["delegates"].as_array().is_some_and(Vec::is_empty)
            && observed.documents.values().all(|doc| number(&doc["task"], "/requester/task") != Some(SUBOWNER)),
        PLAN_DEPTH,
        "A1 must hear its delegation refusal for depth alone, with depth 0 and no child in any read document",
    );
}

fn plan_messages(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{PLAN_ANSWER, PLAN_ANSWER_NUMBER, PLAN_DIRECTOR, PLAN_QUESTION_NUMBER, PLAN_WORDS, PLANNER};
    let questions = plan_tool_results(observed, Agent::Planner, b"message");
    let answers = plan_tool_results(observed, Agent::Plan, b"message");
    let question_at = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Plan && user_words(told).any(|words| contains(words, PLAN_WORDS)));
    let answer_at = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Planner && user_words(told).any(|words| contains(words, PLAN_ANSWER)));
    step_check(
        misses,
        "5.5",
        questions.len() == 1
            && questions.iter().all(|(_, args, output)| {
                args["target"] == PLAN_DIRECTOR
                    && args["form"] == "question"
                    && args["words"] == PLAN_WORDS
                    && contains(output, &format!("Message {PLAN_QUESTION_NUMBER} sent"))
            })
            && answers.len() == 1
            && answers.iter().all(|(_, args, output)| {
                args["target"] == PLANNER
                    && args["form"] == "answer"
                    && args["question"] == PLAN_QUESTION_NUMBER
                    && args["words"] == PLAN_ANSWER
                    && contains(output, &format!("Message {PLAN_ANSWER_NUMBER} sent"))
            })
            && [Read::PlannerEnded, Read::PlannerEarlier, Read::PlannerFirst]
                .iter()
                .flat_map(|read| check_activity(observed, *read))
                .filter(|row| {
                    number(row, "/line/words/message") == Some(PLAN_ANSWER_NUMBER)
                        && number(row, "/line/words/by/task") == Some(PLAN_DIRECTOR)
                        && number(row, "/line/words/kind/answer/question") == Some(PLAN_QUESTION_NUMBER)
                        && text(row, "/line/words/words") == PLAN_ANSWER
                })
                .count()
                == 1
            && plan_deliveries(observed, Agent::Plan, PLAN_WORDS) == 1
            && plan_deliveries(observed, Agent::Planner, PLAN_ANSWER) == 1
            && question_at.zip(answer_at).is_some_and(|(question, answer)| question < answer),
        PLAN_MESSAGES,
        "the numbered plan question and its answer must each be sent and delivered once, in order",
    );
}

// Histories repeat earlier turns. Count new user input once within a growing
// history, and count it again if delivered to another fresh conversation.
fn plan_deliveries(observed: &Observed, agent: Agent, words: &str) -> usize {
    use skein_fake_llm_domain::api::{Part, Role};
    let mut previous: &[skein_fake_llm_domain::api::Message] = &[];
    let mut count = 0;
    for told in observed.told.iter().filter(|told| told.agent == agent) {
        let offset = if told.history.starts_with(previous) { previous.len() } else { 0 };
        count += told.history[offset..]
            .iter()
            .filter(|message| message.role == Role::User)
            .flat_map(|message| &message.parts)
            .map(|part| match part {
                Part::Text { text } | Part::ToolOutput { output: text, .. } => {
                    calls::occurrences(text, words.as_bytes())
                }
                Part::Opaque { .. } | Part::ToolCall { .. } => 0,
            })
            .sum::<usize>();
        previous = &told.history;
    }
    count
}

// Match actual tool outputs to their assistant call identities, never to prose
// repeating the expected words. A replayed prefix is still one completed call.
fn plan_tool_results(observed: &Observed, agent: Agent, tool: &[u8]) -> Vec<(usize, Value, Vec<u8>)> {
    use skein_fake_llm_domain::api::{Part, Role};
    let mut seen = BTreeSet::new();
    let mut results = Vec::new();
    for (at, told) in observed.told.iter().enumerate().filter(|(_, told)| told.agent == agent) {
        let mut calls = BTreeMap::new();
        for message in &told.history {
            for part in &message.parts {
                match part {
                    Part::ToolCall { id, name, arguments }
                        if message.role == Role::Assistant && name.as_ref() == tool =>
                    {
                        calls.insert(id.as_ref(), arguments);
                    }
                    Part::ToolOutput { id, output, .. } if message.role == Role::User => {
                        if let Some(arguments) = calls.get(id.as_ref())
                            && let Ok(args) = serde_json::from_slice(arguments)
                            && seen.insert(id.to_vec())
                        {
                            results.push((at, args, output.to_vec()));
                        }
                    }
                    Part::Text { .. } | Part::Opaque { .. } | Part::ToolCall { .. } | Part::ToolOutput { .. } => {}
                }
            }
        }
    }
    results
}

fn plan_fresh(told: &Told) -> bool {
    use skein_fake_llm_domain::api::{Part, Role};
    !told.history.is_empty()
        && told.history.iter().all(|message| {
            message.role == Role::User && message.parts.iter().all(|part| matches!(part, Part::Text { .. }))
        })
}

fn plan_note_recalled(observed: &Observed, agent: Agent, note: u64, description: &str, body: &str) -> bool {
    let writes = plan_tool_results(observed, agent, b"note");
    let recalls = plan_tool_results(observed, agent, b"recall");
    writes.iter().any(|(written, args, output)| {
        args["scope"] == serde_json::json!({"kind":"project","project":PROJECT})
            && args["description"] == description
            && args["body"] == body
            && contains(output, &format!("Note {note} revision 1 written"))
            && recalls.iter().any(|(recalled, args, output)| {
                written < recalled
                    && args["kind"] == "name"
                    && args["name"] == note
                    && contains(output, &format!("Note {note} revision 1: {description}\n{body}\n"))
            })
    })
}

fn plan_attempts(observed: &Observed, reads: &[Read], task: u64, minimum: usize) -> bool {
    let rows: Vec<_> = reads.iter().flat_map(|read| check_activity(observed, *read)).collect();
    let complete = reads.iter().all(|read| pause_activity(observed, *read)["cut"] == false)
        && reads.last().is_some_and(|read| pause_activity(observed, *read)["before"] == Value::Null)
        && reads.first().is_some_and(|read| pause_activity(observed, *read)["after"] == Value::Null);
    let ordered = rows
        .windows(2)
        .all(|pair| number(pair[0], "/position").zip(number(pair[1], "/position")).is_some_and(|(new, old)| new > old));
    let mut open = None;
    let mut attempts = BTreeSet::new();
    let mut valid = complete && ordered && !rows.is_empty();
    for row in rows.iter().rev() {
        valid &= number(row, "/task") == Some(task);
        if let Some(began) = row["line"].get("run_began") {
            valid &= open.is_none();
            open = number(began, "/attempt");
            valid &= open.is_some_and(|attempt| attempts.insert(attempt));
        }
        if let Some(ended) = row["line"].get("run_ended") {
            valid &= open.is_some() && open.take() == number(ended, "/attempt");
        }
    }
    valid && open.is_none() && attempts.len() >= minimum
}

#[cfg(test)]
mod plan_tests {
    use super::*;
    use crate::case::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn input(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn tool(id: &str, name: &str, args: &Value, output: &str) -> [Message; 2] {
        [
            Message {
                role: Role::Assistant,
                parts: Box::new([Part::ToolCall {
                    id: id.as_bytes().into(),
                    name: name.as_bytes().into(),
                    arguments: serde_json::to_vec(args).expect("args").into(),
                }]),
            },
            Message {
                role: Role::User,
                parts: Box::new([Part::ToolOutput {
                    id: id.as_bytes().into(),
                    output: output.as_bytes().into(),
                    is_error: false,
                }]),
            },
        ]
    }
    fn told(agent: Agent, history: &[Message]) -> Told {
        Told::new(agent.cue(), b"fake", None, history).expect("agent cue")
    }
    fn met() -> Observed {
        let mut documents = BTreeMap::new();
        for (read, task, parent, report) in [
            (Read::PlanEnded, PLAN_DIRECTOR, 0, PLAN_REPORT),
            (Read::PlannerEnded, PLANNER, PLAN_DIRECTOR, PLANNER_REPORT),
            (Read::SubownerEnded, SUBOWNER, PLANNER, SUBOWNER_REPORT),
        ] {
            documents.insert(read, json!({"task":{
                "chip":{"task":task,"phase":"ended","details":{"owner":{"details":{"live_delegates":0}}}},
                "project":PROJECT,"requester":{"task":parent},"priority":1,"goal":PLANNER,"delegates":[],
                "authority":{"authority":{"deadline":if task == SUBOWNER { A1_DEADLINE } else { A_DEADLINE },"delegation":{"depth":0}}},
                "result":{"value":{"report":{"bytes":report}}}
            }}));
        }
        documents.insert(Read::PlannerParked, json!({"task":{"chip":{"task":PLANNER,"phase":"idle"}}}));
        documents.insert(
            Read::PlanAnswerPending,
            json!({"task":PLAN_DIRECTOR,
            "writing":{"text":PLAN_ANSWER_PENDING,"cut":false}}),
        );
        let rows: Vec<_> = [
            json!({"run_ended":{"attempt":2,"end":"finished"}}),
            json!({"run_began":{"attempt":2}}),
            json!({"words":{"message":PLAN_ANSWER_NUMBER,"by":{"task":PLAN_DIRECTOR},
                "kind":{"answer":{"question":PLAN_QUESTION_NUMBER}},"words":PLAN_ANSWER}}),
            json!({"run_ended":{"attempt":1,"end":"parked"}}),
            json!({"run_began":{"attempt":1}}),
        ]
        .into_iter()
        .enumerate()
        .map(|(at, line)| {
            json!({"event":{"owner":{"source":{
                "task":PLANNER,"position":5-at,"line":line
            }}}})
        })
        .collect();
        documents.get_mut(&Read::PlannerEnded).expect("A")["activity"] =
            json!({"cut":false,"before":{"value":"older"},"after":null,"rows":rows[..2]});
        documents.insert(
            Read::PlannerEarlier,
            json!({"cut":false,"before":{"value":"first"},"after":{"value":"newer"},"rows":rows[2..4]}),
        );
        documents
            .insert(Read::PlannerFirst, json!({"cut":false,"before":null,"after":{"value":"newer"},"rows":rows[4..]}));
        let mut opening = tool(
            "note",
            "note",
            &json!({"scope":{"kind":"project","project":PROJECT},
            "description":PLAN_NOTE,"body":PLAN_WORDS}),
            "Note 1 revision 1 written",
        )
        .to_vec();
        let noted = told(Agent::Planner, &opening);
        opening.extend(tool(
            "question",
            "message",
            &json!({"target":PLAN_DIRECTOR,"form":"question","words":PLAN_WORDS}),
            &format!("Message {PLAN_QUESTION_NUMBER} sent"),
        ));
        let mut director = vec![input(PLAN_WORDS)];
        director.extend(tool(
            "answer",
            "message",
            &json!({"target":PLANNER,"form":"answer","question":PLAN_QUESTION_NUMBER,
            "words":PLAN_ANSWER}),
            &format!("Message {PLAN_ANSWER_NUMBER} sent"),
        ));
        let mut next = vec![input(PLAN_ANSWER)];
        let fresh = told(Agent::Planner, &next);
        next.extend(tool(
            "recall",
            "recall",
            &json!({"kind":"name","name":PLAN_NOTE_NUMBER}),
            &format!("Note {PLAN_NOTE_NUMBER} revision 1: {PLAN_NOTE}\n{PLAN_WORDS}\n"),
        ));
        Observed {
            documents,
            commands: Vec::new(),
            kit: Kit { main: BTreeMap::new(), merged_green: true },
            told: vec![
                noted,
                told(Agent::Planner, &opening),
                told(Agent::Plan, &director),
                fresh,
                told(Agent::Planner, &next),
                told(Agent::Subowner, &tool("refused", "delegate", &json!({"batch":[{}]}), SUBOWNER_REFUSAL)),
            ],
        }
    }
    #[test]
    fn answered_plan_meets_the_checkpoint_and_replayed_prefixes_are_one_delivery() {
        let mut observed = met();
        assert_met(&step_5_5(&observed));
        observed.told.push(observed.told[4].clone());
        assert_met(&step_5_5(&observed));
    }
    #[test]
    fn every_required_document_field_rejects_a_mutation() {
        for (read, paths) in [
            (
                Read::PlanEnded,
                vec!["/task/chip/task", "/task/project", "/task/chip/phase", "/task/result/value/report/bytes"],
            ),
            (
                Read::PlannerEnded,
                vec![
                    "/task/chip/task",
                    "/task/project",
                    "/task/requester/task",
                    "/task/chip/phase",
                    "/task/result/value/report/bytes",
                    "/task/priority",
                    "/task/goal",
                    "/task/authority/authority/deadline",
                    "/activity/cut",
                    "/activity/after",
                ],
            ),
            (
                Read::SubownerEnded,
                vec![
                    "/task/chip/task",
                    "/task/project",
                    "/task/requester/task",
                    "/task/chip/phase",
                    "/task/result/value/report/bytes",
                    "/task/authority/authority/deadline",
                    "/task/authority/authority/delegation/depth",
                    "/task/chip/details/owner/details/live_delegates",
                    "/task/delegates",
                ],
            ),
            (Read::PlannerParked, vec!["/task/chip/task", "/task/chip/phase"]),
            (
                Read::PlannerEarlier,
                vec![
                    "/cut",
                    "/rows/0/event/owner/source/line/words/message",
                    "/rows/0/event/owner/source/line/words/by/task",
                    "/rows/0/event/owner/source/line/words/kind/answer/question",
                    "/rows/0/event/owner/source/line/words/words",
                ],
            ),
            (Read::PlannerFirst, vec!["/cut", "/before"]),
            (Read::PlanAnswerPending, vec!["/task", "/writing/text", "/writing/cut"]),
        ] {
            for path in paths {
                let mut observed = met();
                let value = observed.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field");
                *value = if value.is_null() { json!("unexpected cursor") } else { Value::Null };
                assert!(!step_5_5(&observed).is_empty(), "{read:?}{path}");
            }
        }
        for &(read, _) in acts::READS_5_5 {
            let mut observed = met();
            observed.documents.remove(&read);
            assert!(!step_5_5(&observed).is_empty(), "missing {read:?}");
        }
    }
    #[test]
    fn missing_duplicate_reordered_or_resumed_messages_fail() {
        for at in 0..met().told.len() {
            let mut observed = met();
            observed.told.remove(at);
            // The first note result also appears in the opening's growing prefix.
            if at != 0 {
                assert!(!step_5_5(&observed).is_empty(), "missing query {at}");
            }
        }
        let mut duplicate = met();
        duplicate.told[2].history = [duplicate.told[2].history.to_vec(), vec![input(PLAN_WORDS)]].concat().into();
        let mut duplicate_answer = met();
        duplicate_answer.told[3].history =
            [duplicate_answer.told[3].history.to_vec(), vec![input(PLAN_ANSWER)]].concat().into();
        let mut early = met();
        early.told.swap(2, 3);
        let mut resumed = met();
        resumed.told[3].history = resumed.told[1].history.clone();
        resumed.told[3].history = [resumed.told[3].history.to_vec(), vec![input(PLAN_ANSWER)]].concat().into();
        let mut wrong_number = met();
        if let Part::ToolCall { arguments, .. } = &mut wrong_number.told[2].history[1].parts[0] {
            *arguments =
                serde_json::to_vec(&json!({"target":PLANNER,"form":"answer","question":99,"words":PLAN_ANSWER}))
                    .expect("args")
                    .into();
        }
        for observed in [duplicate, duplicate_answer, early, resumed, wrong_number] {
            assert!(!step_5_5(&observed).is_empty());
        }
    }
    #[test]
    fn refusal_must_be_tool_output_and_no_child_may_be_made() {
        let mut prose = met();
        prose.told[5].history = vec![input(SUBOWNER_REFUSAL)].into();
        let mut child = met();
        child.documents.get_mut(&Read::SubownerEnded).expect("A1")["task"]["delegates"] = json!([{"chip":{"task":4}}]);
        let mut wrong_parent = met();
        wrong_parent.documents.get_mut(&Read::PlanEnded).expect("director")["task"]["requester"] =
            json!({"task":SUBOWNER});
        let mut forged_output = met();
        if let Part::ToolOutput { id, .. } = &mut forged_output.told[5].history[1].parts[0] {
            *id = b"unmatched".as_slice().into();
        }
        let mut spend = met();
        if let Part::ToolOutput { output, .. } = &mut spend.told[5].history[1].parts[0] {
            *output = format!("{SUBOWNER_REFUSAL}spend allowance; ").into_bytes().into();
        }
        for observed in [prose, child, wrong_parent, forged_output, spend] {
            assert!(!step_5_5(&observed).is_empty());
        }
    }
    #[test]
    fn plan_recall_must_follow_its_write_and_attempts_cannot_overlap() {
        let mut early = met();
        early.told.swap(0, 4);
        early.told[1].history = Box::new([]);
        let mut overlap = met();
        overlap.documents.get_mut(&Read::PlannerEarlier).expect("older")["rows"][0]["event"]["owner"]["source"]["line"] =
            json!({"run_began":{"attempt":2}});
        let mut unordered = met();
        unordered.documents.get_mut(&Read::PlannerEarlier).expect("older")["rows"][0]["event"]["owner"]["source"]["position"] =
            json!(99);
        let mut unended = met();
        unended.documents.get_mut(&Read::PlannerEnded).expect("A")["activity"]["rows"][0]["event"]["owner"]["source"]
            ["line"] = json!({});
        for observed in [early, overlap, unordered, unended] {
            assert!(!step_5_5(&observed).is_empty());
        }
    }
}
