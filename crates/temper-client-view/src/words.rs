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
    TimeYesterdayWord,
    TimeJustNow,
    TimeUnderMinute,
    TimeMinutes,
    TimeHours,
    TimeHoursMinutes,
    TimeDays,
    TimeAgo,
    TimeAt,
    TimeClock,
    TimePair,
    TimeDate,
    TimeDateYear,
    TimeToday,
    TimeYesterday,
    TimeWaiting,
    TimeWaitingChat,
    TimeEnded,
    TimeEndedReport,
    TimeFailed,
    TimeCancelled,
    TimeSundayShort,
    TimeMondayShort,
    TimeTuesdayShort,
    TimeWednesdayShort,
    TimeThursdayShort,
    TimeFridayShort,
    TimeSaturdayShort,
    TimeSunday,
    TimeMonday,
    TimeTuesday,
    TimeWednesday,
    TimeThursday,
    TimeFriday,
    TimeSaturday,
    TimeJanuary,
    TimeFebruary,
    TimeMarch,
    TimeApril,
    TimeMay,
    TimeJune,
    TimeJuly,
    TimeAugust,
    TimeSeptember,
    TimeOctober,
    TimeNovember,
    TimeDecember,

    ChangeModel,
    UsingModel,
    UsingDefault,
    UsingKey,
    UsingPlan,
    UsingProgram,
    ModelNeedsAccount,
    ModelEffort,

    WaitRefused,
    DependencyParked,
    WaitInvalid,
    ChoiceDraft,
    ChoiceDraftDefault,
    WaitingAccountName,
    WaitingAccount,
    ChoiceProvider,
    NeedsAccount,
    AccountReadinessUnknown,
    AccountBanner,
    AccountSignIn,
    AccountProgram,
    AccountKey,
    ChoiceApply,
    ChoiceReset,
    AccountWaiting,
    WaitRefreshing,
    WaitHost,
    WaitEmpty,
    WaitCredential,
    WaitCapacity,
    Usage,
    ChoicePending,
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
    DeadlineMoment,
    Priority,
    Folded,
    HeldBelow,
    ActivityAt,
    DecisionMade,
    UnreadTurn,
    Opaque,
    Milestone,

    ToolCalls,
    ToolSeparator,
    ToolRepeated,
    ChatEnded,
    ChatEndedReport,
    ChatFailed,
    ChatCancelled,
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
    YourChats,
    AllChats,
    NoChats,
    NoLiveChats,
    NoClosedChats,
    LiveChats,
    ClosedChats,
    MoreClosedChats,
    Chat,
    TaskWho,
    ChatSpend,
    ChatDelegates,
    ChatTime,
    ChatHint,
    ChatSendHint,
    DemoBoth,
    DemoForge,
    ConnectForge,
    ChatTitle,
    DemoModelName,
    DemoUsing,
    Default,
    DemoRepository,
    SignedDeploymentOwner,
    SignedProjectOwner,
    SignedName,
    NewChat,
    Ready,
    Demo,
    DemoModel,
    ConnectModel,
    DemoHeading,
    DemoRequest,
    SuggestedRequests,
    DemoScript,
    DelegationChange,
    DelegationReport,
    DelegationVerdict,
    DelegationApproval,
    Landed,
    LandedMerge,
    Steps,
    BranchPushed,
    PullRequestOpened,
    CiPassed,

    GoalReport,
    GoalChange,
    GoalVerdict,
    GoalApproval,
    GoalResult,
    BudgetAuthority,
    BudgetLeft,
    DelegatesSpent,
    ReservedDelegates,
    FundedBy,
    YourPool,
    PersonPool,
    RecurringBudget,
    ProjectBudget,
    MaySpend,
    MayDelegate,
    DelegateBounds,
    Models,
    AllModels,
    NoModels,
    AuthorityCut,
    Amount,
    BudgetDeadline,
    BudgetDeadlineDate,
    BudgetDeadlineMoment,
    NoDeadline,
    DelegateOne,

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
    Menu,
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

    WaitingForTask,
    WaitingForTasks,
    WaitingForMoreTasks,
    WaitingForHoldings,
    RowSpent,
    Dependencies,
    DependenciesCut,
    WaitingForYourWords,
    DemoContextProject,
    DemoRepositoryForge,
    WaitingForDelegates,
    WaitingForYou,
    WaitingForPerson,
    WaitingForRole,
    WaitingForNews,
    Running,
    Writing,
    RunningTool,
    GoalPriority,
    PhaseWaiting,
    PhaseIdle,
    PhaseDue,
    PhasePreparing,
    PhaseClaimed,
    PhaseActive,
    PhaseRetrying,
    PhaseClosing,
    PhaseHeld,

    TaskBrief,
    RunLive,
    RunWaiting,
    RunOver,
    NoRun,
    RunStarted,
    ItsTasks,
    SteerNote,
    StopHolds,
    Amend,
    AmendTask,
    Specification,
    Tracked,
    AmendPriority,
    SendAmendment,
    Discard,
    NoChange,
    SpecTooLong,
    TaskChanged,
    LookAgain,
    DependencyAddedByYou,
    DependencyAddedByPerson,
    DependencyAddedByTask,
    DependencyAddedByDeployment,
    DependencyKeeps,
    MainNavigation,
    KindFrom,
    NoLiveTasks,
    ItsTasksNoneYet,
    ItsTasksNone,
    ItsTasksMoreEnded,
    AllPlan,
    LivePlan,
    PlanCounts,
    PlanMoreCounts,
    MessageOnceRuns,
}
// Room for temper, jig, account, card and forge phrases.
// Seven chat projection phrases, alongside the agent's original work-line words.
// Eleven amendment editor phrases (client/README.md, 5.11), and five added
// dependency mark phrases (5.8).
// Two names for navigation and cards.
// Five chats heading, filter and empty phrases (client/README.md, 3.5).
// An empty plan's line (5.8) and a waiting agent's Message note (5.11).
// The composer's line and its chooser replace seven choice phrases (5.5).
// A task's name over its words in a chat (client/README.md, 3.5).
// Forty-seven time form, calendar and placement phrases (5.12).
// The change panel adds 86 forge phrases (2000–2085).
// Three empty and partial ended-children phrases (5.10).
const PHRASES: u32 = 770;
const ALL: [Word; 302] = [
    Word::TimeYesterdayWord,
    Word::TimeJustNow,
    Word::TimeUnderMinute,
    Word::TimeMinutes,
    Word::TimeHours,
    Word::TimeHoursMinutes,
    Word::TimeDays,
    Word::TimeAgo,
    Word::TimeAt,
    Word::TimeClock,
    Word::TimePair,
    Word::TimeDate,
    Word::TimeDateYear,
    Word::TimeToday,
    Word::TimeYesterday,
    Word::TimeWaiting,
    Word::TimeWaitingChat,
    Word::TimeEnded,
    Word::TimeEndedReport,
    Word::TimeFailed,
    Word::TimeCancelled,
    Word::TimeSundayShort,
    Word::TimeMondayShort,
    Word::TimeTuesdayShort,
    Word::TimeWednesdayShort,
    Word::TimeThursdayShort,
    Word::TimeFridayShort,
    Word::TimeSaturdayShort,
    Word::TimeSunday,
    Word::TimeMonday,
    Word::TimeTuesday,
    Word::TimeWednesday,
    Word::TimeThursday,
    Word::TimeFriday,
    Word::TimeSaturday,
    Word::TimeJanuary,
    Word::TimeFebruary,
    Word::TimeMarch,
    Word::TimeApril,
    Word::TimeMay,
    Word::TimeJune,
    Word::TimeJuly,
    Word::TimeAugust,
    Word::TimeSeptember,
    Word::TimeOctober,
    Word::TimeNovember,
    Word::TimeDecember,
    Word::ChangeModel,
    Word::UsingModel,
    Word::UsingDefault,
    Word::UsingKey,
    Word::UsingPlan,
    Word::UsingProgram,
    Word::ModelNeedsAccount,
    Word::ModelEffort,
    Word::WaitRefused,
    Word::DependencyParked,
    Word::WaitInvalid,
    Word::ChoiceDraft,
    Word::ChoiceDraftDefault,
    Word::WaitingAccountName,
    Word::WaitingAccount,
    Word::ChoiceProvider,
    Word::NeedsAccount,
    Word::AccountReadinessUnknown,
    Word::AccountBanner,
    Word::AccountSignIn,
    Word::AccountProgram,
    Word::AccountKey,
    Word::ChoiceApply,
    Word::ChoiceReset,
    Word::AccountWaiting,
    Word::WaitRefreshing,
    Word::WaitHost,
    Word::WaitEmpty,
    Word::WaitCredential,
    Word::WaitCapacity,
    Word::Usage,
    Word::ChoicePending,
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
    Word::DeadlineMoment,
    Word::Priority,
    Word::Folded,
    Word::HeldBelow,
    Word::ActivityAt,
    Word::DecisionMade,
    Word::UnreadTurn,
    Word::Opaque,
    Word::Milestone,
    Word::ToolCalls,
    Word::ToolSeparator,
    Word::ToolRepeated,
    Word::ChatEnded,
    Word::ChatEndedReport,
    Word::ChatFailed,
    Word::ChatCancelled,
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
    Word::YourChats,
    Word::AllChats,
    Word::NoChats,
    Word::NoLiveChats,
    Word::NoClosedChats,
    Word::LiveChats,
    Word::ClosedChats,
    Word::MoreClosedChats,
    Word::Chat,
    Word::TaskWho,
    Word::ChatSpend,
    Word::ChatDelegates,
    Word::ChatTime,
    Word::ChatHint,
    Word::ChatSendHint,
    Word::DemoBoth,
    Word::DemoForge,
    Word::ConnectForge,
    Word::ChatTitle,
    Word::DemoModelName,
    Word::DemoUsing,
    Word::Default,
    Word::DemoRepository,
    Word::SignedDeploymentOwner,
    Word::SignedProjectOwner,
    Word::SignedName,
    Word::NewChat,
    Word::Ready,
    Word::Demo,
    Word::DemoModel,
    Word::ConnectModel,
    Word::DemoHeading,
    Word::DemoRequest,
    Word::SuggestedRequests,
    Word::DemoScript,
    Word::DelegationChange,
    Word::DelegationReport,
    Word::DelegationVerdict,
    Word::DelegationApproval,
    Word::Landed,
    Word::LandedMerge,
    Word::Steps,
    Word::BranchPushed,
    Word::PullRequestOpened,
    Word::CiPassed,
    Word::GoalReport,
    Word::GoalChange,
    Word::GoalVerdict,
    Word::GoalApproval,
    Word::GoalResult,
    Word::BudgetAuthority,
    Word::BudgetLeft,
    Word::DelegatesSpent,
    Word::ReservedDelegates,
    Word::FundedBy,
    Word::YourPool,
    Word::PersonPool,
    Word::RecurringBudget,
    Word::ProjectBudget,
    Word::MaySpend,
    Word::MayDelegate,
    Word::DelegateBounds,
    Word::Models,
    Word::AllModels,
    Word::NoModels,
    Word::AuthorityCut,
    Word::Amount,
    Word::BudgetDeadline,
    Word::BudgetDeadlineDate,
    Word::BudgetDeadlineMoment,
    Word::NoDeadline,
    Word::DelegateOne,
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
    Word::Menu,
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
    Word::WaitingForTask,
    Word::WaitingForTasks,
    Word::WaitingForMoreTasks,
    Word::WaitingForHoldings,
    Word::RowSpent,
    Word::Dependencies,
    Word::DependenciesCut,
    Word::WaitingForYourWords,
    Word::DemoContextProject,
    Word::DemoRepositoryForge,
    Word::WaitingForDelegates,
    Word::WaitingForYou,
    Word::WaitingForPerson,
    Word::WaitingForRole,
    Word::WaitingForNews,
    Word::Running,
    Word::Writing,
    Word::RunningTool,
    Word::GoalPriority,
    Word::PhaseWaiting,
    Word::PhaseIdle,
    Word::PhaseDue,
    Word::PhasePreparing,
    Word::PhaseClaimed,
    Word::PhaseActive,
    Word::PhaseRetrying,
    Word::PhaseClosing,
    Word::PhaseHeld,
    Word::TaskBrief,
    Word::RunLive,
    Word::RunWaiting,
    Word::RunOver,
    Word::NoRun,
    Word::RunStarted,
    Word::ItsTasks,
    Word::SteerNote,
    Word::StopHolds,
    Word::Amend,
    Word::AmendTask,
    Word::Specification,
    Word::Tracked,
    Word::AmendPriority,
    Word::SendAmendment,
    Word::Discard,
    Word::NoChange,
    Word::SpecTooLong,
    Word::TaskChanged,
    Word::LookAgain,
    Word::DependencyAddedByYou,
    Word::DependencyAddedByPerson,
    Word::DependencyAddedByTask,
    Word::DependencyAddedByDeployment,
    Word::DependencyKeeps,
    Word::MainNavigation,
    Word::KindFrom,
    Word::NoLiveTasks,
    Word::ItsTasksNoneYet,
    Word::ItsTasksNone,
    Word::ItsTasksMoreEnded,
    Word::AllPlan,
    Word::LivePlan,
    Word::PlanCounts,
    Word::PlanMoreCounts,
    Word::MessageOnceRuns,
];
impl Word {
    #[expect(clippy::too_many_lines, reason = "one exhaustive table keeps the frame phrase IDs stable")]
    fn id(self, connector: u16) -> PhraseId {
        let number = match self {
            Self::TimeYesterdayWord => 1250,
            Self::TimeJustNow => 1200,
            Self::TimeUnderMinute => 1201,
            Self::TimeMinutes => 1202,
            Self::TimeHours => 1203,
            Self::TimeHoursMinutes => 1204,
            Self::TimeDays => 1205,
            Self::TimeAgo => 1206,
            Self::TimeAt => 1207,
            Self::TimeClock => 1208,
            Self::TimePair => 1209,
            Self::TimeDate => 1210,
            Self::TimeDateYear => 1211,
            Self::TimeToday => 1212,
            Self::TimeYesterday => 1213,
            Self::TimeWaiting => 1214,
            Self::TimeWaitingChat => 1215,
            Self::TimeEnded => 1216,
            Self::TimeEndedReport => 1217,
            Self::TimeFailed => 1218,
            Self::TimeCancelled => 1219,
            Self::TimeSundayShort => 1220,
            Self::TimeMondayShort => 1221,
            Self::TimeTuesdayShort => 1222,
            Self::TimeWednesdayShort => 1223,
            Self::TimeThursdayShort => 1224,
            Self::TimeFridayShort => 1225,
            Self::TimeSaturdayShort => 1226,
            Self::TimeSunday => 1227,
            Self::TimeMonday => 1228,
            Self::TimeTuesday => 1229,
            Self::TimeWednesday => 1230,
            Self::TimeThursday => 1231,
            Self::TimeFriday => 1232,
            Self::TimeSaturday => 1233,
            Self::TimeJanuary => 1234,
            Self::TimeFebruary => 1235,
            Self::TimeMarch => 1236,
            Self::TimeApril => 1237,
            Self::TimeMay => 1238,
            Self::TimeJune => 1239,
            Self::TimeJuly => 1240,
            Self::TimeAugust => 1241,
            Self::TimeSeptember => 1242,
            Self::TimeOctober => 1243,
            Self::TimeNovember => 1244,
            Self::TimeDecember => 1245,

            Self::SignedOut => 180,
            Self::OutOfDate => 181,
            Self::SignedOutMain => 182,
            Self::OutOfDateMain => 183,
            Self::ChangeModel => 1067,
            Self::UsingModel => 1068,
            Self::UsingDefault => 1069,
            Self::UsingKey => 1070,
            Self::UsingPlan => 1071,
            Self::UsingProgram => 1072,
            Self::ModelNeedsAccount => 1073,
            Self::ModelEffort => 1074,
            Self::WaitRefused => 1029,
            Self::DependencyParked => 1030,
            Self::WaitInvalid => 1028,
            Self::ChoiceDraft => 1024,
            Self::ChoiceDraftDefault => 1025,
            Self::WaitingAccountName => 1027,
            Self::WaitingAccount => 1023,
            Self::ChoiceProvider => 1020,
            Self::NeedsAccount => 1021,
            Self::AccountReadinessUnknown => 1022,
            Self::AccountBanner => 1000,
            Self::AccountSignIn => 1001,
            Self::AccountProgram => 1002,
            Self::AccountKey => 1003,
            Self::ChoiceApply => 1009,
            Self::ChoiceReset => 1010,
            Self::AccountWaiting => 1011,
            Self::WaitRefreshing => 1012,
            Self::WaitHost => 1013,
            Self::WaitEmpty => 1014,
            Self::WaitCredential => 1015,
            Self::WaitCapacity => 1016,
            Self::Usage => 1017,
            Self::ChoicePending => 1018,
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
            Self::DeadlineMoment => 194,
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
            Self::ToolCalls => 1040,
            Self::ToolSeparator => 1041,
            Self::ToolRepeated => 1042,
            Self::ChatEnded => 1043,
            Self::ChatEndedReport => 1044,
            Self::ChatFailed => 1045,
            Self::ChatCancelled => 1046,
            Self::RunBegan => 150,
            Self::RunEnded => 151,
            Self::LinesLeft => 152,
            Self::Message => 146,
            Self::Send => 147,
            Self::Stop => 148,
            Self::Older => 149,
            Self::StartChat => 138,
            Self::Start => 139,
            Self::YourChats => 347,
            Self::AllChats => 348,
            Self::NoChats => 349,
            Self::NoLiveChats => 350,
            Self::NoClosedChats => 351,
            Self::LiveChats => 140,
            Self::ClosedChats => 141,
            Self::MoreClosedChats => 142,
            Self::Chat => 143,
            Self::TaskWho => 1075,
            Self::ChatSpend => 144,
            Self::ChatDelegates => 145,
            Self::ChatTime => 184,
            Self::ChatHint => 185,
            Self::ChatSendHint => 186,
            Self::DemoBoth => 273,
            Self::DemoForge => 274,
            Self::ConnectForge => 275,
            Self::ChatTitle => 276,
            Self::DemoModelName => 277,
            Self::DemoUsing => 278,
            Self::Default => 279,
            Self::DemoRepository => 282,
            Self::SignedDeploymentOwner => 283,
            Self::SignedProjectOwner => 284,
            Self::SignedName => 285,
            Self::NewChat => 286,
            Self::Ready => 287,
            Self::Demo => 187,
            Self::DemoModel => 188,
            Self::ConnectModel => 189,
            Self::DemoHeading => 190,
            Self::DemoRequest => 191,
            Self::SuggestedRequests => 192,
            Self::DemoScript => 193,
            // 194 is DeadlineMoment; 200..=267 and 280..=281 are card words.
            Self::DelegationChange => 195,
            Self::DelegationReport => 196,
            Self::DelegationVerdict => 197,
            Self::DelegationApproval => 198,
            Self::Landed => 199,
            Self::LandedMerge => 268,
            Self::Steps => 269,
            Self::BranchPushed => 270,
            Self::PullRequestOpened => 271,
            Self::CiPassed => 272,
            Self::GoalReport => 300,
            Self::GoalChange => 301,
            Self::GoalVerdict => 302,
            Self::GoalApproval => 303,
            Self::GoalResult => 304,
            Self::BudgetAuthority => 305,
            Self::BudgetLeft => 306,
            Self::DelegatesSpent => 308,
            Self::ReservedDelegates => 309,
            Self::FundedBy => 310,
            Self::YourPool => 311,
            Self::PersonPool => 312,
            Self::RecurringBudget => 313,
            Self::ProjectBudget => 314,
            Self::MaySpend => 315,
            Self::MayDelegate => 316,
            Self::DelegateBounds => 317,
            Self::Models => 318,
            Self::AllModels => 319,
            Self::NoModels => 320,
            Self::AuthorityCut => 321,
            Self::Amount => 322,
            Self::BudgetDeadline => 323,
            Self::BudgetDeadlineDate => 324,
            Self::BudgetDeadlineMoment => 325,
            Self::NoDeadline => 326,
            Self::DelegateOne => 327,

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
            Self::Menu => 1076,
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

            Self::WaitingForTask => 288,
            Self::WaitingForTasks => 289,
            Self::WaitingForMoreTasks => 290,
            Self::WaitingForHoldings => 291,
            Self::RowSpent => 292,
            Self::Dependencies => 293,
            Self::DependenciesCut => 294,
            Self::WaitingForYourWords => 344,
            Self::DemoContextProject => 345,
            Self::DemoRepositoryForge => 346,
            Self::WaitingForDelegates => 295,
            Self::WaitingForYou => 296,
            Self::WaitingForPerson => 297,
            Self::WaitingForRole => 298,
            Self::WaitingForNews => 299,
            Self::Running => 330,
            Self::Writing => 331,
            Self::RunningTool => 332,
            Self::GoalPriority => 334,
            Self::PhaseWaiting => 335,
            Self::PhaseIdle => 336,
            Self::PhaseDue => 337,
            Self::PhasePreparing => 338,
            Self::PhaseClaimed => 339,
            Self::PhaseActive => 340,
            Self::PhaseRetrying => 341,
            Self::PhaseClosing => 342,
            Self::PhaseHeld => 343,
            Self::TaskBrief => 1031,
            Self::RunLive => 1032,
            Self::RunWaiting => 1033,
            Self::RunOver => 1034,
            Self::NoRun => 1035,
            Self::RunStarted => 1036,
            Self::ItsTasks => 1037,
            Self::SteerNote => 1038,
            Self::StopHolds => 1039,
            Self::Amend => 1047,
            Self::AmendTask => 1048,
            Self::Specification => 1049,
            Self::Tracked => 1050,
            Self::AmendPriority => 1051,
            Self::SendAmendment => 1052,
            Self::Discard => 1053,
            Self::NoChange => 1054,
            Self::SpecTooLong => 1055,
            Self::TaskChanged => 1056,
            Self::LookAgain => 1057,
            Self::DependencyAddedByYou => 1058,
            Self::DependencyAddedByPerson => 1059,
            Self::DependencyAddedByTask => 1060,
            Self::DependencyAddedByDeployment => 1061,
            Self::DependencyKeeps => 1062,
            Self::MainNavigation => 1063,
            Self::KindFrom => 1064,
            Self::NoLiveTasks => 1065,
            Self::MessageOnceRuns => 1066,
            Self::ItsTasksNoneYet => 1251,
            Self::ItsTasksNone => 1252,
            Self::ItsTasksMoreEnded => 1253,
            Self::AllPlan => 1254,
            Self::LivePlan => 1255,
            Self::PlanCounts => 1256,
            Self::PlanMoreCounts => 1257,
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
    /// The checked joined bound of ordinary Name holes.
    pub name_bytes: u32,
}
fn hole_bytes(word: Word, kind: HoleKind, last: bool, joined: u32) -> Result<u32, Invalid> {
    if word == Word::DemoRequest || word == Word::DemoRepository {
        return Ok(temper_client_domain::forge::DEMO_FIELD_BYTES);
    }
    let model_name = joined.min(crate::choice::MODEL_SHOWN);
    if word == Word::ModelEffort {
        return Ok(model_name);
    }
    if last
        && [Word::UsingModel, Word::UsingKey, Word::UsingPlan, Word::UsingProgram, Word::ModelNeedsAccount]
            .contains(&word)
    {
        let doubled = model_name.checked_mul(2).ok_or(Invalid::Overflow)?;
        return doubled.checked_add(3).ok_or(Invalid::Overflow);
    }
    Ok(if kind == HoleKind::Name { joined } else { 0 })
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
        let mut shapes = List::with_capacity(PHRASES);
        let mut phrases = List::with_capacity(PHRASES);
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
                            bytes: hole_bytes(word, *kind, index.checked_add(1) == Some(kinds.len()), joined)?,
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
            let plural = if word == Word::WaitingForDelegates || word == Word::ToolCalls || word == Word::ChatDelegates
            {
                Some(0)
            } else {
                None
            };
            shapes.push(PhraseShape { id, holes: holes.into_boxed(), plural }).expect("closed shapes fit");
            let other = Form { class: PluralClass::Other, segments: segments.into_boxed() };
            let forms: Box<[Form]> = if word == Word::WaitingForDelegates
                || word == Word::ToolCalls
                || word == Word::ChatDelegates
            {
                Box::new([
                    Form {
                        class: PluralClass::One,
                        segments: Box::new([
                            Segment::Text(copy_of(if word == Word::ToolCalls || word == Word::ChatDelegates {
                                b""
                            } else {
                                b"waiting on "
                            })),
                            Segment::Hole { index: 0, kind: HoleKind::Number },
                            Segment::Text(copy_of(if word == Word::ToolCalls { b" tool call" } else { b" delegate" })),
                        ]),
                    },
                    other,
                ])
            } else {
                Box::new([other])
            };
            phrases.push(Phrase { id, forms }).expect("closed phrases fit");
        }
        for word in jig_client::words::ALL {
            shapes.push(word.shape(text)).expect("jig word shapes fit");
        }
        for phrase in jig_language().phrases {
            phrases.push(phrase).expect("jig words fit");
        }
        crate::accounts::words::extend(&mut shapes, &mut phrases, joined);
        crate::cards::words::extend(text, &mut shapes, &mut phrases);
        temper_client_domain::forge::words::extend(connector, text, &mut shapes, &mut phrases);
        temper_client_domain::forge::words::extend_change(connector, &mut shapes, &mut phrases);
        resource_phrases(connector, joined, &mut shapes, &mut phrases);
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed(),
        };
        let limits = PhraseLimits {
            languages: 1,
            phrases: PHRASES,
            holes: 3,
            segments: 1536,
            text_bytes: 24576,
            // Whole demo requests keep their supplied bound.
            hole_bytes: joined.max(temper_client_domain::forge::DEMO_FIELD_BYTES),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &limits)?;
        Ok(Words { book, language: 0, connector, name_bytes: joined })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(self.connector), holes).expect("checked forge phrase").text
    }
    #[expect(clippy::manual_is_multiple_of, reason = "keep the UTC midnight remainder test explicit")]
    pub(crate) fn deadline(&self, at: skein_lib::Wall) -> Box<[u8]> {
        let word = if at.as_secs() % 86_400 == 0 { Word::BudgetDeadlineDate } else { Word::BudgetDeadlineMoment };
        self.say(word, &[Hole::Wall(at)])
    }
    #[must_use]
    pub fn say_word(&self, word: jig_client::words::Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(), holes).expect("checked jig word").text
    }
    /// A run's supplied placement time in absolute UTC.
    pub(crate) fn run_started(&self, at: skein_lib::Wall) -> Box<[u8]> {
        self.say(Word::RunStarted, &[Hole::Name(copy_of(&chat_clock(at)))])
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
#[expect(clippy::too_many_lines, reason = "one exhaustive table of the client's English words")]
fn english(word: Word, shell: Shell) -> (&'static [&'static [u8]], &'static [HoleKind]) {
    use HoleKind::{Name, Number};
    match word {
        Word::DelegationChange => (&[b"T", b" \xc2\xb7 CHANGE"], &[Number]),
        Word::DelegationReport => (&[b"T", b" \xc2\xb7 REPORT"], &[Number]),
        Word::DelegationVerdict => (&[b"T", b" \xc2\xb7 VERDICT"], &[Number]),
        Word::DelegationApproval => (&[b"T", b" \xc2\xb7 APPROVAL"], &[Number]),
        Word::Landed => (&[b"Landed"], &[]),
        Word::LandedMerge => (&[b"CI passed. Landed \xc2\xb7 merge commit ", b""], &[Name]),
        Word::Steps => (&[b"Steps"], &[]),
        Word::BranchPushed => (&[b"Branch pushed"], &[]),
        Word::PullRequestOpened => (&[b"Pull request opened"], &[]),
        Word::CiPassed => (&[b"CI passed"], &[]),
        Word::GoalReport => (&[b"Result \xc2\xb7 report"], &[]),
        Word::GoalChange => (&[b"Result \xc2\xb7 change"], &[]),
        Word::GoalVerdict => (&[b"Result \xc2\xb7 verdict"], &[]),
        Word::GoalApproval => (&[b"Result \xc2\xb7 approval"], &[]),
        Word::GoalResult => (&[b"Result"], &[]),
        Word::BudgetAuthority => (&[b"Budget and authority"], &[]),
        Word::BudgetLeft => (&[b"", b" spent \xc2\xb7 ", b" left of ", b""], &[Number, Number, Number]),
        Word::DelegatesSpent => (&[b"Delegates spent"], &[]),
        Word::ReservedDelegates => (&[b"Reserved for delegates"], &[]),
        Word::FundedBy => (&[b"Funded by"], &[]),
        Word::YourPool => (&[b"Your pool"], &[]),
        Word::PersonPool => (&[b"Person ", b"'s pool"], &[Number]),
        Word::RecurringBudget => (&[b"Task ", b"'s recurring budget"], &[Number]),
        Word::ProjectBudget => (&[b"Project ", b"'s budget"], &[Number]),
        Word::MaySpend => (&[b"May spend"], &[]),
        Word::MayDelegate => (&[b"May delegate"], &[]),
        Word::DelegateBounds => (&[b"Up to ", b" tasks, ", b" deep"], &[Number, Number]),
        Word::DelegateOne => (&[b"Up to 1 task, ", b" deep"], &[Number]),
        Word::Models => (&[b"Models"], &[]),
        Word::AllModels | Word::AllChats | Word::AllPlan => (&[b"All"], &[]),
        Word::NoModels | Word::NoDeadline => (&[b"None"], &[]),
        Word::AuthorityCut => (&[b"Some of its authority is not shown."], &[]),
        Word::Amount => (&[b"", b""], &[Number]),
        Word::BudgetDeadline => (&[b"Deadline"], &[]),
        Word::BudgetDeadlineDate => (&[b"", b""], &[HoleKind::Date]),
        Word::BudgetDeadlineMoment => (&[b"", b" UTC"], &[HoleKind::Moment]),
        Word::ChangeModel => (&[b"Change model"], &[]),
        Word::UsingModel => (&[b"Using ", b""], &[Name]),
        Word::UsingDefault => (&[b"Using the default model"], &[]),
        Word::UsingKey => (&[b"Using your ", b" API key \xc2\xb7 ", b""], &[Name, Name]),
        Word::UsingPlan => (&[b"Using ", b" plan \xc2\xb7 ", b""], &[Name, Name]),
        Word::UsingProgram => (&[b"Using your ", b" subscription, through ", b" \xc2\xb7 ", b""], &[Name, Name, Name]),
        Word::ModelNeedsAccount => (&[b"", b" \xc2\xb7 needs an account"], &[Name]),
        Word::ModelEffort => (&[b"", b" \xc2\xb7 ", b""], &[Name, Name]),
        Word::WaitRefused => (&[b"Next run refused"], &[]),
        Word::DependencyParked => (&[b"parked"], &[]),
        Word::WaitInvalid => (&[b"The requested account cannot serve this work"], &[]),
        Word::ChoiceDraft => (&[b"Next run if applied: ", b""], &[Name]),
        Word::ChoiceDraftDefault => (&[b"Next run if applied: engine default"], &[]),
        Word::WaitingAccountName => (&[b"Waiting account: ", b""], &[Name]),
        Word::WaitingAccount => (&[b"Waiting account: ", b""], &[Number]),
        Word::NeedsAccount => (&[b"needs an account"], &[]),
        Word::AccountReadinessUnknown => (&[b"Account readiness unknown"], &[]),
        Word::AccountBanner => (&[b"Agents need an LLM account to run."], &[]),
        Word::AccountSignIn => (&[b"Sign in with an LLM provider"], &[]),
        Word::AccountProgram => (&[b"Use a program login"], &[]),
        Word::AccountKey => (&[b"Use an API key"], &[]),
        Word::ChoiceApply => (&[b"Apply to next run"], &[]),
        Word::ChoiceReset => (&[b"Reset choice"], &[]),
        Word::AccountWaiting => (&[b"Starting: waiting for an account"], &[]),
        Word::WaitRefreshing => (&[b"Refreshing account credentials"], &[]),
        Word::WaitHost => (&[b"Waiting for an account on a host"], &[]),
        Word::WaitEmpty => (&[b"No account serving this work"], &[]),
        Word::WaitCredential => (&[b"Waiting for a credential"], &[]),
        Word::WaitCapacity => (&[b"Waiting for a host"], &[]),
        Word::Usage => (&[b"Manage usage"], &[]),
        Word::ChoicePending => (&[b"Next-run choice request in progress"], &[]),
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
        Word::Deadline => (&[b"Due ", b""], &[HoleKind::Date]),
        Word::DeadlineMoment => (&[b"Due ", b" UTC"], &[HoleKind::Moment]),
        Word::Priority => (&[b"Priority ", b""], &[Number]),
        Word::Folded => (&[b"Folded: ", b" ", b""], &[Number, Name]),
        Word::HeldBelow => (&[b"Held below: ", b""], &[Number]),
        Word::ActivityAt => (&[b"", b""], &[HoleKind::Moment]),
        Word::DecisionMade => (&[b"Decision made"], &[]),
        Word::UnreadTurn => (&[b"Turn could not be decoded"], &[]),
        Word::Opaque => (&[b"Opaque content: ", b" bytes"], &[Number]),
        Word::Milestone => (&[b"Milestone ", b""], &[Number]),
        Word::ToolCalls => (&[b"", b" tool calls"], &[Number]),
        Word::ToolSeparator => (&[b" \xc2\xb7 "], &[]),
        Word::ToolRepeated => (&[b"", b" \xc3\x97", b""], &[Name, Number]),
        Word::TimeYesterdayWord => (&[b"yesterday"], &[]),
        Word::TimeJustNow => (&[b"just now"], &[]),
        Word::TimeUnderMinute => (&[b"under 1 min"], &[]),
        Word::TimeMinutes => (&[b"", b" min"], &[Number]),
        Word::TimeHours => (&[b"", b" h"], &[Number]),
        Word::TimeHoursMinutes => (&[b"", b" h ", b" min"], &[Number, Number]),
        Word::TimeDays => (&[b"", b" d"], &[Number]),
        Word::TimeAgo => (&[b"", b" ago"], &[Name]),
        Word::TimeAt => (&[b"at ", b""], &[Name]),
        Word::TimeClock => (&[b"", b":", b""], &[Name, Name]),
        Word::TimePair => (&[b"", b" ", b""], &[Name, Name]),
        Word::TimeDate => (&[b"", b" ", b""], &[Number, Name]),
        Word::TimeDateYear => (&[b"", b" ", b""], &[Name, Number]),
        Word::TimeToday => (&[b"Today \xc2\xb7 ", b""], &[Name]),
        Word::TimeYesterday => (&[b"Yesterday \xc2\xb7 ", b""], &[Name]),
        Word::TimeWaiting => (&[b"Waiting ", b""], &[Name]),
        Word::TimeWaitingChat => (&[b"waiting ", b""], &[Name]),
        Word::TimeEnded => (&[b"Ended ", b""], &[Name]),
        Word::TimeEndedReport => (&[b"Ended ", b" \xc2\xb7 its report, \xe2\x80\x9c", b"\xe2\x80\x9d"], &[Name, Name]),
        Word::TimeFailed => (&[b"Failed ", b" \xc2\xb7 ", b""], &[Name, Name]),
        Word::TimeCancelled => (&[b"Cancelled ", b" \xc2\xb7 ", b""], &[Name, Name]),
        Word::TimeSundayShort => (&[b"Sun"], &[]),
        Word::TimeMondayShort => (&[b"Mon"], &[]),
        Word::TimeTuesdayShort => (&[b"Tue"], &[]),
        Word::TimeWednesdayShort => (&[b"Wed"], &[]),
        Word::TimeThursdayShort => (&[b"Thu"], &[]),
        Word::TimeFridayShort => (&[b"Fri"], &[]),
        Word::TimeSaturdayShort => (&[b"Sat"], &[]),
        Word::TimeSunday => (&[b"Sunday"], &[]),
        Word::TimeMonday => (&[b"Monday"], &[]),
        Word::TimeTuesday => (&[b"Tuesday"], &[]),
        Word::TimeWednesday => (&[b"Wednesday"], &[]),
        Word::TimeThursday => (&[b"Thursday"], &[]),
        Word::TimeFriday => (&[b"Friday"], &[]),
        Word::TimeSaturday => (&[b"Saturday"], &[]),
        Word::TimeJanuary => (&[b"January"], &[]),
        Word::TimeFebruary => (&[b"February"], &[]),
        Word::TimeMarch => (&[b"March"], &[]),
        Word::TimeApril => (&[b"April"], &[]),
        Word::TimeMay => (&[b"May"], &[]),
        Word::TimeJune => (&[b"June"], &[]),
        Word::TimeJuly => (&[b"July"], &[]),
        Word::TimeAugust => (&[b"August"], &[]),
        Word::TimeSeptember => (&[b"September"], &[]),
        Word::TimeOctober => (&[b"October"], &[]),
        Word::TimeNovember => (&[b"November"], &[]),
        Word::TimeDecember => (&[b"December"], &[]),
        Word::ChatEnded => (&[b"Ended"], &[]),
        Word::ChatEndedReport => (&[b"Ended \xc2\xb7 its report, \xe2\x80\x9c", b"\xe2\x80\x9d"], &[Name]),
        Word::ChatFailed => (&[b"Failed \xc2\xb7 ", b""], &[Name]),
        Word::ChatCancelled => (&[b"Cancelled \xc2\xb7 ", b""], &[Name]),
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
        Word::YourChats => (&[b"Your chats"], &[]),
        Word::NoChats => (&[b"No chats yet."], &[]),
        Word::NoLiveChats => (&[b"No live chats."], &[]),
        Word::NoClosedChats => (&[b"No closed chats."], &[]),
        Word::LiveChats | Word::Live => (&[b"Live"], &[]),
        Word::ClosedChats => (&[b"Closed"], &[]),
        Word::MoreClosedChats => (&[b"More closed chats"], &[]),
        Word::Chat | Word::TaskWho => (&[b"T", b""], &[Number]),
        Word::ChatSpend => (&[b"Spent ", b"; delegates spent ", b""], &[Number, Number]),
        Word::ChatDelegates => (&[b"", b" delegates"], &[Number]),
        Word::ChoiceProvider | Word::ChatTime => (&[b"", b""], &[Name]),
        Word::ChatSendHint => (&[b"Enter to send \xc2\xb7 Shift+Enter for newline"], &[]),
        Word::DemoBoth => (&[b"This is a demo, running on a demo model and a demo forge."], &[]),
        Word::DemoForge => (&[b"This is a demo, running on a demo forge."], &[]),
        Word::ConnectForge => (&[b"Connect a forge"], &[]),
        Word::ChatTitle => (&[b"T", b" \xc2\xb7 ", b""], &[Number, Name]),
        Word::DemoModelName => (&[b"Demo model"], &[]),
        Word::DemoUsing => (&[b"Using Demo model \xc2\xb7 ", b""], &[Name]),
        Word::Default => (&[b"Default"], &[]),
        Word::DemoContextProject => (&[b"", b" \xc2\xb7 "], &[Name]),
        Word::DemoRepository => (&[b"", b"/", b""], &[Name, Name]),
        Word::DemoRepositoryForge => (&[b" on Demo forge"], &[]),
        Word::WaitingForYourWords => (&[b"waiting on your words"], &[]),
        Word::SignedDeploymentOwner => (&[b"Signed in as the deployment\xe2\x80\x99s owner"], &[]),
        Word::SignedProjectOwner => (&[b"Signed in as the project\xe2\x80\x99s owner"], &[]),
        Word::SignedName => (&[b"Signed in as ", b""], &[Name]),
        Word::NewChat => (&[b"New chat"], &[]),
        Word::Ready => (&[b"Ready"], &[]),
        Word::Demo => (&[b"Demo"], &[]),
        Word::DemoModel => (&[b"This is a demo, running on a demo model."], &[]),
        Word::ConnectModel => (&[b"Connect a model"], &[]),
        Word::DemoHeading => (&[b"What shall we change?"], &[]),
        Word::DemoRequest => (DEMO_REQUEST, &[Name, Name]),
        Word::SuggestedRequests => (&[b"Suggested requests"], &[]),
        Word::DemoScript => (&[b"The demo model plays these requests from a script."], &[]),
        Word::ChatHint => (&[b"temper \xc2\xb7 Enter to start \xc2\xb7 Shift+Enter for newline"], &[]),
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
        Word::Menu => (&[b"Menu"], &[]),
        Word::MainNavigation => (&[b"Main navigation"], &[]),
        Word::KindFrom => (&[b"", b" from ", b""], &[Name, Name]),
        Word::NoLiveTasks => (&[b"No live tasks."], &[]),
        Word::ItsTasksNoneYet => (&[b"None yet."], &[]),
        Word::ItsTasksNone => (&[b"None."], &[]),
        Word::LivePlan => (&[b"Live only"], &[]),
        Word::PlanCounts => (&[b"Its delegates: ", b" live, ", b" ended."], &[Number, Number]),
        Word::PlanMoreCounts => (&[b"Its delegates: ", b" live, more than ", b" ended."], &[Number, Number]),
        Word::ItsTasksMoreEnded => (&[b"More of its ended tasks are not shown."], &[]),
        Word::MessageOnceRuns => (&[b"Messages reach it once it runs."], &[]),
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
        Word::Task => (&[b"Task ", b""], &[Number]),
        Word::Project => (&[b"Project ", b""], &[Number]),
        Word::Reply => (&[b"Reply"], &[]),

        Word::Inbox => (&[b"Inbox ", b""], &[Number]),
        Word::Chats => (&[b"Chats"], &[]),
        Word::Adoption => (&[b"Adopt a repository"], &[]),
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

        Word::WaitingForTask => (&[b"waiting on task ", b""], &[Number]),
        Word::WaitingForTasks => (&[b"waiting on ", b" tasks"], &[Number]),
        Word::WaitingForMoreTasks => (&[b"waiting on more than ", b" tasks"], &[Number]),
        Word::WaitingForHoldings => (&[b"waiting on its resources"], &[]),
        Word::RowSpent => (&[b"", b" spent"], &[Number]),
        Word::Dependencies => (&[b"Dependencies of task ", b""], &[Number]),
        Word::DependenciesCut => (&[b"More dependencies not shown."], &[]),
        Word::WaitingForDelegates => (&[b"waiting on ", b" delegates"], &[Number]),
        Word::WaitingForYou => (&[b"waiting on an answer from you"], &[]),
        Word::WaitingForPerson => (&[b"waiting on an answer from person ", b""], &[Number]),
        Word::WaitingForRole => (&[b"waiting on an answer from role ", b""], &[Number]),
        Word::WaitingForNews => (&[b"waiting on news"], &[]),
        Word::Running => (&[b"running"], &[]),
        Word::Writing => (&[b"running \xc2\xb7 writing"], &[]),
        Word::RunningTool => (&[b"running \xc2\xb7 a tool \xc2\xb7 ", b""], &[Name]),
        Word::GoalPriority => (&[b"Goal \xc2\xb7 priority ", b""], &[Number]),
        Word::PhaseWaiting => (&[b"waiting"], &[]),
        Word::PhaseIdle => (&[b"waiting for news"], &[]),
        Word::PhaseDue => (&[b"due to run"], &[]),
        Word::PhasePreparing => (&[b"preparing a host"], &[]),
        Word::PhaseClaimed => (&[b"claimed by a host"], &[]),
        Word::PhaseActive => (&[b"executor active"], &[]),
        Word::PhaseRetrying => (&[b"retrying"], &[]),
        Word::PhaseClosing => (&[b"closing"], &[]),
        Word::PhaseHeld => (&[b"held"], &[]),
        Word::TaskBrief => (&[b"Task brief"], &[]),
        Word::RunLive => (&[b"Run ", b" \xc2\xb7 live now"], &[Number]),
        Word::RunWaiting => (&[b"Run ", b" \xc2\xb7 waiting"], &[Number]),
        Word::RunOver => (&[b"Run ", b" \xc2\xb7 ended"], &[Number]),
        Word::NoRun => (&[b"No run now"], &[]),
        Word::RunStarted => (&[b"Started ", b""], &[Name]),
        Word::ItsTasks => (&[b"Its tasks"], &[]),
        Word::SteerNote => (&[b"Steer this task \xc2\xb7 words reach its live run at the next turn."], &[]),
        Word::Amend => (&[b"Amend"], &[]),
        Word::AmendTask => (&[b"Amend task ", b""], &[Number]),
        Word::Specification => (&[b"Specification"], &[]),
        Word::Tracked => (&[b"Tracked"], &[]),
        Word::AmendPriority => (&[b"Priority"], &[]),
        Word::SendAmendment => (&[b"Send amendment"], &[]),
        Word::Discard => (&[b"Discard"], &[]),
        Word::NoChange => (&[b"No change"], &[]),
        Word::SpecTooLong => (&[b"This task's specification is too long to edit here: use `temper ctl amend`"], &[]),
        Word::TaskChanged => (&[b"This task changed since the editor opened"], &[]),
        Word::LookAgain => (&[b"Look again"], &[]),
        Word::StopHolds => (&[b"Its run stops, and the task is held until it is released."], &[]),
        Word::DependencyAddedByYou => (&[b"Added by you"], &[]),
        Word::DependencyAddedByPerson => (&[b"Added by party ", b""], &[Number]),
        Word::DependencyAddedByTask => (&[b"Added by task ", b""], &[Number]),
        Word::DependencyAddedByDeployment => (&[b"Added by the deployment"], &[]),
        Word::DependencyKeeps => (&[b"", b" \xc2\xb7 keeps task ", b" from ending"], &[Name, Number]),
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

/// The four forms of client/README.md, 5.12, against the shell's wall clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeForm {
    Ago,
    Span,
    Clock,
    Day,
}
const MINUTE: u64 = 60_000_000_000;
const DAY: u64 = 86_400_000_000_000;

/// A build accumulates only the times it actually draws.
#[derive(Debug)]
pub(crate) struct Times {
    pub(crate) now: skein_lib::Wall,
    pub(crate) next: Option<skein_lib::Wall>,
}
impl Times {
    pub(crate) fn new(now: skein_lib::Wall) -> Self {
        Self { now, next: None }
    }
    pub(crate) fn say(&mut self, words: &Words, form: TimeForm, at: skein_lib::Wall) -> Box<[u8]> {
        let (text, next) = words.time(form, at, self.now);
        self.next = [self.next, next].into_iter().flatten().min();
        text
    }
}
impl Words {
    /// Words and the instant at which to reconsider them. No clock is read here.
    #[must_use]
    pub fn time(
        &self,
        form: TimeForm,
        at: skein_lib::Wall,
        now: skein_lib::Wall,
    ) -> (Box<[u8]>, Option<skein_lib::Wall>) {
        let elapsed = now.as_nanos().saturating_sub(at.as_nanos());
        let minutes = elapsed / MINUTE;
        let unit = match form {
            TimeForm::Ago if minutes < 720 => Some(MINUTE),
            TimeForm::Span if elapsed < DAY => Some(MINUTE),
            TimeForm::Span => Some(DAY),
            TimeForm::Ago | TimeForm::Clock | TimeForm::Day => None,
        };
        let next = next_tick(at, now, elapsed, unit);
        let said = match form {
            TimeForm::Span => self.time_span(minutes),
            TimeForm::Ago if minutes == 0 => self.say(Word::TimeJustNow, &[]),
            TimeForm::Ago if minutes < 720 => self.say(Word::TimeAgo, &[Hole::Name(self.time_span(minutes))]),
            TimeForm::Ago | TimeForm::Clock | TimeForm::Day => self.calendar_time(form, at, now),
        };
        (said, next)
    }
    fn time_span(&self, minutes: u64) -> Box<[u8]> {
        match minutes {
            0 => self.say(Word::TimeUnderMinute, &[]),
            1..60 => self.say(Word::TimeMinutes, &[Hole::Number(minutes)]),
            60..1440 if minutes.is_multiple_of(60) => self.say(Word::TimeHours, &[Hole::Number(minutes / 60)]),
            60..1440 => self.say(Word::TimeHoursMinutes, &[Hole::Number(minutes / 60), Hole::Number(minutes % 60)]),
            _ => self.say(Word::TimeDays, &[Hole::Number(minutes / 1440)]),
        }
    }
    fn calendar_time(&self, form: TimeForm, at: skein_lib::Wall, now: skein_lib::Wall) -> Box<[u8]> {
        let day = at.as_nanos() / DAY;
        let today = now.as_nanos() / DAY;
        let days = today.checked_sub(day);
        let clock = self.turn_clock(at);
        match form {
            TimeForm::Ago if day == today => self.say(Word::TimeAt, &[Hole::Name(clock)]),
            TimeForm::Ago if days == Some(1) => self.say(Word::TimeYesterdayWord, &[]),
            TimeForm::Ago if days >= Some(2) && days <= Some(7) => self.time_weekday(day, false),
            TimeForm::Ago => self.time_date(at, now),
            TimeForm::Clock if day == today => clock,
            TimeForm::Clock => {
                let date = if days >= Some(1) && days <= Some(6) {
                    self.time_weekday(day, false)
                } else {
                    self.time_date(at, now)
                };
                self.say(Word::TimePair, &[Hole::Name(date), Hole::Name(clock)])
            }
            TimeForm::Day => {
                let date = self.say(
                    Word::TimePair,
                    &[Hole::Name(self.time_weekday(day, true)), Hole::Name(self.time_date(at, now))],
                );
                match days {
                    Some(0) => self.say(Word::TimeToday, &[Hole::Name(date)]),
                    Some(1) => self.say(Word::TimeYesterday, &[Hole::Name(date)]),
                    _ => date,
                }
            }
            TimeForm::Span => unreachable!("spans have no calendar words"),
        }
    }
    /// A turn under a day line has a fixed clock and needs no wake of its own.
    pub(crate) fn turn_clock(&self, at: skein_lib::Wall) -> Box<[u8]> {
        let minute = at.as_nanos() / MINUTE;
        self.say(
            Word::TimeClock,
            &[Hole::Name(copy_of(&two_digits((minute / 60) % 24))), Hole::Name(copy_of(&two_digits(minute % 60)))],
        )
    }
    fn time_weekday(&self, day: u64, full: bool) -> Box<[u8]> {
        let short = [
            Word::TimeSundayShort,
            Word::TimeMondayShort,
            Word::TimeTuesdayShort,
            Word::TimeWednesdayShort,
            Word::TimeThursdayShort,
            Word::TimeFridayShort,
            Word::TimeSaturdayShort,
        ];
        let long = [
            Word::TimeSunday,
            Word::TimeMonday,
            Word::TimeTuesday,
            Word::TimeWednesday,
            Word::TimeThursday,
            Word::TimeFriday,
            Word::TimeSaturday,
        ];
        let index = usize::try_from(day.checked_add(4).expect("epoch day fits") % 7).expect("weekday fits");
        self.say(*if full { long.get(index) } else { short.get(index) }.expect("weekday in week"), &[])
    }
    fn time_date(&self, at: skein_lib::Wall, now: skein_lib::Wall) -> Box<[u8]> {
        let (year, month, day) = civil_date(at);
        let months = [
            Word::TimeJanuary,
            Word::TimeFebruary,
            Word::TimeMarch,
            Word::TimeApril,
            Word::TimeMay,
            Word::TimeJune,
            Word::TimeJuly,
            Word::TimeAugust,
            Word::TimeSeptember,
            Word::TimeOctober,
            Word::TimeNovember,
            Word::TimeDecember,
        ];
        let month = months
            .get(usize::try_from(month.checked_sub(1).expect("months start at one")).expect("month fits"))
            .expect("month in year");
        let date = self.say(Word::TimeDate, &[Hole::Number(day), Hole::Name(self.say(*month, &[]))]);
        if year == civil_date(now).0 {
            date
        } else {
            self.say(Word::TimeDateYear, &[Hole::Name(date), Hole::Number(year)])
        }
    }
}
#[expect(clippy::arithmetic_side_effects, reason = "each input is below 100; each output is a decimal digit")]
fn two_digits(value: u64) -> [u8; 2] {
    let value = u8::try_from(value).expect("clock field fits");
    [b'0' + value / 10, b'0' + value % 10]
}
// Gregorian civil date; u64 nanoseconds bound the year to 2554.
#[expect(clippy::arithmetic_side_effects, reason = "epoch days and Gregorian era intermediates fit u64")]
fn civil_date(at: skein_lib::Wall) -> (u64, u64, u64) {
    let days = at.as_nanos() / DAY + 719_468;
    let era = days / 146_097;
    let doe = days % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (era * 400 + yoe + u64::from(month <= 2), month, day)
}

fn next_tick(at: skein_lib::Wall, now: skein_lib::Wall, elapsed: u64, unit: Option<u64>) -> Option<skein_lib::Wall> {
    let nanos = match unit {
        Some(unit) => {
            let delta = elapsed.checked_div(unit)?.checked_add(1)?.checked_mul(unit)?;
            at.as_nanos().checked_add(delta)?
        }
        None => (now.as_nanos() / DAY).checked_add(1)?.checked_mul(DAY)?,
    };
    Some(skein_lib::Wall::from_nanos(nanos))
}
