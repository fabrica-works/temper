//! Account English has its own phrase IDs (client/README.md, 5.5).
use crate::words::Words;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{Form, Hole, HoleKind, HoleShape, Phrase, PhraseId, PhraseShape, PluralClass, Segment};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Word {
    Accounts,
    Own,
    Deployment,
    Unknown,
    Empty,
    Provider,
    Account,
    Source,
    Place,
    Party,
    Project,
    DefaultUses,
    Uses,
    Plan,
    Key,
    SignIn,
    Program,
    None,
    Ready,
    Granted,
    Asking,
    Limited,
    NeedsSignIn,
    Misconfigured,
    Cancelled,
    Closing,
    Expiring,
    Attention,
    Spent,
    Cut,
    Settled,
    Signing,
    LastEnd,
    Made,
    Renewed,
    Declined,
    Later,
    Scope,
    Directory,
    File,
    Unloaded,
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
    Configured,
    NotLimited,
    UsesRefused,
    KeyLong,
    NotStored,
    LimitAccounts,
    LimitHolder,
    LimitSources,
    LimitSignIns,
    Held,
    ProviderRefused,
    Subject,
    UnknownNumber,
    Numbers,
    Kind,
    NotStarted,
    Address,
    Lost,
    Unsent,
    InFlight,
    Confirmed,
    Unconfirmed,
    Saved,
    Changed,
    KeyEmpty,
    Selection,
    Rank,
    ValueAgain,
    ProviderField,
    AccountField,
    SourceField,
    KeyField,
    OwnChoice,
    DeploymentChoice,
    SignInAction,
    AddKey,
    ReplaceKey,
    SignOut,
    RemoveKey,
    RemoveAccount,
    Resume,
    UpAccount,
    DownAccount,
    UpSource,
    DownSource,
    Confirm,
    Cancel,
    ConfirmSignOut,
    ConfirmKey,
    ConfirmAccount,
    OmittedDisplay,
    UsageUntil,
    Removed,
    RemovedUnconfirmed,
    Number,
    Until,
    After,
}
const ROWS: &[(Word, &[u8])] = &[
    (Word::Removed,b"Account removed; confirmed."),
    (Word::RemovedUnconfirmed,b"Account removed; revocation was not confirmed. Disconnect temper in the provider's settings."),
    (Word::Accounts, b"Accounts"),
    (Word::Own, b"Your accounts"),
    (Word::Deployment, b"Deployment accounts"),
    (Word::Unknown, b"Waiting for current account facts."),
    (Word::Empty, b"No accounts shown."),
    (Word::Provider, b"LLM provider "),
    (Word::Account, b"Account "),
    (Word::Source, b"Source "),
    (Word::Place, b"Order "),
    (Word::Party, b"Person "),
    (Word::Project, b"Project "),
    (Word::DefaultUses, b"Default uses"),
    (Word::Uses, b"Uses: "),
    (Word::Plan, b"Signed-in plan"),
    (Word::Key, b"API key"),
    (Word::SignIn, b"Sign-in"),
    (Word::Program, b"Program login"),
    (Word::None, b"No source kind"),
    (Word::Ready, b"Ready"),
    (Word::Granted, b"Granted"),
    (Word::Asking, b"Checking"),
    (Word::Limited, b"Limited. Tasks on it wait."),
    (Word::NeedsSignIn, b"Needs sign-in"),
    (Word::Misconfigured, b"The provider refused temper's sign-in. Check the configuration's registration."),
    (Word::Cancelled, b"Cancelled"),
    (Word::Closing, b"Closing"),
    (Word::Expiring, b"Expiring"),
    (Word::Attention, b"Needs attention"),
    (Word::Spent, b"Usage limit reached"),
    (Word::Cut, b"Some account facts were omitted."),
    (Word::Settled, b"No sign-ins in progress."),
    (Word::Signing, b"Sign-in in progress: source "),
    (Word::LastEnd, b"Last sign-in: source "),
    (Word::Made, b"Signed in; account "),
    (Word::Renewed, b"Signed in to the same account again; account "),
    (Word::Declined, b"You signed in without plan usage."),
    (Word::Later, b"Try later"),
    (Word::Scope, b"Missing required scope "),
    (Word::Directory, b"Directory"),
    (Word::File, b"File"),
    (Word::Unloaded, b"Cannot load "),
    (Word::Unreadable, b"Unreadable"),
    (Word::Refused, b"Refused"),
    (Word::Failed, b"Failed"),
    (Word::Stalled, b"Stalled"),
    (Word::TooLarge, b"Too large"),
    (Word::Configured, b"Configured sources cannot be changed here."),
    (Word::NotLimited, b"This source is not limited."),
    (Word::UsesRefused, b"The provider does not permit these uses."),
    (Word::KeyLong, b"The key is too long."),
    (Word::NotStored, b"The key could not be stored."),
    (Word::LimitAccounts, b"Account limit reached."),
    (Word::LimitHolder, b"Holder account limit reached."),
    (Word::LimitSources, b"Source limit reached."),
    (Word::LimitSignIns, b"Sign-in limit reached."),
    (Word::Held, b"Account already held for LLM provider "),
    (Word::ProviderRefused, b"Unknown or unavailable LLM provider "),
    (Word::Subject, b"This provider subject cannot be joined."),
    (Word::UnknownNumber, b"Unknown account or source."),
    (Word::Numbers, b"No account or source numbers remain."),
    (Word::Kind, b"Source kind refused: "),
    (Word::NotStarted, b"Sign-in has not started."),
    (Word::Address, b"Open this address in a browser on the engine's machine:"),
    (Word::Lost, b"This act may have been applied. Re-enter the same act, or Cancel to abandon it."),
    (Word::Unsent, b"This act was not sent. Re-enter it, or Cancel to abandon it."),
    (Word::InFlight, b"Account request in progress."),
    (Word::Confirmed, b"Signed out; revocation confirmed."),
    (
        Word::Unconfirmed,
        b"temper removed its tokens, but the provider did not confirm. Disconnect temper in the provider's settings.",
    ),
    (Word::Saved, b"Key kept as source "),
    (Word::Changed, b"Account changed."),
    (Word::KeyEmpty, b"Enter a key."),
    (Word::Selection, b"Choose the provider, account and source needed for this act."),
    (Word::Rank, b"Choose a positive position."),
    (Word::ValueAgain, b"Enter the key again for the same act, or Cancel."),
    (Word::ProviderField, b"Provider number"),
    (Word::AccountField, b"Account number (empty for a new account)"),
    (Word::SourceField, b"Source number"),
    (Word::KeyField, b"Key (pasted once)"),
    (Word::OwnChoice, b"Use your accounts"),
    (Word::DeploymentChoice, b"Use deployment accounts"),
    (Word::SignInAction, b"Sign in with provider"),
    (Word::AddKey, b"Add key"),
    (Word::ReplaceKey, b"Replace key"),
    (Word::SignOut, b"Sign source out"),
    (Word::RemoveKey, b"Remove key"),
    (Word::RemoveAccount, b"Remove account"),
    (Word::Resume, b"Resume source"),
    (Word::UpAccount, b"Move account up"),
    (Word::DownAccount, b"Move account down"),
    (Word::UpSource, b"Move source up"),
    (Word::DownSource, b"Move source down"),
    (Word::Confirm, b"Confirm"),
    (Word::Cancel, b"Cancel"),
    (Word::ConfirmSignOut, b"Sign this source out?"),
    (Word::ConfirmKey, b"Remove this key?"),
    (Word::ConfirmAccount, b"Remove this account and all its sources?"),
    (Word::OmittedDisplay, b"Display name omitted."),
    (Word::UsageUntil, b"Usage limited until "),
];
fn id(number: u16) -> PhraseId {
    PhraseId { owner: super::owner(), number }
}
pub(crate) fn say(words: &Words, word: Word, holes: &[Hole]) -> Box<[u8]> {
    let number = if word == Word::Number {
        200
    } else if word == Word::Until {
        201
    } else if word == Word::After {
        202
    } else {
        literal_number(word)
    };
    words.book.say(words.language, id(number), holes).expect("checked account phrase").text
}
pub(crate) fn extend(shapes: &mut List<PhraseShape>, phrases: &mut List<Phrase>) {
    for (index, (_, literal)) in ROWS.iter().enumerate() {
        let id = id(u16::try_from(index).expect("account words fit"));
        shapes.push(PhraseShape { id, holes: Box::new([]), plural: None }).expect("account shapes fit");
        phrases
            .push(Phrase {
                id,
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Text(copy_of(literal))]),
                }]),
            })
            .expect("account phrases fit");
    }
    for (number, prefix, kind) in [
        (200, b"".as_slice(), HoleKind::Number),
        (201, b"Limited until ".as_slice(), HoleKind::Wall),
        (202, b"Try again after ".as_slice(), HoleKind::Span),
    ] {
        let id = id(number);
        shapes
            .push(PhraseShape { id, holes: Box::new([HoleShape { kind, bytes: 0 }]), plural: None })
            .expect("account shape fits");
        phrases
            .push(Phrase {
                id,
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Text(copy_of(prefix)), Segment::Hole { index: 0, kind }]),
                }]),
            })
            .expect("account phrase fits");
    }
}
pub(crate) fn capacity(words: &Words) -> Option<u32> {
    let mut most = 0;
    for number in 0..u16::try_from(ROWS.len()).ok()? {
        most = most.max(words.book.capacity(id(number))?);
    }
    for number in [200, 201, 202] {
        most = most.max(words.book.capacity(id(number))?);
    }
    Some(most)
}
fn literal_number(word: Word) -> u16 {
    for (index, (candidate, _)) in ROWS.iter().enumerate() {
        if *candidate == word {
            return u16::try_from(index).expect("account words fit");
        }
    }
    unreachable!("closed account words")
}
