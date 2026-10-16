//! JSON read plans, streamed rows and their running answer bound (forge.md, 3.2).
use crate::{
    connection::{Body, Heard},
    limits::{self, Limits, Listing, Shared},
    plan::{Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
mod done;
pub(crate) mod size;
use alloc::boxed::Box;
use skein_lib::{List, Time, Wall, bytes};
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::{
    document as d,
    id::ObjectId,
    request::{self as r, Request},
};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Stage {
    Resolve,
    User,
    Read,
    Comments,
    Pulls,
    Status,
    Branches,
    Protection,
    Collaborators,
    Permissions,
}
#[derive(Debug)]
enum Rows {
    None,
    Items(List<api::Summary>),
    Comments(List<api::Comment>),
    Reviews(List<api::Review>),
    Statuses(List<api::Status>),
    Remarks(List<api::Remark>),
    Branches(List<Box<[u8]>>),
    Collaborators(List<api::Collaborator>),
}
#[derive(Debug)]
pub(crate) struct Plan {
    repository: api::Repository,
    read: api::Read,
    stage: Stage,
    row: Option<Repository>,
    request: Option<Exchange>,
    contents: Option<crate::contents::Plan>,
    actions: Option<crate::actions::Plan>,
    rows: Rows,
    item: Option<api::Summary>,
    pull: Option<d::PullRequest>,
    size: u64,
    count: u32,
    page: u32,
    limit: u32,
    position: u32,
    more: bool,
    now: Time,
    cut: bool,
    cap: u32,
    bound: Option<u32>,
    after: u64,
}
impl Plan {
    pub fn new(repository: api::Repository, read: api::Read, cap: u32) -> Self {
        Self {
            repository,
            read,
            stage: Stage::Read,
            row: None,
            request: None,
            contents: None,
            actions: None,
            rows: Rows::None,
            item: None,
            pull: None,
            size: 0,
            count: 0,
            page: 1,
            limit: 1,
            position: 0,
            more: false,
            now: Time::ZERO,
            cut: false,
            cap,
            bound: None,
            after: 0,
        }
    }
    pub fn begin(&mut self, l: &Limits, shared: &Shared, tables: &mut Tables) -> Result<Step, api::Error> {
        self.row = tables.repository(self.repository).cloned();
        match &self.read {
            api::Read::Locate { owner, name } => {
                return Ok(self.send(Request::Repository { at: r::Repo { owner: owner.clone(), name: name.clone() } }));
            }
            api::Read::Settings => {
                return Ok(self.send(Request::RepositoryById { id: u64::from(self.repository.repository) }));
            }
            api::Read::Items { .. }
            | api::Read::Item { .. }
            | api::Read::Pull { .. }
            | api::Read::PullFor { .. }
            | api::Read::Reviews { .. }
            | api::Read::Statuses { .. }
            | api::Read::Remarks { .. }
            | api::Read::Branch { .. }
            | api::Read::Branches
            | api::Read::Protection { .. }
            | api::Read::Collaborators { .. }
            | api::Read::Permission { .. }
            | api::Read::PullFiles { .. }
            | api::Read::Compare { .. }
            | api::Read::Checks { .. }
            | api::Read::Job { .. }
            | api::Read::File { .. } => {}
        }
        if self.row.is_none() {
            self.stage = Stage::Resolve;
            return Ok(self.send(Request::RepositoryById { id: u64::from(self.repository.repository) }));
        }
        self.initial(l, shared, tables)
    }
    pub fn pending(&mut self) -> Option<Exchange> {
        self.request.take()
    }
    pub fn set_pending(&mut self, request: Exchange) {
        self.request = Some(request);
    }
    fn send(&mut self, request: Request) -> Step {
        self.bound = page_bound(&request);
        self.count = 0;
        Step::send(request)
    }
    fn at(&self) -> r::Repo {
        let row = self.row.as_ref().expect("resolved before path request");
        r::Repo { owner: row.owner.clone(), name: row.name.clone() }
    }
    fn paged(&mut self, listing: Listing, page: u32, l: &Limits, shared: &Shared) -> r::Page {
        self.page = page;
        self.limit = limits::page_size(listing, self.cap, l, shared);
        r::Page { page: Some(page), limit: Some(self.limit) }
    }
    fn initial(&mut self, l: &Limits, shared: &Shared, tables: &mut Tables) -> Result<Step, api::Error> {
        self.stage = Stage::Read;
        let at = self.at();
        let capacity = shared.client.rows;
        let request = match self.read.clone() {
            api::Read::Items { since, page, kind } => {
                self.rows = Rows::Items(List::with_capacity(limits::page_size(Listing::Items, self.cap, l, shared)));
                let kind = match kind {
                    Some(api::Kind::Issue) => Some(r::IssueKind::Issues),
                    Some(api::Kind::Pull) => Some(r::IssueKind::Pulls),
                    None => None,
                };
                Request::Issues {
                    at,
                    state: Some(r::StateFilter::All),
                    kind,
                    since: Some(Wall::from_nanos(since.as_nanos())),
                    sort: Some(r::Sort::LeastUpdate),
                    page: self.paged(Listing::Items, page, l, shared),
                }
            }
            api::Read::Item { number, after } => {
                self.after = after;
                self.rows = Rows::Comments(List::with_capacity(0));
                Request::Issue { at, number }
            }
            api::Read::Pull { number } => Request::Pull { at, number },
            api::Read::PullFor { head, base } => Request::PullFor { at, head, base },
            api::Read::Reviews { number, page } => {
                self.rows =
                    Rows::Reviews(List::with_capacity(limits::page_size(Listing::Reviews, self.cap, l, shared)));
                Request::Reviews { at, number, page: self.paged(Listing::Reviews, page, l, shared) }
            }
            api::Read::Statuses { commit, page } => {
                self.rows =
                    Rows::Statuses(List::with_capacity(limits::page_size(Listing::Statuses, self.cap, l, shared)));
                Request::CombinedStatus {
                    at,
                    commit: self.id(commit),
                    page: self.paged(Listing::Statuses, page, l, shared),
                }
            }
            api::Read::Remarks { number, review, page } => {
                self.rows =
                    Rows::Remarks(List::with_capacity(limits::page_size(Listing::Remarks, self.cap, l, shared)));
                self.paged(Listing::Remarks, page, l, shared);
                Request::ReviewComments { at, number, review }
            }
            api::Read::Branch { branch: name } | api::Read::Protection { branch: name } => Request::Branch { at, name },
            api::Read::Branches => {
                let room = shared
                    .client
                    .answer_bytes
                    .checked_div(u32::try_from(size_of::<Box<[u8]>>()).expect("box size"))
                    .expect("nonzero width");
                self.rows = Rows::Branches(List::with_capacity(capacity.min(room)));
                self.stage = Stage::Branches;
                Request::Branches { at, page: self.paged(Listing::Branches, 1, l, shared) }
            }
            api::Read::Collaborators { page } => {
                self.rows = Rows::Collaborators(List::with_capacity(limits::page_size(
                    Listing::Collaborators,
                    self.cap,
                    l,
                    shared,
                )));
                self.stage = Stage::Collaborators;
                Request::Collaborators { at, page: self.paged(Listing::Collaborators, page, l, shared) }
            }
            api::Read::Permission { user } => return Ok(self.permission(user, tables)),
            read @ (api::Read::PullFiles { .. } | api::Read::Compare { .. } | api::Read::File { .. }) => {
                let mut plan = crate::contents::Plan::new(
                    read,
                    self.row.as_ref().expect("resolved").clone(),
                    l,
                    shared,
                    self.cap,
                )?;
                let step = plan.begin(shared)?;
                self.contents = Some(plan);
                return Ok(step);
            }
            read @ (api::Read::Checks { .. } | api::Read::Job { .. }) => {
                let mut plan =
                    crate::actions::Plan::new(read, self.row.as_ref().expect("resolved").clone(), l, shared, self.cap)?;
                let step = plan.begin();
                self.actions = Some(plan);
                return Ok(step);
            }
            api::Read::Locate { .. } | api::Read::Settings => {
                return Err(api::Error::Unavailable);
            }
        };
        Ok(self.send(request))
    }
    fn id(&self, bytes: api::Commit) -> ObjectId {
        ObjectId { bytes, format: self.row.as_ref().expect("resolved format").object_format }
    }
    fn permission(&mut self, user: u64, tables: &mut Tables) -> Step {
        let request = match tables.user(self.repository.forge, user) {
            Some(login) => Request::Permission { at: self.at(), login: bytes::copy_of(login) },
            None => {
                self.stage = Stage::User;
                Request::UserSearch { uid: user, limit: Some(1) }
            }
        };
        self.send(request)
    }
    pub fn after(&self) -> u64 {
        self.after
    }
    pub fn raw_room(&self, shared: &Shared) -> u32 {
        match &self.contents {
            Some(plan) => plan.raw_room(shared),
            None => match &self.actions {
                Some(plan) => plan.raw_room(shared),
                None => shared.client.answer_bytes,
            },
        }
    }
    pub fn take_cut(&mut self) -> bool {
        core::mem::replace(&mut self.cut, false)
    }
    pub fn heard(
        &mut self,
        heard: Heard,
        l: &Limits,
        shared: &Shared,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        if let Some(plan) = &mut self.contents {
            return plan.heard(heard, shared, self.repository, tables);
        }
        if let Some(plan) = &mut self.actions {
            return plan.heard(heard, shared, self.repository, tables);
        }
        match heard {
            Heard::Head { status, fields } => {
                if status != 200 {
                    if status == 404 || (300..400).contains(&status) {
                        tables.drop_repository(self.repository);
                    }
                    return Err(crate::failure::read(status, &fields));
                }
                self.now = match fields.date {
                    Some(wall) => reading::time(wall),
                    None => Time::ZERO,
                };
                Ok(None)
            }
            Heard::Row(row) => {
                self.row(row, shared, tables)?;
                Ok(None)
            }
            Heard::Done(Body::Document(document)) => self.done(document, l, shared, tables),
            Heard::Done(Body::Malformed(_) | Body::Empty | Body::Raw { .. }) => Err(api::Error::InvalidAnswer),
            Heard::Failed(failure) => Err(crate::failure::transport(failure)),
        }
    }
    fn add_size(&mut self, size: u64, shared: &Shared) -> bool {
        let Some(total) = self.size.checked_add(size) else {
            return false;
        };
        if total > u64::from(shared.client.answer_bytes) {
            return false;
        }
        self.size = total;
        true
    }
    fn row(&mut self, row: d::Row, shared: &Shared, tables: &mut Tables) -> Result<(), api::Error> {
        self.count = self.count.checked_add(1).ok_or(api::Error::TooLarge)?;
        if let Some(bound) = self.bound
            && self.count > bound
        {
            return Err(api::Error::TooLarge);
        }
        keep_row(&row, self.repository.forge, tables);
        match row {
            d::Row::Pull(pull) => self.lookup_row(pull),
            d::Row::User(user) => {
                if self.stage == Stage::User {
                    return Ok(());
                }
                let size = u64::try_from(size_of::<api::Collaborator>()).expect("row size");
                if !self.add_size(size, shared) {
                    return Err(api::Error::TooLarge);
                }
                match &mut self.rows {
                    Rows::Collaborators(rows) => {
                        match rows.push(api::Collaborator { user: user.id, permission: api::Permission::None }) {
                            Ok(()) => Ok(()),
                            Err(_) => Err(api::Error::TooLarge),
                        }
                    }
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Comments(_)
                    | Rows::Reviews(_)
                    | Rows::Statuses(_)
                    | Rows::Remarks(_)
                    | Rows::Branches(_) => Err(api::Error::InvalidAnswer),
                }
            }
            d::Row::Protection(_) => Ok(()),
            d::Row::Issue(issue) => self.append(
                api::Answer::Items { items: Box::new([reading::summary(issue)]), more: false, now: self.now },
                shared,
            ),
            d::Row::Comment(comment) => {
                let item = self.item.as_ref().ok_or(api::Error::InvalidAnswer)?.clone();
                self.append(
                    api::Answer::Item { item, comments: Box::new([reading::comment(comment)]), more: false },
                    shared,
                )
            }
            d::Row::Review(review) => {
                match reading::review(review, self.row.as_ref().expect("resolved format").object_format)? {
                    Some(review) => {
                        self.append(api::Answer::Reviews { reviews: Box::new([review]), more: false }, shared)
                    }
                    None => Ok(()),
                }
            }
            d::Row::Status(_) if self.stage == Stage::Status => Ok(()),
            d::Row::Status(status) => self.append(
                api::Answer::Statuses {
                    ci: api::Ci::None,
                    statuses: Box::new([reading::status(status, &self.row.as_ref().expect("resolved URL").html_url)?]),
                    more: false,
                },
                shared,
            ),
            d::Row::ReviewComment(remark) => {
                let skip = self.page.saturating_sub(1).checked_mul(self.limit).ok_or(api::Error::TooLarge)?;
                if self.count <= skip {
                    return Ok(());
                }
                if self.count > skip.saturating_add(self.limit) {
                    self.more = true;
                    self.cut = true;
                    return Ok(());
                }
                self.append(api::Answer::Remarks { remarks: Box::new([reading::remark(remark)]), more: false }, shared)
            }
            d::Row::Branch(branch) => self.append(api::Answer::Branches(Box::new([branch.name])), shared),
            d::Row::File(_) | d::Row::Commit(_) | d::Row::Run(_) | d::Row::Step(_) | d::Row::Job(_) => {
                Err(api::Error::InvalidAnswer)
            }
        }
    }
    fn lookup_row(&mut self, pull: d::PullRequest) -> Result<(), api::Error> {
        if self.stage == Stage::Pulls {
            if self.more {
                return Ok(());
            }
            let found = match &self.read {
                api::Read::PullFor { head, base } => {
                    pull.head.name == *head
                        && pull.head.repo_id == u64::from(self.repository.repository)
                        && pull.base.name == *base
                }
                api::Read::Locate { .. }
                | api::Read::Items { .. }
                | api::Read::Item { .. }
                | api::Read::Pull { .. }
                | api::Read::Reviews { .. }
                | api::Read::Statuses { .. }
                | api::Read::Remarks { .. }
                | api::Read::Branch { .. }
                | api::Read::Branches
                | api::Read::Protection { .. }
                | api::Read::Settings
                | api::Read::Collaborators { .. }
                | api::Read::Permission { .. }
                | api::Read::PullFiles { .. }
                | api::Read::Compare { .. }
                | api::Read::Checks { .. }
                | api::Read::Job { .. }
                | api::Read::File { .. } => false,
            };
            if found {
                self.pull = Some(pull);
                self.more = true;
            }
            return Ok(());
        }
        Err(api::Error::InvalidAnswer)
    }
    fn append(&mut self, answer: api::Answer, shared: &Shared) -> Result<(), api::Error> {
        let mut size = size::answer(&answer, shared.client.rows)?;
        match &answer {
            api::Answer::Item { item, .. } => {
                size = size.saturating_sub(size::summary(item, shared.client.rows)?);
            }
            api::Answer::Items { .. }
            | api::Answer::Pull(_)
            | api::Answer::Reviews { .. }
            | api::Answer::Statuses { .. }
            | api::Answer::Remarks { .. }
            | api::Answer::Commit(_)
            | api::Answer::Branches(_)
            | api::Answer::Located { .. }
            | api::Answer::PullFiles { .. }
            | api::Answer::Compare { .. }
            | api::Answer::Checks(_)
            | api::Answer::Job { .. }
            | api::Answer::File { .. }
            | api::Answer::Protection(_)
            | api::Answer::Settings(_)
            | api::Answer::Collaborators { .. }
            | api::Answer::Permission(_)
            | api::Answer::Created(_)
            | api::Answer::Commented(_)
            | api::Answer::Reviewed(_)
            | api::Answer::Merged(_)
            | api::Answer::Branch(_)
            | api::Answer::Done => {}
        }
        let room = match &self.rows {
            Rows::None => false,
            Rows::Items(rows) => rows.len() < rows.capacity(),
            Rows::Comments(rows) => rows.len() < rows.capacity(),
            Rows::Reviews(rows) => rows.len() < rows.capacity(),
            Rows::Statuses(rows) => rows.len() < rows.capacity(),
            Rows::Remarks(rows) => rows.len() < rows.capacity(),
            Rows::Branches(rows) => rows.len() < rows.capacity(),
            Rows::Collaborators(rows) => rows.len() < rows.capacity(),
        };
        if !room || !self.add_size(size, shared) {
            match &self.rows {
                Rows::Comments(_) | Rows::Remarks(_) => {
                    self.more = true;
                    self.cut = true;
                    return Ok(());
                }
                Rows::None
                | Rows::Items(_)
                | Rows::Reviews(_)
                | Rows::Statuses(_)
                | Rows::Branches(_)
                | Rows::Collaborators(_) => return Err(api::Error::TooLarge),
            }
        }
        self.push(answer)
    }
    fn push_items(&mut self, items: Box<[api::Summary]>) -> Result<(), api::Error> {
        let rows = match &mut self.rows {
            Rows::Items(rows) => rows,
            Rows::None
            | Rows::Comments(_)
            | Rows::Reviews(_)
            | Rows::Statuses(_)
            | Rows::Remarks(_)
            | Rows::Branches(_)
            | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
        };
        for row in items {
            rows.push(row).expect("room checked");
        }
        Ok(())
    }
    fn push(&mut self, answer: api::Answer) -> Result<(), api::Error> {
        match answer {
            api::Answer::Items { items, .. } => self.push_items(items)?,
            api::Answer::Item { comments, .. } => {
                let rows = match &mut self.rows {
                    Rows::Comments(rows) => rows,
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Reviews(_)
                    | Rows::Statuses(_)
                    | Rows::Remarks(_)
                    | Rows::Branches(_)
                    | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
                };
                for row in comments {
                    rows.push(row).expect("room checked");
                }
            }
            api::Answer::Reviews { reviews, .. } => {
                let rows = match &mut self.rows {
                    Rows::Reviews(rows) => rows,
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Comments(_)
                    | Rows::Statuses(_)
                    | Rows::Remarks(_)
                    | Rows::Branches(_)
                    | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
                };
                for row in reviews {
                    rows.push(row).expect("room checked");
                }
            }
            api::Answer::Statuses { statuses, .. } => {
                let rows = match &mut self.rows {
                    Rows::Statuses(rows) => rows,
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Comments(_)
                    | Rows::Reviews(_)
                    | Rows::Remarks(_)
                    | Rows::Branches(_)
                    | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
                };
                for row in statuses {
                    rows.push(row).expect("room checked");
                }
            }
            api::Answer::Remarks { remarks, .. } => {
                let rows = match &mut self.rows {
                    Rows::Remarks(rows) => rows,
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Comments(_)
                    | Rows::Reviews(_)
                    | Rows::Statuses(_)
                    | Rows::Branches(_)
                    | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
                };
                for row in remarks {
                    rows.push(row).expect("room checked");
                }
            }
            api::Answer::Branches(branches) => {
                let rows = match &mut self.rows {
                    Rows::Branches(rows) => rows,
                    Rows::None
                    | Rows::Items(_)
                    | Rows::Comments(_)
                    | Rows::Reviews(_)
                    | Rows::Statuses(_)
                    | Rows::Remarks(_)
                    | Rows::Collaborators(_) => return Err(api::Error::InvalidAnswer),
                };
                for row in branches {
                    rows.push(row).expect("room checked");
                }
            }
            api::Answer::Pull(_)
            | api::Answer::Commit(_)
            | api::Answer::Located { .. }
            | api::Answer::PullFiles { .. }
            | api::Answer::Compare { .. }
            | api::Answer::Checks(_)
            | api::Answer::Job { .. }
            | api::Answer::File { .. }
            | api::Answer::Protection(_)
            | api::Answer::Settings(_)
            | api::Answer::Collaborators { .. }
            | api::Answer::Permission(_)
            | api::Answer::Created(_)
            | api::Answer::Commented(_)
            | api::Answer::Reviewed(_)
            | api::Answer::Merged(_)
            | api::Answer::Branch(_)
            | api::Answer::Done => return Err(api::Error::InvalidAnswer),
        }
        Ok(())
    }
}

fn keep_user(user: &d::User, forge: u16, tables: &mut Tables) {
    tables.keep_user(forge, user.id, user.login.clone());
}
fn keep_pull(pull: &d::PullRequest, forge: u16, tables: &mut Tables) {
    for user in &pull.requested_reviewers {
        keep_user(user, forge, tables);
    }
}
fn keep_row(row: &d::Row, forge: u16, tables: &mut Tables) {
    match row {
        d::Row::User(user) => keep_user(user, forge, tables),
        d::Row::Issue(issue) => keep_user(&issue.user, forge, tables),
        d::Row::Comment(comment) => keep_user(&comment.user, forge, tables),
        d::Row::Review(review) => keep_user(&review.user, forge, tables),
        d::Row::ReviewComment(remark) => keep_user(&remark.user, forge, tables),
        d::Row::Status(status) => {
            if let Some(user) = &status.creator {
                keep_user(user, forge, tables);
            }
        }
        d::Row::Pull(pull) => keep_pull(pull, forge, tables),
        d::Row::Branch(_)
        | d::Row::Protection(_)
        | d::Row::File(_)
        | d::Row::Commit(_)
        | d::Row::Run(_)
        | d::Row::Step(_)
        | d::Row::Job(_) => {}
    }
}
pub(crate) fn keep_document(document: &d::Document, forge: u16, tables: &mut Tables) {
    match document {
        d::Document::User(user) => keep_user(user, forge, tables),
        d::Document::Repository(repo) => keep_user(&repo.owner, forge, tables),
        d::Document::Issue(issue) => keep_user(&issue.user, forge, tables),
        d::Document::Comment(comment) => keep_user(&comment.user, forge, tables),
        d::Document::Status(status) => {
            if let Some(user) = &status.creator {
                keep_user(user, forge, tables);
            }
        }
        d::Document::Review(review) => keep_user(&review.user, forge, tables),
        d::Document::Pull(pull) => keep_pull(pull, forge, tables),
        d::Document::Permission(permission) => keep_user(&permission.user, forge, tables),
        d::Document::ApiSettings(_)
        | d::Document::Search { .. }
        | d::Document::Listed { .. }
        | d::Document::CombinedStatus(_)
        | d::Document::Branch(_)
        | d::Document::Protection(_)
        | d::Document::Compare(_)
        | d::Document::Runs { .. }
        | d::Document::Job(_)
        | d::Document::Run(_)
        | d::Document::Error(_) => {}
    }
}
fn page_bound(request: &Request) -> Option<u32> {
    match request {
        Request::Issues { page, .. }
        | Request::Reviews { page, .. }
        | Request::CombinedStatus { page, .. }
        | Request::Branches { page, .. }
        | Request::Collaborators { page, .. }
        | Request::Pulls { page, .. }
        | Request::PullFiles { page, .. }
        | Request::Runs { page, .. } => page.limit,
        Request::UserSearch { limit, .. } => *limit,
        Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Permission { .. }
        | Request::Compare { .. }
        | Request::Raw { .. }
        | Request::Run { .. }
        | Request::Job { .. }
        | Request::Jobs { .. }
        | Request::JobLog { .. }
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
        | Request::UserOf { .. }
        | Request::CreateUser { .. }
        | Request::DeleteBranch { .. } => None,
    }
}
pub(crate) fn long(heard: &Heard) -> bool {
    match heard {
        Heard::Row(row) => match row {
            d::Row::Issue(row) => long_body(&row.body),
            d::Row::Comment(row) => long_body(&row.body),
            d::Row::Review(row) => long_body(&row.body),
            d::Row::ReviewComment(row) => long_body(&row.body),
            d::Row::User(_)
            | d::Row::Status(_)
            | d::Row::Branch(_)
            | d::Row::Protection(_)
            | d::Row::Pull(_)
            | d::Row::File(_)
            | d::Row::Commit(_)
            | d::Row::Run(_)
            | d::Row::Step(_)
            | d::Row::Job(_) => false,
        },
        Heard::Done(Body::Document(document)) => match document {
            d::Document::Issue(row) => long_body(&row.body),
            d::Document::Comment(row) => long_body(&row.body),
            d::Document::Review(row) => long_body(&row.body),
            d::Document::User(_)
            | d::Document::ApiSettings(_)
            | d::Document::Repository(_)
            | d::Document::Search { .. }
            | d::Document::Listed { .. }
            | d::Document::Status(_)
            | d::Document::Pull(_)
            | d::Document::CombinedStatus(_)
            | d::Document::Branch(_)
            | d::Document::Protection(_)
            | d::Document::Permission(_)
            | d::Document::Compare(_)
            | d::Document::Runs { .. }
            | d::Document::Job(_)
            | d::Document::Run(_)
            | d::Document::Error(_) => false,
        },
        Heard::Done(Body::Empty | Body::Malformed(_) | Body::Raw { .. }) | Heard::Head { .. } | Heard::Failed(_) => {
            false
        }
    }
}
fn long_body(body: &d::Body) -> bool {
    match body {
        d::Body::Kept(_) => false,
        d::Body::Long(_) => true,
    }
}
