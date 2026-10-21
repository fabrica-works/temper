use crate::{Content, Directories, Facts, File, Ports};
use core::num::NonZeroU16;
use serde_json::Value;

pub const OWNED: &[u8] = include_bytes!("../../../../docs/examples/owned.json");
pub const AGENT: &[u8] = include_bytes!("../../../../docs/examples/owned/agent.json");
pub const DIRECTORIES: Directories<'static> = Directories { config: b"/c/temper", data: b"/d/temper/" };

pub fn facts() -> Facts<'static> {
    Facts {
        binary: b"/opt/temper/bin/temper",
        git: Some(b"/usr/bin/git"),
        claude: None,
        bundle: None,
        ports: Ports {
            clients: NonZeroU16::new(8367).expect("the default port is nonzero"),
            forge: NonZeroU16::new(8369).expect("the fake forge port is nonzero"),
            llm: NonZeroU16::new(8370).expect("the default LLM port is nonzero"),
        },
    }
}

pub fn cases() -> [(&'static str, Facts<'static>); 5] {
    [
        ("neither, default port", facts()),
        ("no git on PATH", Facts { git: None, ..facts() }),
        (
            "claude, kernel port",
            Facts {
                claude: Some(b"/opt/claude/bin/claude"),
                ports: Ports {
                    clients: NonZeroU16::new(41023).expect("the kernel-port fixture is nonzero"),
                    forge: NonZeroU16::new(41024).expect("the kernel forge port is nonzero"),
                    llm: NonZeroU16::new(65535).expect("the kernel LLM port is nonzero"),
                },
                ..facts()
            },
        ),
        (
            "bundle, kernel port",
            Facts {
                bundle: Some(b"/opt/temper/share/temper/bundle"),
                ports: Ports {
                    clients: NonZeroU16::new(41023).expect("the kernel-port fixture is nonzero"),
                    forge: NonZeroU16::new(41024).expect("the kernel forge port is nonzero"),
                    llm: NonZeroU16::new(65535).expect("the kernel LLM port is nonzero"),
                },
                ..facts()
            },
        ),
        (
            "both, default port",
            Facts {
                claude: Some(b"/opt/claude/bin/claude"),
                bundle: Some(b"/opt/temper/share/temper/bundle"),
                ..facts()
            },
        ),
    ]
}

pub fn file<'a>(files: &'a [File], name: &[u8]) -> &'a File {
    for file in files {
        if file.name == name {
            return file;
        }
    }
    unreachable!("the requested file is one init writes")
}

pub fn parse(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("a whole JSON document")
}

#[expect(clippy::disallowed_methods, reason = "tests inspect the JSON text at the protocol boundary")]
pub fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("the test's paths are UTF-8")
}

pub fn content<'a>(files: &'a [File], name: &[u8]) -> &'a [u8] {
    match &file(files, name).content {
        Content::Bytes { bytes, .. } => bytes,
        Content::Link { .. } => unreachable!("the requested file holds bytes"),
    }
}
