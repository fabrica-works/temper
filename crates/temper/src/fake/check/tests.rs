use super::*;
use crate::directories::tests::scratch;
use std::fs;

fn store_limits() -> jig_protocol_store::Limits {
    let mut limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).store;
    // Skein's real KV fixture, with chunk >= key + value + 20, and room
    // for the format and the eight rows of the tag-filter case.
    limits.store = skein_kv::Limits {
        key: 64,
        value: 64,
        ops: 8,
        commit: 1024,
        queued: 8,
        queued_bytes: 8192,
        budget: 8192,
        segment: 4096,
        snapshot_after: 4096,
        chunk: 256,
        page: skein_kv::Page { rows: 16, bytes: 1024 },
        deadline: skein_lib::Duration::from_secs(5),
    };
    limits
}
fn rows(keys: &[u8], tag: Option<u8>) -> Rows {
    keys.iter()
        .map(|key| {
            let key = match tag {
                Some(tag) => Box::from([tag, *key]),
                None => Box::from([*key]),
            };
            (key, Box::from(b"value".as_slice()))
        })
        .collect()
}
fn seed(path: &Path, limits: &jig_protocol_store::Limits, format: Option<Format>, rows: Rows) {
    fs::create_dir(path).expect("store directory");
    let mut rows = rows.into_vec();
    if let Some(format) = format {
        rows.push((
            keys::encode_own(&keys::Own::Format, limits.store.key).expect("format key"),
            format.encode().expect("format"),
        ));
    }
    let mut host = StoreHost::new(limits.store);
    host.seed = Some(rows.into_boxed_slice());
    run_store(path, host).expect("seed through the real store");
}
enum Copy {
    Store { format: Option<Format>, rows: Rows },
    Absent,
    File,
}
#[test]
fn store() {
    // The row-count page bound forces three pages for five temper rows.
    const SMALL_PAGE: u32 = 2;
    let root = scratch("fake-check-store");
    let current = format::current(temper_protocol_store::STORE_VERSION, [0; 16]);
    let mut application = current;
    application.application += 1;
    let mut component = current;
    component.component += 1;
    let temper = &[9, 10, 12];
    let many = &[9, 10, 11, 12, 16];
    let mixed = [(0, 9), (1, 10), (2, 9), (2, 10), (2, 12), (3, 12)]
        .into_iter()
        .map(|(tag, key)| (Box::from([tag, key]), Box::from(b"value".as_slice())))
        .collect();
    let limits = store_limits();
    let page_rows = limits.store.page.rows;
    for (name, copy, page, expected) in [
        (
            "good",
            Copy::Store { format: Some(current), rows: rows(temper, Some(keys::APPLICATION)) },
            page_rows,
            Ok(rows(temper, None)),
        ),
        ("absent", Copy::Absent, page_rows, Err(format!("opening {}: errno 2", root.join("absent").display()))),
        ("file", Copy::File, page_rows, Err(format!("opening {}: errno 20", root.join("file").display()))),
        ("empty", Copy::Store { format: None, rows: Box::new([]) }, page_rows, Err("format: Got None".to_owned())),
        (
            "application",
            Copy::Store { format: Some(application), rows: Box::new([]) },
            page_rows,
            Err("format check: Format { family: Application, found: 4, wanted: 3 }".to_owned()),
        ),
        (
            "component",
            Copy::Store { format: Some(component), rows: Box::new([]) },
            page_rows,
            Err("format check: Format { family: Component, found: 3, wanted: 2 }".to_owned()),
        ),
        ("tags", Copy::Store { format: Some(current), rows: mixed }, page_rows, Ok(rows(temper, None))),
        (
            "pages",
            Copy::Store { format: Some(current), rows: rows(many, Some(keys::APPLICATION)) },
            SMALL_PAGE,
            Ok(rows(many, None)),
        ),
    ] {
        let path = root.join(name);
        let mut limits = limits;
        limits.store.page.rows = page;
        match copy {
            Copy::Store { format, rows } => seed(&path, &limits, format, rows),
            Copy::Absent => {}
            Copy::File => fs::write(&path, b"file").expect("file"),
        }
        assert_eq!(read_store(&path, &limits), expected, "{name}");
    }
    fs::remove_dir_all(root).expect("scratch removed");
}

const START: &[u8] = b"{\"observations_lost\":0}\n";
const LOST: &[u8] = b"{\"observations_lost\":7}\n";
const DELETED: &[u8] =
    b"{\"observation\":\"deleted\",\"repository\":\"org/repo\",\"branch\":\"work\",\"at\":7,\"by\":2}\n";
const MISSING: &str = "observations: no checkpoint";
const REFUSED: &str = "line 1: decode: Refusal { variant: None, member: [111, 98, 115, 101, 114, 118, 97, 116, 105, 111, 110], reason: Json }";
fn encoded(records: Records) -> Result<Vec<Vec<u8>>, String> {
    records
        .map(|record| {
            let mut bytes = skein_lib::List::with_capacity(u32::try_from(DELETED.len()).expect("fixture line length"));
            skein_fake_forge_protocol::observations::encode(&record?, &mut bytes).expect("fixture record fits");
            Ok(bytes.as_slice().to_vec())
        })
        .collect()
}
#[test]
fn observations() {
    let root = scratch("fake-check-observations");
    let bound = usize::try_from(temper_fake::limits::OBSERVATION_BYTES).expect("u32 fits usize");
    // Exactly one byte beyond the line bound (including LF).
    let mut overlong = vec![b'x'; bound];
    overlong.push(b'\n');
    let overlong_error = format!("line 1: line past {bound} bytes");
    let missing_file =
        format!("observations {}: {}", root.join("absent").display(), std::io::Error::from_raw_os_error(2));
    for (name, bytes, expected) in [
        ("start", Some(START.to_vec()), Ok((1, 0, vec![START.to_vec()]))),
        ("partial", Some([DELETED, START, b"{partial"].concat()), Ok((2, 0, vec![DELETED.to_vec(), START.to_vec()]))),
        ("refused-tail", Some([DELETED, START, b"{\n"].concat()), Ok((2, 0, vec![DELETED.to_vec(), START.to_vec()]))),
        ("refused-prefix", Some([b"{\n", START].concat()), Err(REFUSED)),
        ("no-checkpoint", Some(DELETED.to_vec()), Err(MISSING)),
        ("empty", Some(vec![]), Err(MISSING)),
        ("absent", None, Err(missing_file.as_str())),
        ("overlong-prefix", Some([overlong.as_slice(), START].concat()), Err(overlong_error.as_str())),
        ("overlong-tail", Some([START, overlong.as_slice()].concat()), Ok((1, 0, vec![START.to_vec()]))),
        (
            "checkpoints",
            Some([START, DELETED, LOST, b"partial"].concat()),
            Ok((3, 7, vec![START.to_vec(), DELETED.to_vec(), LOST.to_vec()])),
        ),
        (
            "checkpoint-tail",
            Some([START, DELETED, b"{\"observations_lost\":7}"].concat()),
            Ok((1, 0, vec![START.to_vec()])),
        ),
    ] {
        let path = root.join(name);
        if let Some(bytes) = bytes {
            fs::write(&path, bytes).expect("observations fixture");
        }
        let found = Observations::open(&path).and_then(|observations| {
            Ok((observations.lines(), observations.lost(), encoded(observations.records()?)?))
        });
        assert_eq!(found.as_ref().map_err(String::as_str), expected.as_ref().map_err(|why| *why), "{name}");
    }
    // The second pass detects a changed prefix, without reading beyond its
    // saved boundary. Appends are the running fake's ordinary case.
    for (name, replacement, expected) in [
        (
            "shortened",
            b"{\"observations_lost\":".as_slice(),
            Err("observations: file shorter than checkpoint boundary"),
        ),
        ("changed", b"xxxxxxxxxxxxxxxxxxxxxxx\n", Err("decode:")),
        ("grown", [START, b"ignored\n"].concat().as_slice(), Ok(vec![START.to_vec()])),
    ] {
        let path = root.join(name);
        fs::write(&path, START).expect("initial prefix");
        let observations = Observations::open(&path).expect("boundary");
        fs::write(&path, replacement).expect("changed file");
        match expected {
            Ok(expected) => assert_eq!(encoded(observations.records().expect(name)).expect(name), expected, "{name}"),
            Err(expected) => {
                assert!(encoded(observations.records().expect(name)).expect_err(name).starts_with(expected), "{name}");
            }
        }
    }
    fs::remove_dir_all(root).expect("scratch removed");
}
