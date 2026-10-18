//! Temper's standalone commands (protocol/README.md, section 4).
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match temper::configure(env::args_os().skip(1)).and_then(temper::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}
