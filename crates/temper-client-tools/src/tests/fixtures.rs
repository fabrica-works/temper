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
