//! GitHub's git as processes (domain/testing.md, 2.6), on the cluster's reader.
use super::{Scratch, WITHIN, cluster, ctl};
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_lib::{Time, base64};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};
use temper_github_app::{APP, records, seed};

const INSTALLATION: u64 = 1;
const CODER: u64 = 3; // Fresh deployment: chat 1, change procedure 2, its coder 3.
const EDIT: &[u8] = b"Changed by temper.\n";

pub fn push() {
    let clock = Clock::new();
    let began = clock.now().now;
    let scratch = Scratch::new("github_git");
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/github");
    let mut description = cluster::Cluster::read_from(scratch.path(), &directory);
    let [clients] = super::free_addresses(&[]);
    description.bind("story.clients", &clients.to_string());
    description.render_files("forge");
    let fixture: Value = serde_json::from_str(
        &fs::read_to_string(description.dir("forge").join("github.json")).expect("GitHub fixture"),
    )
    .expect("GitHub JSON");
    let app = &fixture["github"]["app"];
    assert_eq!((app["id"].as_u64(), app["bot"].as_u64()), (Some(APP.id), Some(APP.bot)));
    assert_eq!(app["slug"], APP.slug);
    assert_eq!(app["key"], APP.key);
    // The configured deployment store is state/ under the engine's data directory.
    let data = description.dir("engine").join("data/state");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(&data).expect("engine data directory");
    let key = description.dir("llm").join("llm.key");
    let observations = description.dir("forge").join("observations");
    let engine = description.dir("engine").join("config/temper.json");
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_temper"), description.dir("w1").join("git"))
        .expect("worker's fake git");
    let (mut members, hosts) = description.members();
    members.up(&["forge", "llm"]);
    let mut world = real::World::new(Judge {
        members,
        deadline: clock.now().now.saturating_add(WITHIN),
        stage: Stage::Fakes,
        data,
        key,
        observations,
        engine,
        client: scratch.path().join("client"),
        began,
        ready: BTreeSet::new(),
        call_marks: 0,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.run(&clock, WITHIN);
    assert!(outcome.procs.iter().all(Host::is_empty), "every process and client settled");
    scratch.remove();
    eprintln!("github git: complete {:?}", clock.now().now.saturating_since(began));
}

#[derive(Debug)]
enum Stage {
    Fakes,
    Worker,
    Chat,
    Tree,
    Coder,
    Stopping,
}
struct Judge {
    members: cluster::Members,
    deadline: Time,
    stage: Stage,
    data: PathBuf,
    key: PathBuf,
    observations: PathBuf,
    engine: PathBuf,
    client: PathBuf,
    began: Time,
    ready: BTreeSet<&'static str>,
    call_marks: usize,
}
impl Judge {
    fn mark(&self, step: &str) {
        eprintln!("github git: {step} {:?}", Clock::new().now().now.saturating_since(self.began));
    }
    fn queue(&mut self, host: &mut cluster::Root, calls: Vec<ctl::Call>) {
        self.call_marks = 0;
        let link = format!("http://{}", self.members.capture("engine", "link").expect("engine link"));
        let mut batch = ctl::Calls::linked(&link, &self.client);
        for call in calls {
            batch.push(ctl::Step::Run(call));
        }
        let cluster::Root::Client { calls, .. } = host else { panic!("client root") };
        *calls = Some(Box::new(batch));
    }
    fn begin(&mut self, host: &mut cluster::Root) {
        use ctl::Call;
        let engine: Value =
            serde_json::from_slice(&fs::read(&self.engine).expect("engine document")).expect("engine JSON");
        assert_eq!(engine["workers"], serde_json::json!([]), "engine alone");
        assert_eq!(engine["application"]["workers"], serde_json::json!([]));
        let workers = engine["engine"]["hosts"]["workers"].as_array().expect("worker positions");
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0]["name"], "w1");
        assert_eq!(self.members.linked("w1"), 1, "the sole worker linked");
        self.queue(
            host,
            vec![
                Call::sign_in(),
                Call::new(&[
                    "add-key",
                    "1",
                    self.key.to_str().expect("key path"),
                    "--deployment",
                    "--key",
                    "github-llm",
                ]),
                Call::new(&["wait", "accounts", "--until", "usable"]),
                Call::new(&[
                    "adopt",
                    "--project",
                    "1",
                    "--forge",
                    "1",
                    "--owner",
                    "acme",
                    "--name",
                    "app",
                    "--landing",
                    "main",
                    "--key",
                    "adopt-app",
                ]),
                Call::new(&["chat", "--project", "1", "--key", "change", "@forge_chat Update README", "--json"]),
            ],
        );
        self.stage = Stage::Chat;
    }
    fn tree(&mut self, host: &mut cluster::Root, packet: &Value) {
        let tree = &packet["tree"];
        // Watches carry their last digest forward, so discovering a new child never polls.
        if tree["rows"].as_array().expect("tree rows").iter().any(|row| row["chip"]["task"] == CODER) {
            self.mark("coder created");
            self.queue(host, vec![ctl::Call::new(&["wait", "task", &CODER.to_string(), "--until", "ended", "--json"])]);
            self.stage = Stage::Coder;
        } else {
            let digest = tree["digest"].as_str().unwrap_or_else(|| panic!("tree digest: {packet}"));
            self.queue(host, vec![ctl::Call::new(&["wait", "tree", "1", "--since", digest, "--json"])]);
        }
    }
    fn coder(task: &Value) {
        let task = &task["task"];
        assert_eq!(task["chip"]["task"], CODER, "{task}");
        let owner = &task["chip"]["details"]["owner"];
        assert!(owner["details"]["ending"]["done"].is_object(), "coder delivered: {task}");
        assert!(owner["details"]["ending"]["done"]["change"].is_object(), "change delivered: {task}");
        // begin() checks that this engine alone has exactly one worker position,
        // w1, and its linked line. Only that worker can have executed this coder.
    }
    fn forge(&self) {
        let lines = fs::read_to_string(&self.observations).expect("forge observations");
        let rows: Vec<Value> =
            lines.lines().map(|line| serde_json::from_str(line).expect("observation JSON")).collect();
        assert_eq!(rows.last().expect("checkpoint")["observations_lost"], 0);
        let calls: Vec<_> = rows.iter().filter(|row| row["observation"] == "git_call").collect();
        assert!(calls.iter().any(|row| row["writes"] == false && row["status"] == 200), "clone/read observed");
        assert_eq!(calls.iter().filter(|row| row["writes"] == true).count(), 1, "exactly the push writes");
        for call in calls {
            // The record's closed schema is the token-secrecy proof: no string/value field.
            let decoded =
                temper_fake::forge::observations::decode(call.to_string().as_bytes()).expect("closed git record");
            assert!(matches!(decoded, Some(temper_fake::forge::observations::Observation::GitCall(_))));
            assert_eq!(call["installation"], INSTALLATION);
            assert!(call["mint"].as_u64().is_some_and(|mint| mint > 0));
            if call["writes"] == true {
                assert_eq!(call["status"], 200, "push succeeds");
            } else {
                // Before its first push git checks whether the new branch exists.
                assert!(call["status"] == 200 || call["status"] == 404, "git read: {call}");
            }
        }
        assert!(!rows.iter().any(|row| row["observation"] == "git_calls_full"));
        let moved: Vec<_> = rows.iter().filter(|row| row["observation"] == "moved").collect();
        assert_eq!(moved.len(), 1, "one pushed branch");
        assert_eq!(moved[0]["repository"], "acme/app");
        assert_ne!(moved[0]["branch"], "main");
        assert!(moved[0]["from"].is_null(), "push creates its branch");
        assert_eq!(moved[0]["by"], APP.bot);
        let commit = moved[0]["to"].as_u64().expect("pushed head");
        let address = self.members.capture("forge", "address").expect("forge address").parse().expect("socket");
        let objects = objects(address, commit);
        let object = objects["objects"]
            .as_array()
            .expect("objects")
            .iter()
            .find(|object| object["commit"] == commit)
            .expect("pushed commit");
        let tree = object["tree"].as_array().expect("pushed tree");
        let file = tree.iter().find(|file| decode(&file["path"]).as_ref() == b"README.md").expect("README");
        let expected = [temper_fake::forge::FILES[0].1, EDIT].concat();
        assert_eq!(decode(&file["content"]).as_ref(), expected, "the branch holds the coder's edit");
    }
}
impl Referee<cluster::Root> for Judge {
    fn act(&mut self, now: Time, hosts: &mut [cluster::Root]) {
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return;
        }
        let host = hosts.last_mut().expect("client root");
        if matches!(self.stage, Stage::Fakes) && self.members.is_up() {
            self.mark("fakes ready");
            let api = self.members.capture("forge", "address").expect("forge address").parse().expect("API socket");
            seed(&self.data, &records(&APP, 1, &[INSTALLATION], api)).expect("durable App seed");
            self.mark("App seeded");
            self.members.up(&["w1"]);
            self.stage = Stage::Worker;
            self.members.act(now, hosts);
        } else if matches!(self.stage, Stage::Worker) && self.members.is_up() {
            self.mark("engine and worker ready");
            self.begin(host);
        } else if host.is_empty() && !matches!(self.stage, Stage::Fakes | Stage::Worker | Stage::Stopping) {
            let cluster::Root::Client { calls, .. } = host else { panic!("client root") };
            let calls = calls.take().expect("completed calls");
            for ended in calls.ends() {
                assert_eq!(
                    ended.exit,
                    Some(Exit::Code(0)),
                    "ctl: {} {}",
                    String::from_utf8_lossy(&ended.stderr),
                    String::from_utf8_lossy(&ended.stdout)
                );
            }
            match self.stage {
                Stage::Chat => {
                    self.mark("signed in, key usable, adopted, chat created");
                    let packet = ctl::json(calls.ends().last().expect("chat creation"));
                    assert_eq!(packet["outcome"]["done"]["owner"]["created"]["first"], 1, "{packet}");
                    self.queue(host, vec![ctl::Call::new(&["task", "1", "--tree", "--json"])]);
                    self.stage = Stage::Tree;
                }
                Stage::Tree => self.tree(host, &ctl::json(&calls.ends()[0])),
                Stage::Coder => {
                    Self::coder(&ctl::json(&calls.ends()[0]));
                    self.mark("coder delivered");
                    self.forge();
                    self.mark("push and branch verified");
                    self.members.stop();
                    self.stage = Stage::Stopping;
                }
                Stage::Fakes | Stage::Worker | Stage::Stopping => unreachable!("active calls"),
            }
        }
    }
    fn observe(&mut self, now: Time, hosts: &[cluster::Root]) {
        self.members.observe(now, &hosts[..hosts.len() - 1]);
        for (process, capture) in [("forge", "address"), ("llm", "address"), ("engine", "link"), ("w1", "engine")] {
            if self.members.capture(process, capture).is_some() && self.ready.insert(process) {
                self.mark(&format!("{process} ready"));
            }
        }
        if let Some(cluster::Root::Client { calls: Some(calls), .. }) = hosts.last() {
            for index in self.call_marks..calls.ends().len() {
                let labels: &[&str] = match self.stage {
                    Stage::Chat => &[
                        "signed in",
                        "key added",
                        "key usable",
                        "adopted (forge connected and minted)",
                        "chat created",
                    ],
                    Stage::Coder => &["coder end watch answered"],
                    Stage::Fakes | Stage::Worker | Stage::Tree | Stage::Stopping => &[],
                };
                if let Some(label) = labels.get(index) {
                    self.mark(label);
                }
            }
            self.call_marks = calls.ends().len();
        }
        if let Some(stopped) = self.members.stopped() {
            assert_eq!(
                stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
                ["w1", "engine", "llm", "forge"]
            );
            for member in stopped {
                assert_eq!(
                    member.exit,
                    Some(Exit::Code(0)),
                    "{}: {}",
                    member.name,
                    String::from_utf8_lossy(&member.stderr)
                );
                assert_eq!(member.ending, cluster::Ending::Cluster);
                assert!(!member.killed && member.survivors.is_empty(), "no surviving process");
            }
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| format!("GitHub process story waited in {:?}", self.stage))
    }
    fn passed(&self) -> bool {
        self.members.passed()
    }
}
fn decode(value: &Value) -> Box<[u8]> {
    base64::decode(base64::Alphabet::Standard, value.as_str().expect("base64 text").as_bytes(), 8192)
        .expect("file bytes")
}
// The fake's object route is credential-free and does not add a repository git call.
// Like support/webhooks.rs, the referee makes one bounded request to a separate process.
fn objects(address: SocketAddr, commit: u64) -> Value {
    let within = std::time::Duration::from_secs(5);
    let mut socket = TcpStream::connect_timeout(&address, within).expect("object route");
    socket.set_read_timeout(Some(within)).expect("read bound");
    socket.set_write_timeout(Some(within)).expect("write bound");
    let body = format!("{{\"commits\":[{commit}]}}");
    write!(socket, "POST /.fake/git/objects HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).expect("object request");
    let mut response = Vec::new();
    socket.take(65_536).read_to_end(&mut response).expect("object response");
    assert!(response.starts_with(b"HTTP/1.1 200"), "{}", String::from_utf8_lossy(&response));
    let at = response.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("HTTP head");
    serde_json::from_slice(&response[at + 4..]).expect("objects JSON")
}
