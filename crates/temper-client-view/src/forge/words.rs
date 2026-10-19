//! UI rendering of the domain's adoption phrases.
use crate::words::Words;
use alloc::boxed::Box;
use skein_ui::{Builder, Run, phrases::Hole};
use temper_client_domain::forge::words::{self, Word};
#[must_use]
pub fn say(words: &Words, word: Word, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, words::id(words.connector, word), holes).expect("checked adoption phrase").text
}
pub fn text(words: &Words, builder: &mut Builder, word: Word, holes: &[Hole]) {
    builder.text(Run::Plain, &say(words, word, holes));
}
#[must_use]
pub fn capacity(words: &Words) -> Option<u32> {
    let mut most = 0;
    for (word, _, _) in words::ROWS {
        most = most.max(words.book.capacity(words::id(words.connector, word))?);
    }
    for word in words::RESOURCES {
        most = most.max(words.book.capacity(word.id(words.connector))?);
    }
    Some(most)
}
