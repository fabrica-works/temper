//! Bounded problems and measured stderr lines (protocol/configuration.md, 2).
use crate::limits;
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{Decimal, List, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    Jig(config::What),
    Standalone,
    Loopback,
    Connectors,
    Absolute,
    NoWebhooks,
    Profile,
    Override(limits::Refused),
    Against { other: &'static [u8] },
    Past { bytes: u64, bound: u64 },
    Within { bytes: u64, limit: &'static [u8], bound: u64 },
    Agent,
    Adapter,
    Redirect(super::Listener),
    NoHub,
    Credentials,
    Escalation,
    Provider,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problem {
    pub path: config::Path,
    pub what: What,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problems {
    pub problems: List<Problem>,
    pub past: u32,
}
impl Problems {
    #[must_use]
    pub fn empty() -> Self {
        Self { problems: List::with_capacity(config::PROBLEMS), past: 0 }
    }
    pub fn push(&mut self, path: config::Path, what: What) {
        if self.problems.push(Problem { path, what }).is_err() {
            self.past = self.past.saturating_add(1);
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.problems.is_empty() && self.past == 0
    }
    #[must_use]
    pub fn jig(found: config::Problems, prefix: &config::Path) -> Self {
        let mut result = Self::empty();
        result.append(found, prefix);
        result
    }
    pub(crate) fn append(&mut self, found: config::Problems, prefix: &config::Path) {
        for problem in &found.problems {
            let mut path = prefix.clone();
            for segment in &problem.path.segments {
                path = path.child(segment.clone());
            }
            self.push(path, What::Jig(problem.what));
        }
        self.past = self.past.saturating_add(found.past);
    }
}
fn put(writer: &mut Writer, length: &mut usize, measuring: bool, bytes: &[u8]) {
    *length = length.checked_add(bytes.len()).expect("bounded diagnostic fits usize");
    if !measuring {
        writer.put(bytes).expect("diagnostic was measured");
    }
}
fn words(what: What) -> &'static [u8] {
    match what {
        What::Standalone => b"standalone takes one worker, joined to the engine by the in-process name \"workers\"",
        What::Loopback => b"must be a socket address on loopback",
        What::Connectors => b"must be [1], the forge connector",
        What::Absolute => b"must be an absolute path",
        What::NoWebhooks => b"needs application.webhooks",
        What::Profile => b"the only profile is \"small\"",
        What::Override(limits::Refused::Unknown) => b"no limit of temper's has this name",
        What::Override(limits::Refused::Range) => b"outside its type",
        What::Against { .. } => b"must agree with ",
        What::Past { .. } | What::Within { .. } => b"",
        What::Redirect(_) => b"fixed port conflicts with ",
        What::Adapter => b"program adapter not declared in the agent's programs",
        What::Agent => b"not in the agent's configuration under the same name",
        What::NoHub => b"the root has no hub, so the fleet must place every run on a worker",
        What::Credentials => b"must be from 1 to accounts.credentials, in engine.limits and in the worker's limits",
        What::Escalation => b"an owner's project must name an escalation role from 0 to 3",
        What::Provider => b"the deployment's provider must not be 0, the forge's sign-in provider",
        What::Jig(what) => match what {
            config::What::Json => b"invalid JSON",
            config::What::Unknown => b"unknown member",
            config::What::Missing => b"missing",
            config::What::Duplicate => b"duplicate",
            config::What::Kind => b"wrong value kind",
            config::What::Bound => b"past its bound",
            config::What::Form => b"invalid form",
            config::What::Unique => b"must be unique",
            config::What::Refers => b"unknown reference",
            config::What::Plaintext => b"plaintext requires loopback or in-process",
            config::What::Confidential => b"a confidential registration is refused: only a public one signs in",
            config::What::NoProvider => b"no provider offers this model",
            config::What::Digest => b"invalid digest",
            config::What::HandedIn => b"invalid handed-in account",
            config::What::Reserve { .. } => b"reserve does not fit charter ",
            config::What::Budget { .. } => b"budget does not fit charter ",
            config::What::Grace { .. } => b"grace does not fit template ",
            config::What::Operations { .. } => b"operations do not fit compound requests: operations ",
            config::What::Message => b"message bounds do not agree",
            config::What::Silence => b"heartbeat does not fit silence bound",
            config::What::StopBound => b"stop bound does not fit grace",
            config::What::Memory { .. } => b"the worst case passes memory_bytes by ",
            config::What::Override => b"invalid override",
            config::What::Unreadable => b"not read",
            config::What::Relative => b"relative path must be non-empty with no .. component",
        },
    }
}
fn redirect_listener(listener: super::Listener, writer: &mut Writer, length: &mut usize, measuring: bool) {
    let name = match listener {
        super::Listener::Clients => b"application.clients.listener".as_slice(),
        super::Listener::Webhooks => b"application.webhooks.listener".as_slice(),
        super::Listener::Owned(index) => {
            put(writer, length, measuring, b"application.setup[");
            put(writer, length, measuring, Decimal::of(u64::from(index)).as_bytes());
            b"].mode.owned.listen".as_slice()
        }
    };
    put(writer, length, measuring, name);
}
fn render(problems: &Problems, writer: &mut Writer, measuring: bool) -> usize {
    let mut length = 0_usize;
    for problem in &problems.problems {
        let mut member = false;
        for segment in &problem.path.segments {
            match segment {
                config::Segment::Member(name) => {
                    if member {
                        put(writer, &mut length, measuring, b".");
                    }
                    put(writer, &mut length, measuring, name);
                    member = true;
                }
                config::Segment::Index(index) => {
                    put(writer, &mut length, measuring, b"[");
                    put(writer, &mut length, measuring, Decimal::of(u64::from(*index)).as_bytes());
                    put(writer, &mut length, measuring, b"]");
                }
            }
        }
        put(writer, &mut length, measuring, b": ");
        put(writer, &mut length, measuring, words(problem.what));
        match problem.what {
            What::Redirect(listener) => redirect_listener(listener, writer, &mut length, measuring),
            What::Against { other } => put(writer, &mut length, measuring, other),
            What::Past { bytes, bound } => {
                put(writer, &mut length, measuring, Decimal::of(bytes).as_bytes());
                put(writer, &mut length, measuring, b" bytes, past its bound of ");
                put(writer, &mut length, measuring, Decimal::of(bound).as_bytes());
            }
            What::Within { bytes, limit, bound } => {
                put(writer, &mut length, measuring, Decimal::of(bytes).as_bytes());
                put(writer, &mut length, measuring, b" bytes, past ");
                put(writer, &mut length, measuring, limit);
                put(writer, &mut length, measuring, b" of ");
                put(writer, &mut length, measuring, Decimal::of(bound).as_bytes());
            }
            What::Jig(config::What::Memory { over }) => {
                put(writer, &mut length, measuring, Decimal::of(over).as_bytes());
                put(writer, &mut length, measuring, b" bytes");
            }
            What::Jig(config::What::Operations { operations, delegates, goals }) => {
                put(writer, &mut length, measuring, Decimal::of(u64::from(operations)).as_bytes());
                put(writer, &mut length, measuring, b", delegates ");
                put(writer, &mut length, measuring, Decimal::of(u64::from(delegates)).as_bytes());
                put(writer, &mut length, measuring, b", goals ");
                put(writer, &mut length, measuring, Decimal::of(u64::from(goals)).as_bytes());
            }
            What::Jig(
                config::What::Reserve { charter }
                | config::What::Budget { charter }
                | config::What::Grace { template: charter },
            ) => {
                put(writer, &mut length, measuring, Decimal::of(u64::from(charter)).as_bytes());
            }
            What::Jig(
                config::What::Json
                | config::What::Unknown
                | config::What::Missing
                | config::What::Duplicate
                | config::What::Kind
                | config::What::Bound
                | config::What::Form
                | config::What::Unique
                | config::What::Refers
                | config::What::Plaintext
                | config::What::Confidential
                | config::What::NoProvider
                | config::What::Digest
                | config::What::HandedIn
                | config::What::Message
                | config::What::Silence
                | config::What::StopBound
                | config::What::Override
                | config::What::Unreadable
                | config::What::Relative,
            )
            | What::Standalone
            | What::Loopback
            | What::Connectors
            | What::Absolute
            | What::NoWebhooks
            | What::Profile
            | What::Override(_)
            | What::Adapter
            | What::Agent
            | What::NoHub
            | What::Credentials
            | What::Escalation
            | What::Provider => {}
        }
        put(writer, &mut length, measuring, b"\n");
    }
    if problems.past > 0 {
        put(writer, &mut length, measuring, b"and ");
        put(writer, &mut length, measuring, Decimal::of(u64::from(problems.past)).as_bytes());
        put(writer, &mut length, measuring, b" more\n");
    }
    length
}
#[must_use]
pub fn lines(problems: &Problems) -> Box<[u8]> {
    let length = render(problems, &mut Writer::new(0), true);
    let mut writer = Writer::new(length);
    assert_eq!(render(problems, &mut writer, false), length, "diagnostic passes measure identically");
    writer.finish()
}
