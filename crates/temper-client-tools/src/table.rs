//! The tools available to this host (client/tools.md, 5).
use crate::jig::{
    argv::{self, Usage},
    call::Parsed,
    rows,
    table::Tool,
};
pub const TOOLS: &[&Tool] = &[
    &rows::ME,
    &rows::INBOX,
    &rows::TASKS,
    &rows::TASK,
    &rows::CHAT,
    &rows::ACTIVITY,
    &rows::TURN,
    &rows::RUN,
    &rows::CHAT_CREATE,
    &rows::MESSAGE,
    &rows::CLOSE,
    &rows::ANSWER,
    &rows::DECIDE,
    &rows::RELEASE,
    &rows::STOP,
    &rows::CANCEL,
    &rows::WAIT,
    &crate::adopt::ROW,
    &rows::SIGN_IN,
    &rows::SIGN_OUT,
    &rows::PENDING,
    &rows::HELP,
    &rows::ACCOUNTS,
    &rows::MODELS,
    &rows::CONNECT,
    &rows::DISCONNECT,
    &rows::ADD_KEY,
    &rows::REPLACE_KEY,
    &rows::FORGET,
    &rows::RANK,
    &rows::RETRY,
    &rows::AMEND,
];
pub fn parse(words: &[&[u8]]) -> Result<Parsed, Usage> {
    argv::parse(TOOLS, words, &crate::limits::tools())
}
