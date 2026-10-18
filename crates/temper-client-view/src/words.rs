//! temper's frame and forge words as checked English data.
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
    TaskHeader,
    Asked,
    Plan,
    Latest,
    Agent,
    Procedure,
    AskedBy,
    AskedByPerson,
    AskedByTask,
    AskedByDeployment,
    Deadline,
    Priority,
    Folded,
    HeldBelow,
    ActivityAt,
    DecisionMade,
    UnreadTurn,
    Opaque,
    Milestone,

    RunBegan,
    RunEnded,
    LinesLeft,
    Message,
    Send,
    Stop,
    Close,
    CloseGrace,
    CloseGraceNanos,
    CloseNow,
    DelegateCancel,
    DelegateMove,
    CancelDelegate,
    TakeDelegate,
    Older,
    StartChat,
    Start,
    LiveChats,
    ClosedChats,
    MoreClosedChats,
    Chat,
    ChatSpend,
    ChatDelegates,
    Accept,
    Reject,
    Answer,
    Release,
    LeaveHeld,
    Take,
    HandBack,
    AnswerWords,
    Reason,
    Confirm,
    Cancel,
    Again,
    DoNotRepeat,
    Brand,
    InboxTitle,
    NeedsYou,
    InboxUpdates,
    AllProjects,
    AllKinds,
    AllGoals,
    Goal,
    Proposal,
    Question,
    Held,
    PersonTask,
    MarkSeen,
    Task,
    Project,
    Reply,

    Inbox,
    Chats,
    Adoption,
    Live,
    Offline,
    Connecting,
    Behind,
    SignOut,
    SignOutLost,
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
const ALL: [Word; 105] = [
    Word::TaskHeader,
    Word::Asked,
    Word::Plan,
    Word::Latest,
    Word::Agent,
    Word::Procedure,
    Word::AskedBy,
    Word::AskedByPerson,
    Word::AskedByTask,
    Word::AskedByDeployment,
    Word::Deadline,
    Word::Priority,
    Word::Folded,
    Word::HeldBelow,
    Word::ActivityAt,
    Word::DecisionMade,
    Word::UnreadTurn,
    Word::Opaque,
    Word::Milestone,
    Word::RunBegan,
    Word::RunEnded,
    Word::LinesLeft,
    Word::Message,
    Word::Send,
    Word::Stop,
    Word::Close,
    Word::CloseGrace,
    Word::CloseGraceNanos,
    Word::CloseNow,
    Word::DelegateCancel,
    Word::DelegateMove,
    Word::CancelDelegate,
    Word::TakeDelegate,
    Word::Older,
    Word::StartChat,
    Word::Start,
    Word::LiveChats,
    Word::ClosedChats,
    Word::MoreClosedChats,
    Word::Chat,
    Word::ChatSpend,
    Word::ChatDelegates,
    Word::Accept,
    Word::Reject,
    Word::Answer,
    Word::Release,
    Word::LeaveHeld,
    Word::Take,
    Word::HandBack,
    Word::AnswerWords,
    Word::Reason,
    Word::Confirm,
    Word::Cancel,
    Word::Again,
    Word::DoNotRepeat,
    Word::Brand,
    Word::InboxTitle,
    Word::NeedsYou,
    Word::InboxUpdates,
    Word::AllProjects,
    Word::AllKinds,
    Word::AllGoals,
    Word::Goal,
    Word::Proposal,
    Word::Question,
    Word::Held,
    Word::PersonTask,
    Word::MarkSeen,
    Word::Task,
    Word::Project,
    Word::Reply,
    Word::Inbox,
    Word::Chats,
    Word::Adoption,
    Word::Live,
    Word::Offline,
    Word::Connecting,
    Word::Behind,
    Word::SignOut,
    Word::SignOutLost,
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
    #[expect(clippy::too_many_lines, reason = "closed phrase identifier codec, exhaustive over Word")]
    fn id(self, connector: u16) -> PhraseId {
        let number = match self {
            Self::TaskHeader => 161,
            Self::Asked => 162,
            Self::Plan => 163,
            Self::Latest => 164,
            Self::Agent => 165,
            Self::Procedure => 166,
            Self::AskedBy => 167,
            Self::AskedByPerson => 168,
            Self::AskedByTask => 169,
            Self::AskedByDeployment => 170,
            Self::Deadline => 171,
            Self::Priority => 172,
            Self::Folded => 173,
            Self::HeldBelow => 174,
            Self::ActivityAt => 175,
            Self::DecisionMade => 176,
            Self::UnreadTurn => 177,
            Self::Opaque => 178,
            Self::Milestone => 179,

            Self::CancelDelegate => 157,
            Self::TakeDelegate => 158,
            Self::Close => 153,
            Self::CloseGrace => 154,
            Self::CloseGraceNanos => 159,
            Self::CloseNow => 160,
            Self::DelegateCancel => 155,
            Self::DelegateMove => 156,
            Self::RunBegan => 150,
            Self::RunEnded => 151,
            Self::LinesLeft => 152,
            Self::Message => 146,
            Self::Send => 147,
            Self::Stop => 148,
            Self::Older => 149,
            Self::StartChat => 138,
            Self::Start => 139,
            Self::LiveChats => 140,
            Self::ClosedChats => 141,
            Self::MoreClosedChats => 142,
            Self::Chat => 143,
            Self::ChatSpend => 144,
            Self::ChatDelegates => 145,
            Self::Accept => 125,
            Self::Reject => 126,
            Self::Answer => 127,
            Self::Release => 128,
            Self::LeaveHeld => 129,
            Self::Take => 130,
            Self::HandBack => 131,
            Self::AnswerWords => 132,
            Self::Reason => 133,
            Self::Confirm => 134,
            Self::Cancel => 135,
            Self::Again => 136,
            Self::DoNotRepeat => 137,
            Self::Brand => 100,
            Self::InboxTitle => 110,
            Self::NeedsYou => 111,
            Self::InboxUpdates => 112,
            Self::AllProjects => 113,
            Self::AllKinds => 114,
            Self::AllGoals => 115,
            Self::Goal => 116,
            Self::Proposal => 117,
            Self::Question => 118,
            Self::Held => 119,
            Self::PersonTask => 120,
            Self::MarkSeen => 121,
            Self::Task => 122,
            Self::Project => 123,
            Self::Reply => 124,

            Self::Inbox => 101,
            Self::Chats => 102,
            Self::Adoption => 103,
            Self::Live => 104,
            Self::Offline => 105,
            Self::Connecting => 106,
            Self::Behind => 107,
            Self::SignOut => 108,
            Self::SignOutLost => 109,
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
        let owner =
            if number >= 100 { crate::frame::owner() } else { (Owner::Application { number: connector }).opaque() };
        PhraseId { owner, number }
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
        let mut shapes = List::with_capacity(256);
        let mut phrases = List::with_capacity(256);
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
        for word in jig_client::words::ALL {
            shapes.push(word.shape(text)).expect("jig word shapes fit");
        }
        for mut phrase in jig_client::words::english().phrases {
            use jig_client::words::{StatusWord, Word as JigWord};
            let literal = if phrase.id == JigWord::Status(StatusWord::WaitingForYou).id() {
                Some(b"Waiting for you".as_slice())
            } else if phrase.id == JigWord::Status(StatusWord::Parked).id() {
                Some(b"Parked".as_slice())
            } else {
                None
            };
            if let Some(literal) = literal {
                phrase.forms.first_mut().expect("English word has a form").segments =
                    Box::new([Segment::Text(copy_of(literal))]);
            }
            phrases.push(phrase).expect("jig words fit");
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
            phrases: 256,
            holes: 3,
            segments: 1024,
            text_bytes: 8192,
            hole_bytes: joined.max(513),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &limits)?;
        Ok(Words { book, language: 0, connector })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(self.connector), holes).expect("checked forge phrase").text
    }
    #[must_use]
    pub fn say_word(&self, word: jig_client::words::Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(), holes).expect("checked jig word").text
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
        for word in jig_client::words::ALL {
            most = most.max(self.book.capacity(word.id())?);
        }
        Some(most.max(crate::forge::words::capacity(self)?))
    }
}
#[expect(clippy::too_many_lines, reason = "closed English phrase codec, one row per Word")]
fn english(word: Word) -> (&'static [&'static [u8]], &'static [HoleKind]) {
    use HoleKind::{Name, Number};
    match word {
        Word::TaskHeader => (&[b"Task details"], &[]),
        Word::Asked => (&[b"What was asked"], &[]),
        Word::Plan => (&[b"Plan"], &[]),
        Word::Latest => (&[b"Latest activity"], &[]),
        Word::Agent => (&[b"Agent"], &[]),
        Word::Procedure => (&[b"Procedure"], &[]),
        Word::AskedBy => (&[b"Asked by ", b""], &[Name]),
        Word::AskedByPerson => (&[b"Asked by person ", b""], &[Number]),
        Word::AskedByTask => (&[b"Asked by task ", b""], &[Number]),
        Word::AskedByDeployment => (&[b"Asked by deployment of project ", b""], &[Number]),
        Word::Deadline => (&[b"Due ", b""], &[HoleKind::Wall]),
        Word::Priority => (&[b"Priority ", b""], &[Number]),
        Word::Folded => (&[b"Folded: ", b" ", b""], &[Number, Name]),
        Word::HeldBelow => (&[b"Held below: ", b""], &[Number]),
        Word::ActivityAt => (&[b"At ", b""], &[HoleKind::Wall]),
        Word::DecisionMade => (&[b"Decision made"], &[]),
        Word::UnreadTurn => (&[b"Turn could not be decoded"], &[]),
        Word::Opaque => (&[b"Opaque content: ", b" bytes"], &[Number]),
        Word::Milestone => (&[b"Milestone ", b""], &[Number]),
        Word::RunBegan => (&[b"Run ", b" began"], &[Number]),
        Word::RunEnded => (&[b"Run ", b" ended"], &[Number]),
        Word::LinesLeft => (&[b"Earlier lines omitted ", b""], &[Number]),
        Word::Message => (&[b"Message"], &[]),
        Word::Send => (&[b"Send"], &[]),
        Word::CancelDelegate => (&[b"Cancel delegate ", b""], &[Number]),
        Word::TakeDelegate => (&[b"Take delegate ", b""], &[Number]),
        Word::Close => (&[b"Close"], &[]),
        Word::CloseGrace => (&[b"Close chat: allow ", b" seconds to finish, then cancel"], &[HoleKind::Span]),
        Word::CloseGraceNanos => (&[b"Close chat: allow ", b" nanoseconds to finish, then cancel"], &[Number]),
        Word::CloseNow => (&[b"Close chat now"], &[]),
        Word::DelegateCancel => (&[b"Delegate ", b" will be cancelled"], &[Number]),
        Word::DelegateMove => (&[b"Delegate ", b" will be moved to you"], &[Number]),
        Word::Stop => (&[b"Stop"], &[]),
        Word::Older => (&[b"Older"], &[]),
        Word::StartChat => (&[b"Start a chat"], &[]),
        Word::Start => (&[b"Start"], &[]),
        Word::LiveChats => (&[b"Live chats"], &[]),
        Word::ClosedChats => (&[b"Closed"], &[]),
        Word::MoreClosedChats => (&[b"More closed chats"], &[]),
        Word::Chat => (&[b"Chat ", b""], &[Number]),
        Word::ChatSpend => (&[b"Spent ", b"; delegates spent ", b""], &[Number, Number]),
        Word::ChatDelegates => (&[b"Live delegates ", b""], &[Number]),
        Word::Accept => (&[b"Accept"], &[]),
        Word::Reject => (&[b"Reject with reason"], &[]),
        Word::Answer => (&[b"Answer"], &[]),
        Word::Release => (&[b"Release task"], &[]),
        Word::LeaveHeld => (&[b"Leave held"], &[]),
        Word::Take => (&[b"Take"], &[]),
        Word::HandBack => (&[b"Hand back"], &[]),
        Word::AnswerWords => (&[b"Answer words"], &[]),
        Word::Reason => (&[b"Reason"], &[]),
        Word::Confirm => (&[b"Confirm"], &[]),
        Word::Cancel => (&[b"Cancel"], &[]),
        Word::Again => (&[b"Repeat the effect"], &[]),
        Word::DoNotRepeat => (&[b"Do not repeat the effect"], &[]),
        Word::Brand => (&[b"temper"], &[]),
        Word::InboxTitle => (&[b"Inbox"], &[]),
        Word::NeedsYou => (&[b"Needs you"], &[]),
        Word::InboxUpdates => (&[b"Updates"], &[]),
        Word::AllProjects => (&[b"All projects"], &[]),
        Word::AllKinds => (&[b"All kinds"], &[]),
        Word::AllGoals => (&[b"All goals"], &[]),
        Word::Goal => (&[b"Goal"], &[]),
        Word::Proposal => (&[b"Proposal"], &[]),
        Word::Question => (&[b"Question"], &[]),
        Word::Held => (&[b"Held"], &[]),
        Word::PersonTask => (&[b"Person task"], &[]),
        Word::MarkSeen => (&[b"Mark seen"], &[]),
        Word::Task => (&[b"Task ", b""], &[Number]),
        Word::Project => (&[b"Project ", b""], &[Number]),
        Word::Reply => (&[b"Reply"], &[]),

        Word::Inbox => (&[b"Inbox ", b""], &[Number]),
        Word::Chats => (&[b"Chats"], &[]),
        Word::Adoption => (&[b"Adopt a repository"], &[]),
        Word::Live => (&[b"Live"], &[]),
        Word::Offline => (&[b"Offline"], &[]),
        Word::Connecting => (&[b"Connecting"], &[]),
        Word::Behind => (&[b"Behind"], &[]),
        Word::SignOut => (&[b"Sign out"], &[]),
        Word::SignOutLost => (&[b"Sign out did not reach the engine."], &[]),
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
