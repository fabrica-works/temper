//! temper's frame and forge words as checked English data.
use alloc::boxed::Box;
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
    ChatTime,
    ChatHint,
    ChatSendHint,
    Demo,
    DemoModel,
    ConnectModel,
    DemoHeading,
    DemoRequest,
    SuggestedRequests,
    DemoScript,
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
    SignedOut,
    OutOfDate,
    SignedOutMain,
    OutOfDateMain,
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
    Unshowable,
}
const ALL: [Word; 95] = [
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
    Word::ChatTime,
    Word::ChatHint,
    Word::ChatSendHint,
    Word::Demo,
    Word::DemoModel,
    Word::ConnectModel,
    Word::DemoHeading,
    Word::DemoRequest,
    Word::SuggestedRequests,
    Word::DemoScript,
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
    Word::SignedOut,
    Word::OutOfDate,
    Word::SignedOutMain,
    Word::OutOfDateMain,
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
    Word::Unshowable,
];
impl Word {
    fn id(self, connector: u16) -> PhraseId {
        let number = match self {
            Self::SignedOut => 180,
            Self::OutOfDate => 181,
            Self::SignedOutMain => 182,
            Self::OutOfDateMain => 183,
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
            Self::ChatTime => 184,
            Self::ChatHint => 185,
            Self::ChatSendHint => 186,
            Self::Demo => 187,
            Self::DemoModel => 188,
            Self::ConnectModel => 189,
            Self::DemoHeading => 190,
            Self::DemoRequest => 191,
            Self::SuggestedRequests => 192,
            Self::DemoScript => 193,
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
            Self::Unshowable => 24,
        };
        let owner =
            if number >= 100 { crate::frame::owner() } else { temper_client_domain::forge::words::owner(connector) };
        PhraseId { owner, number }
    }
}
/// The shell whose words the view says (client/browser.md, 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Terminal,
    Browser,
}
#[derive(Clone, Debug)]
pub struct Words {
    pub book: Phrasebook,
    pub language: u16,
    pub connector: u16,
}
impl Words {
    pub fn english(connector: u16, text: u32, shell: Shell) -> Result<Words, Invalid> {
        let joined = match text.checked_mul(2) {
            Some(bytes) => match bytes.checked_add(1) {
                Some(bytes) => bytes,
                None => return Err(Invalid::Overflow),
            },
            None => return Err(Invalid::Overflow),
        };
        let mut shapes = List::with_capacity(512);
        let mut phrases = List::with_capacity(512);
        for word in ALL {
            let (literals, kinds) = english(word, shell);
            let mut holes = List::with_capacity(3);
            let mut segments = List::with_capacity(7);
            for (index, literal) in literals.iter().enumerate() {
                segments.push(Segment::Text(copy_of(literal))).expect("closed English segments fit");
                if let Some(kind) = kinds.get(index) {
                    holes
                        .push(HoleShape {
                            kind: *kind,
                            bytes: if word == Word::DemoRequest {
                                temper_client_domain::forge::DEMO_FIELD_BYTES
                            } else if *kind == HoleKind::Name {
                                joined
                            } else {
                                0
                            },
                        })
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
        for phrase in jig_language().phrases {
            phrases.push(phrase).expect("jig words fit");
        }
        crate::accounts::words::extend(&mut shapes, &mut phrases);
        crate::cards::words::extend(text, &mut shapes, &mut phrases);
        temper_client_domain::forge::words::extend(connector, text, &mut shapes, &mut phrases);
        resource_phrases(connector, joined, &mut shapes, &mut phrases);
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed(),
        };
        let limits = PhraseLimits {
            languages: 1,
            phrases: 512,
            holes: 3,
            segments: 1536,
            text_bytes: 24576,
            hole_bytes: joined.max(temper_client_domain::forge::DEMO_FIELD_BYTES),
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
    /// Absolute UTC time: the view has delivered instants, but no current clock.
    pub(crate) fn chat_time(&self, at: skein_lib::Wall) -> Box<[u8]> {
        self.say(Word::ChatTime, &[Hole::Name(copy_of(&chat_clock(at)))])
    }

    pub fn text(&self, builder: &mut Builder, word: Word, holes: &[Hole]) {
        builder.text(Run::Plain, &self.say(word, holes));
    }
    pub fn resource_text(
        &self,
        builder: &mut Builder,
        word: temper_client_domain::forge::words::ResourceWord,
        holes: &[Hole],
    ) {
        let text = self.book.say(self.language, word.id(self.connector), holes).expect("checked resource phrase").text;
        builder.text(Run::Plain, &text);
    }
    #[must_use]
    pub(crate) fn path_bytes(&self) -> u32 {
        self.book
            .capacity(temper_client_domain::forge::words::ResourceWord::Path.id(self.connector))
            .expect("checked path word")
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
        Some(most.max(crate::forge::words::capacity(self)?).max(crate::accounts::words::capacity(self)?))
    }
}
const DEMO_REQUEST: &[&[u8]] = &[b"Try a request in ", b"/", b". Follow the task from its branch to a landed change."];
fn english(word: Word, shell: Shell) -> (&'static [&'static [u8]], &'static [HoleKind]) {
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
        Word::ClosedChats => (&[b"Closed chats"], &[]),
        Word::MoreClosedChats => (&[b"More closed chats"], &[]),
        Word::Chat => (&[b"Chat ", b""], &[Number]),
        Word::ChatSpend => (&[b"Spent ", b"; delegates spent ", b""], &[Number, Number]),
        Word::ChatDelegates => (&[b"Live delegates ", b""], &[Number]),
        Word::ChatTime => (&[b"", b""], &[Name]),
        Word::ChatSendHint => (&[b"Enter to send \xc2\xb7 Shift+Enter for a new line"], &[]),
        Word::Demo => (&[b"Demo"], &[]),
        Word::DemoModel => (&[b"This is a demo, running on a demo model."], &[]),
        Word::ConnectModel => (&[b"Connect a model"], &[]),
        Word::DemoHeading => (&[b"What shall we change?"], &[]),
        Word::DemoRequest => (DEMO_REQUEST, &[Name, Name]),
        Word::SuggestedRequests => (&[b"Suggested requests"], &[]),
        Word::DemoScript => (&[b"The demo model plays these requests from a script."], &[]),
        Word::ChatHint => (&[b"Enter to start \xc2\xb7 Shift+Enter for a new line"], &[]),
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
        Word::SignedOut => (&[b"Signed out"], &[]),
        Word::OutOfDate => (&[b"Out of date"], &[]),
        Word::SignedOutMain => match shell {
            Shell::Terminal => (&[b"Signed out."], &[]),
            Shell::Browser => (&[b"Signed out. To sign in, open a sign-in link temper serve printed."], &[]),
        },
        Word::OutOfDateMain => (&[b"This temper is out of date: run the temper the engine runs."], &[]),
        Word::Connecting => (&[b"Connecting"], &[]),
        Word::Behind => (&[b"Behind"], &[]),
        Word::SignOut => (&[b"Sign out"], &[]),
        Word::SignOutLost => (&[b"Sign out did not reach the engine."], &[]),
        Word::Unshowable => (&[b"Cannot show this part."], &[]),
    }
}

/// Public jig words with the overrides temper shows and its referee checks.
#[must_use]
pub fn jig_language() -> Language {
    use jig_client::words::{StatusWord, Word as JigWord};
    let mut language = jig_client::words::english();
    for phrase in &mut language.phrases {
        if phrase.id == jig_client::words::Word::Decision(jig_client::words::DecisionWord::Funds).id() {
            phrase.forms.first_mut().expect("English word has a form").segments = Box::new([
                Segment::Text(copy_of(b"Funds ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" from your pool \xc2\xb7 ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" left, ")),
                Segment::Hole { index: 2, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" after")),
            ]);
        }
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
    }
    language
}

fn resource_phrases(connector: u16, joined: u32, shapes: &mut List<PhraseShape>, phrases: &mut List<Phrase>) {
    for word in temper_client_domain::forge::words::RESOURCES {
        let (literals, kinds) = temper_client_domain::forge::words::resource_english(word);
        let mut holes = List::with_capacity(3);
        let mut segments = List::with_capacity(7);
        for (index, literal) in literals.iter().enumerate() {
            segments.push(Segment::Text(copy_of(literal))).expect("closed English segments fit");
            if let Some(kind) = kinds.get(index) {
                holes
                    .push(HoleShape { kind: *kind, bytes: if *kind == HoleKind::Name { joined } else { 0 } })
                    .expect("three holes fit");
                segments
                    .push(Segment::Hole { index: u16::try_from(index).expect("three hole indices fit"), kind: *kind })
                    .expect("closed segments fit");
            }
        }
        let id = word.id(connector);
        shapes.push(PhraseShape { id, holes: holes.into_boxed(), plural: None }).expect("closed shapes fit");
        phrases
            .push(Phrase { id, forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]) })
            .expect("closed phrases fit");
    }
}

// Gregorian civil month/day from epoch days, using 400-year eras and March-based months.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "u64 nanoseconds bound epoch days to 213503; calendar intermediates fit u64 and decimal digits are at most 9"
)]
fn chat_clock(at: skein_lib::Wall) -> [u8; 12] {
    let days = at.as_secs() / 86_400 + 719_468;
    let day_of_era = days % 146_097;
    let year = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year + year / 4 - year / 100);
    let march_month = (5 * day_of_year + 2) / 153;
    let day = u8::try_from(day_of_year - (153 * march_month + 2) / 5 + 1).expect("day fits");
    let month = u8::try_from(if march_month < 10 { march_month + 3 } else { march_month - 9 }).expect("month fits");
    let minutes = at.as_secs() / 60;
    let hour = u8::try_from((minutes / 60) % 24).expect("hour fits");
    let minute = u8::try_from(minutes % 60).expect("minute fits");
    [
        b'0' + month / 10,
        b'0' + month % 10,
        b'-',
        b'0' + day / 10,
        b'0' + day % 10,
        b' ',
        b'0' + hour / 10,
        b'0' + hour % 10,
        b':',
        b'0' + minute / 10,
        b'0' + minute % 10,
        b'Z',
    ]
}
#[cfg(test)]
mod clock_tests {
    #[test]
    fn the_chat_clock_keeps_day_month_and_leap_boundaries() {
        for (seconds, expected) in [
            (0, b"01-01 00:00Z"),
            (86_399, b"01-01 23:59Z"),
            (86_400, b"01-02 00:00Z"),
            (2_678_399, b"01-31 23:59Z"),
            (2_678_400, b"02-01 00:00Z"),
            (1_709_251_199, b"02-29 23:59Z"),
            (1_709_251_200, b"03-01 00:00Z"),
        ] {
            assert_eq!(&super::chat_clock(skein_lib::Wall::from_nanos(seconds * 1_000_000_000)), expected);
        }
    }
}
