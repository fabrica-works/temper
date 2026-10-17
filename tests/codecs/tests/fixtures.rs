use std::boxed::Box;
use temper_forge_forgejo::document::{AccessToken, User};
pub const BOT: u64 = 4;
pub const PERSON: u64 = 5;
pub const API_TOKEN: u64 = 6;
pub const GIT_TOKEN: u64 = 7;
#[must_use]
pub fn user(id: u64, login: &[u8], is_admin: bool, full_name: &[u8]) -> User {
    User { id, login: Box::from(login), is_admin, full_name: Box::from(full_name) }
}
#[must_use]
pub fn api_token(value: bool) -> AccessToken {
    AccessToken {
        id: API_TOKEN,
        name: Box::from(*b"temper-api"),
        scopes: Box::new([Box::from(*b"write:issue"), Box::from(*b"write:repository"), Box::from(*b"read:user")]),
        last_eight: Box::from(*b"00000003"),
        value: if value {
            Some(skein_lib::Secret::new(Box::from(*b"0000000000000000000000000000000000000004")))
        } else {
            None
        },
    }
}
#[must_use]
pub fn git_token(value: bool) -> AccessToken {
    let mut token = api_token(value);
    token.id = GIT_TOKEN;
    token.name = Box::from(*b"temper-git");
    token.scopes = Box::new([Box::from(*b"write:repository"), Box::from(*b"read:user")]);
    token.last_eight = Box::from(*b"00000005");
    token.value = if value {
        Some(skein_lib::Secret::new(Box::from(*b"0000000000000000000000000000000000000006")))
    } else {
        None
    };
    token
}
