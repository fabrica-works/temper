//! Typed fields retained from Forgejo's answers.
use super::id::ObjectFormat;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Body {
    Kept(Box<[u8]>),
    Long(u64),
}
impl Body {
    /// Reads the tokenizer's capped body string, retaining a long body's length alone.
    pub fn read(event: &skein_json::tokenizer::Event, field: &'static [u8]) -> Result<Self, crate::decode::Malformed> {
        match event {
            skein_json::tokenizer::Event::Token(skein_json::Token::String(bytes)) => Ok(Self::Kept(bytes.clone())),
            skein_json::tokenizer::Event::Long(length) => Ok(Self::Long(*length)),
            skein_json::tokenizer::Event::Failed(error) => Err(crate::decode::Malformed::Json(*error)),
            skein_json::tokenizer::Event::Token(
                skein_json::Token::ObjectStart
                | skein_json::Token::ObjectEnd
                | skein_json::Token::ArrayStart
                | skein_json::Token::ArrayEnd
                | skein_json::Token::Key(_)
                | skein_json::Token::Number(_)
                | skein_json::Token::True
                | skein_json::Token::False
                | skein_json::Token::Null,
            )
            | skein_json::tokenizer::Event::Skipped(_)
            | skein_json::tokenizer::Event::Done
            | skein_json::tokenizer::Event::Closed => Err(crate::decode::Malformed::Shape { field }),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub id: u64,
    pub login: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ApiSettings {
    pub max_response_items: u32,
    pub default_paging_num: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MergeStyle {
    Merge,
    Rebase,
    RebaseMerge,
    Squash,
    FastForwardOnly,
    ManuallyMerged,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub id: u64,
    pub owner: User,
    pub name: Box<[u8]>,
    pub object_format: ObjectFormat,
    pub default_branch: Box<[u8]>,
    pub default_merge_style: MergeStyle,
    pub allow_merge_commits: bool,
    pub allow_squash_merge: bool,
    pub allow_rebase: bool,
    pub html_url: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Row {
    User(User),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Document {
    User(User),
    ApiSettings(ApiSettings),
    Repository(Repository),
    Search { ok: bool },
}
pub(crate) fn user() -> User {
    User { id: 0, login: Box::from([]) }
}
pub(crate) fn repository() -> Repository {
    Repository {
        id: 0,
        owner: user(),
        name: Box::from([]),
        object_format: ObjectFormat::Sha1,
        default_branch: Box::from([]),
        default_merge_style: MergeStyle::Merge,
        allow_merge_commits: false,
        allow_squash_merge: false,
        allow_rebase: false,
        html_url: Box::from([]),
    }
}
pub(crate) fn style(text: &[u8]) -> Option<MergeStyle> {
    match text {
        b"merge" => Some(MergeStyle::Merge),
        b"rebase" => Some(MergeStyle::Rebase),
        b"rebase-merge" => Some(MergeStyle::RebaseMerge),
        b"squash" => Some(MergeStyle::Squash),
        b"fast-forward-only" => Some(MergeStyle::FastForwardOnly),
        b"manually-merged" => Some(MergeStyle::ManuallyMerged),
        _ => None,
    }
}
pub(crate) fn style_text(style: MergeStyle) -> &'static [u8] {
    match style {
        MergeStyle::Merge => b"merge",
        MergeStyle::Rebase => b"rebase",
        MergeStyle::RebaseMerge => b"rebase-merge",
        MergeStyle::Squash => b"squash",
        MergeStyle::FastForwardOnly => b"fast-forward-only",
        MergeStyle::ManuallyMerged => b"manually-merged",
    }
}
