pub mod support;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("configuration-bound");
    let directory = scratch.path();
    fs::File::create(directory.join("config.json"))
        .expect("configuration file")
        .set_len(1_048_577)
        .expect("one byte past bound");
    assert_eq!(
        temper::prepare_in(directory).err().expect("refused"),
        "temper serve: configuration config.json: past 1048576 bytes\n"
    );
    scratch.remove();
}
#[test]
fn a_secret_file_readable_by_others_refuses_the_start_naming_it() {
    let scratch = support::Scratch::startup("public-token");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::set_permissions(directory.join("secrets/forge-1.git-token"), fs::Permissions::from_mode(0o644))
        .expect("public permissions");
    let refused = temper::prepare_in(directory).err().expect("start refused");
    assert_eq!(
        refused,
        "temper serve: application.identities[0].token: secrets/forge-1.git-token: readable by others or reached through a link\n"
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
    scratch.remove();
}
#[test]
fn a_worker_root_reached_through_a_link_is_refused() {
    let scratch = support::Scratch::startup("linked-root");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::create_dir(directory.join("real-root")).expect("root directory");
    symlink(directory.join("real-root"), directory.join("worker-root")).expect("linked root");
    assert_eq!(
        temper::prepare_in(directory).err().expect("refused"),
        "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
    );
    scratch.remove();
}
#[test]
fn problems_reach_stderr_one_line_each() {
    let scratch = support::Scratch::startup("problem-lines");
    let directory = scratch.path();
    fs::write(directory.join("config.json"), b"{}").expect("configuration");
    assert_eq!(
        temper::prepare_in(directory).err().expect("refused"),
        "temper serve: engine: missing\ntemper serve: workers: missing\ntemper serve: application: missing\n"
    );
    scratch.remove();
}

#[test]
fn an_address_with_no_token_in_the_directory_is_not_signed_in() {
    for (row, token, expected) in [
        ("absent", None, "temper open: not signed in: give the link temper serve printed\n"),
        ("empty", Some(b"".as_slice()), "temper open: not signed in: give the link temper serve printed\n"),
    ] {
        let scratch = support::Scratch::startup(&format!("open-{row}"));
        if let Some(token) = token {
            fs::write(scratch.path().join("token"), token).expect("token file");
            fs::set_permissions(scratch.path().join("token"), fs::Permissions::from_mode(0o600))
                .expect("private token");
        }
        assert_eq!(
            temper::open::check_token(scratch.path()).expect_err("not signed in").line(scratch.path()),
            expected,
            "{row}"
        );
        scratch.remove();
    }
}

#[test]
fn a_user_file_readable_by_others_refuses_the_start_naming_it() {
    let scratch = support::Scratch::startup("public-user");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    fs::set_permissions(scratch.path().join("secrets/forge-1.user"), fs::Permissions::from_mode(0o644))
        .expect("fixture operation");
    assert_eq!(
        temper::prepare_in(scratch.path()).err().expect("configuration refused"),
        "temper serve: application.forges[0].user.file: secrets/forge-1.user: readable by others or reached through a link\n"
    );
    scratch.remove();
}
fn managed(directory: &std::path::Path, move_files: bool) -> serde_json::Value {
    support::configuration(directory, &support::Forge::Unreached);
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("config.json")).expect("configuration bytes"))
            .expect("fixture operation");
    value["application"]["setup"] = serde_json::json!([{
        "forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.test"},
        "orgs":["acme"],"mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}
    }]);
    if move_files {
        fs::create_dir(directory.join("forge-1")).expect("fixture operation");
        for (from, to) in [
            ("secrets/forge-1.api-token", "api_token"),
            ("secrets/forge-1.git-token", "git_token"),
            ("secrets/forge-1.user", "user"),
        ] {
            fs::rename(directory.join(from), directory.join("forge-1").join(to)).expect("fixture operation");
        }
        value["application"]["forges"][0]["api_token"] = "forge-1/api_token".into();
        value["application"]["forges"][0]["user"] = serde_json::json!({"file":"forge-1/user"});
        value["application"]["identities"][0]["token"] = "forge-1/git_token".into();
    }
    support::write_private(&directory.join("config.json"), &serde_json::to_vec(&value).expect("configuration JSON"));
    value
}
#[test]
fn a_setup_naming_another_api_token_file_refuses_temper_serve_naming_its_path() {
    let scratch = support::Scratch::startup("setup-token");
    let mut value = managed(scratch.path(), true);
    value["application"]["forges"][0]["api_token"] = "another/api_token".into();
    support::write_private(
        &scratch.path().join("config.json"),
        &serde_json::to_vec(&value).expect("configuration JSON"),
    );
    assert_eq!(
        temper::prepare_in(scratch.path()).err().expect("configuration refused"),
        "temper serve: application.forges[0].api_token: invalid form\n"
    );
    scratch.remove();
}
#[test]
fn a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits() {
    let scratch = support::Scratch::startup("setup-limits");
    let mut value = managed(scratch.path(), true);
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 6000.into();
    support::write_private(
        &scratch.path().join("config.json"),
        &serde_json::to_vec(&value).expect("configuration JSON"),
    );
    assert_eq!(
        temper::prepare_in(scratch.path()).err().expect("configuration refused"),
        "temper serve: application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_managed_entrys_absent_admin_token_file_does_not_stop_temper_serve() {
    let scratch = support::Scratch::startup("setup-no-admin");
    managed(scratch.path(), true);
    assert!(!scratch.path().join("secrets/forgejo-admin").exists());
    assert!(temper::prepare_in(scratch.path()).is_ok());
    scratch.remove();
}
#[test]
fn a_setup_naming_another_api_token_file_refuses_temper_sign_in_naming_its_path() {
    let scratch = support::Scratch::startup("setup-sign-in");
    let mut value = managed(scratch.path(), true);
    value["application"]["forges"][0]["api_token"] = "another/api_token".into();
    let refused = temper::sign_in::account(&serde_json::to_vec(&value).expect("configuration JSON"), 1)
        .err()
        .expect("configuration refused")
        .line();
    assert_eq!(refused, "temper sign-in: application.forges[0].api_token: invalid form\n");
    scratch.remove();
}

fn box_document(scratch: &support::Scratch) -> serde_json::Value {
    support::configuration(scratch.path(), &support::Forge::Unreached);
    serde_json::from_slice(&fs::read(scratch.path().join("config.json")).expect("configuration")).expect("JSON")
}
fn refused_box(scratch: &support::Scratch, value: &serde_json::Value) -> String {
    support::write_private(&scratch.path().join("config.json"), &serde_json::to_vec(value).expect("JSON"));
    temper::run(temper::Command::Box { configuration: scratch.path().join("config.json") }).expect_err("box refused")
}
#[test]
fn a_box_configuration_that_is_absent_is_refused_naming_it() {
    assert_eq!(
        temper::run(temper::Command::Box { configuration: "absent.json".into() }),
        Err("temper box: configuration absent.json: absent\n".into())
    );
}
#[test]
fn a_box_configuration_missing_members_is_refused_one_line_each() {
    let scratch = support::Scratch::startup("box-members");
    assert_eq!(
        refused_box(&scratch, &serde_json::json!({})),
        "temper box: engine: missing\ntemper box: workers: missing\ntemper box: application: missing\n"
    );
    scratch.remove();
}
#[test]
fn a_setup_naming_no_forge_refuses_the_box_before_it_starts() {
    let scratch = support::Scratch::startup("box-forge");
    let mut value = managed(scratch.path(), false);
    value["application"]["setup"][0]["forge"] = 7.into();
    assert_eq!(refused_box(&scratch, &value), "temper box: application.setup[0].forge: unknown reference\n");
    assert!(!scratch.path().join("forge-7").exists());
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_below_its_base_is_refused() {
    let scratch = support::Scratch::startup("box-backoff");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_reaching_its_window_is_refused() {
    let scratch = support::Scratch::startup("box-window");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 300_000.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.backoff_max_ms: must agree with box.window_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("box-restarts");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 65.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.restarts: past its bound\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_outside_its_type_is_refused() {
    let scratch = support::Scratch::startup("box-restarts-type");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 4_294_967_296_u64.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.restarts: outside its type\n"
    );
    scratch.remove();
}
#[test]
fn an_unknown_box_limit_is_refused() {
    let scratch = support::Scratch::startup("box-unknown");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.nope"] = 1.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.nope: no limit of temper's has this name\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_past_its_check_refuses_temper_serve_too() {
    let scratch = support::Scratch::startup("box-serve-refused");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    support::write_private(&scratch.path().join("config.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert_eq!(
        temper::prepare_in(scratch.path()).err().expect("serve refused"),
        "temper serve: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_within_its_check_is_accepted_by_temper_serve() {
    let scratch = support::Scratch::startup("box-serve-accepted");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 3.into();
    support::write_private(&scratch.path().join("config.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert!(temper::prepare_in(scratch.path()).is_ok());
    scratch.remove();
}
