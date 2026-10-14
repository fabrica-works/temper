mod support;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};

fn scratch(case: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("startup").join(case);
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove prior scratch");
    }
    fs::create_dir_all(&path).expect("scratch directory");
    path
}
#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let directory = scratch("configuration-bound");
    fs::File::create(directory.join("config.json"))
        .expect("configuration file")
        .set_len(1_048_577)
        .expect("one byte past bound");
    assert_eq!(
        temper::prepare_in(&directory).err().expect("refused"),
        "temper serve: configuration config.json: past 1048576 bytes\n"
    );
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_secret_file_readable_by_others_refuses_the_start_naming_it() {
    let directory = scratch("public-token");
    support::configuration(&directory, &support::Forge::Unreached);
    fs::set_permissions(directory.join("secrets/forge-1.git-token"), fs::Permissions::from_mode(0o644))
        .expect("public permissions");
    let refused = temper::prepare_in(&directory).err().expect("start refused");
    assert_eq!(
        refused,
        "temper serve: application.identities[0].token: secrets/forge-1.git-token: readable by others or reached through a link\n"
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_worker_root_reached_through_a_link_is_refused() {
    let directory = scratch("linked-root");
    support::configuration(&directory, &support::Forge::Unreached);
    fs::create_dir(directory.join("real-root")).expect("root directory");
    symlink(directory.join("real-root"), directory.join("worker-root")).expect("linked root");
    assert_eq!(
        temper::prepare_in(&directory).err().expect("refused"),
        "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
    );
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn problems_reach_stderr_one_line_each() {
    let directory = scratch("problem-lines");
    fs::write(directory.join("config.json"), b"{}").expect("configuration");
    assert_eq!(
        temper::prepare_in(&directory).err().expect("refused"),
        "temper serve: engine: missing\ntemper serve: workers: missing\ntemper serve: application: missing\n"
    );
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
