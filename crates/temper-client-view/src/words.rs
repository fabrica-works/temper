//! The forge's English phrasebook, copied as checked data from ops's pattern.
use alloc::boxed::Box;
use jig_client_view::Owner;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{
    Builder, Run,
    phrases::{
        Form, Hole, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook,
        PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
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
    Unshowable,
}
const ALL: [Word; 25] = [
    Word::Connector,
    Word::Repository,
    Word::Branch,
    Word::Pull,
    Word::Issue,
    Word::Adopted,
    Word::Path,
    Word::Read,
    Word::Push,
    Word::OpenPull,
    Word::MergePull,
    Word::Review,
    Word::Status,
    Word::Comment,
    Word::CreateIssue,
    Word::CreateBranch,
    Word::EditPull,
    Word::UpdatePull,
    Word::ClosePull,
    Word::EditIssue,
    Word::Reviewers,
    Word::DeleteBranch,
    Word::OtherKind,
    Word::Shown,
    Word::Unshowable,
];
impl Word {
    fn id(self, connector: u16) -> PhraseId {
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
            Self::Unshowable => 24,
        };
        PhraseId { owner: (Owner::Application { number: connector }).opaque(), number }
    }
}
#[must_use]
pub const fn kind(kind: u16) -> Option<Word> {
    match kind {
        1 => Some(Word::Read),
        2 => Some(Word::Push),
        3 => Some(Word::OpenPull),
        4 => Some(Word::MergePull),
        5 => Some(Word::Review),
        6 => Some(Word::Status),
        7 => Some(Word::Comment),
        8 => Some(Word::CreateIssue),
        9 => Some(Word::CreateBranch),
        10 => Some(Word::EditPull),
        11 => Some(Word::UpdatePull),
        12 => Some(Word::ClosePull),
        13 => Some(Word::EditIssue),
        14 => Some(Word::Reviewers),
        15 => Some(Word::DeleteBranch),
        _ => None,
    }
}
#[derive(Clone, Debug)]
pub struct Words {
    pub book: Phrasebook,
    pub language: u16,
    pub connector: u16,
}
impl Words {
    pub fn english(connector: u16, text: u32) -> Result<Words, Invalid> {
        let joined = match text.checked_mul(2) {
            Some(bytes) => match bytes.checked_add(1) {
                Some(bytes) => bytes,
                None => return Err(Invalid::Overflow),
            },
            None => return Err(Invalid::Overflow),
        };
        let mut shapes = List::with_capacity(74);
        let mut phrases = List::with_capacity(74);
        for word in ALL {
            let (literals, kinds) = english(word);
            let mut holes = List::with_capacity(3);
            let mut segments = List::with_capacity(7);
            for (index, literal) in literals.iter().enumerate() {
                segments.push(Segment::Text(copy_of(literal))).expect("closed English segments fit");
                if let Some(kind) = kinds.get(index) {
                    holes
                        .push(HoleShape { kind: *kind, bytes: if *kind == HoleKind::Name { joined } else { 0 } })
                        .expect("three holes fit");
                    segments
                        .push(Segment::Hole {
                            index: u16::try_from(index).expect("three hole indices fit"),
                            kind: *kind,
                        })
                        .expect("closed segments fit");
                }
            }
            let id = word.id(connector);
            shapes.push(PhraseShape { id, holes: holes.into_boxed(), plural: None }).expect("closed shapes fit");
            phrases
                .push(Phrase {
                    id,
                    forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]),
                })
                .expect("closed phrases fit");
        }
        crate::forge::words::extend(connector, text, &mut shapes, &mut phrases);
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed(),
        };
        let limits = PhraseLimits {
            languages: 1,
            phrases: 74,
            holes: 3,
            segments: 518,
            text_bytes: 6144,
            hole_bytes: joined.max(513),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &limits)?;
        Ok(Words { book, language: 0, connector })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(self.connector), holes).expect("checked forge phrase").text
    }
    pub fn text(&self, builder: &mut Builder, word: Word, holes: &[Hole]) {
        builder.text(Run::Plain, &self.say(word, holes));
    }
    #[must_use]
    pub fn capacity(&self) -> Option<u32> {
        let mut most = 0;
        for word in ALL {
            most = most.max(self.book.capacity(word.id(self.connector))?);
        }
        Some(most.max(crate::forge::words::capacity(self)?))
    }
}
fn english(word: Word) -> (&'static [&'static [u8]], &'static [HoleKind]) {
    use HoleKind::{Name, Number};
    match word {
        Word::Connector => (&[b"forge"], &[]),
        Word::Repository => (&[b"", b" on ", b""], &[Name, Name]),
        Word::Branch => (&[b"branch ", b" of ", b" on ", b""], &[Name, Name, Name]),
        Word::Pull => (&[b"pull request #", b" of ", b" on ", b""], &[Name, Name, Name]),
        Word::Issue => (&[b"issue #", b" of ", b" on ", b""], &[Name, Name, Name]),
        Word::Adopted => (&[b"repository ", b" on forge ", b""], &[Number, Number]),
        Word::Path => (&[b"", b""], &[Name]),
        Word::Read => (&[b"read"], &[]),
        Word::Push => (&[b"push"], &[]),
        Word::OpenPull => (&[b"open a pull request"], &[]),
        Word::MergePull => (&[b"merge a pull request"], &[]),
        Word::Review => (&[b"post a review"], &[]),
        Word::Status => (&[b"set a commit status"], &[]),
        Word::Comment => (&[b"post a comment"], &[]),
        Word::CreateIssue => (&[b"create an issue"], &[]),
        Word::CreateBranch => (&[b"create a branch"], &[]),
        Word::EditPull => (&[b"edit a pull request"], &[]),
        Word::UpdatePull => (&[b"update a pull request from its base"], &[]),
        Word::ClosePull => (&[b"close or reopen a pull request"], &[]),
        Word::EditIssue => (&[b"edit, close or reopen an issue"], &[]),
        Word::Reviewers => (&[b"set reviewers"], &[]),
        Word::DeleteBranch => (&[b"delete a branch"], &[]),
        Word::OtherKind => (&[b"kind ", b""], &[Number]),
        Word::Shown => (&[b"Shown on the forge."], &[]),
        Word::Unshowable => (&[b"Cannot show this part."], &[]),
    }
}
