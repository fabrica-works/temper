//! The forge connector reads the record: with no demo, or one ended, it
//! refuses, *not offered* (protocol/setup.md, 8.4).
use crate::{Connection, Demo, DemoConfig, DemoState, Domain, Request, StartAnswer, Stored};
use skein_lib::{Queue, Token};

impl DemoConfig {
    #[must_use]
    pub fn within(&self) -> bool {
        let adoption = &self.adoption;
        for bytes in [
            adoption.host.len(),
            adoption.owner.len(),
            adoption.name.len(),
            adoption.prefix.len(),
            adoption.landing.len(),
        ] {
            if bytes > 1024 {
                return false;
            }
        }
        adoption.checks.is_empty()
    }
}
impl Domain {
    pub fn configure_demo(&mut self, demo: Option<DemoConfig>) {
        if let Some(demo) = &demo {
            assert!(demo.within(), "bounded demo configuration");
        }
        self.demo = demo;
    }
    #[must_use]
    pub fn demo(&self) -> Option<&DemoConfig> {
        self.demo.as_ref()
    }
    #[must_use]
    pub const fn demo_state(&self) -> Option<DemoState> {
        self.demo_state
    }
    #[must_use]
    pub fn demo_adopted(&self) -> bool {
        let Some(demo) = &self.demo else { return false };
        let wanted = &demo.adoption;
        for (_, row) in &self.repositories {
            if row.project == wanted.project
                && row.provider.forge == wanted.forge
                && row.owner == wanted.owner
                && row.name == wanted.name
            {
                return true;
            }
        }
        false
    }
}

fn needs_group(d: &Domain) -> bool {
    let Some(demo) = &d.demo else { return false };
    d.under_box && needs_connection(&d.connection(demo.adoption.forge))
}

fn needs_connection(connection: &Connection) -> bool {
    match connection {
        Connection::Connected { .. } | Connection::Connecting => false,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => true,
    }
}

pub(crate) fn start(d: &mut Domain, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.demo_pending.len() == d.demo_pending.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.demo_pending.is_empty();
    d.demo_pending.insert(request, token).expect("bounded starts");
    if !first {
        return;
    }
    d.demo_answers = skein_lib::Map::with_capacity(d.demo_answers.capacity());
    let Some(demo) = &d.demo else {
        answered(d, StartAnswer::NotOffered, out);
        return;
    };
    if d.demo_state == Some(DemoState::Ended) {
        answered(d, StartAnswer::NotOffered, out);
        return;
    }
    if d.demo_state.is_none() {
        d.demo_state = Some(DemoState::Running);
        out.push(Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Demo(Demo { forge: demo.adoption.forge, state: DemoState::Running }),
        });
    }
    if needs_group(d) {
        out.push(Request::SetUp { forge: d.demo.as_ref().expect("offered demo").adoption.forge, request });
    }
    out.push(Request::DemoStart { request });
}

pub(crate) fn answered(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.demo_pending.len() {
        let Some((request, token)) = d.demo_pending.first() else { break };
        let (request, token) = (*request, *token);
        d.demo_pending.remove(&request);
        d.demo_answers.insert(token, answer.clone()).expect("one answer per bounded start");
        out.push(Request::ApplicationDone { request, token });
    }
}

/// The account and the adoption are jig's records and stay; no act is made again.
pub(crate) fn restart(d: &Domain, out: &mut Queue<Request>) {
    if d.demo_state == Some(DemoState::Running) && needs_group(d) {
        out.push(Request::SetUp {
            forge: d.demo.as_ref().expect("configured demo").adoption.forge,
            request: Token::new(u64::MAX),
        });
    }
}
