//! Pending provider sign-in against the shipped binary and an independent local issuer.
use super::{Process, ctl};
use skein_fake_peers::{Transport, oauth};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{fs, path::Path};

const STOPS_WITHIN: Duration = Duration::from_secs(10);

/// `ChatGPT`'s scopes and callback path, with every endpoint on the fake issuer.
#[must_use]
pub fn configuration(directory: &Path) -> oauth::Peer {
    super::configuration(directory, &super::Forge::Unreached);
    let [address] = super::free_addresses(&[]);
    let origin = format!("http://{address}");
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    let provider = &mut value["engine"]["llm"]["providers"][0];
    assert_eq!(provider["number"], 1);
    provider["signed_in_plan"] = true.into();
    provider["manage_usage"] = format!("https://{address}/usage").into();
    provider["sources"]["sign_in"] = serde_json::json!({
        "issuer":origin, "client":"client", "name_hint":"temper",
        "scopes":["openid","profile","email","offline_access","resource.invoke","chatgpt.tokens.use.direct"],
        "must_grant":["chatgpt.tokens.use.direct"], "redirect_path":"auth/callback",
        "authorization":format!("{origin}/authorize"), "token":format!("{origin}/token"),
        "keys":format!("{origin}/keys"), "resource":format!("{origin}/v1"), "transport":"plaintext"
    });
    // This story needs one pending sign-in, within the shipped socket budget.
    value["engine"]["limits"]["overrides"]["accounts.sign_ins"] = 1.into();
    super::write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
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
    oauth::Peer::new(
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
            authorization_url: format!("{origin}/authorize").into_bytes().into(),
            token_endpoint: format!("{origin}/token").into_bytes().into(),
            client_id: b"client".as_slice().into(),
            client_secret: None,
            redirect_uri: b"http://127.0.0.1:1455/auth/callback".as_slice().into(),
            refresh_token: b"fake-refresh".as_slice().into(),
            terms: Some(skein_fake_oauth::Terms {
                issued: Box::new([b"issued-client".as_slice().into()]),
                omit_issued: false,
                varying_port: true,
                revocation_endpoint: None,
                openid: Some(skein_fake_oauth::OpenId {
                    issuer: origin.as_bytes().into(),
                    key_set_url: format!("{origin}/keys").into_bytes().into(),
                    subject: b"subject".as_slice().into(),
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
    .expect("local issuer")
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
    let mut binary = super::serve(directory);
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
