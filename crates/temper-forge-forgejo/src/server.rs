//! The fake forge's side: origin targets and JSON answers.
use crate::{
    document::{
        ApiSettings, Branch, BranchProtection, CollaboratorPermission, CombinedStatus, Comment, CommitStatus, Issue,
        PullRequest, Repository, Review, ReviewComment, User,
    },
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_json::writer;
mod branches;
mod items;
mod reviews;
mod start;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unrouted {
    NotFound,
    Method,
    Malformed,
}
#[derive(Debug)]
pub enum Route {
    Ready(Request),
}
pub fn route(method: Method, target: &[u8], limits: &Limits) -> Result<Route, Unrouted> {
    match start::route(method, target, limits) {
        Err(Unrouted::NotFound) => match reviews::route(method, target, limits) {
            Err(Unrouted::NotFound) => match branches::route(method, target, limits) {
                Err(Unrouted::NotFound) => items::route(method, target, limits),
                other => other,
            },
            other => other,
        },
        other => other,
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    User(User),
    ApiSettings(ApiSettings),
    Repository(Repository),
    Search { ok: bool, data: Box<[User]> },
    Issues(Box<[Issue]>),
    Issue(Issue),
    Comments(Box<[Comment]>),
    Pull(PullRequest),
    Pulls(Box<[PullRequest]>),
    Reviews(Box<[Review]>),
    ReviewComments(Box<[ReviewComment]>),
    Branch(Branch),
    Branches(Box<[Branch]>),
    Protection(BranchProtection),
    Protections(Box<[BranchProtection]>),
    Collaborators(Box<[User]>),
    Permission(CollaboratorPermission),
    CombinedStatus { state: CombinedStatus, sha: Option<crate::id::ObjectId>, statuses: Box<[CommitStatus]> },
}
pub fn encode(answer: &Answer, limits: &Limits) -> Result<Box<[u8]>, writer::Refusal> {
    let mut measure = writer::Encoder::measure(&limits.writer);
    write(&mut measure, answer)?;
    let length = measure.measured()?;
    let mut write = writer::Encoder::write(length, &limits.writer);
    self::write(&mut write, answer)?;
    Ok(write.finish())
}
fn write(json: &mut writer::Encoder, answer: &Answer) -> Result<(), writer::Refusal> {
    match answer {
        Answer::Pulls(_) | Answer::Reviews(_) | Answer::ReviewComments(_) => reviews::write(json, answer),
        Answer::Branch(_)
        | Answer::Branches(_)
        | Answer::Protection(_)
        | Answer::Protections(_)
        | Answer::Collaborators(_)
        | Answer::Permission(_) => {
            branches::write(json, answer);
            Ok(())
        }
        Answer::User(_) | Answer::ApiSettings(_) | Answer::Repository(_) | Answer::Search { .. } => {
            start::write(json, answer);
            Ok(())
        }
        Answer::Issues(_)
        | Answer::Issue(_)
        | Answer::Comments(_)
        | Answer::Pull(_)
        | Answer::CombinedStatus { .. } => items::write(json, answer),
    }
}
