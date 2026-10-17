use crate::{
    Result,
    cases::Recorder,
    facts::Fact,
    fixture::Fixture,
    mask::Mask,
    write::{SecretKind, Secrets},
};
use std::path::Path;
use temper_throwaway_forgejo::Token;
pub(crate) const ADMIN: &[&str] = &["write:admin", "write:organization", "write:user"];
pub(crate) fn scoped(f: &Fixture, secrets: &mut Secrets, login: &str, name: &str, scopes: &[&str]) -> Result<Token> {
    let row = f.api(
        "POST",
        &format!("/api/v1/admin/users/{login}/tokens"),
        &serde_json::json!({"name":name, "scopes":scopes}),
    )?;
    let value = row["sha1"].as_str().ok_or("token value missing")?;
    secrets.add(SecretKind::ApiToken, value);
    Ok(Token::new(value.to_owned()))
}
pub(crate) fn record(f: &Fixture, out: &Path, secrets: &mut Secrets) -> Result<()> {
    let admin = scoped(f, secrets, "fixture", "setup-admin", ADMIN)?;
    let mut mask = Mask::new(f.instance.port());
    let mut recorder = Recorder { f, out, secrets, mask: &mut mask };
    crate::users::record(&mut recorder, &admin)?;
    crate::tokens::record(&mut recorder, &admin)?;
    crate::orgs::record(&mut recorder, &admin)
}
pub(crate) fn facts(f: &Fixture, out: &Path) -> Result<Vec<Fact>> {
    let mut facts = crate::users::facts(out)?;
    facts.extend(crate::tokens::facts(out)?);
    facts.extend(crate::orgs::facts(f, out)?);
    Ok(facts)
}
