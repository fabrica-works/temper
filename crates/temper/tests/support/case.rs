//! The reference case on the cluster's processes (domain/testing.md, 2.6 and 5.3).
use super::{Ended, Scratch, WITHIN, cluster, ctl};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Script};
use skein_io::kernel::Exit;
use skein_lib::{Time, base64};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use temper_real_loop_world::{
    calls,
    case::{
        acts,
        oracle::{Kit, Observed, Told},
    },
    forge::Fixture,
};

pub struct Run {
    calls: Vec<Ended>,
    told: Vec<Told>,
    kit: Kit,
    workers: BTreeMap<u64, String>,
    steps: Vec<calls::Step>,
}
impl Run {
    #[must_use]
    pub fn calls(&self) -> &[Ended] {
        &self.calls
    }
    #[must_use]
    pub fn observed(&self, reads: &[(acts::Read, usize)]) -> Observed {
        let documents = reads
            .iter()
            .filter_map(|(read, position)| {
                let call = self.calls.get(*position)?;
                if call.exit != Some(Exit::Code(0)) {
                    return None;
                }
                Some((*read, serde_json::from_slice(&call.stdout).expect("case document")))
            })
            .collect();
        Observed { documents, told: self.told.clone(), commands: Vec::new(), kit: self.kit.clone() }
            .with_commands(&self.steps)
    }
    /// Placement only where an act's document retains a live observation channel.
    #[must_use]
    pub fn placed(&self) -> &BTreeMap<u64, String> {
        &self.workers
    }
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("prepared document")).expect("JSON")
}
fn write(path: &Path, document: &Value) {
    super::write_private(path, &serde_json::to_vec(document).expect("JSON"));
}
fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("case UTF-8")
}
fn encode(scripts: &[Script]) -> Value {
    scripts
        .iter()
        .map(|script| {
            let turns: Vec<_> = script
                .turns
                .iter()
                .map(|turn| {
                    let lines: Vec<_> = turn
                        .lines
                        .iter()
                        .map(|line| match line {
                            Line::Text { text: words } => json!({"text": text(words)}),
                            Line::Call { name, arguments } => json!({
                                "call": text(name),
                                "arguments": serde_json::from_slice::<Value>(arguments).expect("arguments")
                            }),
                            other @ (Line::Fail { .. } | Line::Pieces { .. } | Line::Opaque { .. }) => {
                                panic!("case script line unsupported: {other:?}")
                            }
                        })
                        .collect();
                    let finish = match turn.finish {
                        Finish::Stop => "stop",
                        Finish::ToolCalls => "tool_calls",
                        Finish::Length => "length",
                        Finish::ContentFilter => "content_filter",
                    };
                    json!({"lines": lines, "finish": finish, "tokens": turn.tokens})
                })
                .collect();
            json!({"cue": text(&script.cue), "turns": turns})
        })
        .collect()
}
/// Bring lab's combined document to tests/real-loop/src/fixtures.rs's base
/// before applying the case fixture once, as that world's cluster does.
fn base(value: &mut Value, charter: &Path) {
    // The case owns its charters: fixture_placed appends director charter 2,
    // which must not collide with a charter added to the lab's base.
    let case: Value = serde_json::from_slice(&jig_channel_world::fixtures::engine_document()).expect("case base");
    value["engine"]["charters"] = case["engine"]["charters"].clone();
    value["engine"]["charters"][0]["waiting_ms"] = 1.into();
    value["engine"]["templates"][0]["ended_by"] = "result".into();
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        if name.as_ref() == b"charters/chat.txt" {
            super::write_private(charter, bytes.expose());
        }
    }
}
/// Departures from the description's documents: the case uses its own forge
/// and records-only setup until the lab describes one; the inert spare at id 1
/// goes when start-time adoption uses the forge's own ids instead of 1..=len.
fn setup(value: &mut Value, forge: SocketAddr, directory: &Path, git: &Path, engine: &Path) {
    let d = directory.to_str().expect("setup directory");
    value["application"]["webhooks"] = json!({"listener":{"socket":"127.0.0.1:0"}});
    value["application"]["forges"] = json!([{
        "number":1,"kind":"forgejo","api":{"socket":forge.to_string()},
        "transport":"plaintext","git":format!("http://{forge}")
    }]);
    value["application"]["identities"] = json!([{
        "number":1,"forge":1,"login":"temper","author":{"name":"Temper","email":"temper@example.test"}
    }]);
    for file in ["forge-1.api-token", "forge-1.user", "forge-1.git-token"] {
        fs::remove_file(engine.join("data/secrets").join(file)).expect("remove lab's file-backed fixture");
    }
    // Adoption walks ids 1..=len; the setup names are not consulted.
    value["application"]["setup"] = json!([{"forge":1,"directory":d,"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{"listen":{"socket":forge.to_string()},"git":git,"owner":"acme","repositories":[{"name":"spare","landing":"main"},{"name":"app","landing":"main"}]}}}]);
}

pub fn run(
    workers: &[&str],
    fixture: Fixture,
    scripts: fn(SocketAddr) -> Box<[Script]>,
    steps: Vec<calls::Step>,
    placed: &[u64],
) -> Run {
    assert_eq!(workers, ["ws", "s1"], "opener's workers");
    let clock = Clock::new();
    let began = clock.now().now;
    let scratch = Scratch::new(&format!("case_process_{}", std::process::id()));
    let mut description = cluster::Cluster::read(scratch.path());
    description.rename("w1", "ws");
    description.rename("w2", "s1");
    description.record_llm();
    let [hosts] = super::free_addresses(&[]);
    description.bind("proc.engine.ready.workers", &hosts.to_string());
    let forge = description.prepare("forge");
    let mut document = read(&forge.join("forge.json"));
    let kit = (fixture.setup)();
    let ci = match kit.checks.cue {
        Some(skein_fake_forge_domain::api::Cue::File { path, green }) => {
            json!({"path":text(&path),"green":text(&green)})
        }
        other => panic!("case CI cue: {other:?}"),
    };
    document["owner"] = "acme".into();
    document["repositories"] = json!([
        {"id":kit.id,"name":"app","landing":text(&kit.default),"ci":ci,"tree":kit.tree.iter().map(|file| json!({"path":text(&file.path),"content":text(&file.content)})).collect::<Vec<_>>()},
        {"id":1,"name":"spare","landing":"main","ci":null}
    ]);
    write(&forge.join("forge.json"), &document);
    let git = scratch.path().join("git");
    let (mut members, roots) = description.members();
    members.up(&["forge"]);
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = real::World::new(Judge {
        members,
        stage: Stage::Forge,
        deadline: began.saturating_add(WITHIN),
        marked: began,
        phases: Vec::new(),
        fixture,
        scripts,
        hosts,
        git,
        steps: steps.into(),
        ran: Vec::new(),
        calls: Vec::new(),
        slots: Vec::new(),
        client: scratch.path().join("client"),
        provisioning: false,
        live: None,
        placed: placed.to_vec(),
        run: result.clone(),
        stopped: false,
    });
    for root in roots {
        world.spawn(|| root);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.run(&clock, WITHIN);
    assert!(outcome.procs.iter().all(Host::is_empty), "every root empty");
    scratch.remove();
    result.borrow_mut().take().expect("case observations")
}
#[derive(Debug)]
enum Stage {
    Forge,
    Llm,
    Engine,
    Ws,
    S1,
    Acts,
    Provision,
    Live,
    Accounts,
    Stop,
}
struct Judge {
    members: cluster::Members,
    stage: Stage,
    deadline: Time,
    marked: Time,
    phases: Vec<(&'static str, skein_lib::Duration)>,
    fixture: Fixture,
    scripts: fn(SocketAddr) -> Box<[Script]>,
    hosts: SocketAddr,
    git: PathBuf,
    steps: VecDeque<calls::Step>,
    ran: Vec<calls::Step>,
    calls: Vec<Ended>,
    slots: Vec<usize>,
    client: PathBuf,
    provisioning: bool,
    live: Option<Value>,
    placed: Vec<u64>,
    run: std::rc::Rc<std::cell::RefCell<Option<Run>>>,
    stopped: bool,
}
impl Judge {
    fn mark(&mut self, phase: &'static str, now: Time) {
        self.phases.push((phase, now.saturating_since(self.marked)));
        self.marked = now;
    }
    fn calls(root: &mut cluster::Root) -> &mut ctl::Calls {
        let cluster::Root::Client { calls: Some(calls), .. } = root else { panic!("client calls") };
        calls
    }
    fn queue(&self, root: &mut cluster::Root) {
        let link = format!("http://{}", self.members.capture("engine", "link").expect("engine link"));
        let mut batch = ctl::Calls::linked(&link, &self.client);
        batch.hold();
        let cluster::Root::Client { calls, .. } = root else { panic!("client root") };
        *calls = Some(Box::new(batch));
    }
    fn configure(&mut self) {
        let engine_dir = self.members.prepare("engine");
        let ws_dir = self.members.prepare("ws");
        let s1_dir = self.members.prepare("s1");
        let mut value = read(&engine_dir.join("config/temper.json"));
        let mut ws = read(&ws_dir.join("config/temper.json"));
        let mut s1 = read(&s1_dir.join("config/temper.json"));
        let mut agent = read(&ws_dir.join("config/agent.json"));
        value["workers"] = ws["workers"].clone();
        value["application"]["workers"] = ws["application"]["workers"].clone();
        base(&mut value, &engine_dir.join("config/charters/chat.txt"));
        let forge = self.members.capture("forge", "address").expect("forge address").parse().expect("socket");
        (self.fixture.configure)(&mut value, &mut agent, forge);
        setup(&mut value, forge, &self.members.dir("forge").join("setup"), &self.git, &engine_dir);
        value["engine"]["hosts"]["listener"] = json!([{"socket":self.hosts.to_string()}]);
        value["engine"]["hosts"]["workers"][0]["name"] = "ws".into();
        value["engine"]["hosts"]["workers"][1]["name"] = "s1".into();
        for (name, worker, dir) in [("ws", &mut ws, &ws_dir), ("s1", &mut s1, &s1_dir)] {
            let mut shaped = value["workers"][0].clone();
            // Preserve this member's independent paths and secret, copying only fixture policy.
            shaped["name"] = name.into();
            shaped["secret"] = worker["workers"][0]["secret"].clone();
            shaped["agent"] = worker["workers"][0]["agent"].clone();
            worker["workers"] = json!([shaped]);
            worker["application"]["limits"] = value["application"]["limits"].clone();
            worker["application"]["workers"][0]["git"] = self.git.to_str().expect("git path").into();
            write(&dir.join("config/temper.json"), worker);
            write(&dir.join("config/agent.json"), &agent);
        }
        value["workers"] = json!([]);
        value["application"]["workers"] = json!([]);
        write(&engine_dir.join("config/temper.json"), &value);
    }
    fn arguments(&self, argv: &calls::Argv) -> Vec<String> {
        argv.0
            .iter()
            .map(|arg| match arg {
                calls::Arg::Lit(word) => (*word).into(),
                calls::Arg::Number(n) => n.to_string(),
                calls::Arg::File(name) => {
                    self.client.parent().expect("scratch").join(name).to_str().expect("file path").into()
                }
                calls::Arg::Task(n) => text(&self.calls[*n].stdout)
                    .split("done: task ")
                    .nth(1)
                    .expect("created task")
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .expect("task")
                    .into(),
                calls::Arg::Delegate(n) => {
                    let v = ctl::json(&self.calls[*n]);
                    let tree = &v["tree"];
                    let root = &tree["root"]["task"];
                    let children: Vec<_> = tree["rows"]
                        .as_array()
                        .expect("tree rows")
                        .iter()
                        .filter(|row| &row["parent"] == root)
                        .collect();
                    assert_eq!(children.len(), 1);
                    children[0]["chip"]["task"].as_u64().expect("child").to_string()
                }
                calls::Arg::Digest(n) => ctl::json(&self.calls[*n])["task"]["digest"].as_str().expect("digest").into(),
                calls::Arg::ActivityBefore(n) => {
                    let v = ctl::json(&self.calls[*n]);
                    v.pointer("/activity/before/value")
                        .or_else(|| v.pointer("/before/value"))
                        .and_then(Value::as_str)
                        .expect("before")
                        .into()
                }
            })
            .collect()
    }
    fn advance(&mut self, now: Time, root: &mut cluster::Root) {
        use calls::Step;
        let batch = Self::calls(root);
        if !batch.foreground_idle() {
            return;
        }
        self.calls = self
            .slots
            .iter()
            .map(|slot| {
                let ended = &batch.ends()[*slot];
                Ended { exit: ended.exit, stdout: ended.stdout.clone(), stderr: ended.stderr.clone() }
            })
            .collect();
        if !self.provisioning && self.calls.len() == 1 {
            self.provisioning = true;
            let key = self.members.dir("llm").join("llm.key");
            batch.push(ctl::Step::Run(ctl::Call::new(&[
                "add-key",
                "1",
                key.to_str().expect("key"),
                "--deployment",
                "--key",
                "case-llm",
            ])));
            batch.push(ctl::Step::Run(ctl::Call::new(&["wait", "accounts", "--until", "usable"])));
            self.stage = Stage::Provision;
            return;
        }
        if self.live.is_none()
            && matches!(self.ran.last(), Some(Step::Call { argv, .. }) if self.arguments(argv)[0] == "chat")
        {
            // The harness's own read, as the provisioning is: the director's chat
            // can end before an act reads it live, and an ended document names no
            // worker, so read its running document once.
            self.live = Some(Value::Null);
            let director = temper_real_loop_world::case::DIRECTOR.to_string();
            batch.push(ctl::Step::Run(ctl::Call::new(&[
                "wait",
                "task",
                &director,
                "--until",
                "phase:running",
                "--json",
            ])));
            self.stage = Stage::Live;
            return;
        }
        let Some(step) = self.steps.pop_front() else {
            self.mark("acts", now);
            batch.push(ctl::Step::Run(ctl::Call::new(&["accounts", "--json"])));
            self.stage = Stage::Accounts;
            return;
        };
        match &step {
            Step::SignIn { at } => {
                assert_eq!(at.0, 1);
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Run(ctl::Call::sign_in()));
            }
            Step::Call { at, argv } => {
                assert_eq!(at.0, 1);
                let args = self.arguments(argv);
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Run(ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>())));
            }
            Step::Start { at, argv } => {
                assert_eq!(at.0, 1);
                let args = self.arguments(argv);
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Start(ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>())));
            }
            Step::Until { at, argv, holds, early: _ } => {
                assert_eq!(at.0, 1);
                let args = self.arguments(argv);
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Until(
                    ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>()),
                    holds,
                ));
            }
            Step::Join { call } => {
                assert!(*call < self.slots.len());
                batch.push(ctl::Step::Join);
            }
            Step::AddWorker { .. } => panic!("AddWorker unsupported"),
            Step::StopWorker { .. } => panic!("StopWorker unsupported"),
            Step::StartWorker { .. } => panic!("StartWorker unsupported"),
            Step::FixtureReady => panic!("FixtureReady unsupported"),
            Step::Advance => panic!("Advance unsupported"),
            Step::ReleaseHold { .. } => panic!("ReleaseHold unsupported"),
            Step::Probe { .. } => panic!("Probe unsupported"),
            Step::Release { .. } => panic!("Release unsupported"),
            Step::Contended { .. } => panic!("Contended unsupported"),
            Step::SameParty { .. } => panic!("SameParty unsupported"),
            Step::Held { .. } => panic!("Held unsupported"),
            Step::Kill { .. } => panic!("Kill unsupported"),
            Step::Watching { .. } => panic!("Watching unsupported"),
            Step::Terminal(_) => panic!("Terminal unsupported"),
        }
        // The oracle's commands are the acts' verbs (Observed::with_commands).
        self.ran.push(step);
    }
}
impl Referee<cluster::Root> for Judge {
    fn act(&mut self, now: Time, roots: &mut [cluster::Root]) {
        self.members.act(now, roots);
        if self.members.failed() {
            self.stage = Stage::Stop;
            return;
        }
        let root = roots.last_mut().expect("client root");
        match self.stage {
            Stage::Forge if self.members.is_up() => {
                self.mark("forge ready", now);
                let address = self.members.capture("forge", "address").expect("forge address").parse().expect("socket");
                let directory = self.members.prepare("llm");
                let path = directory.join("llm.json");
                let mut document = read(&path);
                document["models"] = json!([{"name":"fake","window":1_048_576,"output":8192},{"name":"owner","window":1_048_576,"output":8192}]);
                document["cued"] = encode(&(self.scripts)(address));
                write(&path, &document);
                self.members.up(&["llm"]);
                self.stage = Stage::Llm;
            }
            Stage::Llm if self.members.is_up() => {
                self.mark("llm ready", now);
                self.configure();
                self.members.up(&["engine"]);
                self.stage = Stage::Engine;
            }
            Stage::Engine if self.members.is_up() && root.is_empty() => {
                self.mark("engine ready", now);
                self.members.up(&["ws"]);
                self.stage = Stage::Ws;
            }
            Stage::Ws if self.members.is_up() => {
                self.mark("ws ready", now);
                self.members.up(&["s1"]);
                self.stage = Stage::S1;
            }
            Stage::S1 if self.members.is_up() => {
                self.mark("s1 ready", now);
                self.queue(root);
                self.stage = Stage::Acts;
            }
            Stage::Acts => self.advance(now, root),
            Stage::Accounts if Self::calls(root).foreground_idle() => {
                let accounts = Self::calls(root).ends().last().expect("post-act accounts");
                success(accounts);
                let accounts = ctl::json(accounts);
                self.collect(&accounts);
                self.mark("reads", now);
                Self::calls(root).release();
                self.members.stop();
                self.stage = Stage::Stop;
            }
            Stage::Provision if Self::calls(root).foreground_idle() => {
                for call in Self::calls(root).ends() {
                    success(call);
                }
                self.stage = Stage::Acts;
            }
            Stage::Live if Self::calls(root).foreground_idle() => {
                let read = Self::calls(root).ends().last().expect("director's running read");
                success(read);
                self.live = Some(ctl::json(read));
                self.stage = Stage::Acts;
            }
            Stage::Forge
            | Stage::Llm
            | Stage::Engine
            | Stage::Ws
            | Stage::S1
            | Stage::Provision
            | Stage::Live
            | Stage::Accounts
            | Stage::Stop => {}
        }
    }
    fn observe(&mut self, now: Time, roots: &[cluster::Root]) {
        self.members.observe(now, &roots[..roots.len() - 1]);
        if let Some(stopped) = self.members.stopped() {
            assert_eq!(
                stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
                ["s1", "ws", "engine", "llm", "forge"]
            );
            for worker in ["ws", "s1"] {
                assert_eq!(
                    self.members.linked(worker),
                    1,
                    "{worker} linked once: the act's channel still maps to this link"
                );
            }
            for member in stopped {
                assert_eq!(
                    member.exit,
                    Some(Exit::Code(0)),
                    "{}: {}",
                    member.name,
                    String::from_utf8_lossy(&member.stderr)
                );
                assert!(!member.killed && member.survivors.is_empty(), "no surviving process");
            }
            if !self.stopped {
                for member in ["llm", "forge"] {
                    let bytes = fs::read(self.members.dir(member).join("observations.jsonl")).expect("final records");
                    let last =
                        bytes.split(|byte| *byte == b'\n').rfind(|line| !line.is_empty()).expect("final checkpoint");
                    assert_eq!(
                        serde_json::from_slice::<Value>(last).expect("checkpoint"),
                        json!({"observations_lost":0})
                    );
                }
                self.check_workspaces();
                self.stopped = true;
                self.mark("stop", now);
                eprintln!("case processes: {:?}", self.phases);
            }
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| format!("case processes waited in {:?}", self.stage))
    }
    fn passed(&self) -> bool {
        self.members.passed()
    }
}
fn success(call: &Ended) {
    assert_eq!(
        call.exit,
        Some(Exit::Code(0)),
        "call: {} {}",
        String::from_utf8_lossy(&call.stderr),
        String::from_utf8_lossy(&call.stdout)
    );
}
impl Judge {
    fn collect(&mut self, accounts: &Value) {
        let path = self.members.dir("llm").join("observations.jsonl");
        let mut told = Vec::new();
        let mut checkpoints = 0;
        for line in fs::read(path).expect("LLM records").split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
            use temper_fake::llm::observations::Observation;
            match temper_fake::llm::observations::decode(line).expect("query record") {
                Observation::Query(query) => {
                    if let Some(query) = Told::new(&query.system, &query.model, query.credential, &query.messages) {
                        told.push(query);
                    }
                }
                Observation::Checkpoint { lost } => {
                    assert_eq!(lost, 0, "lossless query stream");
                    checkpoints += 1;
                }
                Observation::Full { .. } => panic!("LLM record full"),
            }
        }
        assert!(checkpoints > 0, "query checkpoint");
        let rows: Vec<Value> = fs::read_to_string(self.members.dir("forge").join("observations.jsonl"))
            .expect("forge records")
            .lines()
            .map(|line| serde_json::from_str(line).expect("forge JSON"))
            .collect();
        assert_eq!(rows.last().expect("forge checkpoint")["observations_lost"], 0);
        let merged = rows
            .iter()
            .rev()
            .find(|row| row["observation"] == "merged" && row["repository"] == "acme/app")
            .expect("kit merged");
        let head = merged["head"].as_u64().expect("merged head");
        let merge = merged["commit"].as_u64().expect("merge commit");
        let mut checks = BTreeMap::new();
        for row in &rows {
            if std::ptr::eq(row, merged) {
                break;
            }
            if row["observation"] == "reported" && row["repository"] == "acme/app" && row["commit"] == head {
                checks.insert(row["context"].as_str().expect("check context"), row["state"] == "passed");
            }
        }
        let green = !checks.is_empty() && checks.values().all(|passed| *passed);
        let address = self.members.capture("forge", "address").expect("forge address").parse().expect("socket");
        let objects = super::github::objects(address, merge);
        let object = objects["objects"]
            .as_array()
            .expect("objects")
            .iter()
            .find(|object| object["commit"] == merge)
            .expect("merge object");
        let main = object["tree"]
            .as_array()
            .expect("main tree")
            .iter()
            .map(|file| (decode(&file["path"]), decode(&file["content"])))
            .collect();
        let workers = self.placements(accounts);
        *self.run.borrow_mut() = Some(Run {
            calls: std::mem::take(&mut self.calls),
            told,
            kit: Kit { main, merged_green: green },
            workers,
            steps: std::mem::take(&mut self.ran),
        });
    }
}
impl Judge {
    fn check_workspaces(&self) {
        use temper_real_loop_world::case;
        let workspaces = |worker| self.members.dir(worker).join("data/worker-root/workspaces");
        assert!(fs::read_dir(workspaces("ws")).expect("ws workspaces").next().is_none(), "ws holds no workspace");
        assert!(
            fs::read_dir(workspaces("s1")).expect("s1 workspaces").any(|entry| {
                let path = entry.expect("workspace").path().join("app/docs/design/export.md");
                fs::read(path).is_ok_and(|words| words == case::DESIGN_WORDS.as_bytes())
            }),
            "s1's kit workspace holds the design"
        );
    }
    fn placements(&self, accounts: &Value) -> BTreeMap<u64, String> {
        // jig retains a task's channel only while claimed/running. Use only JSON
        // stdout already read: the director's running read, the acts', and the
        // post-act accounts read with the hosts. Each worker must link exactly
        // once; ended tasks remain a named gap.
        let mut workers = BTreeMap::new();
        let mut pending = BTreeMap::new();
        for call in &self.calls {
            success(call);
        }
        let documents = self.calls.iter().filter_map(|call| serde_json::from_slice::<Value>(&call.stdout).ok());
        for document in self.live.iter().cloned().chain(documents).chain(std::iter::once(accounts.clone())) {
            live_channels(&document, &mut pending);
            if let Some(hosts) = document.pointer("/accounts/deployment_facts/program_hosts").and_then(Value::as_array)
            {
                pending.retain(|task, channel| {
                    let Some(host) = hosts.iter().find(|host| host["channel"] == *channel && host["connected"] == true)
                    else {
                        return true;
                    };
                    if self.placed.contains(task) {
                        workers.insert(*task, host["name"].as_str().expect("worker name").into());
                    }
                    false
                });
            }
        }
        workers
    }
}

fn decode(value: &Value) -> Vec<u8> {
    base64::decode(
        base64::Alphabet::Standard,
        value.as_str().expect("base64").as_bytes(),
        temper_fake::forge::LIMITS.limits.content_bytes,
    )
    .expect("file bytes")
    .into_vec()
}
fn live_channels(value: &Value, channels: &mut BTreeMap<u64, u64>) {
    if let (Some(task), Some(channel)) = (
        value.get("task").and_then(Value::as_u64),
        value.pointer("/details/owner/details/observation/channel").and_then(Value::as_u64),
    ) {
        channels.insert(task, channel);
    }
    for child in
        value.as_object().into_iter().flat_map(serde_json::Map::values).chain(value.as_array().into_iter().flatten())
    {
        live_channels(child, channels);
    }
}
