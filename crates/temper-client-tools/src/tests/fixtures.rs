use crate::jig::key::Name;
use skein_lib::List;
use std::path::PathBuf;
/// Source files under the extraction boundary; capacity is a source-tree bound.
const SOURCE_FILES: u32 = 128;
pub(super) fn name(bytes: &[u8]) -> Name {
    Name::new(bytes).expect("a fixture key name")
}
pub(super) fn sources() -> List<(PathBuf, Box<str>)> {
    let mut paths = List::with_capacity(SOURCE_FILES);
    let mut sources = List::with_capacity(SOURCE_FILES);
    paths.push(PathBuf::from("src/jig.rs")).expect("module root");
    paths.push(PathBuf::from("src/jig")).expect("module directory");
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
    &["chat", "hello", "--project", "shop", "--key", "start-chat"],
    &["message", "41", "hello", "--key", "say-hello"],
    &["close", "41", "--key", "close-chat"],
    &["answer", "41", "3", "yes"],
    &["decide", "t41/3", "accept"],
    &["decide", "41", "release", "--hold", "2"],
    &["release", "41", "--hold", "2"],
    &["stop", "41"],
    &["cancel", "41", "finished"],
    &["wait", "me"],
    ADOPTION,
    &["sign-in", "-"],
    &["sign-out"],
    &["pending"],
    &["help"],
];

/// Parse the printer's bytes back without re-creating argv in each test.
pub(super) fn printed(call: &crate::jig::call::Call) -> crate::jig::call::Parsed {
    let words = back(call);
    let mut borrowed = List::with_capacity(WORDS);
    for word in &words {
        borrowed.push(word.as_ref()).expect("bounded words");
    }
    crate::table::parse(borrowed.as_slice()).expect("printed argv")
}
/// The common adoption form; optional flags stay absent.
pub(super) fn adoption() -> crate::jig::call::Call {
    call(ADOPTION)
}

/// Required adoption fields and its named key, shared by form tests.
pub(super) const ADOPTION: &[&str] =
    &["adopt", "--project", "shop", "--owner", "o", "--name", "n", "--landing", "main", "--key", "adopt-repo"];

pub(super) fn from_json(bytes: &[u8]) -> crate::jig::call::Call {
    json(bytes).expect("fixture JSON call")
}
pub(super) fn json(bytes: &[u8]) -> Result<crate::jig::call::Call, crate::jig::argv::Usage> {
    crate::jig::json::call(
        crate::table::TOOLS,
        bytes,
        crate::jig::call::Source::Path(Box::from(b"call.json".as_slice())),
        &crate::limits::tools(),
    )
}
/// Shared JSON wait, deliberately putting the condition before the subject.
pub(super) fn json_wait(subject: &str, until: Option<&str>) -> Box<[u8]> {
    let limits = skein_json::writer::Limits { depth: 2, length: crate::limits::TEXT };
    let mut measured = skein_json::writer::Encoder::measure(&limits);
    encode_wait(&mut measured, subject, until);
    let length = measured.measured().expect("fixture JSON size");
    let mut output = skein_json::writer::Encoder::write(length, &limits);
    encode_wait(&mut output, subject, until);
    output.finish()
}
fn encode_wait(out: &mut skein_json::writer::Encoder, subject: &str, until: Option<&str>) {
    out.object_start();
    out.key(b"tool");
    out.string(b"wait");
    out.key(b"arguments");
    out.object_start();
    if let Some(until) = until {
        out.key(b"until");
        out.string(until.as_bytes());
    }
    out.key(b"since");
    out.string(b"0011223344556677");
    out.key(b"subject");
    out.string(subject.as_bytes());
    out.object_end();
    out.object_end();
}
pub(super) fn wait_call(subject: &str, until: Option<&str>) -> crate::jig::call::Call {
    let mut words = List::with_capacity(WORDS);
    words.push(b"wait".as_slice()).expect("bounded words");
    for word in subject.split_ascii_whitespace() {
        words.push(word.as_bytes()).expect("bounded words");
    }
    words.push(b"--since".as_slice()).expect("bounded words");
    words.push(b"0011223344556677".as_slice()).expect("bounded words");
    if let Some(until) = until {
        words.push(b"--until".as_slice()).expect("bounded words");
        words.push(until.as_bytes()).expect("bounded words");
    }
    let crate::jig::call::Parsed::Call { call, json: false } =
        crate::table::parse(words.as_slice()).expect("wait argv")
    else {
        panic!("a plain wait call");
    };
    call
}
