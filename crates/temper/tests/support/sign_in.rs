//! The public sign-in binary, issuer and browser, hosted on one real world.
use super::*;
use skein_fake_peers::oauth::Peer;
use smith_local_process_world::oauth::Browser;
pub const ACCOUNT: u32 = 1;
pub const AUTHORIZATION: &str = "http://127.0.0.1:34544/authorize";
const ACCESS: &[u8] = b"access-new";
pub fn configuration(directory: &Path) {
    super::configuration(directory, &Forge::Unreached);
    fs::set_permissions(directory.join("accounts/records"), fs::Permissions::from_mode(0o700))
        .expect("private record directory");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("config.json")).expect("configuration")).expect("JSON");
    document["engine"]["llm"]["records"] = "accounts/records".into();
    document["engine"]["limits"]["overrides"]["accounts.retry_ms"] = 100.into();
    document["engine"]["llm"]["accounts"] = serde_json::json!([{"number":ACCOUNT,"source":{"sign_in":{
        "client":"client","authorization":AUTHORIZATION,"token":"http://127.0.0.1:34544/token",
        "endpoint":{"socket":"127.0.0.1:34544"},"transport":"plaintext","redirect":"http://127.0.0.1:2345/callback","scope":"openid"}}}]);
    write_private(&directory.join("config.json"), &serde_json::to_vec(&document).expect("JSON"));
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("agent.json")).expect("agent configuration")).expect("JSON");
    agent["endpoints"][0]["provider"] = "anthropic".into();
    agent["endpoints"][0]["address"] = "127.0.0.1:34545".into();
    agent["endpoints"][0]["transport"] = "plaintext".into();
    let endpoint = agent["endpoints"][0].as_object_mut().expect("endpoint object");
    endpoint.remove("server_name");
    endpoint.remove("trust_der");
    write_private(&directory.join("agent.json"), &serde_json::to_vec(&agent).expect("JSON"));
}
pub struct Running {
    binary: Binary,
    streams: Streams,
    link: Vec<u8>,
}
/// The engine runs without an account record before the explicit sign-in.
#[must_use]
pub fn running(directory: &Path) -> Running {
    let mut binary = super::serve(directory);
    let streams = binary.take_streams().expect("engine pipes");
    let Streams::Pipes { output, .. } = &streams else { panic!("pipes mode") };
    let link = super::read_line(*output, WITHIN);
    Running { binary, streams, link }
}
#[must_use]
pub fn issuer() -> Peer {
    smith_local_process_world::process::issuer_at(skein_fake_peers::Transport::Plaintext, 34544)
}
#[must_use]
pub fn browse(_issuer: &Peer, visit: &str) -> Browser {
    Browser::new(visit.as_bytes())
}
#[must_use]
pub fn start(directory: &Path, account: u32) -> Child {
    let mut binary = Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: vec!["sign-in".into(), "config.json".into(), account.to_string().into()],
            directory: directory.into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("sign-in starts");
    let streams = binary.take_streams().expect("sign-in pipes");
    Child { binary: Some(binary), streams: Some(streams), stderr: Vec::new(), directory: directory.into() }
}
pub struct Child {
    binary: Option<Binary>,
    streams: Option<Streams>,
    stderr: Vec<u8>,
    directory: std::path::PathBuf,
}
impl Child {
    #[must_use]
    pub fn line(&self, within: Duration) -> String {
        let Streams::Pipes { output, .. } = self.streams.as_ref().expect("sign-in pipes") else { panic!("pipes mode") };
        String::from_utf8(super::read_line(*output, within)).expect("visit URL")
    }
    #[must_use]
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
    pub fn exit(&mut self, issuer: Peer, browser: Browser, engine: Option<Running>, within: Duration) -> u8 {
        let clock = Clock::new();
        let running = engine.is_some();
        let mut world = real::World::new(Judge {
            deadline: clock.now().now.saturating_add(within),
            stopped: false,
            settled: false,
            running,
            wake: None,
        });
        let binary = self.binary.take().expect("sign-in child");
        world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
        let streams = self.streams.take().expect("sign-in streams");
        let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
        world.spawn_with_fds(vec![input, output], || Process::Stdout(Box::new(StdoutReader::new(input, output))));
        world.spawn(|| Process::Issuer(Box::new(issuer)));
        world.spawn(|| Process::Browser { browser: Box::new(browser), active: false });
        if let Some(Running { binary: engine, streams, link }) = engine {
            let mut fds = streams.descriptors();
            let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
            let root = skein_shell::open_root(&self.directory).expect("client root");
            fds.push(root);
            world.spawn_with_fds(engine.descriptors(), || Process::Binary(Box::new(engine)));
            world.spawn_with_fds(fds, || Process::Person {
                person: Box::new(person::Person::with_link(
                    root,
                    input,
                    output,
                    person::answered_chat(b"Hello", super::ANSWER),
                    &link,
                )),
                active: false,
            });
            let peer = llm();
            world.spawn(|| Process::Llm(Box::new(peer)));
        }
        let outcome = world.run(&clock, within.saturating_add(Duration::from_secs(5)));
        let Process::Binary(binary) = &outcome.procs[0] else { panic!("sign-in child") };
        self.stderr = binary.stderr().to_vec();
        let Process::Browser { browser, .. } = &outcome.procs[3] else { panic!("browser") };
        assert!(browser.saw_reply(), "callback answered");
        if running {
            let Process::Person { person, .. } = &outcome.procs[5] else { panic!("native person") };
            assert!(person.finished, "person saw the answer with the new credential");
            let Process::Llm(peer) = &outcome.procs[6] else { panic!("LLM peer") };
            assert!(smith_agent_process_world::fake::replied(peer), "the peer accepted the new credential and replied");
        }
        match binary.exit_status().expect("child exited") {
            Exit::Code(code) => code,
            Exit::Signal(_) => 1,
        }
    }
}
fn llm() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_protocol::{documents, provider};
    let config = skein_llm_world::fake::config();
    let mut protocol = skein_llm_world::fake::limits(&skein_llm_world::limits());
    protocol.documents.anthropic.request = 32768;
    protocol.documents.anthropic.strings = 16384;
    protocol.documents.anthropic.tokens = 4096;
    protocol.documents.anthropic.output_items = 256;
    skein_fake_peers::llm::Peer::new(
        (std::net::Ipv4Addr::LOCALHOST, 34545).into(),
        skein_fake_peers::Transport::Plaintext,
        smith_agent_process_world::fake::limits(),
        provider::Config {
            echo: skein_llm::openai::Echo::NONE,
            usage_fields: documents::UsageFields::ALL,
            provider: documents::Provider::Anthropic,
            path: skein_llm::Endpoint::anthropic().target,
            headers: Box::new([]),
        },
        skein_llm::Credential { access_token: ACCESS.into(), account_id: Box::new([]) },
        protocol,
        skein_fake_llm_domain::Domain::try_scripted(&config, 17, Box::new([chat_script(super::ANSWER)]))
            .expect("sign-in chat script"),
        config,
    )
    .expect("sign-in LLM peer")
}
enum Process {
    Binary(Box<Binary>),
    Stdout(Box<StdoutReader>),
    Issuer(Box<Peer>),
    Browser { browser: Box<Browser>, active: bool },
    Person { person: Box<person::Person>, active: bool },
    Llm(Box<skein_fake_peers::llm::Peer>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Stdout(x) => x.as_ref(),
            Self::Issuer(x) => x.as_ref(),
            Self::Browser { browser, .. } => browser.as_ref(),
            Self::Person { person, .. } => person.as_ref(),
            Self::Llm(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Stdout(x) => x.as_mut(),
            Self::Issuer(x) => x.as_mut(),
            Self::Browser { browser, .. } => browser.as_mut(),
            Self::Person { person, .. } => person.as_mut(),
            Self::Llm(x) => x.as_mut(),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if !matches!(self, Self::Person { active: false, .. } | Self::Browser { active: false, .. }) {
            self.host_mut().iterate(now, wall);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !matches!(self, Self::Person { active: false, .. } | Self::Browser { active: false, .. })
            && self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
struct Judge {
    deadline: Time,
    stopped: bool,
    settled: bool,
    running: bool,
    wake: Option<Time>,
}
fn llm_listening(procs: &[Process]) -> bool {
    let Process::Llm(peer) = &procs[6] else { panic!("LLM peer") };
    peer.address().is_some()
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Issuer(issuer) = &procs[2] else { panic!("issuer") };
        if issuer.address().is_some() {
            let Process::Browser { active, .. } = &mut procs[3] else { panic!("browser") };
            *active = true;
        }
        if procs[0].is_empty() && procs[1].is_empty() && procs[3].is_empty() && (!self.running || llm_listening(procs))
        {
            let mut finished = !self.running;
            if self.running {
                let Process::Person { active, person } = &mut procs[5] else { panic!("person") };
                *active = true;
                finished = person.finished && person.client.as_ref().is_some_and(Host::is_empty);
            }
            if !self.stopped && finished {
                for proc in procs {
                    match proc {
                        Process::Issuer(peer) => peer.shutdown(),
                        Process::Llm(peer) => peer.shutdown(),
                        Process::Binary(binary) if binary.exit_status().is_none() => binary.signal(Signal::Kill),
                        Process::Binary(_) | Process::Stdout(_) | Process::Browser { .. } | Process::Person { .. } => {}
                    }
                }
                self.stopped = true;
            }
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = self.stopped && procs.iter().all(Host::is_empty);
        let ready = if self.running {
            let Process::Person { active, person } = &procs[5] else { panic!("person") };
            (!*active && llm_listening(procs))
                || (person.finished && person.client.as_ref().is_some_and(Host::is_empty))
        } else {
            true
        };
        self.wake = (!self.stopped && ready && procs[0].is_empty() && procs[1].is_empty() && procs[3].is_empty())
            .then_some(now);
        let Process::Binary(binary) = &procs[0] else { panic!("child") };
        assert!(
            now < self.deadline || self.settled,
            "sign-in world deadline; stopped: {}; empty hosts: {:?}; issuer deadline: {:?}; stderr: {}",
            self.stopped,
            procs.iter().map(Host::is_empty).collect::<Vec<_>>(),
            procs[2].next_deadline(),
            String::from_utf8_lossy(binary.stderr())
        );
        assert!(
            binary.exit_status().is_none_or(|exit| exit == Exit::Code(0)),
            "sign-in refused: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or_else(|| (!self.settled).then_some(self.deadline))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "sign-in world did not drain".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
