//! A committed chat reply followed by a parked run on the real transport.
use skein_fake_llm_domain::api;

pub const WORDS: &str = "Hello";
pub const REPLY: &[u8] = b"World answer";
pub const EARLY: &[u8] = b"World";

#[must_use]
pub fn chat_scripts(chats: usize) -> Box<[api::Script]> {
    (0..chats)
        .map(|_| api::Script {
            cue: Box::default(),
            turns: Box::new([api::Turn {
                lines: Box::new([
                    api::Line::Text { text: REPLY.into() },
                    api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            }]),
        })
        .collect()
}
