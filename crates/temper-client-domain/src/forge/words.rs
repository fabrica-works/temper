//! Adoption phrases as checked English data (client/README.md, 4.4 and 4.7).
use super::{Broken, Kinds, Role};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer, bytes::copy_of};
use skein_ui::phrases::{Form, Hole, HoleKind, HoleShape, Phrase, PhraseId, PhraseShape, PluralClass, Segment};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Title,
    TitleNumber,
    Forge,
    Owner,
    Name,
    Role,
    OwnedChoice,
    ForkChoice,
    ContextChoice,
    Home,
    Prefix,
    Landing,
    Ci,
    Checks,
    Adopt,
    Required,
    ForgeNumber,
    BadChecks,
    ContextHome,
    NoCheck,
    Long,
    OutOfDate,
    Adopted,
    Owned,
    Fork,
    Context,
    AdoptedRole,
    IsHome,
    May,
    Protection,
    NoSuchForge,
    NotFound,
    NoAccess,
    Refused,
    InFlight,
    TooLarge,
    Unreachable,
    InvalidAnswer,
    Unknown,
    Read,
    Push,
    Open,
    Land,
    Review,
    Status,
    Comment,
    Issue,
    Branch,
}
use HoleKind::{Name, Number};
pub type Row = (Word, &'static [&'static [u8]], &'static [HoleKind]);
pub const ROWS: [Row; 48] = [
    (Word::Title, &[b"Adopt a repository into ", b""], &[Name]),
    (Word::TitleNumber, &[b"Adopt a repository into project ", b""], &[Number]),
    (Word::Forge, &[b"Forge number"], &[]),
    (Word::Owner, &[b"Owner"], &[]),
    (Word::Name, &[b"Name"], &[]),
    (Word::Role, &[b"Role"], &[]),
    (Word::OwnedChoice, &[b"Owned: temper pushes, opens pull requests and lands"], &[]),
    (Word::ForkChoice, &[b"Fork: temper contributes through a fork"], &[]),
    (Word::ContextChoice, &[b"Context: temper only reads"], &[]),
    (Word::Home, &[b"Make it the project's home"], &[]),
    (Word::Prefix, &[b"Branch prefix"], &[]),
    (Word::Landing, &[b"Landing branch"], &[]),
    (Word::Ci, &[b"It runs CI"], &[]),
    (Word::Checks, &[b"Checks: gate numbers, separated by spaces or commas"], &[]),
    (Word::Adopt, &[b"Adopt"], &[]),
    (Word::Required, &[b"Required."], &[]),
    (Word::ForgeNumber, &[b"A forge number is from 1 to 65535."], &[]),
    (Word::BadChecks, &[b"Checks are numbers from 1, each once."], &[]),
    (Word::ContextHome, &[b"A context repository cannot be the project's home."], &[]),
    (Word::NoCheck, &[b"Without CI, name at least one check."], &[]),
    (Word::Long, &[b"At most ", b" bytes."], &[Number]),
    (Word::OutOfDate, &[b"This temper is out of date for adopting repositories: run the temper the engine runs."], &[]),
    (Word::Adopted, &[b"Adopted ", b" on forge ", b" as ", b"."], &[Name, Number, Name]),
    (Word::Owned, &[b"owned"], &[]),
    (Word::Fork, &[b"fork"], &[]),
    (Word::Context, &[b"context"], &[]),
    (Word::AdoptedRole, &[b"adopted"], &[]),
    (Word::IsHome, &[b"It is the project's home."], &[]),
    (Word::May, &[b"temper may: ", b"."], &[Name]),
    (Word::Protection, &[b"Branch protection could not be read."], &[]),
    (Word::NoSuchForge, &[b"No forge ", b" is configured."], &[Name]),
    (Word::NotFound, &[b"No repository ", b" on forge ", b" that temper's forge user can see."], &[Name, Name]),
    (Word::NoAccess, &[b"temper's forge user may not read ", b" on forge ", b"."], &[Name, Name]),
    (
        Word::Refused,
        &[
            b"The forge connector refused it: another project writes ",
            b", the project has a home already, or branches under ",
            b" are not this deployment's.",
        ],
        &[Name, Name],
    ),
    (Word::InFlight, &[b"", b" is being adopted already; try again when that ends."], &[Name]),
    (Word::TooLarge, &[b"A field is past the forge's bounds, or the forge's repositories are full."], &[]),
    (Word::Unreachable, &[b"Forge ", b" did not answer; nothing was adopted. Try again."], &[Name]),
    (Word::InvalidAnswer, &[b"Forge ", b" answered what temper did not expect; nothing was adopted."], &[Name]),
    (Word::Unknown, &[b"It is not known whether ", b" was adopted."], &[Name]),
    (Word::Read, &[b"read"], &[]),
    (Word::Push, &[b"push"], &[]),
    (Word::Open, &[b"open pull requests"], &[]),
    (Word::Land, &[b"land"], &[]),
    (Word::Review, &[b"review"], &[]),
    (Word::Status, &[b"set statuses"], &[]),
    (Word::Comment, &[b"comment"], &[]),
    (Word::Issue, &[b"create issues"], &[]),
    (Word::Branch, &[b"create and delete branches"], &[]),
];
fn phrase_id(connector: u16, index: usize) -> PhraseId {
    PhraseId {
        owner: owner(connector),
        number: u16::try_from(index).expect("48 phrases").checked_add(1000).expect("phrase id fits"),
    }
}
pub fn extend(connector: u16, text: u32, shapes: &mut List<PhraseShape>, phrases: &mut List<Phrase>) {
    for (index, (word, literals, kinds)) in ROWS.iter().enumerate() {
        let mut holes = List::with_capacity(3);
        let mut segments = List::with_capacity(7);
        for (index, literal) in literals.iter().enumerate() {
            segments.push(Segment::Text(copy_of(literal))).expect("phrase segments fit");
            if let Some(kind) = kinds.get(index) {
                holes.push(HoleShape { kind: *kind, bytes: name_bytes(*word, index, text) }).expect("holes fit");
                segments
                    .push(Segment::Hole { index: u16::try_from(index).expect("three holes"), kind: *kind })
                    .expect("segments fit");
            }
        }
        let id = phrase_id(connector, index);
        shapes.push(PhraseShape { id, holes: holes.into_boxed(), plural: None }).expect("shapes fit");
        phrases
            .push(Phrase { id, forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]) })
            .expect("phrases fit");
    }
}
#[must_use]
pub fn name_bytes(word: Word, index: usize, text: u32) -> u32 {
    match word {
        Word::Title => text.checked_add(16).expect("checked text bound"),
        Word::Adopted => {
            if index == 0 {
                513
            } else if index == 2 {
                7
            } else {
                0
            }
        }
        Word::May => 128,
        Word::NotFound | Word::NoAccess | Word::Refused => {
            if index == 0 {
                513
            } else {
                256
            }
        }
        Word::InFlight | Word::Unknown => 513,
        Word::NoSuchForge | Word::Unreachable | Word::InvalidAnswer => 256,
        Word::TitleNumber
        | Word::Forge
        | Word::Owner
        | Word::Name
        | Word::Role
        | Word::OwnedChoice
        | Word::ForkChoice
        | Word::ContextChoice
        | Word::Home
        | Word::Prefix
        | Word::Landing
        | Word::Ci
        | Word::Checks
        | Word::Adopt
        | Word::Required
        | Word::ForgeNumber
        | Word::BadChecks
        | Word::ContextHome
        | Word::NoCheck
        | Word::Long
        | Word::OutOfDate
        | Word::Owned
        | Word::Fork
        | Word::Context
        | Word::AdoptedRole
        | Word::IsHome
        | Word::Protection
        | Word::TooLarge
        | Word::Read
        | Word::Push
        | Word::Open
        | Word::Land
        | Word::Review
        | Word::Status
        | Word::Comment
        | Word::Issue
        | Word::Branch => 0,
    }
}

#[must_use]
pub fn id(connector: u16, word: Word) -> PhraseId {
    for (index, (candidate, _, _)) in ROWS.iter().enumerate() {
        if *candidate == word {
            return phrase_id(connector, index);
        }
    }
    unreachable!("closed phrase vocabulary")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceWord {
    Connector,
    Repository,
    Branch,
    Pull,
    Issue,
    Adopted,
    Path,
    Read,
    Push,
    OpenPull,
    MergePull,
    Review,
    Status,
    Comment,
    CreateIssue,
    CreateBranch,
    EditPull,
    UpdatePull,
    ClosePull,
    EditIssue,
    Reviewers,
    DeleteBranch,
    OtherKind,
    Shown,
}
pub const RESOURCES: [ResourceWord; 24] = [
    ResourceWord::Connector,
    ResourceWord::Repository,
    ResourceWord::Branch,
    ResourceWord::Pull,
    ResourceWord::Issue,
    ResourceWord::Adopted,
    ResourceWord::Path,
    ResourceWord::Read,
    ResourceWord::Push,
    ResourceWord::OpenPull,
    ResourceWord::MergePull,
    ResourceWord::Review,
    ResourceWord::Status,
    ResourceWord::Comment,
    ResourceWord::CreateIssue,
    ResourceWord::CreateBranch,
    ResourceWord::EditPull,
    ResourceWord::UpdatePull,
    ResourceWord::ClosePull,
    ResourceWord::EditIssue,
    ResourceWord::Reviewers,
    ResourceWord::DeleteBranch,
    ResourceWord::OtherKind,
    ResourceWord::Shown,
];
impl ResourceWord {
    #[must_use]
    pub fn id(self, connector: u16) -> PhraseId {
        let number = match self {
            Self::Connector => 0,
            Self::Repository => 1,
            Self::Branch => 2,
            Self::Pull => 3,
            Self::Issue => 4,
            Self::Adopted => 5,
            Self::Path => 6,
            Self::Read => 7,
            Self::Push => 8,
            Self::OpenPull => 9,
            Self::MergePull => 10,
            Self::Review => 11,
            Self::Status => 12,
            Self::Comment => 13,
            Self::CreateIssue => 14,
            Self::CreateBranch => 15,
            Self::EditPull => 16,
            Self::UpdatePull => 17,
            Self::ClosePull => 18,
            Self::EditIssue => 19,
            Self::Reviewers => 20,
            Self::DeleteBranch => 21,
            Self::OtherKind => 22,
            Self::Shown => 23,
        };
        PhraseId { owner: owner(connector), number }
    }
}
#[must_use]
pub fn resource_english(word: ResourceWord) -> (&'static [&'static [u8]], &'static [HoleKind]) {
    match word {
        ResourceWord::Connector => (&[b"forge"], &[]),
        ResourceWord::Repository => (&[b"", b" on ", b""], &[Name, Name]),
        ResourceWord::Branch => (&[b"branch ", b" of ", b" on ", b""], &[Name, Name, Name]),
        ResourceWord::Pull => (&[b"pull request #", b" of ", b" on ", b""], &[Name, Name, Name]),
        ResourceWord::Issue => (&[b"issue #", b" of ", b" on ", b""], &[Name, Name, Name]),
        ResourceWord::Adopted => (&[b"repository ", b" on forge ", b""], &[Number, Number]),
        ResourceWord::Path => (&[b"", b""], &[Name]),
        ResourceWord::Read => (&[b"read"], &[]),
        ResourceWord::Push => (&[b"push"], &[]),
        ResourceWord::OpenPull => (&[b"open a pull request"], &[]),
        ResourceWord::MergePull => (&[b"merge a pull request"], &[]),
        ResourceWord::Review => (&[b"post a review"], &[]),
        ResourceWord::Status => (&[b"set a commit status"], &[]),
        ResourceWord::Comment => (&[b"post a comment"], &[]),
        ResourceWord::CreateIssue => (&[b"create an issue"], &[]),
        ResourceWord::CreateBranch => (&[b"create a branch"], &[]),
        ResourceWord::EditPull => (&[b"edit a pull request"], &[]),
        ResourceWord::UpdatePull => (&[b"update a pull request from its base"], &[]),
        ResourceWord::ClosePull => (&[b"close or reopen a pull request"], &[]),
        ResourceWord::EditIssue => (&[b"edit, close or reopen an issue"], &[]),
        ResourceWord::Reviewers => (&[b"set reviewers"], &[]),
        ResourceWord::DeleteBranch => (&[b"delete a branch"], &[]),
        ResourceWord::OtherKind => (&[b"kind ", b""], &[Number]),
        ResourceWord::Shown => (&[b"Shown on the forge."], &[]),
    }
}
#[must_use]
pub const fn kind(kind: u16) -> Option<ResourceWord> {
    match kind {
        1 => Some(ResourceWord::Read),
        2 => Some(ResourceWord::Push),
        3 => Some(ResourceWord::OpenPull),
        4 => Some(ResourceWord::MergePull),
        5 => Some(ResourceWord::Review),
        6 => Some(ResourceWord::Status),
        7 => Some(ResourceWord::Comment),
        8 => Some(ResourceWord::CreateIssue),
        9 => Some(ResourceWord::CreateBranch),
        10 => Some(ResourceWord::EditPull),
        11 => Some(ResourceWord::UpdatePull),
        12 => Some(ResourceWord::ClosePull),
        13 => Some(ResourceWord::EditIssue),
        14 => Some(ResourceWord::Reviewers),
        15 => Some(ResourceWord::DeleteBranch),
        _ => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource<'a> {
    Repository { host: &'a [u8], owner: &'a [u8], name: &'a [u8] },
    Branch { host: &'a [u8], owner: &'a [u8], name: &'a [u8], branch: &'a [Box<[u8]>] },
    Pull { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    Issue { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    Adopted { forge: u16, repository: u32 },
    Other,
}
#[must_use]
pub fn resource(path: &[Box<[u8]>]) -> Resource<'_> {
    match path {
        [forge, repository] if forge.len() == 2 && repository.len() == 4 => Resource::Adopted {
            forge: Reader::new(forge).u16().expect("two bytes"),
            repository: Reader::new(repository).u32().expect("four bytes"),
        },
        [prefix, host, owner, name] if prefix.as_ref() == b"forge" => Resource::Repository { host, owner, name },
        [prefix, host, owner, name, tag, branch @ ..]
            if prefix.as_ref() == b"forge" && tag.as_ref() == b"branch" && !branch.is_empty() =>
        {
            Resource::Branch { host, owner, name, branch }
        }
        [prefix, host, owner, name, tag, number] if prefix.as_ref() == b"forge" && tag.as_ref() == b"pull" => {
            Resource::Pull { host, owner, name, number }
        }
        [prefix, host, owner, name, tag, number] if prefix.as_ref() == b"forge" && tag.as_ref() == b"issue" => {
            Resource::Issue { host, owner, name, number }
        }
        _ => Resource::Other,
    }
}
#[must_use]
pub fn joined(segments: &[Box<[u8]>], most: u32) -> Box<[u8]> {
    let mut bytes = segments.len().saturating_sub(1);
    for segment in segments {
        bytes = bytes.checked_add(segment.len()).expect("bounded resource path fits");
    }
    let maximum = usize::try_from(most).expect("path text bound fits");
    let mut first = 0;
    let cut = bytes > maximum;
    if cut {
        let available = maximum.saturating_sub(4);
        for (index, segment) in segments.iter().enumerate() {
            if bytes <= available {
                break;
            }
            bytes = bytes.saturating_sub(segment.len()).saturating_sub(1);
            first = index.checked_add(1).expect("path index fits");
        }
    }
    let tail = segments.get(first..).expect("tail lies in path");
    let mut writer = Writer::new(bytes.checked_add(if cut { 4 } else { 0 }).expect("path mark fits"));
    if cut {
        writer.put(b"\xe2\x80\xa6/").expect("path mark counted");
    }
    for (index, segment) in tail.iter().enumerate() {
        if index != 0 {
            writer.put(b"/").expect("separator counted");
        }
        writer.put(segment).expect("segment counted");
    }
    writer.finish()
}
#[must_use]
pub fn full_name(owner: &[u8], name: &[u8]) -> Box<[u8]> {
    let size =
        owner.len().checked_add(name.len()).expect("bounded full name fits").checked_add(1).expect("separator fits");
    let mut writer = Writer::new(size);
    writer.put(owner).expect("owner counted");
    writer.put(b"/").expect("separator counted");
    writer.put(name).expect("name counted");
    writer.finish()
}
#[must_use]
pub fn owner(connector: u16) -> skein_ui::Owner {
    skein_ui::Owner { number: u32::from(connector) }
}
#[must_use]
pub fn path(path: &[Box<[u8]>], most: u32) -> (ResourceWord, Box<[Hole]>) {
    match resource(path) {
        Resource::Repository { host, owner, name } => {
            (ResourceWord::Repository, Box::new([Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))]))
        }
        Resource::Branch { host, owner, name, branch } => (
            ResourceWord::Branch,
            Box::new([Hole::Name(joined(branch, most)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))]),
        ),
        Resource::Pull { host, owner, name, number } => (
            ResourceWord::Pull,
            Box::new([Hole::Name(copy_of(number)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))]),
        ),
        Resource::Issue { host, owner, name, number } => (
            ResourceWord::Issue,
            Box::new([Hole::Name(copy_of(number)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))]),
        ),
        Resource::Adopted { forge, repository } => {
            (ResourceWord::Adopted, Box::new([Hole::Number(u64::from(repository)), Hole::Number(u64::from(forge))]))
        }
        Resource::Other => (ResourceWord::Path, Box::new([Hole::Name(joined(path, most))])),
    }
}
#[must_use]
pub fn broken(broken: Option<Broken>) -> Option<(Word, Box<[Hole]>)> {
    let word = match broken {
        None => return None,
        Some(Broken::Required) => Word::Required,
        Some(Broken::ForgeNumber) => Word::ForgeNumber,
        Some(Broken::Checks) => Word::BadChecks,
        Some(Broken::ContextHome) => Word::ContextHome,
        Some(Broken::NoCheck) => Word::NoCheck,
        Some(Broken::Long { bound }) => return Some((Word::Long, Box::new([Hole::Number(u64::from(bound))]))),
    };
    Some((word, Box::new([])))
}
#[must_use]
pub const fn kinds(kinds: Kinds) -> [(bool, Word); 9] {
    [
        (kinds.read, Word::Read),
        (kinds.push, Word::Push),
        (kinds.open, Word::Open),
        (kinds.land, Word::Land),
        (kinds.review, Word::Review),
        (kinds.status, Word::Status),
        (kinds.comment, Word::Comment),
        (kinds.issue, Word::Issue),
        (kinds.branch, Word::Branch),
    ]
}
#[must_use]
pub const fn role(role: Role) -> Word {
    match role {
        Role::Owned => Word::Owned,
        Role::Adopted => Word::AdoptedRole,
        Role::Fork => Word::Fork,
        Role::Context => Word::Context,
    }
}
