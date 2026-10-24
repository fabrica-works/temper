//! Each module owns a figure file; the checked total is derived, never stored.
extern crate alloc;
use alloc::collections::BTreeSet;
use std::{fs, path::Path};
pub fn check(directory: &Path, terms: &[(&str, u64)], total: u64) {
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let mut names = BTreeSet::new();
    let mut sum = 0_u64;
    if write {
        fs::create_dir_all(directory).expect("memory figure directory");
    }
    for (name, bytes) in terms {
        assert!(!name.is_empty() && !name.contains('/') && !name.contains('\\'), "module file name: {name}");
        let name = format!("{name}.txt");
        assert!(names.insert(name.clone()), "one figure per term: {name}");
        let path = directory.join(&name);
        if write {
            fs::write(&path, format!("{bytes}\n")).expect("write memory term");
        }
        let kept = fs::read_to_string(&path).unwrap_or_else(|why| {
            panic!("{}: {why}; regenerate with SKEIN_CODEGEN_WRITE=1 and the same test filter", path.display())
        });
        assert_eq!(
            kept,
            format!("{bytes}\n"),
            "{}; regenerate with SKEIN_CODEGEN_WRITE=1 and the same test filter",
            path.display()
        );
        sum = sum.checked_add(kept.trim().parse::<u64>().expect("term bytes")).expect("checked figure sum");
    }
    let mut kept = BTreeSet::new();
    for entry in fs::read_dir(directory).expect("memory figure directory") {
        let entry = entry.expect("memory figure file");
        assert!(entry.file_type().expect("figure type").is_file(), "one file per term");
        let name = entry.file_name().into_string().expect("module name");
        if write && !names.contains(&name) {
            fs::remove_file(entry.path()).expect("remove departed term");
        } else {
            kept.insert(name);
        }
    }
    assert_eq!(kept, names, "memory term files; regenerate with SKEIN_CODEGEN_WRITE=1 and the same test filter");
    assert_eq!(sum, total, "term files sum to the computed envelope: {}", directory.display());
}
