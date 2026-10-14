use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scratch = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().ok_or("expected --scratch <dir> --out <dir>")?;
        match arg.as_str() {
            "--scratch" => scratch = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err("expected --scratch <dir> --out <dir>".into()),
        }
    }
    forgejo_conformance::run(&scratch.ok_or("--scratch required")?, &out.ok_or("--out required")?)
}
