use crate::{
    Setup,
    check::{Forge, Members, User},
    limits,
    problem::Problems,
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
use skein_json::Document;
use skein_lib::{Writer, bytes};
pub const TWO: &[u8] = br#"[
{"forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["acme"],"mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}},
{"forge":2,"directory":"forge-2","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["work"],"mode":{"owned":{"forgejo":"/opt/forgejo/forgejo","runner":{"binary":"/opt/forgejo/forgejo-runner","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"},"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"ada","email":"ada@example.invalid"}}}}
]"#;
pub fn document(bytes: &[u8]) -> Document {
    jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("test JSON")
}
pub fn setup() -> Setup {
    Setup::read(&document(TWO), 0, &limits::small(), &mut Problems::empty()).expect("design's setup")
}
pub fn members() -> Members {
    Members {
        forges: Box::new([
            Forge {
                number: 1,
                api: None,
                plaintext: false,
                git: bytes::copy_of(b"https://forge.example"),
                user: User::File(bytes::copy_of(b"forge-1/user")),
                api_token: bytes::copy_of(b"forge-1/api_token"),
                identity_token: bytes::copy_of(b"forge-1/git_token"),
                identity_login: bytes::copy_of(b"temper"),
            },
            Forge {
                number: 2,
                api: Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
                plaintext: true,
                git: bytes::copy_of(b"http://127.0.0.1:3000"),
                user: User::File(bytes::copy_of(b"forge-2/user")),
                api_token: bytes::copy_of(b"forge-2/api_token"),
                identity_token: bytes::copy_of(b"forge-2/git_token"),
                identity_login: bytes::copy_of(b"temper"),
            },
        ]),
        listeners: Box::new([]),
    }
}
pub fn changed(from: &[u8], to: &[u8]) -> Box<[u8]> {
    let at = bytes::find(TWO, from).expect("fixture text exists");
    let after = at.checked_add(from.len()).unwrap();
    let len = TWO.len().checked_sub(from.len()).unwrap().checked_add(to.len()).unwrap();
    let mut w = Writer::new(len);
    for part in [TWO.get(..at).unwrap(), to, TWO.get(after..).unwrap()] {
        w.put(part).unwrap();
    }
    w.finish()
}
pub fn owned() -> crate::setup::Owned {
    match setup().entries[1].mode.clone() {
        crate::setup::Mode::Owned(owned) => owned,
        crate::setup::Mode::Managed { .. } => panic!("owned fixture"),
    }
}
pub fn place() -> crate::place::Place {
    crate::place::Place { directory: b"/srv/forge-2".as_slice().into(), user: b"ada".as_slice().into() }
}
pub fn secrets() -> crate::state::Secrets {
    use skein_lib::Secret;
    crate::state::Secrets {
        secret_key: Secret::new(b"key-secret".as_slice().into()),
        internal_token: Secret::new(b"internal-secret".as_slice().into()),
        jwt_secret: Secret::new(b"jwt-secret".as_slice().into()),
    }
}

pub struct Drive {
    machine: crate::prepare::Prepare,
    out: skein_lib::Queue<crate::prepare::Request>,
}
impl Drive {
    pub fn new(owned: crate::setup::Owned) -> Self {
        Self {
            machine: crate::prepare::Prepare::new(owned, place(), None),
            out: skein_lib::Queue::with_capacity(crate::prepare::MAX_OUT),
        }
    }
    pub fn send(&mut self, event: crate::prepare::Event) -> crate::prepare::Request {
        crate::prepare::step(&mut self.machine, event, &mut self.out);
        let request = self.out.pop().expect("one request");
        assert!(self.out.is_empty());
        request
    }
    pub fn versions(&mut self, found: crate::prepare::Found, runner: bool) -> crate::prepare::Request {
        use crate::{
            prepare::{Event, Request},
            spec::Which,
        };
        assert_eq!(
            self.send(Event::Found(found)),
            Request::Version { which: Which::Forgejo, spec: crate::spec::version(Which::Forgejo, &owned(), &place()) }
        );
        let mut request = self.send(Event::Printed(line(Which::Forgejo)));
        if runner {
            assert_eq!(
                request,
                Request::Version {
                    which: Which::Runner,
                    spec: crate::spec::version(Which::Runner, &owned(), &place())
                }
            );
            request = self.send(Event::Printed(line(Which::Runner)));
        }
        request
    }
    pub fn drawn(&mut self, runner: bool) -> crate::prepare::Request {
        use crate::prepare::{Event, Request};
        let count = if runner { 5 } else { 4 };
        for value in 0..count {
            let next = self.send(Event::Drew(draw(value)));
            if value.checked_add(1).unwrap() == count {
                return next;
            }
            assert_eq!(next, Request::Draw);
        }
        unreachable!("four or five draws")
    }
}
pub fn draw(value: u8) -> skein_lib::Secret {
    skein_lib::Secret::new(Box::new([value; 32]))
}
pub fn line(which: crate::spec::Which) -> Box<str> {
    use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION};
    let (prefix, version) = match which {
        crate::spec::Which::Forgejo => (b"forgejo version ".as_slice(), VERSION),
        crate::spec::Which::Runner => (b"forgejo-runner version v".as_slice(), RUNNER_VERSION),
    };
    let mut w = Writer::new(prefix.len().checked_add(version.len()).unwrap());
    w.put(prefix).unwrap();
    w.put(version.as_bytes()).unwrap();
    let bytes = w.finish();
    let mut chunks = bytes.utf8_chunks();
    let chunk = chunks.next().unwrap();
    assert!(chunk.invalid().is_empty());
    assert!(chunks.next().is_none());
    chunk.valid().into()
}
pub fn first() -> crate::prepare::Found {
    crate::prepare::Found { state: None, admin_token: false, files: false }
}
pub fn later() -> crate::prepare::Found {
    use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION};
    crate::prepare::Found {
        state: Some(crate::state::encode(&crate::state::State {
            forgejo: VERSION.as_bytes().into(),
            runner: Some(RUNNER_VERSION.as_bytes().into()),
            prepared: true,
            secrets: secrets(),
        })),
        admin_token: true,
        files: true,
    }
}
pub fn ran(stdout: &[u8]) -> crate::prepare::Event {
    use crate::prepare::{Event, Exit, Ran};
    Event::Ran(Ran {
        exit: Exit::Code(0),
        stdout: skein_lib::Secret::new(stdout.into()),
        stderr: skein_lib::Secret::new(Box::new([])),
    })
}
pub const ADMIN: &[u8] = b"site-admin-token";
pub fn secret(value: &[u8]) -> skein_lib::Secret {
    skein_lib::Secret::new(bytes::copy_of(value))
}
pub fn admin() -> temper_forge_forgejo::document::User {
    temper_forge_forgejo::document::User {
        id: 1,
        login: b"temper-admin".as_slice().into(),
        is_admin: true,
        full_name: Box::new([]),
    }
}
pub fn person(login: &[u8], full_name: &[u8]) -> temper_forge_forgejo::document::User {
    temper_forge_forgejo::document::User {
        id: 42,
        login: bytes::copy_of(login),
        is_admin: false,
        full_name: bytes::copy_of(full_name),
    }
}
pub struct Run {
    machine: crate::reconcile::Reconcile,
    entry: crate::setup::Entry,
}
impl Run {
    pub fn managed() -> Self {
        Self::new(1)
    }
    pub fn owned() -> Self {
        Self::new(2)
    }
    fn new(forge: u16) -> Self {
        let entry = setup().entry(forge).expect("fixture forge").clone();
        Self { machine: crate::reconcile::Reconcile::new(&entry, None), entry }
    }
    pub fn step(&mut self, event: crate::reconcile::Event) -> crate::reconcile::Need {
        let mut out = skein_lib::Queue::with_capacity(crate::reconcile::MAX_OUT);
        crate::reconcile::step(&mut self.machine, event, &mut out);
        assert_eq!(out.len(), 1);
        out.pop().expect("one need")
    }
    pub fn forge(&self) -> u16 {
        self.entry.forge
    }
    pub fn bot(&self) -> Box<[u8]> {
        self.entry.bot.login.clone()
    }
    pub fn bot_lookup(&mut self) {
        use crate::{
            place::File,
            reconcile::{Event, Need, Reply},
        };
        use temper_forge_forgejo::{document::Document, request::Request};
        assert_eq!(self.step(Event::Begin), Need::Read(File::AdminToken));
        assert_eq!(
            self.step(Event::Read(Some(secret(ADMIN)))),
            Need::Send { request: Request::User, token: secret(ADMIN) }
        );
        assert_eq!(
            self.step(Event::Reply(Reply::Document(Document::User(admin())))),
            Need::Send { request: Request::UserOf { login: self.bot() }, token: secret(ADMIN) }
        );
    }
}
pub fn prepared() -> crate::state::State {
    use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION};
    crate::state::State {
        forgejo: VERSION.as_bytes().into(),
        runner: Some(RUNNER_VERSION.as_bytes().into()),
        prepared: true,
        secrets: secrets(),
    }
}
pub fn launch(which: crate::spec::Which) -> crate::run::Launch {
    crate::run::launch(&owned(), &place(), which).expect("configured process")
}
pub fn suffixed_line(which: crate::spec::Which, suffix: &str) -> Box<str> {
    let line = line(which);
    let mut writer = Writer::new(line.len().checked_add(suffix.len()).unwrap());
    writer.put(line.as_bytes()).unwrap();
    writer.put(suffix.as_bytes()).unwrap();
    let bytes = writer.finish();
    let mut chunks = bytes.utf8_chunks();
    let chunk = chunks.next().unwrap();
    assert!(chunk.invalid().is_empty());
    assert!(chunks.next().is_none());
    chunk.valid().into()
}
impl Run {
    pub fn kept_tokens(&mut self) {
        use crate::{
            place::File,
            reconcile::{Event, Need, Reply},
        };
        use temper_forge_forgejo::{document::Document, request::Request};
        for next in [File::GitToken, File::User] {
            assert_eq!(
                self.step(Event::Read(Some(secret(b"bot-token")))),
                Need::Send { request: Request::User, token: secret(b"bot-token") }
            );
            assert_eq!(
                self.step(Event::Reply(Reply::Document(Document::User(person(b"temper", b"temper:bot"))))),
                Need::Read(next)
            );
        }
    }
}
