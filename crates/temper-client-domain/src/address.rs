//! temper's addresses and the inbox's filters (client/README.md, 3.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Inbox { filter: Filter },
    Chats,
    Chat { task: u64 },
    Task { task: u64 },
    Adopt { project: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Filter {
    pub project: Option<u32>,
    pub kind: Option<Kind>,
    pub goal: Option<u64>,
}
impl Filter {
    pub const ALL: Self = Self { project: None, kind: None, goal: None };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Proposal,
    Question,
    Held,
    PersonTask,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    Missing,
}
