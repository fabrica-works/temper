use std::{env, path::Path, process};
fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(out) = args.next() else {
        eprintln!("usage: temper-gallery OUT");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: temper-gallery OUT");
        process::exit(2);
    }
    if let Err(why) = temper_gallery::print(Path::new(&out)) {
        eprintln!("temper-gallery: {why:?}");
        process::exit(1);
    }
}
