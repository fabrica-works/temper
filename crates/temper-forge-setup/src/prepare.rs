//! The owned preparation's one-request machine (setup.md, 6.3 and 13.1).
use crate::{
    check::joined,
    ini,
    place::{File, Place},
    runner,
    setup::Owned,
    spec::{self, Command, Spec, Which},
    state::{self, Secrets, State},
    upgrade::{self, Course},
};
use alloc::boxed::Box;
use core::net::IpAddr;
use skein_lib::{
    Queue, Secret, Writer,
    base64::{self, Alphabet},
    bytes,
};
use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION, names_forgejo, names_runner};
/// One request is emitted per step.
pub const MAX_OUT: u32 = 1;
/// A completed process's status.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exit {
    /// The process exited with this code.
    Code(u8),
    /// The process ended on this signal.
    Signal(u8),
}
/// Files discovered by the host while it holds Forgejo's lock.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Found {
    pub state: Option<Secret>, // Whole record, absent on a fresh preparation.
    pub admin_token: bool,     // Whether the token file exists.
    pub runner_secret: bool,   // Whether an added runner can keep its secret.
    pub files: bool,           // Whether Forgejo's files exist.
}
/// A command's bounded output, kept private until masked for a failure.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ran {
    pub exit: Exit,     // Exit code or terminating signal.
    pub stdout: Secret, // First bounded stream.
    pub stderr: Secret, // Second bounded stream.
}
/// Answers to the previous request, beginning with files found under the lock.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    /// The locked directory has been examined.
    Found(Found),
    /// A version command printed its first line.
    Printed(Box<str>),
    /// A preparation command ended.
    Ran(Ran),
    /// The host drew exactly 32 random bytes.
    Drew(Secret),
    /// Whether the requested backup exists.
    Exists(bool),
    /// A lock or file effect completed.
    Done,
}
/// Host effects; only End is terminal, and no effect runs inside the machine.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Request {
    /// Check the binary before deciding from its state.
    Version { which: Which, spec: Spec }, // Read the first stdout line.
    /// Take and retain the runner lock before writing its files.
    Lock(File), // Take the runner's lock without waiting.
    /// Remove the interrupted first preparation’s files.
    Clear, // Remove only an interrupted preparation's files.
    /// Create the private directory layout.
    Layout, // Make the private directories.
    /// Draw fresh entropy for credentials or a token name.
    Draw, // Draw 32 random bytes.
    /// Run one offline preparation command.
    Run { command: Command, spec: Spec }, // Execute a preparation command.
    /// Generate the administrator’s token.
    Token { spec: Spec }, // Execute the admin token command.
    /// Write a complete private file.
    Write { file: File, contents: Secret }, // Write the whole private file.
    /// Check for an upgrade backup without changing it.
    Exists { path: Box<[u8]> },
    /// Commit the completed partial dump.
    Rename { from: Box<[u8]>, to: Box<[u8]> },
    /// Finish successfully or with a refusal.
    End(Result<(), Failure>), // Terminal answer.
}
/// A refusal or a process failure; no variant holds unmasked credentials.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    /// A binary’s version line does not name its pin.
    Binary { which: Which, program: Box<[u8]>, printed: Box<str> },
    /// A preparation command failed; its output tail is masked.
    Command { command: Command, exit: Exit, tail: Box<[u8]> },
    /// The admin token command failed or printed no token.
    Token { exit: Exit, tail: Box<[u8]> },
    /// A successful command printed no required first line.
    Unread { command: Command },
    /// The preparation record could not be decoded.
    State(state::Refused),
    /// Forgejo’s files exist without a preparation record.
    Unrecorded,
    /// A recorded version is newer than its pin.
    Newer { which: Which, recorded: Box<[u8]> },
    /// A recorded version cannot be compared.
    Unreadable { which: Which },
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Found,
    Version(Which),
    Lock,
    Clear,
    Draw(u8),
    Start,
    Layout,
    Ini,
    Backup,
    Rename,
    Run(Command),
    Token,
    AdminToken,
    Password,
    RunnerSecret,
    RunnerConfiguration,
    Finish,
    End,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RunnerCourse {
    Version(Course),
    Added,
    Removed,
    None,
}
/// The opaque state of one preparation, with private credentials and no host.
#[derive(Debug)]
pub struct Prepare {
    owned: Owned,
    place: Place,
    allowed_host: Option<IpAddr>,
    phase: Phase,
    found: Option<Found>,
    state: State,
    first: bool,
    clear: bool,
    admin_token: bool,
    forgejo: Course,
    runner: RunnerCourse,
    backup: Box<[u8]>,
    name: Box<[u8]>,
    runner_secret: Option<Secret>,
    token: Option<Secret>,
    password: Option<Secret>,
}
impl Prepare {
    /// Starts idle, awaiting the host's files under Forgejo's lock.
    #[must_use]
    pub fn new(owned: Owned, place: Place, allowed_host: Option<IpAddr>) -> Self {
        let runner = owned.runner.is_some().then_some(RUNNER_VERSION.as_bytes().into());
        Self {
            owned,
            place,
            allowed_host,
            phase: Phase::Found,
            found: None,
            state: State {
                forgejo: VERSION.as_bytes().into(),
                runner,
                prepared: false,
                secrets: Secrets {
                    secret_key: Secret::new(Box::new([])),
                    internal_token: Secret::new(Box::new([])),
                    jwt_secret: Secret::new(Box::new([])),
                },
            },
            first: true,
            clear: false,
            admin_token: false,
            forgejo: Course::Same,
            runner: RunnerCourse::None,
            backup: Box::new([]),
            name: Box::new([]),
            runner_secret: None,
            token: None,
            password: None,
        }
    }
    fn request(&mut self, phase: Phase, request: Request) -> Request {
        self.phase = phase;
        request
    }
    fn end(&mut self, result: Result<(), Failure>) -> Request {
        self.request(Phase::End, Request::End(result))
    }
    fn version(&mut self, which: Which) -> Request {
        self.request(
            Phase::Version(which),
            Request::Version { which, spec: spec::version(which, &self.owned, &self.place) },
        )
    }
    fn run(&mut self, command: Command) -> Request {
        self.request(
            Phase::Run(command),
            Request::Run { command, spec: spec::command(command, &self.owned, &self.place) },
        )
    }
    fn write(&mut self, file: File, contents: Secret, phase: Phase) -> Request {
        self.request(phase, Request::Write { file, contents })
    }
    fn token(&mut self) -> Request {
        self.request(Phase::Token, Request::Token { spec: spec::admin_token(&self.owned, &self.place, &self.name) })
    }
    fn begin(&mut self) -> Request {
        if self.clear {
            self.request(Phase::Clear, Request::Clear)
        } else {
            self.request(Phase::Draw(0), Request::Draw)
        }
    }
    fn decide(&mut self) -> Request {
        let found = self.found.take().expect("Found received first");
        if let Some(bytes) = found.state {
            self.state = match state::decode(bytes.expose()) {
                Ok(state) => state,
                Err(why) => return self.end(Err(Failure::State(why))),
            };
            if self.state.prepared {
                self.first = false;
                self.forgejo = upgrade::course(&self.state.forgejo, VERSION.as_bytes());
                if let Some(why) = Self::refusal(Which::Forgejo, self.forgejo, &self.state.forgejo) {
                    return self.end(Err(why));
                }
                self.runner = match (&self.owned.runner, &self.state.runner) {
                    (Some(_), Some(version)) => {
                        let course = upgrade::course(version, RUNNER_VERSION.as_bytes());
                        if let Some(why) = Self::refusal(Which::Runner, course, version) {
                            return self.end(Err(why));
                        }
                        RunnerCourse::Version(course)
                    }
                    (Some(_), None) => RunnerCourse::Added,
                    (None, Some(_)) => RunnerCourse::Removed,
                    (None, None) => RunnerCourse::None,
                };
                self.backup = self.place.backup(&upgrade::backup_name(&self.state.forgejo, VERSION.as_bytes()));
                if self.runner == RunnerCourse::Added && found.runner_secret {
                    // The existing file remains the registration's token; no entropy is drawn.
                    self.runner_secret = None;
                } else if self.runner == RunnerCourse::Added {
                    self.runner_secret = Some(Secret::new(Box::new([])));
                }
                return if self.runner_changes() {
                    self.request(Phase::Lock, Request::Lock(File::RunnerLock))
                } else {
                    self.request(Phase::Layout, Request::Layout)
                };
            }
            self.clear = true;
        } else if found.files {
            return self.end(Err(Failure::Unrecorded));
        }
        self.state.forgejo = VERSION.as_bytes().into();
        self.state.runner = self.owned.runner.is_some().then_some(RUNNER_VERSION.as_bytes().into());
        if self.owned.runner.is_some() {
            self.request(Phase::Lock, Request::Lock(File::RunnerLock))
        } else {
            self.begin()
        }
    }
    fn refusal(which: Which, course: Course, recorded: &[u8]) -> Option<Failure> {
        match course {
            Course::Newer => Some(Failure::Newer { which, recorded: bytes::copy_of(recorded) }),
            Course::Unreadable => Some(Failure::Unreadable { which }),
            Course::Same | Course::Older => None,
        }
    }
    fn runner_changes(&self) -> bool {
        self.runner == RunnerCourse::Added || self.runner == RunnerCourse::Version(Course::Older)
    }
    fn upgrading(&self) -> bool {
        self.forgejo == Course::Older || self.runner_changes() || self.runner == RunnerCourse::Removed
    }
    fn after_ini(&mut self) -> Request {
        if self.first || self.forgejo == Course::Older { self.run(Command::Migrate) } else { self.after_migrate() }
    }
    fn after_migrate(&mut self) -> Request {
        if self.runner_changes() {
            if self.runner == RunnerCourse::Added && self.runner_secret.is_some() {
                self.request(Phase::Draw(4), Request::Draw)
            } else {
                self.run(Command::Register)
            }
        } else if self.upgrading() {
            self.finish()
        } else {
            self.later_token()
        }
    }
    fn later_token(&mut self) -> Request {
        if self.admin_token { self.end(Ok(())) } else { self.request(Phase::Draw(3), Request::Draw) }
    }
    fn ini(&mut self) -> Request {
        self.write(
            File::AppIni,
            ini::text(&self.owned, &self.place, &self.state.secrets, self.allowed_host),
            Phase::Ini,
        )
    }
    fn draw(&mut self, index: u8, value: &Secret) -> Request {
        assert_eq!(value.expose().len(), 32, "host draws 32 bytes");
        match index {
            0 => self.state.secrets.secret_key = hex(value.expose(), 32),
            1 => self.state.secrets.internal_token = hex(value.expose(), 32),
            2 => {
                let mut text = Writer::new(43);
                base64::encode(Alphabet::Url, value.expose(), &mut text).expect("32 bytes encode to 43 characters");
                self.state.secrets.jwt_secret = Secret::new(text.finish());
            }
            3 => self.name = joined(&[b"temper-admin-", hex(value.expose(), 8).expose()]),
            4 => self.runner_secret = Some(hex(value.expose(), 20)),
            _ => unreachable!("five draws at most"),
        }
        if !self.first {
            if index == 4 {
                return self.write(
                    File::RunnerSecret,
                    self.runner_secret.clone().expect("drawn secret"),
                    Phase::RunnerSecret,
                );
            }
            return self.token();
        }
        let count = if self.owned.runner.is_some() { 5 } else { 4 };
        let next = index.checked_add(1).expect("five draws");
        if next < count {
            return self.request(Phase::Draw(next), Request::Draw);
        }
        self.state.prepared = false;
        self.write(File::State, state::encode(&self.state), Phase::Start)
    }
    fn finish(&mut self) -> Request {
        self.state.forgejo = VERSION.as_bytes().into();
        self.state.runner = self.owned.runner.is_some().then_some(RUNNER_VERSION.as_bytes().into());
        self.state.prepared = true;
        self.write(File::State, state::encode(&self.state), Phase::Finish)
    }
    fn ran(&mut self, command: Command, ran: &Ran) -> Request {
        if ran.exit != Exit::Code(0) {
            return self.end(Err(Failure::Command { command, exit: ran.exit, tail: self.tail(ran) }));
        }
        match command {
            Command::Dump => self.request(
                Phase::Rename,
                Request::Rename { from: self.place.path(File::DumpPartial), to: self.backup.clone() },
            ),
            Command::Migrate if self.first => self.run(Command::CreateAdmin),
            Command::Migrate => self.after_migrate(),
            Command::CreateAdmin => self.token(),
            Command::CreatePerson => {
                let Some(password) = first_line(&ran.stdout) else {
                    return self.end(Err(Failure::Unread { command }));
                };
                self.password = Some(password.clone());
                self.write(File::PersonPassword, password, Phase::Password)
            }
            Command::Register => {
                let Some(uuid) = first_line(&ran.stdout) else {
                    return self.end(Err(Failure::Unread { command }));
                };
                let text = runner::configuration(
                    self.owned.runner.as_ref().expect("runner configured"),
                    &self.owned,
                    &self.place,
                    uuid.expose(),
                );
                self.write(File::RunnerConfiguration, Secret::new(text), Phase::RunnerConfiguration)
            }
            Command::Web | Command::Daemon => unreachable!("preparation never runs a daemon"),
        }
    }
    fn tail(&self, ran: &Ran) -> Box<[u8]> {
        for stream in [&ran.stdout, &ran.stderr] {
            assert!(
                stream.expose().len() <= usize::try_from(spec::OUTPUT).expect("stream bound"),
                "host bounds each stream"
            );
        }
        let input = Secret::new(joined(&[ran.stdout.expose(), ran.stderr.expose()]));
        let mut out = Writer::new(self.mask(input.expose(), None));
        self.mask(input.expose(), Some(&mut out));
        let masked = out.finish();
        bytes::copy_of(masked.get(masked.len().saturating_sub(4096)..).expect("failure tail"))
    }
    fn mask(&self, input: &[u8], mut output: Option<&mut Writer>) -> usize {
        let secrets =
            [&self.state.secrets.secret_key, &self.state.secrets.internal_token, &self.state.secrets.jwt_secret];
        let extra = [self.runner_secret.as_ref(), self.token.as_ref(), self.password.as_ref()];
        let mut length = 0_usize;
        let mut at = 0_usize;
        while at < input.len() {
            let remaining = input.get(at..).expect("bounded output");
            let mut skip = 0_usize;
            for secret in secrets {
                if !secret.expose().is_empty() && remaining.starts_with(secret.expose()) {
                    skip = skip.max(secret.expose().len());
                }
            }
            for secret in extra.into_iter().flatten() {
                if !secret.expose().is_empty() && remaining.starts_with(secret.expose()) {
                    skip = skip.max(secret.expose().len());
                }
            }
            let piece = if skip > 0 {
                b"[redacted]".as_slice()
            } else {
                skip = 1;
                remaining.get(..1).expect("output byte")
            };
            length = length.checked_add(piece.len()).expect("bounded masked output");
            if let Some(out) = &mut output {
                out.put(piece).expect("measured masked output");
            }
            at = at.checked_add(skip).expect("bounded output");
        }
        length
    }
}
fn first_line(value: &Secret) -> Option<Secret> {
    let input = value.expose();
    let end = bytes::find(input, b"\n").unwrap_or(input.len());
    let line = input.get(..end)?.strip_suffix(b"\r").unwrap_or(input.get(..end)?);
    if line.is_empty() { None } else { Some(Secret::new(bytes::copy_of(line))) }
}
fn hex(input: &[u8], count: usize) -> Secret {
    let mut w = Writer::new(count.checked_mul(2).expect("bounded hex"));
    for byte in input.iter().take(count) {
        for digit in [byte >> 4_u8, byte & 15] {
            let c = b"0123456789abcdef".get(usize::from(digit)).expect("hex digit");
            w.put(&[*c]).expect("measured hex");
        }
    }
    Secret::new(w.finish())
}
/// Consumes one host answer and emits exactly one next request.
pub fn step(machine: &mut Prepare, event: Event, out: &mut Queue<Request>) {
    let request = match machine.phase {
        Phase::Found => {
            let Event::Found(found) = event else { unreachable!("Found first") };
            machine.first = true;
            // A later preparation needs only the file's presence, never its bytes.
            machine.admin_token = found.admin_token;
            machine.found = Some(found);
            machine.version(Which::Forgejo)
        }
        Phase::Version(which) => {
            let Event::Printed(printed) = event else { unreachable!("version line expected") };
            let accepted = match which {
                Which::Forgejo => names_forgejo(&printed),
                Which::Runner => names_runner(&printed),
            };
            if !accepted {
                let program = spec::version(which, &machine.owned, &machine.place).program;
                machine.end(Err(Failure::Binary { which, program, printed }))
            } else if which == Which::Forgejo && machine.owned.runner.is_some() {
                machine.version(Which::Runner)
            } else {
                machine.decide()
            }
        }
        Phase::Draw(index) => {
            let Event::Drew(value) = event else { unreachable!("entropy expected") };
            machine.draw(index, &value)
        }
        Phase::Run(command) => {
            let Event::Ran(ran) = event else { unreachable!("command result expected") };
            machine.ran(command, &ran)
        }
        Phase::Backup => {
            let Event::Exists(exists) = event else { unreachable!("existence expected") };
            if exists { machine.ini() } else { machine.run(Command::Dump) }
        }
        Phase::Token => {
            let Event::Ran(ran) = event else { unreachable!("token result expected") };
            if ran.exit != Exit::Code(0) {
                machine.end(Err(Failure::Token { exit: ran.exit, tail: machine.tail(&ran) }))
            } else if let Some(token) = first_line(&ran.stdout) {
                machine.token = Some(token.clone());
                machine.write(File::AdminToken, token, Phase::AdminToken)
            } else {
                machine.end(Err(Failure::Token { exit: ran.exit, tail: machine.tail(&ran) }))
            }
        }
        phase @ (Phase::Lock
        | Phase::Clear
        | Phase::Start
        | Phase::Layout
        | Phase::Ini
        | Phase::Rename
        | Phase::AdminToken
        | Phase::Password
        | Phase::RunnerSecret
        | Phase::RunnerConfiguration
        | Phase::Finish
        | Phase::End) => {
            let Event::Done = event else { unreachable!("effect completion expected") };
            match phase {
                Phase::Lock if machine.first => machine.begin(),
                Phase::Clear => machine.request(Phase::Draw(0), Request::Draw),
                Phase::Lock | Phase::Start => machine.request(Phase::Layout, Request::Layout),
                Phase::Layout if !machine.first && machine.forgejo == Course::Older => {
                    machine.request(Phase::Backup, Request::Exists { path: machine.backup.clone() })
                }
                Phase::Layout | Phase::Rename => machine.ini(),
                Phase::Ini => machine.after_ini(),
                Phase::Finish if !machine.first => machine.later_token(),
                Phase::AdminToken if machine.first => machine.run(Command::CreatePerson),
                Phase::Finish | Phase::AdminToken => machine.end(Ok(())),
                Phase::Password => match &machine.runner_secret {
                    Some(secret) => machine.write(File::RunnerSecret, secret.clone(), Phase::RunnerSecret),
                    None => machine.finish(),
                },
                Phase::RunnerSecret => machine.run(Command::Register),
                Phase::RunnerConfiguration => machine.finish(),
                Phase::Found
                | Phase::Version(_)
                | Phase::Draw(_)
                | Phase::Run(_)
                | Phase::Token
                | Phase::Backup
                | Phase::End => {
                    unreachable!("phase handled or terminal")
                }
            }
        }
    };
    out.push(request);
}
