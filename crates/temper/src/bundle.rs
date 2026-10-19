//! Reading `application.clients.bundle`'s directory whole at start, within `clients.bundle_files` and
//! `clients.bundle_bytes` (client/browser.md, 2.2). Copied from jig's testing/jig-test/src/bundle.rs, with
//! temper's bounds: the page counts among the files, each file is read at the bytes left, and a directory is
//! read in name order.
use skein_lib::Map;
use skein_shell::Unread;
use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};
use temper_service::{
    clients::bundle::{Bundle, File, NAME_BYTES, kind},
    limits::Clients,
};

fn refusal(shown: &Path, why: impl std::fmt::Display) -> String {
    format!("application.clients.bundle: {}: {why}", shown.display())
}
/// The directory's entries in name order, at most twice `bundle_files` of them in all.
fn entries(shown: &Path, directory: &Path, count: &mut u32, limits: Clients) -> Result<Vec<fs::DirEntry>, String> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).map_err(|why| refusal(shown, why))? {
        let entry = entry.map_err(|why| refusal(shown, why))?;
        *count += 1;
        if *count > limits.bundle_files.saturating_mul(2) {
            return Err(refusal(&shown.join(entry.file_name()), "too many entries"));
        }
        found.push(entry);
    }
    found.sort_by_key(fs::DirEntry::file_name);
    Ok(found)
}
struct Files {
    limits: Clients,
    bytes: u32,
    count: u32,
}
impl Files {
    /// One file, read at the bytes left; `shown` names it in a refusal.
    fn read(&mut self, shown: &Path, path: &Path) -> Result<Box<[u8]>, String> {
        if self.count >= self.limits.bundle_files {
            return Err(refusal(shown, "past clients.bundle_files"));
        }
        let left = self.limits.bundle_bytes.saturating_sub(self.bytes);
        let content = skein_shell::read_file(path, left).map_err(|why| match why {
            Unread::TooLarge { .. } => refusal(shown, "past clients.bundle_bytes"),
            why @ (Unread::Absent | Unread::NotAFile | Unread::Unsafe(_) | Unread::Kernel(_)) => {
                refusal(shown, crate::unread(why))
            }
        })?;
        self.bytes += u32::try_from(content.len()).expect("read within the bytes left");
        self.count += 1;
        Ok(content)
    }
}
/// The digest directory's name under `shown`, the only directory beside the page.
fn digest(shown: &Path, root: &Path, limits: Clients) -> Result<Box<[u8]>, String> {
    let mut found = None;
    let mut count = 0;
    for entry in entries(shown, root, &mut count, limits)? {
        let at = shown.join(entry.file_name());
        let kind = entry.file_type().map_err(|why| refusal(&at, why))?;
        if kind.is_dir() {
            if found.is_some() {
                return Err(refusal(&at, "more than one digest directory"));
            }
            let name = entry.file_name();
            let bytes = name.as_bytes();
            if bytes.len() != 16 || !bytes.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
                return Err(refusal(&at, "digest must be 16 lowercase hex digits"));
            }
            found = Some(Box::<[u8]>::from(bytes));
        } else if !kind.is_file() {
            return Err(refusal(&at, "entry must be a regular file or directory"));
        }
    }
    found.ok_or_else(|| refusal(shown, "no digest directory"))
}

/// Read the bundle in `configured` (a path under `directory`) whole, within `limits`; `Err` is the refusal's
/// line.
pub fn read(directory: &Path, configured: &[u8], limits: Clients) -> Result<Bundle, String> {
    let shown = PathBuf::from(std::ffi::OsStr::from_bytes(configured));
    let root = directory.join(&shown);
    let mut state = Files { limits, bytes: 0, count: 0 };
    let page = state.read(&shown.join("index.html"), &root.join("index.html"))?;
    let digest = digest(&shown, &root, limits)?;
    let tree = std::ffi::OsStr::from_bytes(&digest);
    let (shown_tree, root_tree) = (shown.join(tree), root.join(tree));
    let mut files = Map::with_capacity(limits.bundle_files);
    let mut pending = vec![PathBuf::new()];
    let mut count = 0;
    while let Some(relative) = pending.pop() {
        let mut directories = Vec::new();
        for entry in entries(&shown_tree.join(&relative), &root_tree.join(&relative), &mut count, limits)? {
            let name = relative.join(entry.file_name());
            let at = shown_tree.join(&name);
            if name.as_os_str().len() > NAME_BYTES as usize
                || !name.components().all(|part| matches!(part, Component::Normal(_)))
            {
                return Err(refusal(&at, "path must be plain and at most 1024 bytes"));
            }
            let file_type = entry.file_type().map_err(|why| refusal(&at, why))?;
            if file_type.is_dir() {
                directories.push(name);
            } else if file_type.is_file() {
                let key = name.to_str().ok_or_else(|| refusal(&at, "path is not UTF-8"))?.as_bytes();
                let typed = kind(key).ok_or_else(|| refusal(&at, "file type is not on the closed list"))?;
                let content = state.read(&at, &root_tree.join(&name))?;
                files.insert(Box::from(key), File { kind: typed, bytes: content }).expect("file bound checked");
            } else {
                return Err(refusal(&at, "entry must be a regular file or directory"));
            }
        }
        pending.extend(directories.into_iter().rev());
    }
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in &page {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    Ok(Bundle { page, etag: format!("\"{hash:016x}\"").into_bytes().into_boxed_slice(), digest, files })
}
