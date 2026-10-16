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
