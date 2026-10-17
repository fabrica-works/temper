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
    run::{self, Refusal},
    setup::Mode,
    spec::Which,
    upgrade::Course,
};
use temper_forge_setup_shell::{command, prepare, run as host};
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
            Failure::Newer { which, recorded } => {
                let (name, pin) = pinned(which);
                format!(
                    "{name} is prepared at {}, newer than this temper's pin {pin}: its migrations only go forward",
                    text(&recorded)
                )
            }
            Failure::Unreadable { which } => {
                format!("{} is prepared at a version this temper cannot read", pinned(which).0)
            }
        },
    }
}
fn pinned(which: Which) -> (&'static str, &'static str) {
    use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION};
    match which {
        Which::Forgejo => ("forgejo", VERSION),
        Which::Runner => ("runner", RUNNER_VERSION),
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
fn run_refusal(why: Refusal, launch: &run::Launch, number: u16) -> String {
    let prepare = |why: String| format!("{why}: run temper forge prepare CONFIG.json {number}");
    match why {
        Refusal::NoRunner => "the setup has no runner".into(),
        Refusal::NotPrepared => prepare("not prepared".into()),
        Refusal::Unfinished => prepare("its preparation did not end".into()),
        Refusal::Runner { recorded } => prepare(if recorded {
            "prepared with a runner, and the setup has none".into()
        } else {
            "prepared without a runner, and the setup has one".into()
        }),
        Refusal::Versions { course: Course::Unreadable, .. } => {
            "prepared at a version this temper cannot read: no command of this temper runs it".into()
        }
        Refusal::Versions { forgejo, runner, course } => {
            use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION};
            let recorded =
                runner.map_or_else(|| text(&forgejo), |runner| format!("{} / {}", text(&forgejo), text(&runner)));
            let pinned = if launch.runner { format!("{VERSION} / {RUNNER_VERSION}") } else { VERSION.into() };
            let reason = format!("prepared at {recorded}, this temper pins {pinned}");
            if course == Course::Newer {
                format!("{reason}: no command of this temper runs it")
            } else {
                prepare(reason)
            }
        }
        Refusal::Binary { printed } => format!("{} printed {printed:?}, not the pinned release", text(&launch.program)),
    }
}
fn run_failure(why: host::Failed, launch: &run::Launch, number: u16) -> String {
    match why {
        host::Failed::Refused(why) => run_refusal(why, launch, number),
        host::Failed::Locked => format!(
            "{} is running or being prepared already",
            if launch.which == Which::Forgejo { "forgejo" } else { "runner" }
        ),
        host::Failed::Lock(why) => format!("{}: lock: {why}", text(&launch.lock)),
        host::Failed::State(why) => format!("{}: state {why}", text(&launch.state)),
        host::Failed::Version(why) => command_error(why),
        host::Failed::Exec(why) => format!("{}: {why}", text(&launch.program)),
    }
}
pub(crate) fn start(arguments: &[OsString]) -> Result<u8, Refused> {
    let usage = || Refused {
        verb: "prepare",
        number: None,
        why: "usage: temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner".into(),
    };
    let (verb, configuration, number, which) = match arguments {
        [verb, configuration, number] if verb == "prepare" => ("prepare", configuration, number, None),
        [verb, configuration, number, which] if verb == "run" => {
            let which = match which.to_str() {
                Some("forgejo") => Which::Forgejo,
                Some("runner") => Which::Runner,
                _ => return Err(usage()),
            };
            ("run", configuration, number, Some(which))
        }
        _ => return Err(usage()),
    };
    let number = number.to_str().and_then(|n| n.parse::<u16>().ok()).ok_or_else(usage)?;
    let refused = |why| Refused { verb, number: Some(number), why };
    let bytes = skein_shell::read_file(Path::new(configuration), super::CONFIG_BYTES).map_err(|why| {
        refused(format!("configuration {}: {}", Path::new(configuration).display(), super::unread(why)))
    })?;
    let (_, checked) = super::setup::check(&bytes).map_err(refused)?;
    let entry = checked.setup.entry(number).ok_or_else(|| refused("the setup has no entry for it".into()))?;
    let Mode::Owned(owned) = &entry.mode else {
        return Err(refused("it is managed, not owned".into()));
    };
    let directory = std::path::absolute(super::path(&entry.directory)).map_err(|why| refused(why.to_string()))?;
    if which.is_none() && !directory.exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)
            .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    }
    let user = skein_shell::effective_user();
    if directory.exists() {
        super::private_root(&directory, user).map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    }
    let place = Place { directory: directory.as_os_str().as_bytes().into(), user: login(user).map_err(refused)? };
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).io;
    if let Some(which) = which {
        let launch = run::launch(owned, &place, which).map_err(|_| refused("the setup has no runner".into()))?;
        return Err(refused(run_failure(host::run(&limits, &launch), &launch, number)));
    }
    prepare::run(&limits, &place, Prepare::new(owned.clone(), place.clone(), None))
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
