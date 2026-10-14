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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    User,
    ApiSettings,
    RepositoryById { id: u64 },
    Repository { at: Repo },
    UserSearch { uid: u64, limit: Option<u32> },
}
