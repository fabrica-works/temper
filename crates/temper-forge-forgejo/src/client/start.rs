use super::{Encoded, Refusal};
use crate::{
    decode::{
        Malformed, Step,
        walk::{Kind, Move},
    },
    document::{self, ApiSettings, Document, Repository, Row, User},
    id::ObjectFormat,
    limits::Limits,
    path,
    request::{Method, Request},
};
use skein_json::{Token, tokenizer};
use skein_lib::{Decimal, Writer};
pub(super) fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
    let length = match request {
        Request::User => 12,
        Request::ApiSettings => 20,
        Request::RepositoryById { id } => 21usize.saturating_add(Decimal::of(*id).as_bytes().len()),
        Request::Repository { at } => {
            15usize.saturating_add(path::segment_length(&at.owner)).saturating_add(path::segment_length(&at.name))
        }
        Request::UserSearch { uid, limit } => {
            let mut size = 25usize.saturating_add(Decimal::of(*uid).as_bytes().len());
            if let Some(limit) = limit {
                size = size.saturating_add(7).saturating_add(Decimal::of(u64::from(*limit)).as_bytes().len());
            }
            size
        }
    };
    if length > usize::try_from(limits.target_bytes).expect("u32 fits") {
        return Err(Refusal::TooLong);
    }
    let mut out = Writer::new(length);
    let result = target(request, &mut out);
    if result.is_err() {
        return Err(Refusal::TooLong);
    }
    let target = out.finish();
    Ok(Encoded { method: Method::Get, target, body: None })
}
fn target(request: &Request, out: &mut Writer) -> Result<(), skein_lib::Overflow> {
    out.put(b"/api/v1/")?;
    match request {
        Request::User => out.put(b"user"),
        Request::ApiSettings => out.put(b"settings/api"),
        Request::RepositoryById { id } => {
            out.put(b"repositories/")?;
            out.put(Decimal::of(*id).as_bytes())
        }
        Request::Repository { at } => {
            out.put(b"repos/")?;
            path::segment(&at.owner, out)?;
            out.put(b"/")?;
            path::segment(&at.name, out)
        }
        Request::UserSearch { uid, limit } => {
            out.put(b"users/search?uid=")?;
            out.put(Decimal::of(*uid).as_bytes())?;
            if let Some(limit) = limit {
                out.put(b"&limit=")?;
                out.put(Decimal::of(u64::from(*limit)).as_bytes())?;
            }
            Ok(())
        }
    }
}
#[derive(Debug)]
pub(super) struct Start {
    kind: Kind,
    user: User,
    repo: Repository,
    settings: ApiSettings,
    ok: bool,
}
impl Start {
    pub(super) fn new(kind: Kind) -> Self {
        Self {
            kind,
            user: document::user(),
            repo: document::repository(),
            settings: ApiSettings { max_response_items: 0, default_paging_num: 0 },
            ok: false,
        }
    }
    pub(super) fn step(&mut self, movement: Move, parent: Option<Kind>, event: &tokenizer::Event) -> Step {
        match movement {
            Move::Demand(demand) => Step::Demand(demand),
            Move::Scalar { kind, key } => match self.scalar(kind, key, event) {
                Ok(()) => Step::Demand(tokenizer::Request::Next),
                Err(error) => Step::Malformed(error),
            },
            Move::End(Kind::User) => match parent {
                Some(Kind::Repository) => {
                    self.repo.owner = core::mem::replace(&mut self.user, document::user());
                    Step::Demand(tokenizer::Request::Next)
                }
                Some(Kind::Users) => {
                    let user = core::mem::replace(&mut self.user, document::user());
                    Step::Row(Row::User(user), tokenizer::Request::Next)
                }
                None | Some(Kind::User | Kind::Settings | Kind::Search) => Step::Demand(tokenizer::Request::Next),
            },
            Move::End(Kind::Settings | Kind::Repository | Kind::Search | Kind::Users) => {
                Step::Demand(tokenizer::Request::Next)
            }
            Move::Done => {
                let document = match self.kind {
                    Kind::User => Document::User(core::mem::replace(&mut self.user, document::user())),
                    Kind::Settings => Document::ApiSettings(self.settings),
                    Kind::Repository => {
                        Document::Repository(core::mem::replace(&mut self.repo, document::repository()))
                    }
                    Kind::Search => Document::Search { ok: self.ok },
                    Kind::Users => unreachable!("no standalone users answer"),
                };
                Step::Done(document)
            }
        }
    }
    fn scalar(&mut self, kind: Kind, key: &'static [u8], event: &tokenizer::Event) -> Result<(), Malformed> {
        match event {
            tokenizer::Event::Token(Token::Number(text)) => {
                let number = crate::id::unsigned(text).ok_or(Malformed::Range { field: key })?;
                match kind {
                    Kind::User => self.user.id = number,
                    Kind::Repository => self.repo.id = number,
                    Kind::Settings => {
                        let Ok(n) = u32::try_from(number) else {
                            return Err(Malformed::Range { field: key });
                        };
                        match key {
                            b"max_response_items" => self.settings.max_response_items = n,
                            b"default_paging_num" => self.settings.default_paging_num = n,
                            _ => unreachable!("settings table"),
                        }
                    }
                    Kind::Search | Kind::Users => unreachable!("number table"),
                }
            }
            tokenizer::Event::Token(Token::String(text)) => match kind {
                Kind::User => self.user.login.clone_from(text),
                Kind::Repository => match key {
                    b"name" => self.repo.name.clone_from(text),
                    b"default_branch" => self.repo.default_branch.clone_from(text),
                    b"html_url" => self.repo.html_url.clone_from(text),
                    b"object_format_name" => {
                        self.repo.object_format = match &**text {
                            b"sha1" => ObjectFormat::Sha1,
                            b"sha256" => ObjectFormat::Sha256,
                            _ => return Err(Malformed::Value { field: key }),
                        }
                    }
                    b"default_merge_style" => {
                        self.repo.default_merge_style = document::style(text).ok_or(Malformed::Value { field: key })?;
                    }
                    _ => unreachable!("repository string table"),
                },
                Kind::Settings | Kind::Search | Kind::Users => unreachable!("string table"),
            },
            tokenizer::Event::Token(Token::True | Token::False) => {
                let value = event == &tokenizer::Event::Token(Token::True);
                match kind {
                    Kind::Search => self.ok = value,
                    Kind::Repository => match key {
                        b"allow_merge_commits" => self.repo.allow_merge_commits = value,
                        b"allow_squash_merge" => self.repo.allow_squash_merge = value,
                        b"allow_rebase" => self.repo.allow_rebase = value,
                        _ => unreachable!("repository bool table"),
                    },
                    Kind::User | Kind::Settings | Kind::Users => unreachable!("boolean table"),
                }
            }
            tokenizer::Event::Token(
                Token::ObjectStart
                | Token::ObjectEnd
                | Token::ArrayStart
                | Token::ArrayEnd
                | Token::Key(_)
                | Token::Null,
            )
            | tokenizer::Event::Long(_)
            | tokenizer::Event::Skipped(_)
            | tokenizer::Event::Done
            | tokenizer::Event::Failed(_)
            | tokenizer::Event::Closed => unreachable!("walker validates scalar types"),
        }
        Ok(())
    }
}
