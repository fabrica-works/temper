//! The browser's bundle, read whole at start, and its closed list of types (client/browser.md, 2.2; skein's
//! ui-web.md, 4.3 and 4.4). Copied from jig's testing/jig-test-service/src/server/bundle.rs.
use alloc::boxed::Box;
use skein_lib::Map;

/// A bundle path's bound (jig's testing/jig-test/src/bundle.rs).
pub const NAME_BYTES: u32 = 1024;
/// The digest directory's name: sixteen lowercase hex digits.
const DIGEST_BYTES: u64 = 16;
/// The page's `ETag`: a quoted FNV-1a of sixteen hex digits.
const ETAG_BYTES: u64 = 18;

/// One digest-directory file, typed by the closed list (skein's ui-web.md, 4.4).
#[derive(Debug)]
pub struct File {
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
}
/// The bundle read whole at start (client/browser.md, 2.2).
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}
/// A path's type by its extension, from the closed list; `None` off it.
#[must_use]
pub fn kind(path: &[u8]) -> Option<&'static [u8]> {
    let mut dot = None;
    for (at, byte) in path.iter().enumerate() {
        if *byte == b'.' {
            dot = Some(at);
        }
        if *byte == b'/' {
            dot = None;
        }
    }
    let dot = dot?;
    match path.get(dot.checked_add(1)?..)? {
        b"html" => Some(b"text/html; charset=utf-8"),
        b"js" => Some(b"text/javascript"),
        b"wasm" => Some(b"application/wasm"),
        b"css" => Some(b"text/css"),
        b"svg" => Some(b"image/svg+xml"),
        b"png" => Some(b"image/png"),
        b"webp" => Some(b"image/webp"),
        b"ico" => Some(b"image/x-icon"),
        b"woff2" => Some(b"font/woff2"),
        b"txt" => Some(b"text/plain; charset=utf-8"),
        b"webmanifest" => Some(b"application/manifest+json"),
        _ => None,
    }
}
/// The bundle's bytes at its limits: its files (the page among them), their names, the digest and the `ETag`.
#[must_use]
pub fn worst_case(limits: &crate::limits::Clients) -> Option<u64> {
    u64::from(limits.bundle_bytes)
        .checked_add(Map::<Box<[u8]>, File>::worst_case(limits.bundle_files)?)?
        .checked_add(u64::from(limits.bundle_files).checked_mul(u64::from(NAME_BYTES))?)?
        .checked_add(DIGEST_BYTES)?
        .checked_add(ETAG_BYTES)?
        .checked_add(u64::try_from(size_of::<Bundle>()).ok()?)
}
