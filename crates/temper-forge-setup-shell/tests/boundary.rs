pub mod support;
use std::{fs, path::Path};
fn forbidden(source: &str) -> bool {
    let compact: String = source.chars().filter(|c| !c.is_whitespace()).collect();
    compact.contains("std::process")
        || compact.contains("process::") && compact.contains("usestd::{")
        || ["CommandExt", "try_lock", "lock_shared", ".lock()"].iter().any(|name| compact.contains(name))
}
fn scan(directory: &Path) {
    for entry in fs::read_dir(directory).expect("source directory") {
        let path = entry.expect("source entry").path();
        if path.file_name().is_some_and(|name| name == "tests" || name == "tests.rs") {
            continue;
        }
        if path.is_dir() {
            scan(&path);
        } else if path.extension().is_some_and(|name| name == "rs") {
            if path.ends_with("temper-forge-setup-shell/src/process.rs") {
                continue;
            }
            let source = fs::read_to_string(&path).expect("Rust source");
            let source = if path.ends_with("temper/src/main.rs") {
                source.replace("std::process::ExitCode", "ExitCode").replace("process::ExitCode", "ExitCode")
            } else {
                source
            };
            assert!(!forbidden(&source), "process or lock outside its boundary: {}", path.display());
        }
    }
}
#[test]
fn std_process_and_file_locks_appear_only_in_the_process_module() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crates");
    for entry in fs::read_dir(crates).expect("crates") {
        let source = entry.expect("crate").path().join("src");
        if source.is_dir() {
            scan(&source);
        }
    }
}
#[test]
fn the_scan_catches_each_form_and_passes_other_process_modules() {
    for (source, expected) in [
        ("use std::{fs, process::Child};", true),
        ("std::process::exit(1)", true),
        ("CommandExt", true),
        ("try_lock", true),
        ("smith::process::Event", false),
    ] {
        assert_eq!(forbidden(source), expected, "{source}");
    }
}
