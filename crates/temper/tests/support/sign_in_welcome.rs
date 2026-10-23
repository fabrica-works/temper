//! The welcome on a terminal root, its sign-in link and durable account.
use super::{Process, person};
use skein_fake_peers::oauth;
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::path::Path;

// No model call or forge work: two local service starts and one fake grant.
const WITHIN: Duration = Duration::from_secs(20);
// configuration's issuer queues exactly one authorization-code grant.
const GRANTS: usize = 1;

struct Judge {
    authorization: Vec<u8>,
    visited: bool,
    stopped: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(visitor)] =
            procs
        else {
            panic!("welcome's service, person, issuer and visitor");
        };
        if !self.visited
            && let Some(client) = &person.client
            && let Some(address) = client
                .tree()
                .nodes()
                .iter()
                .find_map(|node| (node.role == skein_ui::Role::Link).then_some(node.external.as_deref()).flatten())
        {
            let query = address.strip_prefix(self.authorization.as_slice()).expect("page links to the fake issuer");
            assert!(query.starts_with(b"?"), "page links to the authorization endpoint");
            visitor.hand(address);
            self.visited = true;
        }
        if !self.stopped && person.finished && person.client.as_ref().is_some_and(Host::is_empty) && visitor.is_empty()
        {
            binary.signal(Signal::Terminate);
            issuer.shutdown();
            self.stopped = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(visitor)] =
            procs
        else {
            panic!("welcome's processes");
        };
        assert!(person.failed.is_none(), "{}", person.failed.as_deref().unwrap_or("person running"));
        assert!(
            self.stopped || binary.exit_status().is_none(),
            "serve exited early: {}",
            String::from_utf8_lossy(binary.stderr())
        );
        self.settled = self.stopped && procs.iter().all(Host::is_empty);
        assert!(
            now < self.deadline || self.settled,
            "welcome sign-in stalled at step {}; visited {}; response {:?}; authorize {}; posts {}",
            person.steps_seen(),
            self.visited,
            visitor.last_response().map(|(status, _)| status),
            issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Authorize { .. })).count(),
            issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Post { .. })).count(),
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "welcome sign-in did not settle".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

pub fn run(directory: &Path, issuer: oauth::Peer) {
    let configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("temper.json")).expect("configuration"))
            .expect("configuration JSON");
    let authorization = configuration["engine"]["llm"]["providers"][0]["sources"]["sign_in"]["authorization"]
        .as_str()
        .expect("fake authorization endpoint")
        .as_bytes()
        .to_vec();
    let clock = Clock::new();
    let mut binary = super::sign_in::start(directory);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let root = skein_shell::open_root(directory).expect("client parent root");
    fds.push(root);
    let mut world = real::World::new(Judge {
        authorization,
        visited: false,
        stopped: false,
        settled: false,
        deadline: clock.now().now.saturating_add(WITHIN),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Person(Box::new(person::Person::new(root, input, output, person::welcome_sign_in())))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer)));
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, WITHIN);
    let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(visitor)] =
        outcome.procs.as_slice()
    else {
        panic!("welcome's processes");
    };
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Authorize { .. })).count(),
        GRANTS,
        "one authorization visit"
    );
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Post { .. })).count(),
        GRANTS,
        "one code exchange"
    );
    let (status, location) = visitor.last_response().expect("listener answered the issuer's redirect");
    assert_eq!(status, 200, "the engine listener answers its fixed page");
    assert!(location.is_none(), "the terminal sent no Return: the listener offers no Location");
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "a healthy stop before restart");
    assert!(outcome.procs.iter().all(Host::is_empty), "all welcome sign-in io settled");
    let tree = person.client.as_ref().expect("terminal client").tree();
    let account = tree
        .nodes()
        .iter()
        .filter(|node| node.role == skein_ui::Role::Paragraph)
        .find_map(|node| node.name.as_deref()?.strip_prefix(b"Model: ChatGPT"))
        .expect("Ready named the made ChatGPT account");
    let heading = [b"ChatGPT".as_slice(), account].concat();
    let restarted = super::run(directory, person::kept_chatgpt_account(&heading), None, WITHIN);
    assert!(restarted.finished, "the same account is signed in after restart");
}
