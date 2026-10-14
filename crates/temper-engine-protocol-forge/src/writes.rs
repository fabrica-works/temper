//! Forge writes, with validation before resolution and no retry (forge.md, 3.2–3.4).
use crate::{
    connection::{Body, Heard},
    failure::{self, Asked, Drop},
    limits::Limits,
    marker,
    plan::{Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
use alloc::boxed::Box;
use skein_lib::Duration;
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::{
    document as d,
    id::ObjectId,
    request::{self as r, Request},
};
#[derive(Debug)]
pub(crate) struct Plan {
    repository: api::Repository,
    write: api::Write,
    row: Option<Repository>,
    resolving: bool,
    status: u16,
    retry: Option<Duration>,
    request: Option<Exchange>,
}
impl Plan {
    pub fn pending(&mut self) -> Option<Exchange> {
        self.request.take()
    }
    pub fn set_pending(&mut self, request: Exchange) {
        self.request = Some(request);
    }
    pub fn new(repository: api::Repository, write: api::Write) -> Self {
        Self { repository, write, row: None, resolving: false, status: 0, retry: None, request: None }
    }
    pub fn begin(&mut self, l: &Limits, tables: &mut Tables) -> Result<Step, api::Error> {
        validate(&self.write)?;
        self.row = tables.repository(self.repository).cloned();
        if self.row.is_none() {
            self.resolving = true;
            return Ok(Step::send(Request::RepositoryById { id: u64::from(self.repository.repository) }));
        }
        self.initial(l)
    }
    fn at(&self) -> r::Repo {
        let row = self.row.as_ref().expect("resolved repository");
        r::Repo { owner: row.owner.clone(), name: row.name.clone() }
    }
    fn id(&self, bytes: api::Commit) -> ObjectId {
        ObjectId { bytes, format: self.row.as_ref().expect("repository format").object_format }
    }
    fn initial(&mut self, l: &Limits) -> Result<Step, api::Error> {
        let at = self.at();
        let request = match self.write.clone() {
            api::Write::CreateIssue { key, title, body } => {
                Request::CreateIssue { at, title, body: marked(&key, &body, l)? }
            }
            api::Write::Post { number, key, body } => {
                Request::PostComment { at, number, body: marked(&key, &body, l)? }
            }
            api::Write::Review { number, key, verdict, body } => {
                let event = match verdict {
                    api::Verdict::Approve => r::ReviewEvent::Approved,
                    api::Verdict::RequestChanges => r::ReviewEvent::RequestChanges,
                    api::Verdict::Comment => r::ReviewEvent::Comment,
                };
                Request::PostReview { at, number, event, body: marked(&key, &body, l)? }
            }
            api::Write::Edit { number, title, body } => Request::EditIssue { at, number, title, body, state: None },
            api::Write::Close { number } => {
                Request::EditIssue { at, number, title: None, body: None, state: Some(d::IssueState::Closed) }
            }
            api::Write::Reopen { number } => {
                Request::EditIssue { at, number, title: None, body: None, state: Some(d::IssueState::Open) }
            }
            api::Write::Update { number } => Request::Update { at, number, style: Some(r::UpdateStyle::Merge) },
            api::Write::Status { commit, context, check } => {
                let state = match check {
                    api::Check::Pending => d::StatusState::Pending,
                    api::Check::Passed => d::StatusState::Success,
                    api::Check::Failed => d::StatusState::Failure,
                };
                Request::PostStatus { at, commit: self.id(commit), state, context }
            }
            api::Write::CreateBranch { branch, commit } => Request::CreateBranch { at, branch, from: self.id(commit) },
            api::Write::DeleteBranch { branch } => Request::DeleteBranch { at, branch },
            api::Write::OpenPull { .. } | api::Write::SetReviewers { .. } | api::Write::Merge { .. } => {
                unreachable!("multiexchange writes not assigned yet")
            }
        };
        Ok(Step::send(request))
    }
    fn asked(&self) -> Asked {
        match self.write {
            api::Write::OpenPull { .. } => Asked::OpenPull,
            api::Write::Merge { .. } => Asked::Merge,
            api::Write::Update { .. } => Asked::Update,
            api::Write::DeleteBranch { .. } => Asked::DeleteBranch,
            api::Write::CreateIssue { .. }
            | api::Write::Post { .. }
            | api::Write::Review { .. }
            | api::Write::Edit { .. }
            | api::Write::SetReviewers { .. }
            | api::Write::Close { .. }
            | api::Write::Reopen { .. }
            | api::Write::Status { .. }
            | api::Write::CreateBranch { .. } => Asked::Write,
        }
    }
    fn expected(&self) -> u16 {
        if self.resolving {
            return 200;
        }
        match self.write {
            api::Write::Update { .. } => 200,
            api::Write::DeleteBranch { .. } => 204,
            api::Write::CreateIssue { .. }
            | api::Write::Post { .. }
            | api::Write::Review { .. }
            | api::Write::Edit { .. }
            | api::Write::Close { .. }
            | api::Write::Reopen { .. }
            | api::Write::Status { .. }
            | api::Write::CreateBranch { .. } => 201,
            api::Write::OpenPull { .. } | api::Write::SetReviewers { .. } | api::Write::Merge { .. } => {
                unreachable!("multiexchange writes not assigned yet")
            }
        }
    }
    pub fn heard(&mut self, heard: Heard, l: &Limits, tables: &mut Tables) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                self.status = status;
                self.retry = fields.retry_after;
                Ok(None)
            }
            Heard::Row(_) => Ok(None),
            Heard::Failed(fault) => Err(failure::transport(fault)),
            Heard::Done(body) => {
                if !self.resolving && self.status == 409 {
                    match self.write {
                        api::Write::CreateBranch { .. } => {
                            return Ok(Some(Step::Done(Ok(api::Answer::Branch(api::BranchCreation::Exists)))));
                        }
                        api::Write::CreateIssue { .. }
                        | api::Write::Post { .. }
                        | api::Write::Review { .. }
                        | api::Write::Edit { .. }
                        | api::Write::Close { .. }
                        | api::Write::Reopen { .. }
                        | api::Write::Status { .. }
                        | api::Write::Update { .. }
                        | api::Write::DeleteBranch { .. }
                        | api::Write::OpenPull { .. }
                        | api::Write::SetReviewers { .. }
                        | api::Write::Merge { .. } => {}
                    }
                }
                if self.status != self.expected() {
                    let refusal = match &body {
                        Body::Document(d::Document::Error(error)) => match &error.message {
                            d::Body::Kept(message) => temper_forge_forgejo::failure::refusal(self.status, message),
                            d::Body::Long(_) => None,
                        },
                        Body::Document(_) | Body::Empty | Body::Raw { .. } | Body::Malformed(_) => None,
                    };
                    let classified = failure::classify(self.asked(), self.status, refusal, self.retry);
                    if classified.drop != Drop::Nothing {
                        tables.drop_repository(self.repository);
                    }
                    return Ok(Some(Step::Done(Err(classified.error))));
                }
                let document = match body {
                    Body::Document(document) => Some(document),
                    Body::Empty => None,
                    Body::Malformed(_) | Body::Raw { .. } => return Err(api::Error::Timeout),
                };
                if let Some(document) = &document {
                    crate::reads::keep_document(document, self.repository.forge, tables);
                }
                if self.resolving {
                    let Some(d::Document::Repository(repository)) = document else {
                        return Err(api::Error::Timeout);
                    };
                    let Ok(id) = u32::try_from(repository.id) else {
                        return Err(api::Error::TooLarge);
                    };
                    if id != self.repository.repository {
                        return Err(api::Error::Timeout);
                    }
                    let row = reading::repository(repository);
                    tables.keep_repository(self.repository, row.clone());
                    self.row = Some(row);
                    self.resolving = false;
                    return Ok(Some(self.initial(l)?));
                }
                let answer = self.answer(document)?;
                Ok(Some(Step::Done(Ok(answer))))
            }
        }
    }
    fn answer(&self, document: Option<d::Document>) -> Result<api::Answer, api::Error> {
        match &self.write {
            api::Write::CreateIssue { .. } => match document {
                Some(d::Document::Issue(issue)) => Ok(api::Answer::Created(issue.number)),
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::Post { .. } => match document {
                Some(d::Document::Comment(comment)) => Ok(api::Answer::Commented(comment.id)),
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::Review { .. } => match document {
                Some(d::Document::Review(review)) => {
                    if reading::commit(review.commit_id, self.row.as_ref().expect("format").object_format).is_none() {
                        return Err(api::Error::Timeout);
                    }
                    Ok(api::Answer::Reviewed(review.id))
                }
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::Edit { .. } | api::Write::Close { .. } | api::Write::Reopen { .. } => match document {
                Some(d::Document::Issue(_)) => Ok(api::Answer::Done),
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::Status { .. } => match document {
                Some(d::Document::Status(_)) => Ok(api::Answer::Done),
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::CreateBranch { .. } => match document {
                Some(d::Document::Branch(branch)) => {
                    if reading::commit(branch.commit, self.row.as_ref().expect("format").object_format).is_none() {
                        return Err(api::Error::Timeout);
                    }
                    Ok(api::Answer::Branch(api::BranchCreation::Created))
                }
                Some(_) | None => Err(api::Error::Timeout),
            },
            api::Write::Update { .. } | api::Write::DeleteBranch { .. } => match document {
                None => Ok(api::Answer::Done),
                Some(_) => Err(api::Error::Timeout),
            },
            api::Write::OpenPull { .. } | api::Write::SetReviewers { .. } | api::Write::Merge { .. } => {
                unreachable!("multiexchange writes not assigned yet")
            }
        }
    }
}
fn marked(key: &[u8], body: &[u8], l: &Limits) -> Result<Box<[u8]>, api::Error> {
    marker::encode(key, body, l.body_bytes).ok_or(api::Error::Refused)
}
fn text(bytes: &[u8]) -> Result<(), api::Error> {
    let l = skein_json::writer::Limits { length: u32::MAX, depth: 1 };
    let mut json = skein_json::writer::Encoder::measure(&l);
    json.string(bytes);
    match json.measured() {
        Ok(_) => Ok(()),
        Err(_) => Err(api::Error::Refused),
    }
}
fn validate(write: &api::Write) -> Result<(), api::Error> {
    match write {
        api::Write::CreateIssue { title, body, .. } => {
            text(title)?;
            text(body)
        }
        api::Write::OpenPull { title, body, head, base } => {
            text(title)?;
            text(body)?;
            text(head)?;
            text(base)
        }
        api::Write::Post { body, .. } | api::Write::Review { body, .. } => text(body),
        api::Write::Edit { title, body, .. } => {
            if let Some(title) = title {
                text(title)?;
            }
            if let Some(body) = body {
                text(body)?;
            }
            Ok(())
        }
        api::Write::Status { context, .. } => text(context),
        api::Write::CreateBranch { branch, .. } | api::Write::DeleteBranch { branch } => text(branch),
        api::Write::SetReviewers { .. }
        | api::Write::Close { .. }
        | api::Write::Reopen { .. }
        | api::Write::Merge { .. }
        | api::Write::Update { .. } => Ok(()),
    }
}
