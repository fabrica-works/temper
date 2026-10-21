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
    let mut rows = Vec::new();
    for held in [false, true] {
        for claude in [false, true] {
            for bundle in [false, true] {
                rows.push((held, claude, bundle, true, false));
            }
        }
    }
    rows.push((false, false, false, false, false));
    rows.push((false, false, false, true, true));
    for (held, claude, bundle, git, same) in rows {
        let row = format!("facts-{held}-{claude}-{bundle}-{git}-{same}");
        let root = scratch(&row);
        let dirs = directories(&root);
        let env = environment(&root, claude);
        if !git {
            fs::remove_file(root.join("bin/git")).expect("no git on PATH");
        }
        let binary = root.join("bin/temper");
        if bundle {
            fs::create_dir_all(root.join("share/temper/bundle")).expect("bundle");
            fs::write(root.join("share/temper/bundle/index.html"), b"page").expect("page");
        }
        let listeners = [
            TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("preferred clients listener"),
            TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("preferred LLM listener"),
        ];
        let mut preferred = listeners.each_ref().map(|listener| listener.local_addr().expect("port").port());
        if same {
            preferred[1] = preferred[0];
        }
        let holder = if held {
            Some(listeners)
        } else {
            drop(listeners);
            None
        };
        let written = write(&dirs, &env, &binary, preferred).expect("init");
        check_files(&dirs, &binary, &written, &row);
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(dirs.document()).expect("document")).expect("JSON");
        let address: std::net::SocketAddr = document["application"]["clients"]["listener"]["socket"]
            .as_str()
            .expect("address")
            .parse()
            .expect("socket");
        assert_eq!(address.port() == preferred[0], !held, "{row}");
        assert!(TcpListener::bind(address).is_ok(), "{row}: reservation released");
        assert_eq!(document["application"]["clients"]["bundle"].is_string(), bundle);
        let agent: serde_json::Value =
            serde_json::from_slice(&fs::read(path(&dirs.config).join("agent.json")).expect("agent")).expect("JSON");
        let llm: std::net::SocketAddr =
            agent["endpoints"][1]["address"].as_str().expect("LLM address").parse().expect("LLM socket");
        assert_ne!(address.port(), llm.port(), "{row}: held reservations differ");
        assert_eq!(
            document["engine"]["llm"]["endpoints"][1]["address"],
            serde_json::json!({"socket": llm.to_string()}),
            "{row}"
        );
        assert_eq!(llm.port() == preferred[1], !held && !same, "{row}");
        assert!(TcpListener::bind(llm).is_ok(), "{row}: LLM reservation released");
        assert_eq!(agent["programs"].as_object().expect("programs").contains_key("claude_code"), claude);
        assert_eq!(written.missing_bundle.is_none(), bundle);
        assert_eq!(written.missing_git, !git);
        assert_eq!(document["application"]["workers"][0]["git"].is_null(), !git);
        check_agent(&dirs, agent);
        drop(holder);
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn check_agent(dirs: &Directories, mut document: serde_json::Value) {
    let config = path(&dirs.config);
    // The generated route is asserted above; metadata preflight needs no public DNS
    // or machine trust bundle (as in tests/support/owned.rs's local_agent).
    document["endpoints"][0]["address"] = "127.0.0.1:443".into();
    document["endpoints"][0]["trust_der"] = serde_json::json!(config.join("root.der"));
    fs::write(config.join("root.der"), skein_tls_world::pki::ROOT).expect("fixture trust");
    let checked = config.join("agent-check.json");
    fs::write(&checked, serde_json::to_vec(&document).expect("agent JSON")).expect("metadata fixture");
    crate::agent::read(&checked).expect("written agent configuration parses");
}

fn check_files(dirs: &Directories, binary: &Path, written: &Written, row: &str) {
    let names = [
        ("worker.secret", true, Some(0o600)),
        ("worker.digest", true, Some(0o600)),
        ("demo/llm.key", true, Some(0o600)),
        ("demo/bin/git", true, None),
        ("charters/chat.txt", false, Some(0o644)),
        ("charters/coding.txt", false, Some(0o644)),
        ("agent.json", false, Some(0o644)),
        ("temper.json", false, Some(0o644)),
    ];
    let expected: Vec<_> =
        names.iter().map(|(name, data, _)| path(if *data { &dirs.data } else { &dirs.config }).join(name)).collect();
    assert_eq!(written.paths, expected, "{row}: file order");
    for (file, (_, _, mode)) in written.paths.iter().zip(names) {
        match mode {
            Some(mode) => {
                assert_eq!(fs::metadata(file).expect("file").mode() & 0o777, mode, "{row}: {}", file.display());
            }
            None => assert_eq!(fs::read_link(file).expect("git link"), binary, "{row}"),
        }
    }
    let key = fs::read(path(&dirs.data).join("demo/llm.key")).expect("demo key");
    assert_eq!(key.len(), 65, "{row}: 64 digits and newline");
    assert!(key[..64].iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)), "{row}");
    assert_eq!(key[64], b'\n', "{row}");
    for directory in ["demo", "demo/bin"] {
        assert_eq!(
            fs::metadata(path(&dirs.data).join(directory)).expect("demo directory").mode() & 0o777,
            0o700,
            "{row}"
        );
    }
    for directory in [&dirs.config, &dirs.data, &dirs.cache] {
        assert_eq!(fs::metadata(path(directory)).expect("directory").mode() & 0o777, 0o700);
    }
    assert_eq!(fs::metadata(path(&dirs.config).join("charters")).expect("charters").mode() & 0o777, 0o700);
}

#[test]
fn init_refusals_leave_every_file_and_directory_untouched() {
    for (row, data, dangling) in [
        ("temper.json", false, false),
        ("worker.secret", true, false),
        ("worker.digest", true, false),
        ("demo/llm.key", true, false),
        ("demo/bin/git", true, true),
    ] {
        let root = scratch(&row.replace('/', "-"));
        let dirs = directories(&root);
        let env = environment(&root, false);
        let destination = path(if data { &dirs.data } else { &dirs.config }).join(row);
        fs::create_dir_all(destination.parent().expect("parent")).expect("parent");
        if dangling {
            std::os::unix::fs::symlink("/nonexistent/temper-init-demo", &destination).expect("dangling link");
        } else {
            fs::write(&destination, b"kept").expect("existing file");
        }
        let before = snapshot(&root);
        let why = write(&dirs, &env, &root.join("bin/temper"), [0, 0]).expect_err("refused");
        assert_eq!(why, format!("{}: already exists", destination.display()));
        assert_eq!(snapshot(&root), before, "{row}: every file and directory untouched");
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn snapshot(root: &Path) -> Vec<(PathBuf, u32, Vec<u8>)> {
    let mut pending = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).expect("entry");
        let content = if metadata.is_dir() {
            pending.extend(fs::read_dir(&path).expect("directory").map(|entry| entry.expect("child").path()));
            Vec::new()
        } else if metadata.is_symlink() {
            fs::read_link(&path).expect("link").as_os_str().as_bytes().to_vec()
        } else {
            fs::read(&path).expect("file")
        };
        entries.push((path, metadata.mode(), content));
    }
    entries.sort();
    entries
}

#[test]
fn generated_configuration_passes_the_same_checks_as_serve() {
    let root = scratch("acceptance");
    let dirs = directories(&root);
    let env = environment(&root, true);
    write(&dirs, &env, &root.join("bin/temper"), [0, 0]).expect("init");
    let bytes = fs::read(dirs.document()).expect("document");
    crate::setup::check(&bytes, &dirs.jig()).expect("init document passes setup and named");
    fs::remove_dir_all(root).expect("cleanup");
}
