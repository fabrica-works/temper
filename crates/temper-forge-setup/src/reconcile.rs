//! One forge reconciled from its answers, with one need outstanding (setup.md, 5).
use crate::{
    place::File,
    setup::{Account, Entry},
};
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_lib::{Decimal, Queue, Secret, Writer, bytes};
use temper_forge_forgejo::{
    document::{Document, Row},
    request::Request,
};
/// Room the host reserves before stepping.
pub const MAX_OUT: u32 = 1;
/// One effect for the host, or the run's terminal outcome.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Need {
    Send { request: Request, token: Secret },
    Read(File),
    Write { file: File, bytes: Secret },
    Draw(u32),
    Done,
    Stop(Failure),
}
/// The outcome of the outstanding need.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "a reply's document moves whole, as the store's records do")]
pub enum Event {
    Begin,
    Read(Option<Secret>),
    Reply(Reply),
    Written,
    Drawn(Secret),
}
/// An HTTP answer decoded by the host.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "a document moves whole, as the store's records do")]
pub enum Reply {
    Document(Document),
    Rows(Box<[Row]>),
    Empty,
    Status(u16),
    Down,
}
/// Names the stopped step, never its credentials.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure {
    pub forge: u16,
    pub step: Stage,
    pub subject: Box<[u8]>,
    pub why: Why,
}
/// Setup's six steps, with the bot's id included in Bot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Admin,
    Bot,
    Token,
    Org,
    Hook,
}
/// Why the forge could not be reconciled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Missing,
    Refused(u16),
    NotAdmin,
    BotIsAdmin,
    Unmarked,
    OrgMissing,
    Down,
    Unexpected,
}
#[derive(Debug)]
enum State {
    Begin,
    AdminFile,
    Admin,
    Bot,
    Draw,
    Create,
    Id(u64),
    Written,
    Ended,
}
/// One forge's state; credentials remain beside the machine's requests.
#[derive(Debug)]
pub struct Reconcile {
    forge: u16,
    bot: Account,
    admin: Option<Secret>,
    state: State,
}
impl Reconcile {
    /// Starts one entry; hooks use the listener when their step is reached.
    #[must_use]
    pub fn new(entry: &Entry, _listener: Option<SocketAddr>) -> Self {
        Self { forge: entry.forge, bot: entry.bot.clone(), admin: None, state: State::Begin }
    }
    fn send(&self, request: Request) -> Need {
        Need::Send { request, token: self.admin.clone().expect("admin checked before bot") }
    }
    fn stop(&mut self, stage: Stage, why: Why) -> Need {
        self.state = State::Ended;
        self.admin = None;
        let subject = match stage {
            Stage::Admin => bytes::copy_of(b"temper-admin"),
            Stage::Bot | Stage::Token | Stage::Org | Stage::Hook => self.bot.login.clone(),
        };
        Need::Stop(Failure { forge: self.forge, step: stage, subject, why })
    }
    fn id(&mut self, user: temper_forge_forgejo::document::User) -> Need {
        if user.is_admin {
            return self.stop(Stage::Bot, Why::BotIsAdmin);
        }
        if bytes::find(&user.full_name, b"temper:bot").is_none() {
            return self.stop(Stage::Bot, Why::Unmarked);
        }
        self.state = State::Id(user.id);
        Need::Read(File::User)
    }
    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::Begin => match event {
                Event::Begin => {
                    self.state = State::AdminFile;
                    Need::Read(File::AdminToken)
                }
                Event::Read(_) | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Admin, Why::Unexpected)
                }
            },
            State::AdminFile => match event {
                Event::Read(Some(token)) if !token.is_empty() => {
                    self.admin = Some(token);
                    self.state = State::Admin;
                    self.send(Request::User)
                }
                Event::Read(_) => self.stop(Stage::Admin, Why::Missing),
                Event::Begin | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Admin, Why::Unexpected)
                }
            },
            State::Admin => self.admin(event),
            State::Bot => self.bot(event),
            State::Draw => self.draw(event),
            State::Create => match account(event) {
                Ok(user) => self.id(user),
                Err(why) => self.stop(Stage::Bot, why),
            },
            State::Id(id) => match event {
                Event::Read(value) => {
                    let digits = Decimal::of(id);
                    if match value {
                        Some(value) => value.expose() == digits.as_bytes(),
                        None => false,
                    } {
                        self.admin = None;
                        Need::Done
                    } else {
                        self.state = State::Written;
                        Need::Write { file: File::User, bytes: Secret::new(bytes::copy_of(digits.as_bytes())) }
                    }
                }
                Event::Begin | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Bot, Why::Unexpected)
                }
            },
            State::Written => match event {
                Event::Written => {
                    self.admin = None;
                    Need::Done
                }
                Event::Begin | Event::Read(_) | Event::Reply(_) | Event::Drawn(_) => {
                    self.stop(Stage::Bot, Why::Unexpected)
                }
            },
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }
    fn admin(&mut self, event: Event) -> Need {
        match account(event) {
            Ok(user) => {
                if !user.is_admin {
                    return self.stop(Stage::Admin, Why::NotAdmin);
                }
                self.state = State::Bot;
                self.send(Request::UserOf { login: self.bot.login.clone() })
            }
            Err(why) => self.stop(Stage::Admin, why),
        }
    }
    fn bot(&mut self, event: Event) -> Need {
        match account(event) {
            Ok(user) => self.id(user),
            Err(Why::Refused(404)) => {
                self.state = State::Draw;
                Need::Draw(32)
            }
            Err(why) => self.stop(Stage::Bot, why),
        }
    }
    fn draw(&mut self, event: Event) -> Need {
        match event {
            Event::Drawn(value) if value.len() == 32 => {
                let mut password = Writer::new(64);
                for byte in value.expose() {
                    for nibble in [byte >> 4, byte & 15] {
                        let digit = b"0123456789abcdef".get(usize::from(nibble)).expect("hex digit");
                        password.put(&[*digit]).expect("32 bytes as 64 hex digits");
                    }
                }
                self.state = State::Create;
                self.send(Request::CreateUser {
                    login: self.bot.login.clone(),
                    email: self.bot.email.clone(),
                    full_name: bytes::copy_of(b"temper:bot"),
                    password: Secret::new(password.finish()),
                    must_change_password: false,
                })
            }
            Event::Begin | Event::Read(_) | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                self.stop(Stage::Bot, Why::Unexpected)
            }
        }
    }
}
fn account(event: Event) -> Result<temper_forge_forgejo::document::User, Why> {
    match event {
        Event::Reply(reply) => match reply {
            Reply::Document(document) => match document {
                Document::User(user) => Ok(user),
                Document::Token(_)
                | Document::ApiSettings(_)
                | Document::Repository(_)
                | Document::Search { .. }
                | Document::Listed { .. }
                | Document::Issue(_)
                | Document::Comment(_)
                | Document::Status(_)
                | Document::Review(_)
                | Document::Pull(_)
                | Document::CombinedStatus(_)
                | Document::Branch(_)
                | Document::Protection(_)
                | Document::Permission(_)
                | Document::Compare(_)
                | Document::Runs { .. }
                | Document::Run(_)
                | Document::Job(_)
                | Document::Error(_) => Err(Why::Unexpected),
            },
            Reply::Status(status) => Err(Why::Refused(status)),
            Reply::Down => Err(Why::Down),
            Reply::Rows(_) | Reply::Empty => Err(Why::Unexpected),
        },
        Event::Begin | Event::Read(_) | Event::Written | Event::Drawn(_) => Err(Why::Unexpected),
    }
}
/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Reconcile, event: Event, out: &mut Queue<Need>) {
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Begin
        | State::AdminFile
        | State::Admin
        | State::Bot
        | State::Draw
        | State::Create
        | State::Id(_)
        | State::Written => {
            let need = machine.advance(state, event);
            out.push(need);
        }
    }
}
