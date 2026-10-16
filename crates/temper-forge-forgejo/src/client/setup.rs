use crate::{
    decode::{
        Malformed, Step,
        walk::{Field, Kind, Move, Read, Value},
    },
    document::{self, Document, User},
};
use skein_json::tokenizer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SetupKind {
    Account,
}
pub(crate) fn fields(kind: SetupKind) -> &'static [Field] {
    match kind {
        SetupKind::Account => {
            const {
                &[
                    Field { key: b"id", read: Read::Number, required: true },
                    Field { key: b"login", read: Read::Text, required: true },
                    Field { key: b"is_admin", read: Read::Boolean, required: false },
                    Field { key: b"full_name", read: Read::Text, required: false },
                ]
            }
        }
    }
}
#[derive(Debug)]
pub(super) struct Setup {
    user: User,
}
impl Setup {
    pub(super) fn new() -> Self {
        Self { user: document::user() }
    }
    pub(super) fn step(&mut self, movement: Move) -> Step {
        match movement {
            Move::Demand(demand) => return Step::Demand(demand),
            Move::Scalar { key, value, .. } => match value {
                Value::Number(value) => self.user.id = value,
                Value::Boolean(value) => self.user.is_admin = value,
                Value::Text(text) => {
                    if key == b"login" {
                        self.user.login = text;
                    } else {
                        self.user.full_name = text;
                    }
                }
                Value::Body(_) | Value::Time(_) | Value::Id(_) => {
                    return Step::Malformed(Malformed::Shape { field: key });
                }
            },
            Move::End(Kind::Setup(_)) => {}
            Move::End(
                Kind::User
                | Kind::Error
                | Kind::Write(_)
                | Kind::Settings
                | Kind::Repository
                | Kind::Search
                | Kind::Users
                | Kind::File(_)
                | Kind::Action(_)
                | Kind::Item(_)
                | Kind::Review(_)
                | Kind::Branch(_),
            ) => unreachable!("setup tables"),
            Move::Done => return Step::Done(Document::User(core::mem::replace(&mut self.user, document::user()))),
        }
        Step::Demand(tokenizer::Request::Next)
    }
}

use super::{Encoded, Refusal, items::Target};
use crate::{
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_json::writer::{self, Encoder};
use skein_lib::Writer;
pub(super) fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
    if let Request::UserOf { login } = request {
        let mut measure = Target { length: 0, writer: None };
        measure.put(b"/api/v1/users/");
        measure.segment(login);
        if measure.length > usize::try_from(limits.target_bytes).expect("u32 fits") {
            return Err(Refusal::TooLong);
        }
        let mut out = Target { length: 0, writer: Some(Writer::new(measure.length)) };
        out.put(b"/api/v1/users/");
        out.segment(login);
        return Ok(Encoded { method: Method::Get, target: out.writer.expect("writing pass").finish(), body: None });
    }
    let target = b"/api/v1/admin/users".as_slice();
    if target.len() > usize::try_from(limits.target_bytes).expect("u32 fits") {
        return Err(Refusal::TooLong);
    }
    let body = {
        let mut measure = Encoder::measure(&limits.writer);
        write(&mut measure, request);
        let length = match measure.measured() {
            Ok(length) => length,
            Err(writer::Refusal::Text) => return Err(Refusal::Text),
            Err(writer::Refusal::TooLong | writer::Refusal::TooDeep | writer::Refusal::Number) => {
                return Err(Refusal::TooLong);
            }
        };
        let mut out = Encoder::write(length, &limits.writer);
        write(&mut out, request);
        Some(out.finish())
    };
    Ok(Encoded { method: Method::Post, target: Box::from(target), body })
}
fn write(json: &mut Encoder, request: &Request) {
    let Request::CreateUser { login, email, password, full_name, must_change_password } = request else {
        unreachable!("user body selection")
    };
    json.object_start();
    for (key, value) in [
        (b"username".as_slice(), login.as_ref()),
        (b"email", email),
        (b"password", password.expose()),
        (b"full_name", full_name),
    ] {
        json.key(key);
        json.string(value);
    }
    json.key(b"must_change_password");
    json.boolean(*must_change_password);
    json.object_end();
}
