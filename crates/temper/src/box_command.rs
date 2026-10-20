//! The checked configuration's steps, supervised on one skein-io loop.
use skein_io::kernel::{Exit, Fd};
use skein_shell::{Clock, Config, Host, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_box::host::{Service, Settings};

/// Checks the document before starting its steps; refusals contain unprefixed lines.
pub fn run(directories: &super::Directories) -> Result<Exit, String> {
    let bytes = directories.read()?;
    let (_, checked, limits) = super::setup::check(&bytes, &directories.jig())?;
    let directory = super::path(&directories.data);
    let configuration = directories.document();
    let program = std::env::current_exe().map_err(|why| why.to_string())?;
    let environment = std::env::vars_os()
        .map(|(name, value)| {
            let mut entry = name.as_bytes().to_vec();
            entry.push(b'=');
            entry.extend_from_slice(value.as_bytes());
            entry.into_boxed_slice()
        })
        .collect();
    let steps = temper_forge_setup::steps(&checked.setup, configuration.as_os_str().as_bytes());
    let mut owned = Vec::new();
    let opened = (|| {
        let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
        owned.push(signals);
        let root = skein_shell::open_root(directory).map_err(|errno| format!("directory: errno {errno}"))?;
        owned.push(root);
        let settings = Settings {
            program: program.as_os_str().as_bytes().into(),
            environment,
            plain: b"serve".as_slice().into(),
            root,
            signals,
            stdout: Fd::new(1),
            stderr: Fd::new(2),
        };
        let service = Service::new(&limits, steps, settings).map_err(|why| format!("box limits: {why:?}"))?;
        let kernel = Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string())?;
        Ok((service, kernel))
    })();
    let (mut service, mut kernel) = match opened {
        Ok(opened) => opened,
        Err(why) => {
            super::close(owned);
            return Err(why);
        }
    };
    Ok(skein_shell::drive(&mut kernel, &Clock::new(), &mut service))
}
