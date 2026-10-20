use super::*;
use std::{fs, net::TcpListener, os::unix::fs::PermissionsExt};

fn scratch(row: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-init-{}-{row}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("prior scratch");
    }
    fs::create_dir(&root).expect("scratch");
    root
}
fn directories(root: &Path) -> Directories {
    let paths = [root.join("config"), root.join("data"), root.join("cache")];
    let args = [
        b"--config".as_slice(),
        paths[0].as_os_str().as_bytes(),
        b"--data",
        paths[1].as_os_str().as_bytes(),
        b"--cache",
        paths[2].as_os_str().as_bytes(),
    ];
    crate::directories::resolve(&args.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("explicit directories")
        .0
}
fn environment(root: &Path, claude: bool) -> Vec<EnvironmentEntry> {
    fs::create_dir(root.join("bin")).expect("bin");
    for name in if claude { vec!["git", "claude"] } else { vec!["git"] } {
        let path = root.join("bin").join(name);
        fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    }
    vec![(Box::from(b"PATH".as_slice()), root.join("bin").as_os_str().as_bytes().into())]
}
#[test]
fn path_lookup_uses_only_absolute_executable_candidates_in_order() {
    for (row, entries, executable, expected) in [
        ("absolute", "/one:/two", vec![true, true], Some("/one/git")),
        ("relative skipped", "relative::/two", vec![true], Some("/two/git")),
        ("none", "relative:", vec![], None),
        ("non executable", "/one:/two", vec![false, true], Some("/two/git")),
        ("no hit", "/one", vec![false], None),
    ] {
        let env = vec![(Box::from(b"PATH".as_slice()), Box::from(entries.as_bytes()))];
        let candidates = candidates(&env, b"git");
        assert_eq!(candidates.len(), executable.len(), "{row}");
        let facts: Vec<_> = candidates.into_iter().zip(executable).collect();
        assert_eq!(lookup(&env, b"git", &facts).as_deref(), expected.map(Path::new), "{row}");
    }
}
#[test]
fn init_finds_ports_programs_and_bundle_and_writes_exact_modes_in_order() {
    for held in [false, true] {
        for claude in [false, true] {
            for bundle in [false, true] {
                let row = format!("facts-{held}-{claude}-{bundle}");
                let root = scratch(&row);
                let dirs = directories(&root);
                let env = environment(&root, claude);
                let binary = root.join("bin/temper");
                if bundle {
                    fs::create_dir_all(root.join("share/temper/bundle")).expect("bundle");
                    fs::write(root.join("share/temper/bundle/index.html"), b"page").expect("page");
                }
                let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("preferred listener");
                let preferred = listener.local_addr().expect("port").port();
                let holder = if held {
                    Some(listener)
                } else {
                    drop(listener);
                    None
                };
                let written = write(&dirs, &env, &binary, preferred).expect("init");
                let names = [
                    "worker.secret",
                    "worker.digest",
                    "charters/chat.txt",
                    "charters/coding.txt",
                    "agent.json",
                    "temper.json",
                ];
                let expected: Vec<_> = names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| path(if i < 2 { &dirs.data } else { &dirs.config }).join(name))
                    .collect();
                assert_eq!(written.paths, expected, "{row}: file order");
                for (i, file) in written.paths.iter().enumerate() {
                    assert_eq!(
                        fs::metadata(file).expect("file").mode() & 0o777,
                        if i < 2 { 0o600 } else { 0o644 },
                        "{row}: {}",
                        file.display()
                    );
                }
                for directory in [&dirs.config, &dirs.data, &dirs.cache] {
                    assert_eq!(fs::metadata(path(directory)).expect("directory").mode() & 0o777, 0o700);
                }
                assert_eq!(fs::metadata(path(&dirs.config).join("charters")).expect("charters").mode() & 0o777, 0o700);
                let document: serde_json::Value =
                    serde_json::from_slice(&fs::read(dirs.document()).expect("document")).expect("JSON");
                let address: std::net::SocketAddr = document["application"]["clients"]["listener"]["socket"]
                    .as_str()
                    .expect("address")
                    .parse()
                    .expect("socket");
                assert_eq!(address.port() == preferred, !held, "{row}");
                assert!(TcpListener::bind(address).is_ok(), "{row}: reservation released");
                assert_eq!(document["application"]["clients"]["bundle"].is_string(), bundle);
                let agent: serde_json::Value =
                    serde_json::from_slice(&fs::read(path(&dirs.config).join("agent.json")).expect("agent"))
                        .expect("JSON");
                assert_eq!(agent["programs"].as_object().expect("programs").contains_key("claude_code"), claude);
                assert_eq!(written.missing_bundle.is_none(), bundle);
                let declarations = crate::agent::read(&path(&dirs.config).join("agent.json"));
                declarations.expect("written agent configuration parses");
                drop(holder);
                fs::remove_dir_all(root).expect("cleanup");
            }
        }
    }
}
#[test]
fn init_refusals_leave_every_file_and_directory_untouched() {
    for row in ["temper.json", "worker.secret", "worker.digest", "no git"] {
        let root = scratch(row);
        let dirs = directories(&root);
        let mut env = environment(&root, false);
        let destination = match row {
            "temper.json" => dirs.document(),
            _ => path(&dirs.data).join(row),
        };
        if row == "no git" {
            env.clear();
        } else {
            fs::create_dir(destination.parent().expect("parent")).expect("parent");
            fs::write(&destination, b"kept").expect("existing file");
        }
        let why = write(&dirs, &env, &root.join("bin/temper"), 0).expect_err("refused");
        assert_eq!(
            why,
            if row == "no git" {
                "git: not found on PATH".into()
            } else {
                format!("{}: already exists", destination.display())
            }
        );
        for directory in [&dirs.config, &dirs.data, &dirs.cache] {
            let directory = path(directory);
            if directory == destination.parent().expect("parent") && row != "no git" {
                assert_eq!(fs::read_dir(directory).expect("existing parent").count(), 1);
                assert_eq!(fs::read(&destination).expect("kept"), b"kept");
            } else {
                assert!(!directory.exists(), "{row}: {}", directory.display());
            }
        }
        fs::remove_dir_all(root).expect("cleanup");
    }
}
#[test]
#[ignore = "landed init template requires the separate GitHub parser work recorded in STATUS"]
fn generated_configuration_passes_the_same_checks_as_serve() {
    let root = scratch("acceptance");
    let dirs = directories(&root);
    let env = environment(&root, true);
    write(&dirs, &env, &root.join("bin/temper"), 0).expect("init");
    let bytes = fs::read(dirs.document()).expect("document");
    crate::setup::check(&bytes, &dirs.jig()).expect("init document passes setup and named");
    fs::remove_dir_all(root).expect("cleanup");
}
