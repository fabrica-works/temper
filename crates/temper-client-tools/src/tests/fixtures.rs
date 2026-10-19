use crate::jig::key::Name;
use skein_lib::List;
use std::path::{Path, PathBuf};
/// Source files under the extraction boundary; capacity is a source-tree bound.
const SOURCE_FILES: u32 = 128;
pub(super) fn name(bytes: &[u8]) -> Name {
    Name::new(bytes).expect("a fixture key name")
}
#[expect(clippy::disallowed_macros, reason = "the boundary fixture reads this crate's source tree")]
pub(super) fn sources() -> List<(PathBuf, Box<str>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = List::with_capacity(SOURCE_FILES);
    let mut sources = List::with_capacity(SOURCE_FILES);
    paths.push(root.join("src/jig.rs")).expect("module root");
    paths.push(root.join("src/jig")).expect("module directory");
    for index in 0..SOURCE_FILES {
        let Some(path) = paths.get(index).cloned() else {
            break;
        };
        if path.is_dir() {
            for entry in std::fs::read_dir(path).expect("read source directory") {
                paths.push(entry.expect("source entry").path()).expect("bounded sources");
            }
        } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
            let text = std::fs::read_to_string(&path).expect("read source").into_boxed_str();
            sources.push((path, text)).expect("bounded sources");
        }
    }
    sources
}
/// Enough words for every published call, including repeated checks and moves.
const WORDS: u32 = 128;
pub(super) fn parse(words: &[&str]) -> Result<crate::jig::call::Parsed, crate::jig::argv::Usage> {
    let mut bytes = List::with_capacity(WORDS);
    for word in words {
        bytes.push(word.as_bytes()).expect("bounded words");
    }
    crate::table::parse(bytes.as_slice())
}
pub(super) fn call(words: &[&str]) -> crate::jig::call::Call {
    let crate::jig::call::Parsed::Call { call, json: false } = parse(words).expect("fixture argv") else {
        panic!("a plain call");
    };
    call
}
pub(super) fn back(call: &crate::jig::call::Call) -> Box<[Box<[u8]>]> {
    let mut words = List::with_capacity(WORDS);
    crate::jig::argv::back(crate::table::TOOLS, call, &mut words).expect("a call with argv");
    words.into_boxed()
}
pub(super) fn owned(words: &[&str]) -> Box<[Box<[u8]>]> {
    let mut owned = List::with_capacity(WORDS);
    for word in words {
        owned.push(Box::from(word.as_bytes())).expect("bounded words");
    }
    owned.into_boxed()
}
/// One argv per table row and form; the help derivation shares these examples.
pub(super) const EVERY_TOOL: &[&[&str]] = &[
    &["me"],
    &["inbox"],
    &["tasks", "--project", "shop"],
    &["task", "41"],
    &["chat", "41"],
    &["activity", "41"],
    &["turn", "41", "3.7"],
    &["run", "41"],
    &["sign-in", "-"],
    &["sign-out"],
    &["pending"],
    &["help"],
];
