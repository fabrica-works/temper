//! The read and host-tool records, in their published order.
use super::table::{
    Keyed, Kind,
    Occurs::{Optional, Required},
    Tool, flag, positional, switch,
};
pub const ME: Tool = Tool {
    name: b"me",
    line: b"Read the party, projects and waiting counts",
    result: b"me",
    forms: &[&[]],
    keyed: Keyed::Never,
    area: 1,
};
pub const INBOX: Tool = Tool {
    name: b"inbox",
    line: b"Read what needs the party and its updates",
    result: b"inbox",
    forms: &[&[
        flag(b"project", b"P", Kind::Project, Optional, b"Project name or number"),
        flag(b"kind", b"K", Kind::Group, Optional, b"Inbox group"),
        flag(b"root", b"N", Kind::Number, Optional, b"Root task"),
        flag(b"after", b"C", Kind::Cursor, Optional, b"Continue after this cursor"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const TASKS: Tool = Tool {
    name: b"tasks",
    line: b"Read task lines",
    result: b"tasks",
    forms: &[&[
        flag(b"project", b"P", Kind::Project, Required, b"Project name or number"),
        flag(b"label", b"L", Kind::Word { set: &[b"chat", b"goal"] }, Optional, b"Task label"),
        switch(b"tracked", b"Only tracked tasks"),
        switch(b"mine", b"Only my tasks"),
        switch(b"ended", b"Only ended tasks"),
        flag(b"order", b"O", Kind::Order, Optional, b"activity or priority; number by default"),
        flag(b"after", b"C", Kind::Cursor, Optional, b"Continue after this cursor"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const TASK: Tool = Tool {
    name: b"task",
    line: b"Read a task and selected related documents",
    result: b"task",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        switch(b"tree", b"Include the live tree"),
        flag(b"activity", b"K", Kind::Count, Optional, b"Latest activity lines"),
        switch(b"run", b"Include the live run"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const CHAT: Tool = Tool {
    name: b"chat",
    line: b"Read a conversation from its end",
    result: b"chat",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        flag(b"before", b"C", Kind::Cursor, Optional, b"Continue before this cursor"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const ACTIVITY: Tool = Tool {
    name: b"activity",
    line: b"Read task activity from its end",
    result: b"activity",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        flag(b"before", b"C", Kind::Cursor, Optional, b"Continue before this cursor"),
        flag(b"limit", b"K", Kind::Count, Optional, b"Most activity lines"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const TURN: Tool = Tool {
    name: b"turn",
    line: b"Read one turn and its parts",
    result: b"turn",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        positional(b"turn", b"A.T", Kind::Turn, Required, b"Attempt and turn"),
        flag(b"part", b"K", Kind::Number, Optional, b"Part number"),
        flag(b"from", b"B", Kind::Number, Optional, b"Byte offset"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
pub const RUN: Tool = Tool {
    name: b"run",
    line: b"Read a task's live run",
    result: b"run",
    forms: &[&[positional(b"task", b"N", Kind::Number, Required, b"Task number")]],
    keyed: Keyed::Never,
    area: 1,
};
pub const SIGN_IN: Tool = Tool {
    name: b"sign-in",
    line: b"Sign in with the link the engine's host printed",
    result: b"sign_in",
    forms: &[&[positional(b"file", b"FILE|-", Kind::File, Required, b"File holding the sign-in link")]],
    keyed: Keyed::Never,
    area: 1,
};
pub const SIGN_OUT: Tool = Tool {
    name: b"sign-out",
    line: b"End sign-in and leave the token empty",
    result: b"sign_out",
    forms: &[&[]],
    keyed: Keyed::Never,
    area: 1,
};
pub const PENDING: Tool = Tool {
    name: b"pending",
    line: b"List requests kept in the store",
    result: b"pending",
    forms: &[&[]],
    keyed: Keyed::Never,
    area: 1,
};
pub const HELP: Tool = Tool {
    name: b"help",
    line: b"Read the guide or one tool's record",
    result: b"help",
    forms: &[&[positional(b"tool", b"TOOL", Kind::Tool, Optional, b"Tool to describe")]],
    keyed: Keyed::Never,
    area: 1,
};
pub const CHAT_CREATE: Tool = Tool {
    name: b"chat",
    line: b"Start a chat",
    result: b"chat_create",
    forms: &[&[
        flag(b"project", b"P", Kind::Project, Required, b"Project name or number"),
        positional(b"text", b"TEXT", Kind::Text, Required, b"Opening words"),
    ]],
    keyed: Keyed::Required,
    area: 1,
};
pub const MESSAGE: Tool = Tool {
    name: b"message",
    line: b"Send words to a task",
    result: b"message",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        positional(b"text", b"TEXT", Kind::Text, Required, b"Words to send"),
    ]],
    keyed: Keyed::Required,
    area: 1,
};
pub const CLOSE: Tool = Tool {
    name: b"close",
    line: b"Close a chat and cancel or move its delegates",
    result: b"close",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        super::table::repeated(b"move", b"D", Kind::Number, 8, b"Delegate to move to the party"),
        flag(b"reason", b"TEXT", Kind::Text, Optional, b"Reason, empty by default"),
    ]],
    keyed: Keyed::Required,
    area: 1,
};
pub const ANSWER: Tool = Tool {
    name: b"answer",
    line: b"Answer a task's question",
    result: b"answer",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        positional(b"question", b"M", Kind::Number, Required, b"Question number"),
        positional(b"text", b"TEXT", Kind::Text, Required, b"Answer"),
    ]],
    keyed: Keyed::Optional,
    area: 1,
};
pub const DECIDE: Tool = Tool {
    name: b"decide",
    line: b"Decide a proposal or an escalation",
    result: b"decide",
    forms: &[
        &[
            positional(b"proposal", b"P/K", Kind::Proposal, Required, b"Typed proposer and proposal number"),
            positional(
                b"outcome",
                b"OUTCOME",
                Kind::Word { set: &[b"accept", b"reject", b"pass-up"] },
                Required,
                b"Decision",
            ),
            positional(b"reason", b"REASON", Kind::Text, Optional, b"Reason"),
        ],
        &[
            positional(b"task", b"N", Kind::Number, Required, b"Task number"),
            flag(b"hold", b"R", Kind::Number, Required, b"Hold revision"),
            positional(
                b"outcome",
                b"OUTCOME",
                Kind::Word { set: &[b"release", b"leave", b"pass-up"] },
                Required,
                b"Decision",
            ),
            positional(b"reason", b"REASON", Kind::Text, Optional, b"Reason"),
        ],
    ],
    keyed: Keyed::Optional,
    area: 1,
};
pub const RELEASE: Tool = Tool {
    name: b"release",
    line: b"Lift a task's hold",
    result: b"release",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        flag(b"hold", b"R", Kind::Number, Required, b"Hold revision"),
        switch(b"again", b"Run again"),
    ]],
    keyed: Keyed::Optional,
    area: 1,
};
pub const STOP: Tool = Tool {
    name: b"stop",
    line: b"Stop a task's run and hold it",
    result: b"stop",
    forms: &[&[positional(b"task", b"N", Kind::Number, Required, b"Task number")]],
    keyed: Keyed::Optional,
    area: 1,
};
pub const CANCEL: Tool = Tool {
    name: b"cancel",
    line: b"Cancel a task and its tree",
    result: b"cancel",
    forms: &[&[
        positional(b"task", b"N", Kind::Number, Required, b"Task number"),
        flag(b"grace", b"D", Kind::Duration, Optional, b"Grace before cancellation"),
        positional(b"reason", b"REASON", Kind::Text, Required, b"Reason"),
    ]],
    keyed: Keyed::Optional,
    area: 1,
};
pub const WAIT: Tool = Tool {
    name: b"wait",
    line: b"Wait for a watch's change or condition",
    result: b"wait",
    forms: &[&[
        positional(b"subject", b"SUBJECT", Kind::Subject, Required, b"Watch subject"),
        flag(b"since", b"DIGEST", Kind::Digest, Optional, b"Wait for a different digest"),
        flag(b"until", b"CONDITION", Kind::Condition, Optional, b"Wait for a condition"),
        flag(b"timeout", b"DURATION", Kind::Duration, Optional, b"Timeout, 300 seconds by default"),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
