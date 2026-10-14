use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scratch = None;
    let mut out = None;
    let mut component = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--component" {
            component = true;
            continue;
        }
        let value = args.next().ok_or("expected --scratch <dir> --out <dir>")?;
        match arg.as_str() {
            "--scratch" => scratch = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err("expected --scratch <dir> --out <dir>".into()),
        }
    }
    let scratch = scratch.ok_or("--scratch required")?;
    let out = out.ok_or("--out required")?;
    if component {
        forgejo_conformance::component::record(&scratch, &out)
    } else {
        forgejo_conformance::run(&scratch, &out)
    }
}
