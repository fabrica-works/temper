//! Adoption phrases as checked English data (client.md, 4.4 and 4.7).
use crate::words::Words;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{
    Builder, Run,
    phrases::{Form, Hole, HoleKind, HoleShape, Phrase, PhraseId, PhraseShape, PluralClass, Segment},
};
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
type Row = (Word, &'static [&'static [u8]], &'static [HoleKind]);
const ROWS: [Row; 48] = [
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
fn id(connector: u16, index: usize) -> PhraseId {
    PhraseId {
        owner: crate::forge::bindings::owner(connector),
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
        let id = id(connector, index);
        shapes.push(PhraseShape { id, holes: holes.into_boxed(), plural: None }).expect("shapes fit");
        phrases
            .push(Phrase { id, forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]) })
            .expect("phrases fit");
    }
}
fn name_bytes(word: Word, index: usize, text: u32) -> u32 {
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
pub fn say(words: &Words, word: Word, holes: &[Hole]) -> Box<[u8]> {
    for (index, (candidate, _, _)) in ROWS.iter().enumerate() {
        if *candidate == word {
            return words
                .book
                .say(words.language, id(words.connector, index), holes)
                .expect("checked adoption phrase")
                .text;
        }
    }
    unreachable!("closed phrase vocabulary")
}
pub fn text(words: &Words, builder: &mut Builder, word: Word, holes: &[Hole]) {
    builder.text(Run::Plain, &say(words, word, holes));
}
#[must_use]
pub fn capacity(words: &Words) -> Option<u32> {
    let mut most = 0;
    for (index, _) in ROWS.iter().enumerate() {
        most = most.max(words.book.capacity(id(words.connector, index))?);
    }
    Some(most)
}
