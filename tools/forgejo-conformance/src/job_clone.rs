use crate::{
    Result,
    facts::{Fact, add},
    fixture::Fixture,
};
use serde_json::json;
/// A private clone first without authentication, then with the job's token.
pub(crate) fn workflow() -> &'static str {
    "name: clone-check\non: push\njobs:\n  clone:\n    runs-on: temper-host\n    steps:\n      - run: |\n          if GIT_TERMINAL_PROMPT=0 git clone \"${{ github.server_url }}/${{ github.repository }}.git\" a; then exit 1; fi\n      - run: |\n          auth=$(printf 'x-access-token:%s' \"${{ github.token }}\" | base64 -w0)\n          git -c \"http.extraHeader=Authorization: basic $auth\" clone \"${{ github.server_url }}/${{ github.repository }}.git\" b\n          test -s b/README.md\n"
}
pub(crate) fn fact(f: &Fixture) -> Result<Fact> {
    let repo = "/api/v1/repos/fixture/clone-check";
    f.api(
        "POST",
        "/api/v1/user/repos",
        &json!({"name":"clone-check","private":true,"auto_init":true,"default_branch":"main"}),
    )?;
    f.api("PATCH", repo, &json!({"has_actions":true}))?;
    f.put(repo, "main", ".forgejo/workflows/clone.yaml", workflow())?;
    let head =
        f.get(&format!("{repo}/branches/main"))?["commit"]["id"].as_str().ok_or("clone head missing")?.to_owned();
    let mut success = false;
    for _ in 0..120 {
        let row = f.get(&format!("{repo}/commits/{head}/status"))?;
        if row["state"] == "success" {
            success = true;
            break;
        }
        if row["state"] == "failure" || row["state"] == "error" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    let mut facts = Vec::new();
    add(&mut facts, crate::registration::NAMES[9], success, "clone job did not succeed in 60 seconds");
    Ok(facts.pop().expect("one clone fact"))
}
