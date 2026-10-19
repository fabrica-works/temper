use crate::{
    Result,
    cases::{Auth, Recorder},
    facts::{Fact, add, case},
    setup,
    write::SecretKind,
};
use skein_throwaway_forgejo::{Token, fresh_secret};
use std::path::Path;
pub(crate) fn record(r: &mut Recorder<'_>, admin: &Token) -> Result<()> {
    let password = fresh_secret().map_err(|e| crate::failure(&e))?;
    let body = format!(
        "{{\"username\":\"bot\",\"email\":\"bot@example.invalid\",\"password\":\"{password}\",\"full_name\":\"temper:bot\",\"must_change_password\":false}}"
    );
    std::thread::sleep(std::time::Duration::from_secs(1));
    r.send_new(
        "setup-user-create",
        "POST",
        "/api/v1/admin/users",
        Some(&body),
        Auth::Token(admin),
        &[(SecretKind::Password, &password)],
    )?;
    r.send_new("setup-user-create-again", "POST", "/api/v1/admin/users", Some(&body), Auth::Token(admin), &[])?;
    r.send_new(
        "setup-user-create-by-writer",
        "POST",
        "/api/v1/admin/users",
        Some(&body),
        Auth::Token(&r.f.writer.token),
        &[],
    )?;
    for (name, target) in [
        ("setup-user-of", "/api/v1/users/bot"),
        ("setup-user-missing", "/api/v1/users/missing"),
        ("setup-user-admin", "/api/v1/user"),
    ] {
        r.send_new(name, "GET", target, None, Auth::Token(admin), &[])?;
    }
    let password = fresh_secret().map_err(|e| crate::failure(&e))?;
    let body = format!(
        "{{\"username\":\"person\",\"email\":\"person@example.invalid\",\"password\":\"{password}\",\"full_name\":\"Person\",\"must_change_password\":true}}"
    );
    std::thread::sleep(std::time::Duration::from_secs(1));
    r.send_new(
        "setup-user-create-person",
        "POST",
        "/api/v1/admin/users",
        Some(&body),
        Auth::Token(admin),
        &[(SecretKind::Password, &password)],
    )?;
    r.send_new("setup-user-person-basic", "GET", "/api/v1/user", None, Auth::Basic("person", &password), &[])?;
    let person = setup::scoped(r.f, r.secrets, "person", "setup-person", &["read:user"])?;
    r.send_new("setup-user-person-token", "GET", "/api/v1/user", None, Auth::Token(&person), &[])?;
    r.f.api(
        "PATCH",
        "/api/v1/admin/users/person",
        &serde_json::json!({"login_name":"person", "source_id":0, "must_change_password":false}),
    )?;
    r.send_new("setup-user-person-cleared", "GET", "/api/v1/user", None, Auth::Token(&person), &[])?;
    Ok(())
}
pub(crate) fn facts(out: &Path) -> Result<Vec<Fact>> {
    let made = case(out, "setup-user-create")?;
    let user = case(out, "setup-user-of")?;
    let again = case(out, "setup-user-create-again")?;
    let basic = case(out, "setup-user-person-basic")?;
    let token = case(out, "setup-user-person-token")?;
    let cleared = case(out, "setup-user-person-cleared")?;
    let admin = case(out, "setup-user-admin")?;
    let mut facts = Vec::new();
    for (name, holds) in [
        (
            "setup-create-user-answers-201-with-login-email-and-full-name",
            made.status == 201
                && made.body["login"] == "bot"
                && made.body["email"] == "bot@example.invalid"
                && made.body["full_name"] == "temper:bot",
        ),
        (
            "setup-created-user-is-no-site-admin-and-keeps-its-full-name",
            made.body["is_admin"] == false && made.body["full_name"] == "temper:bot",
        ),
        ("setup-create-user-again-is-refused-with-a-message", again.status >= 400 && again.body["message"].is_string()),
        (
            "setup-user-of-a-login-reads-admin-flag-and-full-name",
            user.status == 200
                && user.body["login"] == "bot"
                && user.body["is_admin"] == false
                && user.body["full_name"] == "temper:bot",
        ),
        ("setup-user-of-a-missing-login-is-404", case(out, "setup-user-missing")?.status == 404),
        (
            "setup-a-non-admin-token-is-refused-the-admin-routes",
            case(out, "setup-user-create-by-writer")?.status == 403,
        ),
        ("setup-the-admin-token-reads-user-as-a-site-admin", admin.status == 200 && admin.body["is_admin"] == true),
        (
            "setup-a-user-made-to-change-its-password-is-refused-the-api-under-basic-auth",
            basic.status == 403 && basic.body["message"].as_str().is_some_and(|s| s.contains("change your password")),
        ),
        (
            "setup-a-user-made-to-change-its-password-is-refused-under-its-token-until-the-admin-clears-the-flag",
            token.status == 403
                && token.body["message"].as_str().is_some_and(|s| s.contains("change your password"))
                && cleared.status == 200
                && cleared.body["login"] == "person",
        ),
    ] {
        add(&mut facts, name, holds, "captured status or members differ");
    }
    let mut whole = true;
    for name in [
        "user",
        "user-fixture",
        "setup-user-create",
        "setup-user-of",
        "setup-user-admin",
        "setup-user-create-person",
        "setup-user-person-cleared",
    ] {
        let row = case(out, name)?;
        whole &= row.body["is_admin"].is_boolean() && row.body["full_name"].is_string();
    }
    add(&mut facts, "setup-every-recorded-user-carries-is-admin-and-full-name", whole, "user members absent");
    Ok(facts)
}
