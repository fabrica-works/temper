//! A request value shared by the client and server.
use alloc::boxed::Box;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repo {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Page {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StateFilter {
    Open,
    Closed,
    All,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IssueKind {
    Issues,
    Pulls,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sort {
    LeastUpdate,
    Other,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    User,
    ApiSettings,
    RepositoryById {
        id: u64,
    },
    Repository {
        at: Repo,
    },
    UserSearch {
        uid: u64,
        limit: Option<u32>,
    },
    Issues {
        at: Repo,
        state: Option<StateFilter>,
        kind: Option<IssueKind>,
        since: Option<skein_lib::Wall>,
        sort: Option<Sort>,
        page: Page,
    },
    Issue {
        at: Repo,
        number: u64,
    },
    Comments {
        at: Repo,
        number: u64,
    },
    Pull {
        at: Repo,
        number: u64,
    },
    PullFor {
        at: Repo,
        base: Box<[u8]>,
        head: Box<[u8]>,
    },
    CombinedStatus {
        at: Repo,
        commit: crate::id::ObjectId,
        page: Page,
    },
    Reviews {
        at: Repo,
        number: u64,
        page: Page,
    },
    ReviewComments {
        at: Repo,
        number: u64,
        review: u64,
    },
    Branch {
        at: Repo,
        name: Box<[u8]>,
    },
    Branches {
        at: Repo,
        page: Page,
    },
    Protection {
        at: Repo,
        name: Box<[u8]>,
    },
    Protections {
        at: Repo,
    },
    Collaborators {
        at: Repo,
        page: Page,
    },
    Permission {
        at: Repo,
        login: Box<[u8]>,
    },
    Pulls {
        at: Repo,
        page: Page,
    },
    PullFiles {
        at: Repo,
        number: u64,
        page: Page,
    },
    Compare {
        at: Repo,
        before: crate::id::ObjectId,
        after: crate::id::ObjectId,
    },
    Raw {
        at: Repo,
        path: Box<[u8]>,
        reference: crate::id::ObjectId,
    },
    Runs {
        at: Repo,
        head_sha: crate::id::ObjectId,
        run_number: u64,
        page: Page,
    },
    Run {
        at: Repo,
        run: u64,
    },
    Jobs {
        at: Repo,
        run: u64,
    },
    JobLog {
        at: Repo,
        job: u64,
        attempt: u32,
    },
}
