//! temper's addresses and the inbox's filters (client/README.md, 3.5), and their paths (client/browser.md, 2.3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};
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

/// The page's path (client/browser.md, 2.3): `/`, `/chats`, `/chats/<task>`, `/tasks/<task>`,
/// `/projects/<project>/adopt`. The inbox prints `/` whatever its filter.
#[must_use]
pub fn print(address: &Address) -> Box<[u8]> {
    let (name, number, tail) = match address {
        Address::Inbox { .. } => return Box::from(b"/".as_slice()),
        Address::Chats => (b"chats".as_slice(), None, b"".as_slice()),
        Address::Chat { task } => (b"chats".as_slice(), Some(*task), b"".as_slice()),
        Address::Task { task } => (b"tasks".as_slice(), Some(*task), b"".as_slice()),
        Address::Adopt { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"adopt".as_slice()),
    };
    let digits = Decimal::of(number.unwrap_or(0));
    let number = if number.is_some() { digits.as_bytes() } else { b"" };
    let parts = [name, number, tail];
    let mut size = 0_usize;
    for part in parts {
        if !part.is_empty() {
            size = size.checked_add(part.len()).expect("path fits usize");
            size = size.checked_add(1).expect("path fits usize");
        }
    }
    let mut path = Writer::new(size);
    for part in parts {
        if !part.is_empty() {
            path.put(b"/").expect("separator fits");
            path.put(part).expect("part fits");
        }
    }
    path.finish()
}

/// The page a path names, ignoring its query and one trailing slash; any other path is `Place::Missing`.
#[must_use]
pub fn parse(path: &[u8]) -> Place {
    let query = skein_lib::bytes::find(path, b"?").unwrap_or(path.len());
    let path = path.get(..query).expect("query in path");
    let path = path.strip_suffix(b"/").unwrap_or(path);
    if path.is_empty() {
        return Place::Address(Address::Inbox { filter: Filter::ALL });
    }
    let Some(path) = path.strip_prefix(b"/") else {
        return Place::Missing;
    };
    if path.ends_with(b"/") {
        return Place::Missing;
    }
    let (name, rest) = segment(path);
    let (number, tail) = segment(rest);
    let address = match (name, number, tail) {
        (b"chats", b"", b"") => Address::Chats,
        (b"chats", _, b"") => {
            let Some(task) = decimal(number) else {
                return Place::Missing;
            };
            Address::Chat { task }
        }
        (b"tasks", _, b"") => {
            let Some(task) = decimal(number) else {
                return Place::Missing;
            };
            Address::Task { task }
        }
        (b"projects", _, b"adopt") => {
            let Some(project) = project_decimal(number) else {
                return Place::Missing;
            };
            Address::Adopt { project }
        }
        _ => return Place::Missing,
    };
    Place::Address(address)
}

fn segment(path: &[u8]) -> (&[u8], &[u8]) {
    let split = skein_lib::bytes::find(path, b"/").unwrap_or(path.len());
    let first = path.get(..split).expect("separator in path");
    let rest = path.get(split..).expect("separator in path").strip_prefix(b"/").unwrap_or(b"");
    (first, rest)
}

fn decimal(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || bytes.len() > 20 || (bytes.len() > 1 && bytes.starts_with(b"0")) {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn project_decimal(bytes: &[u8]) -> Option<u32> {
    let number = decimal(bytes)?;
    u32::try_from(number).ok()
}
