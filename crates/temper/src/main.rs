//! Temper's standalone commands (protocol/README.md, section 4).
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new(smith_agent_shell::relay::FLAG)) {
        return smith_agent_shell::relay::main();
    }
    match temper::configure(env::args_os().skip(1)).and_then(temper::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}
