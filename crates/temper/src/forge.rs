//! Forge commands check the document, then host only their own setup part.
use std::{
    ffi::OsString,
    fs,
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::Path,
};
use temper_forge_setup::{
    place::{File, Place},
    prepare::{Exit, Failure, Prepare},
    setup::Mode,
};
use temper_forge_setup_shell::{command, prepare};
/// A command's diagnostic, without any credential contents.
#[derive(Debug)]
pub struct Refused {
    verb: &'static str,
    number: Option<u16>,
    why: String,
}
impl Refused {
    #[must_use]
    pub fn line(&self) -> String {
        let forge = self.number.map_or_else(String::new, |n| format!("forge {n}: "));
        super::prefixed(&format!("temper forge {}", self.verb), &format!("{forge}{}", self.why))
    }
}
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn status(exit: Exit) -> String {
    match exit {
        Exit::Code(n) => n.to_string(),
        Exit::Signal(n) => format!("signal {n}"),
    }
}
fn command_error(why: command::Error) -> String {
    match why {
        command::Error::Interrupted => "interrupted".into(),
        command::Error::Kernel(why) => format!("command kernel: {why}"),
        command::Error::Spawn(why) => format!("command could not start: {why:?}"),
    }
}
fn failure(why: prepare::Failed, place: &Place) -> String {
    match why {
        prepare::Failed::Held(file) => format!(
            "{} is running or being prepared already",
            if file == File::RunnerLock { "runner" } else { "forgejo" }
        ),
        prepare::Failed::Io(file, why) => format!("{}: {file}: {why}", text(&place.directory)),
        prepare::Failed::Command(why) => command_error(why),
        prepare::Failed::Refused(why) => match why {
            Failure::Binary { program, printed, .. } => {
                format!("{} printed {printed:?}, not the pinned release", text(&program))
            }
            Failure::Command { command, exit, tail } => {
                format!("{command:?} ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::Token { exit, tail } => {
                format!("no token for temper-admin: ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::Unread { command } => format!("{command:?} printed no required value"),
            Failure::State(why) => format!("{}: state cannot be decoded: {why:?}", text(&place.directory)),
            Failure::Unrecorded => format!("{}: Forgejo files have no preparation state", text(&place.directory)),
            Failure::Recorded => format!("{}: state records other prepared versions", text(&place.directory)),
        },
    }
}
fn login(user: u32) -> Result<Box<[u8]>, String> {
    let passwd = skein_shell::read_file(Path::new("/etc/passwd"), 1 << 20)
        .map_err(|why| format!("user: {}", super::unread(why)))?;
    for line in passwd.split(|c| *c == b'\n') {
        let fields: Vec<_> = line.split(|c| *c == b':').collect();
        if fields.get(2).and_then(|id| std::str::from_utf8(id).ok()).and_then(|id| id.parse::<u32>().ok()) == Some(user)
        {
            return Ok(fields[0].into());
        }
    }
    Err(format!("user {user}: no login in /etc/passwd"))
}
pub(crate) fn start(arguments: &[OsString]) -> Result<u8, Refused> {
    let usage =
        || Refused { verb: "prepare", number: None, why: "usage: temper forge prepare CONFIG.json FORGE".into() };
    let [verb, configuration, number] = arguments else {
        return Err(usage());
    };
    if verb != "prepare" {
        return Err(usage());
    }
    let number = number.to_str().and_then(|n| n.parse::<u16>().ok()).ok_or_else(usage)?;
    let refused = |why| Refused { verb: "prepare", number: Some(number), why };
    let bytes = skein_shell::read_file(Path::new(configuration), super::CONFIG_BYTES).map_err(|why| {
        refused(format!("configuration {}: {}", Path::new(configuration).display(), super::unread(why)))
    })?;
    let (_, checked) = super::setup::check(&bytes).map_err(refused)?;
    let entry = checked.setup.entry(number).ok_or_else(|| refused("the setup has no entry for it".into()))?;
    let Mode::Owned(owned) = &entry.mode else {
        return Err(refused("it is managed, not owned".into()));
    };
    let directory = std::path::absolute(super::path(&entry.directory)).map_err(|why| refused(why.to_string()))?;
    if !directory.exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)
            .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    }
    let user = skein_shell::effective_user();
    super::private_root(&directory, user).map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    let place = Place { directory: directory.as_os_str().as_bytes().into(), user: login(user).map_err(refused)? };
    prepare::run(
        &jig_protocol_config::profile(jig_protocol_config::Profile::Small).io,
        &place,
        Prepare::new(owned.clone(), place.clone(), None),
    )
    .map_err(|why| refused(failure(why, &place)))?;
    Ok(0)
}
#[must_use]
pub fn run(arguments: &[OsString]) -> u8 {
    match start(arguments) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line());
            1
        }
    }
}
