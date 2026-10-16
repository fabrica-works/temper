use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use temper_throwaway_forgejo::{Failure, Forgejo, Init, Outgoing, Refusal, Settings, VERSION};
#[test]
fn a_binary_printing_another_version_is_refused_naming_it() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("wrong-version");
    fs::create_dir_all(&scratch).expect("scratch");
    let binary = scratch.join("forgejo");
    fs::write(&binary, b"#!/bin/sh\nprintf 'forgejo version 15.0.0+gitea-1.22.0\\n'\n").expect("script");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    assert_eq!(
        Forgejo::from_path(&binary).expect_err("refused"),
        Refusal::Version {
            path: fs::canonicalize(&binary).expect("absolute"),
            printed: "forgejo version 15.0.0+gitea-1.22.0".into()
        }
    );
    fs::remove_dir_all(scratch).expect("removed");
}
#[test]
fn a_scratch_directory_that_is_not_empty_is_refused() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("occupied");
    fs::create_dir_all(&scratch).expect("scratch");
    fs::write(scratch.join("keep"), b"mine").expect("file");
    let binary = scratch.join("forgejo");
    fs::write(&binary, format!("#!/bin/sh\nprintf 'forgejo version {VERSION}+gitea-1.22.0 (release name {VERSION}) built with GNU Make 4.4.1, go1.26.9\\n'\n")).expect("script");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    let forgejo = Forgejo::from_path(&binary).expect("version accepted");
    let result = forgejo.start(&scratch, &Settings::small());
    assert!(matches!(result, Err(Failure::Scratch { .. })));
    assert_eq!(fs::read(scratch.join("keep")).expect("kept"), b"mine");
    fs::remove_dir_all(scratch).expect("removed");
}
#[test]
#[ignore = "needs TEMPER_FORGEJO"]
fn a_throwaway_forgejo_starts_serves_its_admin_and_is_removed() {
    let forgejo = Forgejo::from_env().expect("TEMPER_FORGEJO names the pinned Forgejo");
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("throwaway-start");
    let mut instance = forgejo.start(&scratch, &Settings::small()).expect("started");
    let admin = instance.admin().clone();
    let user = instance.exchange(&Outgoing::get("/api/v1/user", Some(&admin.token))).expect("answered");
    assert_eq!(user.status, 200, "the admin's token reads its user");
    let made = instance.create_repository(&admin, "specimen", Init::Readme).expect("made");
    assert_eq!(made.name, "specimen");
    instance.stop().expect("stopped");
    assert!(!scratch.exists(), "the scratch directory is removed");
}
