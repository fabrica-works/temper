//! Browser sign-in ways supplied by the page (protocol/sign-in.md, 7).
use crate::words::{Shell, Word, Words};
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_ui::{Builder, Role};

/// The longest page origin a sign-in link is written from.
pub const ORIGIN_BYTES: usize = 260;
/// Known ways only, and the page's origin their links leave from; no other page-owned text is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ways {
    pub google: bool,
    /// The page's `http` or `https` origin, as `location.origin` gives it.
    pub origin: Option<Box<[u8]>>,
}
impl Ways {
    pub const NONE: Self = Self { google: false, origin: None };
    /// The meta's ways, offered only from an `http` or `https` origin within `ORIGIN_BYTES`.
    #[must_use]
    pub fn parse(content: &str, origin: &[u8]) -> Self {
        let mut google = false;
        for way in content.split_ascii_whitespace() {
            if way == "google" {
                google = true;
            }
        }
        let web = origin.starts_with(b"https://") || origin.starts_with(b"http://");
        if !web || origin.len() > ORIGIN_BYTES {
            return Self::NONE;
        }
        Self { google, origin: Some(Box::from(origin)) }
    }
}
pub(crate) fn offered(root: &temper_client_domain::Domain, words: &Words, ways: &Ways) -> bool {
    words.shell == Shell::Browser
        && ways.google
        && ways.origin.is_some()
        && root.client().link().state == jig_client::Link::SignedOut
        && root.client().link().not_signed_in.is_none()
}
/// Unreserved bytes and `/` stay as they are in the return path.
fn kept(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte)
}
fn google(origin: &[u8], path: &[u8]) -> Box<[u8]> {
    const PREFIX: &[u8] = b"/sign-in/google?then=";
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut size = origin.len().checked_add(PREFIX.len()).expect("bounded URL");
    for byte in path {
        size = size.checked_add(if kept(*byte) { 1 } else { 3 }).expect("bounded URL");
    }
    let mut out = Writer::new(size);
    out.put(origin).expect("sized URL");
    out.put(PREFIX).expect("sized URL");
    for byte in path {
        if kept(*byte) {
            out.put(&[*byte]).expect("sized URL");
        } else {
            out.put(&[
                b'%',
                *HEX.get(usize::from(byte >> 4_u8)).expect("high nibble"),
                *HEX.get(usize::from(byte & 15_u8)).expect("low nibble"),
            ])
            .expect("sized URL");
        }
    }
    out.finish()
}
pub(crate) fn main(b: &mut Builder, root: &temper_client_domain::Domain, words: &Words, origin: &[u8]) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    b.style(crate::look::SIGN_IN);
    b.open(Role::Slot);
    b.style(crate::look::CARD);
    b.open(Role::Paragraph);
    b.style(crate::look::BRAND);
    words.text(b, Word::Brand, &[]);
    b.close();
    b.open(Role::Heading { level: 1 });
    let heading = words.say(Word::SignInTitle, &[]);
    b.name(&heading);
    words.text(b, Word::SignInTitle, &[]);
    b.close();
    b.open(Role::Link);
    b.name(&words.say(Word::ContinueGoogle, &[]));
    b.external(&google(origin, &temper_client_domain::address::print(root.pages().address())));
    words.text(b, Word::ContinueGoogle, &[]);
    b.close();
    b.close();
    b.close();
}

#[cfg(test)]
mod tests {
    #[test]
    fn return_paths_keep_only_unreserved_bytes_and_slashes() {
        for (path, expected) in [
            (b"/a-b._~/".as_slice(), b"https://t.example/sign-in/google?then=/a-b._~/".as_slice()),
            (
                b"/a b/%?q=\xc3\xa9".as_slice(),
                b"https://t.example/sign-in/google?then=/a%20b/%25%3Fq%3D%C3%A9".as_slice(),
            ),
        ] {
            assert_eq!(super::google(b"https://t.example", path).as_ref(), expected);
        }
    }
}
