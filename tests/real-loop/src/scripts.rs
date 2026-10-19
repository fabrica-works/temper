//! The same chat script as the one-process world, on the real transport.
use skein_fake_llm_domain::api;

pub const WORDS: &str = "Hello";
pub const REPLY: &[u8] = b"World answer";
pub const EARLY: &[u8] = b"World";

#[must_use]
pub fn chat_scripts(chats: usize) -> Box<[api::Script]> {
    (0..chats)
        .map(|_| {
            let mut script = jig_channel_world::fixtures::one_chat(WORDS.as_bytes());
            script.result = br#"{"form":"report","text":"World answer","fields":{}}"#.as_slice().into();
            api::Script { cue: Box::default(), turns: jig_channel_world::fixtures::provider_turns(&script) }
        })
        .collect()
}
