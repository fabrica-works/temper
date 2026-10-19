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
        switch(b"ended", b"Include ended tasks"),
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
    line: b"End sign-in and erase the token",
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
