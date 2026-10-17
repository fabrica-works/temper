//! Hosts of the forge setup machines (setup.md, 11).
pub mod command;
pub mod files;
pub mod prepare;
pub mod process;
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
pub(crate) fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}
