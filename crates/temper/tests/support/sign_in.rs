//! Pending provider sign-in against the shipped binary and an independent local issuer.
use super::{Process, ctl};
use skein_fake_peers::{Transport, oauth};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{fs, path::Path};

const STOPS_WITHIN: Duration = Duration::from_secs(10);

const ACCESS: &[u8] = b"process-sign-in-access";

/// Initialize the shipped document once; only the issuer addresses and LLM endpoint move.
#[must_use]
pub fn configuration(directory: &Path) -> oauth::Peer {
    let initialized = std::process::Command::new(env!("CARGO_BIN_EXE_temper"))
        .arg("init")
        .args(super::configuration::flags(directory))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir(directory)
        .output()
        .expect("initialize a fresh deployment");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    let [address] = super::free_addresses(&["127.0.0.1:34443".parse().expect("LLM address")]);
    let origin = format!("http://{address}");
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("init document")).expect("JSON");
    let provider = &mut value["engine"]["llm"]["providers"][0];
    assert_eq!(provider["number"], 1);
    let sign_in = &mut provider["sources"]["sign_in"];
    let original = sign_in["issuer"].as_str().expect("shipped issuer").to_owned();
    for member in ["issuer", "authorization", "token", "revocation", "keys"] {
        sign_in[member] = sign_in[member].as_str().expect("shipped address").replace(&original, &origin).into();
    }
    let redirect = format!(
        "http://127.0.0.1:{}/{}",
        sign_in["redirect_port"].as_u64().expect("shipped redirect port"),
        sign_in["redirect_path"].as_str().expect("shipped callback path")
    );
    sign_in["transport"] = "plaintext".into();
    sign_in.as_object_mut().expect("registration").remove("server_name");
    sign_in.as_object_mut().expect("registration").remove("redirect_port");
    let registration = sign_in.clone();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:34443"});
    value["engine"]["llm"]["endpoints"][0]["server_name"] = "skein.test".into();
    value["engine"]["llm"]["endpoints"][0]["trust"] = serde_json::json!(directory.join("root.der"));
    super::write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
    let agent_path = directory.join("agent.json");
    let mut agent: serde_json::Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent")).expect("JSON");
    agent["endpoints"][0]["address"] = "127.0.0.1:34443".into();
    agent["endpoints"][0]["server_name"] = "skein.test".into();
    agent["endpoints"][0]["trust_der"] = serde_json::json!(directory.join("root.der"));
    super::write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    super::write_private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    issuer(address, &registration, redirect.as_bytes())
}

fn issuer(address: std::net::SocketAddr, registration: &serde_json::Value, redirect: &[u8]) -> oauth::Peer {
    let bytes =
        |member: &str| -> Box<[u8]> { registration[member].as_str().expect("registration member").as_bytes().into() };
    let profile = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small);
    let limits = skein_fake_oauth::Limits {
        document: profile.driver.client.document,
        uri_bytes: 1024,
        request_bytes: 4096,
        codes: 1,
        rotations: 2,
        carried: 5,
        plans: 1,
    };
    let mut peer = oauth::Peer::new(
        address,
        Transport::Plaintext,
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: 64,
            plaintext: 131_072,
            ciphertext: 131_072,
            observations: 64,
            observation_bytes: 4096,
        },
        skein_fake_oauth::Config {
            authorization_url: bytes("authorization"),
            token_endpoint: bytes("token"),
            client_id: bytes("client"),
            client_secret: None,
            redirect_uri: redirect.into(),
            refresh_token: b"fake-refresh".as_slice().into(),
            terms: Some(skein_fake_oauth::Terms {
                issued: Box::new([b"issued-client".as_slice().into()]),
                omit_issued: false,
                varying_port: true,
                revocation_endpoint: Some(bytes("revocation")),
                openid: Some(skein_fake_oauth::OpenId {
                    issuer: bytes("issuer"),
                    key_set_url: bytes("keys"),
                    subject: b"process-person".as_slice().into(),
                    lifetime: Duration::from_secs(300),
                    header_kid: true,
                    fault: None,
                }),
            }),
        },
        limits,
        skein_http::server::Limits {
            response: oauth::least_response(&limits).expect("issuer response"),
            body: u64::from(limits.request_bytes),
            ..profile.driver.server
        },
    )
    .expect("local issuer");
    let scopes = registration["scopes"]
        .as_array()
        .expect("shipped scopes")
        .iter()
        .map(|scope| scope.as_str().expect("scope"))
        .collect::<Vec<_>>()
        .join(" ");
    peer.queue(skein_fake_oauth::Plan {
        status: 200,
        body: skein_fake_oauth::Body::Token(skein_oauth::TokenResponse {
            access_token: ACCESS.into(),
            refresh_token: Some(b"fake-refresh".as_slice().into()),
            expires_in: 300,
            scope: Some(scopes.into_bytes().into()),
            id_token: None,
            earliest_refresh_at: None,
        }),
        delay: Duration::ZERO,
        retry_after: Duration::ZERO,
    })
    .expect("one fake grant");
    peer
}

fn start(directory: &Path) -> skein_world::end_to_end::Binary {
    use skein_world::end_to_end::{Binary, Command, Mode};
    // A fresh init's demo forge is prepared by box before it starts serve.
    Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: std::iter::once("box".into()).chain(super::configuration::flags(directory)).collect(),
            directory: directory.into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("box starts the initialized deployment")
}

struct Judge {
    sent: Option<Time>,
    killed: bool,
    deadline: Time,
    settled: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.sent.is_none() && now >= self.deadline {
            let Process::Calls(calls) = &mut procs[1] else { panic!("ctl calls") };
            for ended in calls.ends().iter().chain(calls.pending()) {
                eprintln!(
                    "setup call {:?}: stdout {}; stderr {}",
                    ended.exit,
                    String::from_utf8_lossy(&ended.stdout),
                    String::from_utf8_lossy(&ended.stderr)
                );
            }
            calls.cancel();
            let Process::Binary(binary) = &mut procs[0] else { panic!("temper") };
            eprintln!("serve setup stderr: {}", String::from_utf8_lossy(binary.stderr()));
            binary.signal(Signal::Kill);
            self.sent = Some(now);
            self.killed = true;
            self.deadline = now.saturating_add(STOPS_WITHIN);
        }
        let Process::Calls(calls) = &procs[1] else { panic!("ctl calls") };
        if self.sent.is_none()
            && let [signed, connected] = calls.ends()
        {
            assert_eq!(signed.exit, Some(Exit::Code(0)), "sign-in answered");
            assert_eq!(
                connected.exit,
                Some(Exit::Code(0)),
                "connect answered: {}",
                String::from_utf8_lossy(&connected.stderr)
            );
            let answer = ctl::json(connected);
            let address = answer["outcome"]["done"]["owner"]["account_sign_in_started"]["address"]
                .as_str()
                .expect("sign-in address");
            assert!(address.starts_with("http://127.0.0.1:"), "local authorization address");
            // Nobody visits this address. Keep the issuer alive until temper exits.
            let Process::Binary(binary) = &mut procs[0] else { panic!("temper") };
            binary.signal(Signal::Terminate);
            self.sent = Some(now);
            self.deadline = now.saturating_add(STOPS_WITHIN);
        }
        let Process::Binary(binary) = &mut procs[0] else { panic!("temper") };
        if binary.exit_status().is_some() {
            assert!(self.killed || now <= self.deadline, "SIGTERM exceeded {STOPS_WITHIN:?}");
            let Process::Issuer(issuer) = &mut procs[2] else { panic!("issuer") };
            issuer.shutdown();
        } else if self.sent.is_some() && now >= self.deadline && !self.killed {
            // Reap our owned child even in the deliberately broken regression run.
            binary.signal(Signal::Kill);
            self.killed = true;
            self.deadline = now.saturating_add(STOPS_WITHIN);
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("temper") };
        assert!(
            binary.exit_status().is_none() || self.sent.is_some(),
            "serve exited early: {}",
            String::from_utf8_lossy(binary.stderr())
        );
        self.settled = self.sent.is_some() && procs.iter().all(Host::is_empty);
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "sign-in stop did not settle".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

pub fn run(directory: &Path, issuer: oauth::Peer) {
    let clock = Clock::new();
    let mut binary = start(directory);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(directory).expect("scratch root");
    fds.push(root);
    let mut world = real::World::new(Judge {
        sent: None,
        killed: false,
        deadline: clock.now().now.saturating_add(super::WITHIN),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Calls(Box::new(ctl::Calls::new(
            root,
            vec![
                ctl::Step::Run(ctl::Call::sign_in()),
                ctl::Step::Run(ctl::Call::new(&["connect", "1", "--key", "connect-llm", "--json"])),
            ],
            Some(&streams),
            None,
        )))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer)));
    let outcome = world.run(&clock, Duration::from_secs(90));
    let Process::Issuer(issuer) = &outcome.procs[2] else { panic!("issuer") };
    assert!(
        !issuer.observations().iter().any(|event| matches!(event, oauth::Observation::Authorize { .. })),
        "no authorization visit"
    );
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("temper") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "SIGTERM must stop within {STOPS_WITHIN:?}");
    assert!(outcome.procs.iter().all(Host::is_empty), "all sign-in io settled");
}

struct AnswerJudge {
    visited: bool,
    stopped: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for AnswerJudge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Calls(calls) = &procs[1] else { panic!("ctl calls") };
        if !self.visited
            && let [_, me, connected, pending, ..] = calls.ends()
        {
            let party = ctl::json(me)["party"]["number"].as_u64().expect("signed-in person");
            let pending = ctl::json(pending);
            let [signing] = pending["accounts"]["signing"].as_array().expect("signing list").as_slice() else {
                panic!("one sign-in under way: {pending}")
            };
            assert_eq!(signing["provider"], 1);
            assert_eq!(signing["holder"]["party"], party);
            let address = connected
                .stdout
                .split(|byte| *byte == b'\n')
                .find(|line| line.starts_with(b"http://127.0.0.1:"))
                .expect("connect prints the address");
            let address = address.to_vec();
            let Process::Visitor(visitor) = &mut procs[3] else { panic!("browser") };
            visitor.hand(&address);
            self.visited = true;
        }
        let Process::Calls(calls) = &procs[1] else { panic!("ctl calls") };
        for (index, ended) in calls.ends().iter().enumerate() {
            assert_eq!(
                ended.exit,
                Some(Exit::Code(0)),
                "ctl call {index}: {}; output {}",
                String::from_utf8_lossy(&ended.stderr),
                String::from_utf8_lossy(&ended.stdout)
            );
        }
        if !self.stopped && calls.ends().len() == ANSWER_CALLS {
            let Process::Binary(binary) = &mut procs[0] else { panic!("temper") };
            binary.signal(Signal::Terminate);
            for proc in procs {
                match proc {
                    Process::Peer(peer) => peer.shutdown(),
                    Process::Issuer(issuer) => issuer.shutdown(),
                    Process::Binary(_)
                    | Process::Calls(_)
                    | Process::Person(_)
                    | Process::Stdout(_)
                    | Process::Visitor(_) => {}
                }
            }
            self.stopped = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("temper") };
        assert!(self.stopped || binary.exit_status().is_none(), "serve: {}", String::from_utf8_lossy(binary.stderr()));
        self.settled = self.stopped && procs.iter().all(Host::is_empty);
        assert!(now < self.deadline || self.settled, "signed-in chat did not settle");
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "signed-in chat deadline".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
const ANSWER_CALLS: usize = 8;

pub fn answered(directory: &Path, issuer: oauth::Peer) {
    let mut script = jig_channel_world::fixtures::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    let mut recipe = temper_real_loop_world::llm::scripted(Box::new([skein_fake_llm_domain::api::Script {
        cue: Box::default(),
        turns: jig_channel_world::fixtures::provider_turns(&script),
    }]));
    recipe.credential = skein_llm::Credential::bearer(ACCESS.into());
    let peer = skein_fake_peers::llm::Peer::new(
        "127.0.0.1:34443".parse().expect("fake LLM address"),
        Transport::Tls,
        recipe.limits,
        recipe.config,
        recipe.credential,
        recipe.protocol_limits,
        recipe.domain,
        recipe.domain_limits,
    )
    .expect("fake LLM peer");
    let clock = Clock::new();
    let mut binary = start(directory);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(directory).expect("scratch root");
    fds.push(root);
    let mut world = real::World::new(AnswerJudge {
        visited: false,
        stopped: false,
        settled: false,
        deadline: clock.now().now.saturating_add(super::WITHIN),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Calls(Box::new(ctl::Calls::new(
            root,
            vec![
                ctl::Step::Run(ctl::Call::sign_in()),
                ctl::Step::Run(ctl::Call::new(&["me", "--json"])),
                ctl::Step::Run(ctl::Call::new(&["connect", "1", "--key", "connect-llm"])),
                ctl::Step::Run(ctl::Call::new(&["accounts", "--json"])),
                ctl::Step::Run(ctl::Call::new(&["wait", "accounts", "--until", "settled", "--json"])),
                ctl::Step::Run(ctl::Call::new(&["accounts", "--json"])),
                ctl::Step::Run(ctl::Call::new(&[
                    "chat",
                    "--project",
                    "1",
                    "--model",
                    "gpt-6-luna",
                    "--key",
                    "chat-on-plan",
                    "Hello",
                    "--json",
                ])),
                ctl::Step::Run(ctl::Call::new(&["wait", "task", "1", "--until", "replied", "--json"])),
            ],
            Some(&streams),
            None,
        )))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer)));
    world.spawn(|| Process::Visitor(Box::default()));
    world.spawn(|| Process::Peer(Box::new(peer)));
    let outcome = world.run(&clock, super::WITHIN);
    let Process::Calls(calls) = &outcome.procs[1] else { panic!("ctl calls") };
    let [_, me, _, _, _, ready, _, conversation] = calls.ends() else { panic!("signed-in chat calls") };
    let person = ctl::json(me)["party"]["number"].as_u64().expect("person number");
    let ready = ctl::json(ready);
    let account = ready["accounts"]["party"]
        .as_array()
        .expect("person's accounts")
        .iter()
        .find(|row| row["provider"] == 1)
        .expect("OpenAI account");
    assert_eq!(account["plan"], true);
    assert_eq!(account["holder"]["party"], person);
    assert!(
        account["sources"].as_array().expect("sources").iter().any(|source| {
            source["kind"] == "sign_in" && source["usable"] == true && source["state"]["kind"] == "granted"
        }),
        "usable signed-in source"
    );
    assert!(
        String::from_utf8_lossy(&conversation.stdout).contains("Process answer"),
        "chat answered: {}",
        String::from_utf8_lossy(&conversation.stdout)
    );
    let Process::Peer(peer) = &outcome.procs[4] else { panic!("LLM peer") };
    assert!(
        peer.observations().iter().any(|event| matches!(
            event,
            skein_fake_peers::llm::Observation::Credential { scheme: Some(skein_llm::Scheme::Bearer), .. }
        )),
        "the peer observed a bearer scheme"
    );
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("temper") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)));
    assert!(outcome.procs.iter().all(Host::is_empty), "all signed-in chat io settled");
}
