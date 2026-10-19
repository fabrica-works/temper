//! Classification uses the tools' single argument table.
use temper_client_tools::{
    jig::{
        argv::Usage,
        call::{Call, Parsed},
        rows,
        table::Keyed,
    },
    table::{self, TOOLS},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Act,
    Read,
    Wait,
    SignIn,
    SignOut,
    Pending,
    Help,
}
impl Class {
    #[must_use]
    pub const fn locks(self) -> bool {
        matches!(self, Self::Act | Self::SignIn | Self::SignOut)
    }
}
/// Parse the words after DIRECTORY through the tools' own table.
/// # Errors
/// Returns the table's usage error unchanged.
pub fn parse(words: &[&[u8]]) -> Result<Parsed, Usage> {
    table::parse(words)
}
#[must_use]
pub fn class(call: &Call) -> Class {
    let tool = TOOLS.get(usize::from(call.tool)).expect("parsed tool");
    if tool.keyed != Keyed::Never {
        Class::Act
    } else if tool.name == rows::SIGN_IN.name {
        Class::SignIn
    } else if tool.name == rows::SIGN_OUT.name {
        Class::SignOut
    } else if tool.name == rows::PENDING.name {
        Class::Pending
    } else if tool.name == rows::HELP.name {
        Class::Help
    } else if tool.name == rows::WAIT.name {
        Class::Wait
    } else {
        Class::Read
    }
}
