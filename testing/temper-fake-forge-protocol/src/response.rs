//! Per-route projections and post-write reads; one write precedes its readback.
use crate::{Config, Limits, answers, errors, routes};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, bytes};
use temper_fake_forge_domain::api::{self as a, Answer};
use temper_forge_forgejo::{
    document as d,
    request::{Page, Request},
    server,
};
#[derive(PartialEq, Eq, Debug)]
pub enum Select {
    Document,
    Permission(u64),
    Branch(Box<[u8]>),
    Branches(Page),
    Review(u64),
    Status(Box<[u8]>),
    Raw(Option<(u64, Option<u64>)>),
}
#[derive(PartialEq, Eq, Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Box<[skein_http::Header]>,
    pub body: Box<[u8]>,
}
fn number(answer: &Answer) -> Option<u64> {
    match answer {
        Answer::Created(id) | Answer::Commented(id) | Answer::Reviewed(id) => Some(*id),
        Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Pulls(_)
        | Answer::Comments(_)
        | Answer::Runs(_)
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::PullFiles { .. }
        | Answer::Comparison { .. }
        | Answer::Checks(_)
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators(_)
        | Answer::Statuses(_)
        | Answer::Permission(_)
        | Answer::Comment { .. }
        | Answer::Dependencies(_)
        | Answer::Labels(_)
        | Answer::Commit(_)
        | Answer::Tree(_)
        | Answer::File(_)
        | Answer::Pages { .. }
        | Answer::Page(_)
        | Answer::Merged(_)
        | Answer::Revision(_)
        | Answer::Done
        | Answer::Cloned { .. }
        | Answer::Pushed(_)
        | Answer::Branch(_) => None,
    }
}
pub fn follow(request: &Request, answer: &Answer) -> Result<Option<(a::Read, Select)>, a::Error> {
    let next = match request {
        Request::CreateIssue { .. } => {
            (a::Read::Item { number: number(answer).ok_or(a::Error::Refused)?, after: 0 }, Select::Document)
        }
        Request::OpenPull { .. } => {
            (a::Read::Pull { number: number(answer).ok_or(a::Error::Refused)? }, Select::Document)
        }
        Request::PostComment { .. } => {
            (a::Read::Comment { id: number(answer).ok_or(a::Error::Refused)? }, Select::Document)
        }
        Request::PostReview { number: pull, .. } => {
            (a::Read::Pull { number: *pull }, Select::Review(number(answer).ok_or(a::Error::Refused)?))
        }
        Request::EditIssue { number, .. } => (a::Read::Item { number: *number, after: 0 }, Select::Document),
        Request::PostStatus { commit, context, .. } => (
            a::Read::Checks { commit: routes::commit(commit).ok_or(a::Error::Missing(a::What::Commit))? },
            Select::Status(context.clone()),
        ),
        Request::CreateBranch { branch, .. } => {
            (a::Read::Branch { branch: branch.clone() }, Select::Branch(branch.clone()))
        }
        Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::Pulls { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Raw { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Jobs { .. }
        | Request::JobLog { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::DeleteBranch { .. } => return Ok(None),
    };
    Ok(Some(next))
}
pub fn select(config: &Config, request: &Request, range: Option<(u64, Option<u64>)>) -> Result<Select, a::Error> {
    match request {
        Request::Branch { name, .. } => Ok(Select::Branch(name.clone())),
        Request::Branches { page, .. } => Ok(Select::Branches(*page)),
        Request::Permission { login, .. } => {
            for (user, name) in &config.users {
                if name == login {
                    return Ok(Select::Permission(*user));
                }
            }
            let id = login.strip_prefix(b"u").ok_or(a::Error::Missing(a::What::Repository))?;
            Ok(Select::Permission(
                temper_forge_forgejo::id::unsigned(id).ok_or(a::Error::Missing(a::What::Repository))?,
            ))
        }
        Request::Raw { .. } | Request::JobLog { .. } => Ok(Select::Raw(range)),
        Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Pulls { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Jobs { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::PostComment { .. }
        | Request::PostReview { .. }
        | Request::EditIssue { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::PostStatus { .. }
        | Request::CreateBranch { .. }
        | Request::DeleteBranch { .. } => Ok(Select::Document),
    }
}
fn status(request: &Request) -> u16 {
    match crate::route(request) {
        crate::Route::CreateIssue
        | crate::Route::OpenPull
        | crate::Route::PostComment
        | crate::Route::PostStatus
        | crate::Route::CreateBranch => 201,
        crate::Route::User
        | crate::Route::ApiSettings
        | crate::Route::RepositoryById
        | crate::Route::Repository
        | crate::Route::UserSearch
        | crate::Route::Issues
        | crate::Route::Issue
        | crate::Route::Comments
        | crate::Route::Pull
        | crate::Route::PullFor
        | crate::Route::Pulls
        | crate::Route::CombinedStatus
        | crate::Route::Reviews
        | crate::Route::ReviewComments
        | crate::Route::Branch
        | crate::Route::Branches
        | crate::Route::Protection
        | crate::Route::Protections
        | crate::Route::Collaborators
        | crate::Route::Permission
        | crate::Route::PullFiles
        | crate::Route::Compare
        | crate::Route::Raw
        | crate::Route::Runs
        | crate::Route::Run
        | crate::Route::Jobs
        | crate::Route::JobLog
        | crate::Route::PostReview
        | crate::Route::EditIssue
        | crate::Route::RequestReviewers
        | crate::Route::RemoveReviewers
        | crate::Route::Merge
        | crate::Route::Update
        | crate::Route::DeleteBranch => 200,
    }
}
#[must_use]
pub fn failure(limits: &Limits, failure: errors::Failure) -> Reply {
    let body = if failure.message.is_empty() {
        Box::from([])
    } else {
        server::encode(
            &server::Answer::Error(d::ErrorBody { message: d::Body::Kept(bytes::copy_of(failure.message)) }),
            &limits.documents,
        )
        .unwrap_or(Box::from([]))
    };
    let mut headers = List::with_capacity(1);
    if let Some(seconds) = failure.retry_after {
        let digits = Decimal::of(seconds);
        headers
            .push(skein_http::Header { name: bytes::copy_of(b"retry-after"), value: bytes::copy_of(digits.as_bytes()) })
            .expect("one header");
    }
    Reply { status: failure.status, body, headers: headers.into_boxed() }
}
fn document(config: &Config, request: &Request, select: &Select, answer: &Answer) -> Result<server::Answer, a::Error> {
    match answer {
        Answer::Comment { comment, .. } => Ok(server::Answer::Comment(answers::comment(config, comment))),
        Answer::Permission(permission) => {
            let user = match select {
                Select::Permission(user) => *user,
                Select::Document
                | Select::Branch(_)
                | Select::Branches(_)
                | Select::Review(_)
                | Select::Status(_)
                | Select::Raw(_) => return Err(a::Error::Refused),
            };
            Ok(server::Answer::Permission(d::CollaboratorPermission {
                permission: answers::permission(*permission),
                user: routes::user(config, user),
            }))
        }
        Answer::Commit(commit) => {
            let name = match select {
                Select::Branch(name) => name,
                Select::Document
                | Select::Permission(_)
                | Select::Branches(_)
                | Select::Review(_)
                | Select::Status(_)
                | Select::Raw(_) => return Err(a::Error::Refused),
            };
            Ok(server::Answer::Branch(answers::branch(config, request, name, *commit)))
        }
        Answer::Cloned { branches: values, .. } => branches(config, request, select, values),
        Answer::Pull(pull) => pull_document(config, request, select, pull, answer),
        Answer::Checks(checks) => match select {
            Select::Status(context) => {
                for check in checks {
                    if check.status.context == *context {
                        return Ok(server::Answer::Status(d::CommitStatus {
                            creator: if check.job.is_some() {
                                None
                            } else {
                                Some(routes::user(config, check.status.author))
                            },
                            updated_at: skein_lib::Wall::from_nanos(check.status.at.as_nanos()),
                            context: check.status.context.clone(),
                            status: answers::status(check.status.state),
                            description: check.description.clone(),
                            target_url: check.link.clone(),
                        }));
                    }
                }
                Err(a::Error::Missing(a::What::Commit))
            }
            Select::Document => answers::read(config, request, answer),
            Select::Permission(_) | Select::Branch(_) | Select::Branches(_) | Select::Review(_) | Select::Raw(_) => {
                Err(a::Error::Refused)
            }
        },
        Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pulls(_)
        | Answer::Comments(_)
        | Answer::Runs(_)
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::PullFiles { .. }
        | Answer::Comparison { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators(_)
        | Answer::Statuses(_)
        | Answer::Dependencies(_)
        | Answer::Labels(_)
        | Answer::Tree(_)
        | Answer::File(_)
        | Answer::Pages { .. }
        | Answer::Page(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Revision(_)
        | Answer::Done
        | Answer::Pushed(_)
        | Answer::Branch(_) => answers::read(config, request, answer),
    }
}
pub fn answer(
    config: &Config,
    limits: &Limits,
    request: &Request,
    select: &Select,
    answer: &Answer,
) -> Result<Reply, a::Error> {
    match answer {
        Answer::Done | Answer::Merged(_) => {
            return Ok(Reply { status: status(request), headers: Box::from([]), body: Box::from([]) });
        }
        Answer::File(body) => {
            let range = match select {
                Select::Raw(range) => *range,
                Select::Document
                | Select::Permission(_)
                | Select::Branch(_)
                | Select::Branches(_)
                | Select::Review(_)
                | Select::Status(_) => return Err(a::Error::Refused),
            };
            return raw(limits, body, range);
        }
        Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Pulls(_)
        | Answer::Comments(_)
        | Answer::Runs(_)
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::PullFiles { .. }
        | Answer::Comparison { .. }
        | Answer::Checks(_)
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators(_)
        | Answer::Statuses(_)
        | Answer::Permission(_)
        | Answer::Comment { .. }
        | Answer::Dependencies(_)
        | Answer::Labels(_)
        | Answer::Commit(_)
        | Answer::Tree(_)
        | Answer::Pages { .. }
        | Answer::Page(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Revision(_)
        | Answer::Cloned { .. }
        | Answer::Pushed(_)
        | Answer::Branch(_) => {}
    }
    let document = document(config, request, select, answer)?;
    let Ok(body) = server::encode(&document, &limits.documents) else {
        return Err(a::Error::TooLarge);
    };
    let mut headers = List::with_capacity(1);
    match answer {
        Answer::PullFiles { more, .. } => headers
            .push(skein_http::Header {
                name: bytes::copy_of(b"x-hasmore"),
                value: bytes::copy_of(if *more { b"true" } else { b"false" }),
            })
            .expect("one header"),
        Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Pulls(_)
        | Answer::Comments(_)
        | Answer::Runs(_)
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::Comparison { .. }
        | Answer::Checks(_)
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators(_)
        | Answer::Statuses(_)
        | Answer::Permission(_)
        | Answer::Comment { .. }
        | Answer::Dependencies(_)
        | Answer::Labels(_)
        | Answer::Commit(_)
        | Answer::Tree(_)
        | Answer::File(_)
        | Answer::Pages { .. }
        | Answer::Page(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Revision(_)
        | Answer::Done
        | Answer::Cloned { .. }
        | Answer::Pushed(_)
        | Answer::Branch(_) => {}
    }
    Ok(Reply { status: status(request), headers: headers.into_boxed(), body })
}
pub fn range(value: Option<&[u8]>) -> Result<Option<(u64, Option<u64>)>, a::Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.strip_prefix(b"bytes=").ok_or(a::Error::Refused)?;
    let dash = bytes::find(value, b"-").ok_or(a::Error::Refused)?;
    let first =
        temper_forge_forgejo::id::unsigned(value.get(..dash).ok_or(a::Error::Refused)?).ok_or(a::Error::Refused)?;
    let last = value.get(dash.saturating_add(1)..).ok_or(a::Error::Refused)?;
    let last =
        if last.is_empty() { None } else { Some(temper_forge_forgejo::id::unsigned(last).ok_or(a::Error::Refused)?) };
    if let Some(last) = last
        && last < first
    {
        return Err(a::Error::Refused);
    }
    Ok(Some((first, last)))
}
fn raw(limits: &Limits, body: &[u8], range: Option<(u64, Option<u64>)>) -> Result<Reply, a::Error> {
    let mut headers = List::with_capacity(1);
    let mut status = 200;
    let bytes = if let Some((first, last)) = range {
        let total = u64::try_from(body.len()).expect("bounded body");
        if first >= total {
            return Ok(Reply { status: 416, body: Box::from([]), headers: Box::from([]) });
        }
        let last = last.unwrap_or(total.saturating_sub(1)).min(total.saturating_sub(1));
        let first_text = Decimal::of(first);
        let last_text = Decimal::of(last);
        let total_text = Decimal::of(total);
        let mut value = Writer::new(
            6usize
                .saturating_add(first_text.as_bytes().len())
                .saturating_add(last_text.as_bytes().len())
                .saturating_add(total_text.as_bytes().len())
                .saturating_add(2),
        );
        for part in
            [b"bytes ".as_slice(), first_text.as_bytes(), b"-", last_text.as_bytes(), b"/", total_text.as_bytes()]
        {
            value.put(part).expect("sized content range");
        }
        headers
            .push(skein_http::Header { name: bytes::copy_of(b"content-range"), value: value.finish() })
            .expect("one header");
        status = 206;
        let Ok(first) = usize::try_from(first) else {
            return Err(a::Error::TooLarge);
        };
        let Ok(end) = usize::try_from(last.saturating_add(1)) else {
            return Err(a::Error::TooLarge);
        };
        body.get(first..end).ok_or(a::Error::Refused)?
    } else {
        body
    };
    if bytes.len() > usize::try_from(limits.raw_bytes).expect("u32 fits") {
        return Err(a::Error::TooLarge);
    }
    Ok(Reply { status, headers: headers.into_boxed(), body: bytes::copy_of(bytes) })
}

fn pull_document(
    config: &Config,
    request: &Request,
    select: &Select,
    pull: &a::Pull,
    answer: &Answer,
) -> Result<server::Answer, a::Error> {
    match select {
        Select::Review(id) => {
            for value in &pull.reviews {
                if value.id == *id {
                    return Ok(server::Answer::Review(answers::review(config, value)));
                }
            }
            Err(a::Error::Missing(a::What::Review))
        }
        Select::Document => {
            let path = routes::repository(config, request).ok_or(a::Error::Missing(a::What::Repository))?;
            if crate::route(request) == crate::Route::OpenPull {
                for repo in &config.repositories {
                    if repo.full == path {
                        return Ok(server::Answer::Pull(answers::pull(config, u64::from(repo.id), pull)));
                    }
                }
                Err(a::Error::Missing(a::What::Repository))
            } else {
                answers::read(config, request, answer)
            }
        }
        Select::Permission(_) | Select::Branch(_) | Select::Branches(_) | Select::Status(_) | Select::Raw(_) => {
            Err(a::Error::Refused)
        }
    }
}

fn branches(
    config: &Config,
    request: &Request,
    select: &Select,
    branches: &[a::Head],
) -> Result<server::Answer, a::Error> {
    let page = match select {
        Select::Branches(page) => *page,
        Select::Document
        | Select::Permission(_)
        | Select::Branch(_)
        | Select::Review(_)
        | Select::Status(_)
        | Select::Raw(_) => return Err(a::Error::Refused),
    };
    let (page, limit) = routes::page(config, page);
    let skip = u64::from(page.saturating_sub(1)).saturating_mul(u64::from(limit));
    let mut out = List::with_capacity(limit);
    for branch in branches
        .iter()
        .skip(usize::try_from(skip).unwrap_or(usize::MAX))
        .take(usize::try_from(limit).expect("u32 fits"))
    {
        out.push(answers::branch(config, request, &branch.branch, branch.commit)).expect("page cap");
    }
    Ok(server::Answer::Branches(out.into_boxed()))
}
