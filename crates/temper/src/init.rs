//! Facts and file creation for `temper init` (protocol/setup.md, 8.2).
use crate::{
    directories::{Directories, make_private},
    path,
};
use skein_io::kernel::{Complete, Done, Error, Family, Fd, Op, OpenHow, Submit};
use skein_lib::{Queue, Token};
use skein_shell::{Config, Kernel, Wait};
use std::{
    num::NonZeroU16,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
/// Candidate names only: relative PATH entries are never searched.
pub(crate) fn candidates(environment: &[EnvironmentEntry], name: &[u8]) -> Vec<PathBuf> {
    let Some((_, value)) = environment.iter().find(|(key, _)| key.as_ref() == b"PATH") else { return Vec::new() };
    value
        .split(|byte| *byte == b':')
        .filter(|entry| entry.starts_with(b"/"))
        .map(|entry| path(entry).join(std::ffi::OsStr::from_bytes(name)))
        .collect()
}
/// Pure choice over supplied facts; the shell alone inspects candidates.
pub(crate) fn lookup(environment: &[EnvironmentEntry], name: &[u8], facts: &[(PathBuf, bool)]) -> Option<PathBuf> {
    candidates(environment, name)
        .into_iter()
        .find(|candidate| facts.iter().any(|(path, executable)| path == candidate && *executable))
}
fn find(environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
    let entries = candidates(environment, name)
        .into_iter()
        .map(|path| {
            let executable =
                std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.mode() & 0o111 != 0);
            (path, executable)
        })
        .collect::<Vec<_>>();
    lookup(environment, name, &entries)
}
/// A small, synchronous startup shell over the binary's io adapter.
struct Io {
    kernel: Kernel,
    submissions: Queue<Submit>,
    completions: Queue<Complete>,
}
impl Io {
    fn open() -> Result<Self, String> {
        Ok(Self {
            kernel: Kernel::open(Config { operations: 1 }).map_err(|why| why.to_string())?,
            submissions: Queue::with_capacity(1),
            completions: Queue::with_capacity(1),
        })
    }
    fn complete(&mut self, kind: Op) -> Complete {
        self.submissions.push(Submit { op: Token::new(1), kind });
        loop {
            self.kernel.submit(&mut self.submissions, Wait::Forever);
            self.kernel.reap(&mut self.completions);
            if let Some(completion) = self.completions.pop() {
                return completion;
            }
        }
    }
    fn perform(&mut self, kind: Op) -> Result<Done, Error> {
        self.complete(kind).result
    }
    fn socket(&mut self) -> Result<Fd, String> {
        let Done::Fd(fd) = self.perform(Op::Socket { family: Family::Ipv4 }).map_err(io_error)? else {
            unreachable!("socket answers a descriptor")
        };
        Ok(fd)
    }
    fn port(&mut self, preferred: u16) -> Result<(Fd, NonZeroU16), String> {
        let fd = self.socket()?;
        let result = (|| {
            let preferred = ([127, 0, 0, 1], preferred).into();
            let bound = match self.perform(Op::Bind { fd, addr: preferred }) {
                Ok(bound) => bound,
                Err(_) => self.perform(Op::Bind { fd, addr: ([127, 0, 0, 1], 0).into() }).map_err(io_error)?,
            };
            let Done::Bound(address) = bound else { unreachable!("bind answers an address") };
            Ok((fd, NonZeroU16::new(address.port()).expect("bound port")))
        })();
        if result.is_err() {
            self.close(fd);
        }
        result
    }
    fn close(&mut self, fd: Fd) {
        let result = self.perform(Op::Close { fd });
        debug_assert!(result.is_ok(), "startup descriptor closes");
    }
    fn write(&mut self, destination: &Path, bytes: Box<[u8]>, mode: u32) -> Result<(), String> {
        let root = skein_shell::open_root(destination.parent().expect("file parent"))
            .map_err(|errno| format!("errno {errno}"))?;
        let opened = self.perform(Op::Open {
            root,
            path: destination.file_name().expect("file name").as_bytes().into(),
            how: OpenHow::Create { mode: Some(mode) },
        });
        self.close(root);
        let Done::Fd(fd) = opened.map_err(io_error)? else { unreachable!("open answers descriptor") };
        let mut bytes = bytes;
        let mut from = 0;
        let result = (|| {
            while usize::try_from(from).expect("file length fits") < bytes.len() {
                let completion = self.complete(Op::Write { fd, bytes, from, at: u64::from(from) });
                let Op::Write { bytes: returned, .. } = completion.kind else { unreachable!("write returned") };
                bytes = returned;
                let Done::Count(count) = completion.result.map_err(io_error)? else {
                    unreachable!("write answers count")
                };
                from += count;
            }
            // Create's mode is filtered by umask; init's public files have an exact mode.
            std::fs::set_permissions(destination, std::fs::Permissions::from_mode(mode))
                .map_err(|why| why.to_string())?;
            Ok(())
        })();
        self.close(fd);
        result
    }
}
fn io_error(why: Error) -> String {
    format!("errno {}", crate::errno(why))
}

#[derive(Debug)]
pub(crate) struct Written {
    pub paths: Vec<PathBuf>,
    pub missing_bundle: Option<PathBuf>,
}
/// Refusals are checked before directories or files are created. The held
/// listener reserves the chosen port until the files have all been answered.
pub(crate) fn write(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: u16,
) -> Result<Written, String> {
    let document = directories.document();
    for destination in
        [&document, &path(&directories.data).join("worker.secret"), &path(&directories.data).join("worker.digest")]
    {
        match std::fs::symlink_metadata(destination) {
            Ok(_) => return Err(format!("{}: already exists", destination.display())),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", destination.display())),
        }
    }
    let git = find(environment, b"git").ok_or("git: not found on PATH")?;
    let claude = find(environment, b"claude");
    let bundle_path = binary.parent().ok_or("binary: no parent directory")?.join("../share/temper/bundle");
    let bundle = bundle_path.join("index.html").is_file().then_some(&bundle_path);
    let mut io = Io::open()?;
    let (socket, port) = io.port(preferred)?;
    let result = (|| {
        let secret = skein_shell::crypto_seed().map_err(|errno| format!("random secret: errno {errno}"))?;
        let files = temper_init::files(
            &temper_init::Facts {
                binary: binary.as_os_str().as_bytes(),
                git: git.as_os_str().as_bytes(),
                claude: claude.as_ref().map(|path| path.as_os_str().as_bytes()),
                bundle: bundle.map(|path| path.as_os_str().as_bytes()),
                port,
            },
            &temper_init::Directories { config: &directories.config, data: &directories.data },
            secret.expose().try_into().expect("32-byte seed"),
        )
        .map_err(|why| format!("{:?}: {:?}", why.fact, why.reason))?;
        directories.open(skein_shell::effective_user())?;
        let config = path(&directories.config);
        if !config.exists() {
            make_private(config, skein_shell::effective_user())?;
        }
        make_private(&config.join("charters"), skein_shell::effective_user())?;
        let mut paths = Vec::with_capacity(files.len());
        for file in files {
            let root = match file.directory {
                temper_init::Directory::Config => config,
                temper_init::Directory::Data => path(&directories.data),
            };
            let destination = root.join(path(file.name));
            io.write(&destination, file.bytes, if file.private { 0o600 } else { 0o644 })
                .map_err(|why| format!("{}: {why}", destination.display()))?;
            paths.push(destination);
        }
        Ok(Written { paths, missing_bundle: bundle.is_none().then_some(bundle_path.clone()) })
    })();
    io.close(socket);
    result
}
pub(crate) fn core(directories: &Directories) -> Result<Written, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    write(directories, &skein_shell::environment(), &binary, 8367)
}
pub(crate) fn run(directories: &Directories) -> Result<u8, String> {
    let written = core(directories)?;
    for path in written.paths {
        eprintln!("init: wrote {}", path.display());
    }
    if let Some(path) = written.missing_bundle {
        eprintln!("init: no bundle at {}: the browser is not served", path.display());
    }
    let mut line = String::from("init: start it with temper box");
    for (index, (name, value)) in
        [("config", &directories.config), ("data", &directories.data), ("cache", &directories.cache)].iter().enumerate()
    {
        if directories.origins[index] == skein_shell::Origin::Flag {
            use std::fmt::Write;
            write!(line, " --{name} {}", path(value).display()).expect("string");
        }
    }
    eprintln!("{line}");
    Ok(0)
}
#[cfg(test)]
mod tests;
