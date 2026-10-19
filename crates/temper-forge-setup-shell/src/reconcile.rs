//! Reconcile's bounded host: one machine, one connection at a time.
use crate::exchange::{Exchange, pause};
use skein_forge_setup::reconcile::{self, Event, Failure, Need, Reconcile, Reply, Stage, Why};
use skein_io::{
    Io, Request,
    kernel::{Complete, Submit},
};
use skein_lib::{Duration, Env, Queue, Secret, Time, Token, Wall};
use skein_shell::{Clock, Config, Host, Kernel};
use std::{net::SocketAddr, os::unix::ffi::OsStrExt, path::PathBuf};
use temper_forge_setup::{
    given,
    place::{self, File, Place},
    problem::{self, Problems},
};
/// Checked command settings, including the bounds supplied by the profile.
pub struct Settings {
    pub setup_index: u32,
    pub given: skein_forge_setup::given::Given,
    pub address: SocketAddr,
    pub host: Box<[u8]>,
    pub user_agent: &'static str,
    pub directory: PathBuf,
    pub admin_token: PathBuf,
    pub token_bytes: u32,
    pub hook_secret_bytes: u32,
    pub http: skein_http::client::Limits,
    pub documents: skein_forge::forgejo::limits::Limits,
    pub io: skein_io::Limits,
}
/// Why the command did not converge; credentials are never retained here.
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    Unanswered { forge: u16, deadline: Duration },
    Limits { forge: u16, name: &'static str },
    File { forge: u16, file: File, why: String },
    Stopped(Failure),
    Setup { forge: u16, problems: Problems },
}
impl Refused {
    /// One diagnostic line for the command's standard error.
    #[must_use]
    pub fn line(&self) -> String {
        let (forge, words) = match self {
            Self::Setup { forge, problems } => {
                return format!(
                    "temper forge reconcile: forge {forge}: {}",
                    String::from_utf8_lossy(&problem::lines(problems))
                );
            }
            Self::Unanswered { forge, deadline } => {
                (*forge, format!("Forgejo did not answer within {} ms", deadline.as_nanos() / 1_000_000))
            }
            Self::Limits { forge, name } => (*forge, format!("io.{name}: below the http client's largest read")),
            Self::File { forge, file, why } => (*forge, format!("{}: {why}", file_name(*file))),
            Self::Stopped(failure) => {
                let stage = match failure.step {
                    Stage::Admin => "admin",
                    Stage::Bot => "bot",
                    Stage::Token => "token",
                    Stage::Org => "org",
                    Stage::Hook => "hook",
                };
                let why = match failure.why {
                    Why::Missing | Why::OrgMissing => "missing".into(),
                    Why::Refused(status) => format!("HTTP {status}"),
                    Why::NotAdmin => "not a site admin".into(),
                    Why::BotIsAdmin => "is a site admin".into(),
                    Why::Unmarked => "has no temper:bot marker".into(),
                    Why::Down => "unavailable".into(),
                    Why::Unexpected => "unexpected answer".into(),
                };
                (failure.forge, format!("{stage} {}: {why}", String::from_utf8_lossy(&failure.subject)))
            }
        };
        format!("temper forge reconcile: forge {forge}: {words}\n")
    }
}
/// The private-file bound, including the user's optional final newline.
#[must_use]
pub fn bound(settings: &Settings, file: File) -> u32 {
    match file {
        File::User => 21,
        File::WebhookSecret => settings.hook_secret_bytes,
        File::AppIni
        | File::State
        | File::AdminToken
        | File::PersonPassword
        | File::ApiToken
        | File::GitToken
        | File::RunnerSecret
        | File::RunnerConfiguration
        | File::DumpPartial
        | File::ForgejoLock
        | File::RunnerLock => settings.token_bytes,
    }
}
fn file_name(file: File) -> String {
    let place = Place { directory: Box::new([]), user: Box::new([]) };
    String::from_utf8_lossy(&place.path(file)).trim_start_matches('/').into()
}
fn file_path(settings: &Settings, file: File) -> PathBuf {
    if file == File::AdminToken {
        return settings.admin_token.clone();
    }
    let place = Place { directory: settings.directory.as_os_str().as_bytes().into(), user: Box::new([]) };
    crate::path(&place.path(file)).into()
}
fn unread(why: skein_shell::Unread) -> String {
    match why {
        skein_shell::Unread::Absent => "absent".into(),
        skein_shell::Unread::NotAFile => "not a file".into(),
        skein_shell::Unread::TooLarge { max } => format!("past {max} bytes"),
        skein_shell::Unread::Unsafe(_) => "readable by others or reached through a link".into(),
        skein_shell::Unread::Kernel(_) => "could not read".into(),
    }
}
const OWNER: Token = Token::new(1);
struct Running {
    settings: Settings,
    io: Io,
    machine: Reconcile,
    needs: Queue<Need>,
    event: Option<Event>,
    exchange: Option<Exchange>,
    pending: Option<(skein_forge::forgejo::request::Request, Secret)>,
    socket: Option<Token>,
    connected: bool,
    closing: bool,
    first: bool,
    attempt: u32,
    retry: Option<Time>,
    began: Time,
    end: Time,
    result: Option<Result<(), Refused>>,
    operations: u32,
    complete: Queue<Complete>,
    subs: Queue<Submit>,
    events: Queue<skein_io::Event>,
}
impl Running {
    fn down(&mut self, env: &Env<skein_io::Limits>, request: Request) {
        skein_io::down(&mut self.io, env, request, &mut self.subs);
    }
    fn file_error(&self, file: File, why: String) -> Refused {
        Refused::File { forge: self.settings.given.forge, file, why }
    }
    fn local(&mut self, need: Need, now: Time, wall: Wall) {
        match need {
            Need::Send { request, token } => {
                self.pending = Some((request, token));
                self.begin(now, wall);
            }
            Need::Read(item) => {
                let file = place::item(item);
                match skein_shell::read_private_file(
                    &file_path(&self.settings, file),
                    bound(&self.settings, file),
                    skein_shell::effective_user(),
                ) {
                    Ok(bytes) => {
                        let bytes = bytes.expose();
                        self.event =
                            Some(Event::Read(Some(Secret::new(bytes.strip_suffix(b"\n").unwrap_or(bytes).into()))));
                    }
                    Err(skein_shell::Unread::Absent) => self.event = Some(Event::Read(None)),
                    Err(why) => self.result = Some(Err(self.file_error(file, unread(why)))),
                }
            }
            Need::Write { item, bytes } => {
                let file = place::item(item);
                match crate::files::write_private(&file_path(&self.settings, file), bytes.expose()) {
                    Ok(()) => self.event = Some(Event::Written),
                    Err(_) => self.result = Some(Err(self.file_error(file, "could not write whole".into()))),
                }
            }
            Need::Draw(_) => match skein_shell::crypto_seed() {
                Ok(seed) => self.event = Some(Event::Drawn(seed)),
                Err(_) => self.result = Some(Err(self.file_error(File::AdminToken, "entropy unavailable".into()))),
            },
            Need::Done => self.result = Some(Ok(())),
            Need::Stop(failure) => self.result = Some(Err(Refused::Stopped(failure))),
        }
    }
    fn begin(&mut self, now: Time, wall: Wall) {
        let (request, token) = self.pending.as_ref().expect("pending request");
        if let Ok(exchange) = Exchange::new(&self.settings, request, token, now, wall) {
            self.exchange = Some(exchange);
            self.down(
                &Env { now, wall, limits: self.settings.io },
                Request::Connect { owner: OWNER, addr: skein_io::Addr::Ip(self.settings.address) },
            );
        } else {
            self.first = false;
            self.event = Some(Event::Reply(Reply::Down));
            self.pending = None;
        }
    }

    fn answered(&mut self, reply: Reply, now: Time) {
        self.exchange = None;
        if self.first && matches!(reply, Reply::Down | Reply::Status(500..=599)) {
            if let Some(wait) = pause(
                self.settings.given.limits.backoff,
                self.settings.given.limits.backoff_max,
                self.settings.given.limits.deadline,
                self.attempt,
                self.began,
                now,
            ) {
                self.attempt = self.attempt.saturating_add(1);
                self.retry = Some(now.saturating_add(wait));
            } else {
                self.result = Some(Err(Refused::Unanswered {
                    forge: self.settings.given.forge,
                    deadline: self.settings.given.limits.deadline,
                }));
            }
        } else {
            self.first = false;
            self.pending = None;
            self.event = Some(Event::Reply(reply));
        }
    }
    fn events(&mut self, env: &Env<skein_io::Limits>) {
        while let Some(event) = self.events.pop() {
            match event {
                skein_io::Event::Connecting { socket, .. } => self.socket = Some(socket),
                skein_io::Event::Connected { .. } => self.connected = true,
                skein_io::Event::Stream { up, .. } => {
                    if let Some(exchange) = &mut self.exchange {
                        exchange.stream(&self.settings, env.now, env.wall, up);
                    }
                }
                skein_io::Event::Failed { .. } => {
                    if self.exchange.is_some() {
                        self.answered(Reply::Down, env.now);
                    }
                    self.connected = false;
                    self.closing = self.socket.is_some();
                }
                skein_io::Event::Closed { .. } => {
                    self.closing = false;
                    self.connected = false;
                    self.socket = None;
                }
                skein_io::Event::Listening { .. }
                | skein_io::Event::Accepted { .. }
                | skein_io::Event::Output { .. }
                | skein_io::Event::Spawned { .. }
                | skein_io::Event::Exited { .. }
                | skein_io::Event::Usage { .. }
                | skein_io::Event::Resized
                | skein_io::Event::Window { .. }
                | skein_io::Event::Shutdown { .. } => unreachable!("only socket operations"),
            }
        }
    }
}
impl Host for Running {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.settings.io };
        while self.io.is_ready() {
            skein_io::resume(&mut self.io, &env, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        while let Some(complete) = self.complete.pop() {
            skein_io::up(&mut self.io, &env, complete, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        while self.io.is_due(now) {
            skein_io::fire(&mut self.io, &env, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        if self.result.is_none() && now >= self.end {
            self.retry = None;
            if self.first {
                self.result = Some(Err(Refused::Unanswered {
                    forge: self.settings.given.forge,
                    deadline: self.settings.given.limits.deadline,
                }));
                self.exchange = None;
            } else if self.exchange.is_some() {
                self.answered(Reply::Down, now);
            }
        }
        if self.exchange.is_none() && self.socket.is_some() && !self.closing {
            self.closing = true;
            self.down(&env, Request::Close { entity: self.socket.expect("socket to close") });
        }
        if self.closing {
            self.io.reclaim();
            return;
        }
        if self.retry.is_some_and(|time| now >= time) && self.result.is_none() {
            self.retry = None;
            self.begin(now, wall);
        }
        loop {
            if self.result.is_some() || self.retry.is_some() {
                break;
            }
            if let Some(event) = self.event.take() {
                reconcile::step(&mut self.machine, event, &mut self.needs);
            }
            if let Some(need) = self.needs.pop() {
                self.local(need, now, wall);
                continue;
            }
            if !self.connected {
                break;
            }
            let down = self.exchange.as_mut().and_then(|exchange| exchange.pump(&self.settings, now, wall));
            if let Some(down) = down {
                self.down(&env, Request::Stream { stream: self.socket.expect("connected socket"), down });
                continue;
            }
            if let Some(reply) = self.exchange.as_mut().and_then(|exchange| exchange.reply.take()) {
                self.answered(reply, now);
                self.closing = true;
                self.down(&env, Request::Close { entity: self.socket.expect("socket to close") });
            }
            break;
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.subs
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.complete.is_empty()
            || (!self.closing && (self.event.is_some() || !self.needs.is_empty()))
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.retry, self.result.is_none().then_some(self.end)].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.result.is_some() && self.io.is_empty()
    }
    fn worst_case(&self) -> u64 {
        skein_io::worst_case(&self.settings.io).expect("io memory fits")
            + skein_http::client::worst_case(&self.settings.http).expect("http memory fits")
            + skein_json::tokenizer::worst_case(&self.settings.documents.json).expect("json memory fits")
            + Queue::<Complete>::worst_case(self.operations).expect("completions fit")
            + Queue::<Submit>::worst_case(self.operations).expect("submissions fit")
            + Queue::<skein_io::Event>::worst_case(self.events.capacity()).expect("events fit")
            // Each row needs at least one input byte; retained text is bounded by the input too.
            + u64::from(self.settings.documents.json.length).saturating_mul(u64::try_from(size_of::<skein_forge::forgejo::document::Row>()).expect("row size fits").saturating_add(1))
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}
/// Runs one reconcile machine to completion on its own kernel and loop.
pub fn reconcile(settings: Settings, clock: &Clock) -> Result<(), Refused> {
    let machine = Reconcile::new(&settings.given).map_err(|why| Refused::Setup {
        forge: settings.given.forge,
        problems: given::refusal(settings.setup_index, why),
    })?;
    for (name, enough) in [
        ("intake", settings.io.intake >= skein_http::client::largest_read(&settings.http)),
        ("output", settings.io.output >= skein_http::client::largest_room(&settings.http)),
    ] {
        if !enough {
            return Err(Refused::Limits { forge: settings.given.forge, name });
        }
    }
    let operations = skein_io::operations(&settings.io).expect("usable io limits");
    let mut kernel = Kernel::open(Config { operations }).map_err(|_| Refused::File {
        forge: settings.given.forge,
        file: File::AdminToken,
        why: "kernel unavailable".into(),
    })?;
    let began = clock.now().now;
    let mut running = Running {
        machine,
        io: Io::new(&settings.io),
        end: began.saturating_add(settings.given.limits.deadline),
        settings,
        needs: Queue::with_capacity(reconcile::MAX_OUT),
        event: Some(Event::Begin),
        exchange: None,
        pending: None,
        socket: None,
        connected: false,
        closing: false,
        first: true,
        attempt: 0,
        retry: None,
        began,
        result: None,
        operations,
        complete: Queue::with_capacity(operations),
        subs: Queue::with_capacity(operations),
        events: Queue::with_capacity(
            skein_io::MAX_OUT_UP.events.max(skein_io::MAX_OUT_RESUME.events).max(skein_io::MAX_OUT_FIRE.events),
        ),
    };
    skein_shell::drive(&mut kernel, clock, &mut running);
    running.result.expect("terminal reconcile")
}
