use super::reconcile_settings;
use crate::tests::configuration;
use std::{fs, os::unix::ffi::OsStrExt};

#[test]
fn reconcile_adopts_no_repositories() {
    let directory = std::env::temp_dir().join(format!("temper-reconcile-settings-{}", std::process::id()));
    fs::create_dir(&directory).expect("settings scratch");
    let directories = jig_protocol_config::Directories::given(
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
    )
    .expect("fixture directories");
    for fake in [true, false] {
        let mut value = configuration::check_document(&directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        let entry = &mut value["application"]["setup"][0];
        if fake {
            entry["mode"]["fake"]["repositories"] = serde_json::json!([
                {"name":"z-last", "landing":"main"}, {"name":"a-first", "landing":"trunk"},
            ]);
        } else {
            entry["mode"] = serde_json::json!({"managed":{"admin_token":"admin_token"}});
            entry["orgs"] = serde_json::json!(["example"]);
        }
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        let settings = reconcile_settings(&deployment.named, &deployment.checked, 1).expect("settings");
        assert!(settings.given.adopted.is_empty());
        assert_eq!(settings.given.listener, None, "the box makes no hook");
    }
    fs::remove_dir_all(directory).expect("settings scratch removed");
}
