//! The process tier's isolated Forgejo, credentials and landing scripts.
use super::{Forge, Scratch, owned, write_private};
use serde_json::{Value, json};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use temper_throwaway_forgejo::{Instance, Outgoing, Token};
pub type BranchHead = (Box<[u8]>, Box<[u8]>);
// A created member owns all ten grants, their segment boxes and executor kinds.
const MEMBER_BYTES: u32 = 2048;

pub struct Throwaway {
    instance: Instance,
    forgejo: PathBuf,
    runner: owned::Running,
    person: Token,
    repository: u64,
}
impl Throwaway {
    #[must_use]
    pub fn start(scratch: &Scratch) -> Self {
        let address = owned::tier_configuration(scratch.path());
        let running = owned::start(scratch.path());
        let began = skein_shell::Clock::new().now().now;
        let instance = owned::attach(scratch.path(), address.port(), running);
        stage("attach", began);
        let began = skein_shell::Clock::new().now().now;
        let mut runner = owned::run(scratch.path(), "runner");
        runner.declared();
        stage("runner", began);
        let began = skein_shell::Clock::new().now().now;
        let reconciled = owned::reconcile(scratch.path(), 1);
        assert_eq!(reconciled.status.code(), Some(0), "{}", String::from_utf8_lossy(&reconciled.stderr));
        assert!(
            skein_shell::Clock::new().now().now.saturating_since(began) < super::WITHIN,
            "reconcile within its bound"
        );
        stage("reconcile", began);
        let person = person_token(&instance);
        let repository = repository(&instance, &person);
        Self { instance, forgejo: owned::binaries().0, runner, person, repository }
    }

    #[must_use]
    pub fn forge(&self) -> Forge<'_> {
        Forge::Throwaway {
            forgejo: &self.forgejo,
            api: SocketAddr::from(([127, 0, 0, 1], self.instance.port())),
            git: self.instance.base(),
        }
    }
    fn get(&self, suffix: &str) -> temper_throwaway_forgejo::Exchange {
        self.instance
            .exchange(&Outgoing::get(&format!("/api/v1/repos/acme/widget/{suffix}"), Some(&self.person)))
            .expect("throwaway forge answer")
    }
    #[must_use]
    pub fn merged_pull(&self) -> Option<u64> {
        let response = self.get("pulls?state=closed&limit=2");
        assert_eq!(response.status, 200, "closed pull list");
        let value: Value = serde_json::from_slice(&response.body).expect("pull JSON");
        value.as_array().expect("pull list").iter().find_map(|pull| {
            (pull["merged"].as_bool() == Some(true)).then(|| pull["number"].as_u64().expect("pull number"))
        })
    }
    #[must_use]
    pub fn job_state(&self, pull: u64) -> String {
        let response = self.get(&format!("pulls/{pull}"));
        assert_eq!(response.status, 200, "pull head");
        let pull: Value = serde_json::from_slice(&response.body).expect("pull JSON");
        let head = pull["head"]["sha"].as_str().expect("pull head");
        let response = self.get(&format!("commits/{head}/status"));
        assert_eq!(response.status, 200, "job status");
        let status: Value = serde_json::from_slice(&response.body).expect("status JSON");
        status["state"].as_str().expect("job state").into()
    }
    #[must_use]
    pub fn main_readme(&self) -> Vec<u8> {
        let response = self.get("raw/README.md?ref=main");
        assert_eq!(response.status, 200, "main README");
        response.body
    }
    #[must_use]
    pub fn branch_head(&self, branch: &[u8]) -> Option<Box<[u8]>> {
        let branch = std::str::from_utf8(branch).expect("branch words");
        let response = self.get(&format!("branches/{branch}"));
        if response.status == 404 {
            return None;
        }
        assert_eq!(response.status, 200, "branch answer");
        let value: Value = serde_json::from_slice(&response.body).expect("branch JSON");
        Some(value["commit"]["id"].as_str().expect("branch commit").as_bytes().into())
    }
    #[must_use]
    pub fn branch_heads(&self) -> Vec<BranchHead> {
        let response = self.get("branches?limit=8");
        assert_eq!(response.status, 200, "branch list");
        let value: Value = serde_json::from_slice(&response.body).expect("branches JSON");
        value
            .as_array()
            .expect("branches")
            .iter()
            .map(|branch| {
                (
                    branch["name"].as_str().expect("branch name").as_bytes().into(),
                    branch["commit"]["id"].as_str().expect("branch commit").as_bytes().into(),
                )
            })
            .collect()
    }
    #[must_use]
    pub fn failure_state(&self) -> String {
        let response = self.get("pulls?state=all&limit=8");
        assert_eq!(response.status, 200, "all pull list");
        let pulls: Value = serde_json::from_slice(&response.body).expect("pull JSON");
        let response = self.get("branches?limit=8");
        assert_eq!(response.status, 200, "branch list");
        let branches: Value = serde_json::from_slice(&response.body).expect("branches JSON");
        let names: Vec<_> = branches
            .as_array()
            .expect("branches")
            .iter()
            .map(|branch| branch["name"].as_str().expect("branch name"))
            .collect();
        format!(
            "{} pulls, merged={}, branches={names:?}, main edited={}",
            pulls.as_array().expect("pulls").len(),
            self.merged_pull().is_some(),
            self.main_readme().ends_with(super::EDIT)
        )
    }
    pub fn stop(self) {
        self.runner.stop();
        self.instance.stop().expect("owned Forgejo stopped");
    }
    #[must_use]
    pub fn repository(&self) -> u64 {
        self.repository
    }
}

pub fn configuration(directory: &Path, forge: &Forge<'_>, value: &mut Value) {
    let Forge::Throwaway { api, forgejo, .. } = forge else { return };
    let mut owned = owned::Owned::tier();
    owned.forgejo = *api;
    owned.forgejo_binary = forgejo.into();
    let path = directory.join("config.json");
    if let Ok(bytes) = fs::read(&path) {
        let previous: Value = serde_json::from_slice(&bytes).expect("previous configuration");
        for (name, listener) in [("clients", &mut owned.clients), ("webhooks", &mut owned.webhooks)] {
            if let Some(address) = previous["application"][name]["listener"]["socket"].as_str() {
                *listener = address.parse().expect("listener address");
            }
        }
    }
    let limits = value["application"]["limits"].take();
    value["application"] = owned::application(&owned);
    value["application"]["limits"] = limits;
    let grant = |kind| json!({"connector":1,"kind":kind,"pattern":{"segments":["forge",api.to_string()],"last":"open","value":"acme"}});
    let grants =
        json!([grant(1), grant(2), grant(3), grant(4), grant(7), grant(8), grant(10), grant(11), grant(12), grant(13)]);
    let authority = json!({"tools":1,"grants":grants,"delegation":{"kinds":[{"kind":"charter","number":1},{"kind":"procedure","number":2}],"tasks":8,"depth":4},"budget":{"spend":100,"deadline_unix_seconds":null},"notes":0,"note_resources":[]});
    value["engine"]["rules"]["ceiling"] = authority.clone();
    value["engine"]["rules"]["maximum_run_spend"] = 10.into();
    value["engine"]["projects"][0]["policy"]["ceiling"] = authority.clone();
    value["engine"]["projects"][0]["policy"]["roles"][0]["authority"] = authority;
    value["engine"]["projects"][0]["policy"]["roles"][0]["authority"]["budget"]["spend"] = 40.into();
    value["engine"]["projects"][0]["policy"]["projections"] = grants;
    let limits = value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides");
    for (name, size) in [
        ("people.operations", 5),
        ("tasks.tasks", 8),
        ("documents.lines", 2),
        ("documents.chips", 2),
        ("tasks.project_tasks", 8),
        ("tasks.tree_tasks", 8),
        ("tasks.charters", 1),
        ("tasks.message_bytes", 4094),
        ("people.words", 512),
        ("tasks.authority_bytes", 512),
        ("core.call_answer_bytes", 2048),
        ("tasks.spec_bytes", 512),
        ("tasks.result_bytes", 128),
        ("tasks.delegates", 4),
        ("tasks.parameters", 5),
        ("tasks.contract_choices", 2),
        ("tasks.authority_grants", 12),
        ("tasks.authority_segments", 9),
        ("tasks.hold_segments", 9),
        ("tasks.saved_resources", 4),
        ("authority.grants", 12),
        ("authority.segments", 9),
        ("authority.roles", 1),
        ("authority.requirements", 2),
        ("notes.scopes", 15),
        ("brief.briefs", 2),
        ("brief.sections", 4),
        ("fleet.attempts", 2),
        ("people.inbox_tasks", 16),
        ("people.pending", 4),
        ("people.waiters", 4),
        ("people.amendment_bytes", MEMBER_BYTES),
        ("core.policy_bytes", 4096),
    ] {
        limits.insert(name.into(), size.into());
    }
    value["workers"][0]["slots"] = 1.into();
    let limits = &mut value["workers"][0]["limits"]["overrides"];
    limits["hub.slots"] = 1.into();
    limits["hub.event_bytes"] = 4094.into();
    limits["tasks.message_bytes"] = 4096.into();
    let charter = &mut value["engine"]["charters"][0];
    for flag in ["inspect", "modify", "agents"] {
        charter[flag] = true.into();
    }
    charter["turns"] = 8.into();
    charter["time_ms"] = 120_000.into();
    charter["reserve_time_ms"] = 60000.into();
    let path = directory.join("agent.json");
    let mut agent: Value = serde_json::from_slice(&fs::read(&path).expect("agent configuration")).expect("agent JSON");
    agent["profile"]["declared"]["tool_payload"] = 4096.into();
    agent["profile"]["policy"]["max_turns"] = 8.into();
    agent["profile"]["policy"]["check_timeout"] = 1000.into();
    agent["endpoints"][0]["models"][0]["output"] = 8192.into();
    write_private(&path, &serde_json::to_vec(&agent).expect("agent JSON"));
    admit(value);
}

fn admit(value: &Value) {
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("engine document");
    let jig = jig_protocol_config::engine(&document, 0).expect("engine configuration");
    let mut own = temper_service::limits::profile(temper_service::limits::Profile::Small);
    for (name, number) in value["application"]["limits"]["overrides"].as_object().expect("own overrides") {
        temper_service::limits::apply(&mut own, name.as_bytes(), number.as_u64().expect("limit number"))
            .expect("application override");
    }
    let root = temper_service::limits::engine(&jig.limits, &own).expect("engine root admits the fixture");
    jig_protocol_store::sizes::check(
        &jig.limits.core,
        &temper_protocol_store::application(&root.forge).expect("application sizes"),
        &temper_engine_domain::journal_limits(&root.journal),
        &jig.limits.store.store,
        jig.limits.store.orphan_erases,
    )
    .expect("store admits the fixture");
}

#[must_use]
pub fn landing_scripts(answer: &[u8], forgejo: &Throwaway) -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let call = |name: &[u8], value: Value| Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(&value).expect("tool JSON").into_boxed_slice(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let Forge::Throwaway { api, .. } = forgejo.forge() else { panic!("throwaway forge") };
    let grant =
        |kind| json!({"connector":1,"kind":kind,"segments":["forge",api.to_string()],"terminal":"open","last":"acme"});
    let delegate = json!({"batch":[{"executor":{"kind":"procedure","connector":1,"code":2},"spec":{"words":"@coding Small fix","parameters":[{"name":1,"kind":"resource","connector":1,"resource":forgejo.repository()},{"name":2,"kind":"bytes","value":"main"}]},"contract":{"kind":"change","connector":1,"change_kind":1,"words":128},"authority":{"tools":1,"grants":[grant(2),grant(3),grant(4),grant(10),grant(11),grant(12)],"delegation":{"kinds":[{"kind":"agent","number":1}],"tasks":4,"depth":2},"budget":{"spend":10},"notes":0}}]});
    let original = String::from_utf8(forgejo.main_readme()).expect("README UTF-8");
    let changed = format!("{original}{}", std::str::from_utf8(super::EDIT).expect("edit UTF-8"));
    let mut initial = super::chat_script(answer);
    initial.cue = b"Hello".as_slice().into();
    Box::new([
        Script {
            cue: b"@forge_chat".as_slice().into(),
            turns: Box::new([
                call(b"delegate", delegate),
                Turn {
                    lines: Box::new([
                        Line::Text { text: b"Change requested".as_slice().into() },
                        Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                    ]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
            ]),
        },
        Script {
            cue: b"@coding Small fix".as_slice().into(),
            turns: Box::new([
                call(b"read", json!({"path":"README.md"})),
                call(b"edit", json!({"path":"README.md","edits":[{"old":original,"new":changed}]})),
                call(
                    b"finish",
                    json!({"form":"change","fields":{"title":"Update README","body":"Changed by temper."}}),
                ),
            ]),
        },
        Script {
            cue: b"Review this change at the named head".as_slice().into(),
            turns: Box::new([call(
                b"finish",
                json!({"form":"verdict","label":"1","text":"Checked","fields":{},"items":[]}),
            )]),
        },
        initial,
    ])
}

fn stage(name: &str, began: skein_lib::Time) {
    eprintln!(
        "stage {name}: {} ms",
        skein_shell::Clock::new().now().now.saturating_since(began).as_nanos() / 1_000_000
    );
}
fn request(instance: &Instance, token: &Token, method: &str, path: &str, body: &Value, status: u16) -> Value {
    let body = serde_json::to_vec(body).expect("request JSON");
    let response = instance.exchange(&Outgoing::json(method, path, Some(token), &body)).expect("Forgejo answer");
    assert_eq!(response.status, status, "{method} {path}");
    serde_json::from_slice(&response.body).expect("answer JSON")
}
fn person_token(instance: &Instance) -> Token {
    let began = skein_shell::Clock::new().now().now;
    let admin = &instance.admin().token;
    request(
        instance,
        admin,
        "PATCH",
        "/api/v1/admin/users/ada",
        &json!({"login_name":"ada","source_id":0,"must_change_password":false}),
        200,
    );
    let value = request(
        instance,
        admin,
        "POST",
        "/api/v1/admin/users/ada/tokens",
        &json!({"name":"person","scopes":["write:repository","write:organization"]}),
        201,
    );
    stage("person", began);
    Token::new(value["sha1"].as_str().expect("person token").into())
}
fn repository(instance: &Instance, person: &Token) -> u64 {
    let began = skein_shell::Clock::new().now().now;
    let value = request(
        instance,
        person,
        "POST",
        "/api/v1/orgs/acme/repos",
        &json!({"name":"widget","auto_init":true,"default_branch":"main","private":true}),
        201,
    );
    request(instance, person, "PATCH", "/api/v1/repos/acme/widget", &json!({"has_actions":true}), 200);
    for (path, content) in [("check.sh", "test -s README.md\n"), (".forgejo/workflows/check.yaml", WORKFLOW)] {
        let length = skein_lib::base64::encoded_len(
            skein_lib::base64::Alphabet::Standard,
            u32::try_from(content.len()).expect("workflow length"),
        )
        .expect("base64 length");
        let mut encoded = skein_lib::Writer::new(usize::try_from(length).expect("bounded length"));
        skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, content.as_bytes(), &mut encoded)
            .expect("bounded workflow");
        let encoded = encoded.finish();
        let content = std::str::from_utf8(&encoded).expect("base64 text");
        request(
            instance,
            person,
            "POST",
            &format!("/api/v1/repos/acme/widget/contents/{path}"),
            &json!({"branch":"main","message":"Add check","content":content}),
            201,
        );
    }
    stage("repository", began);
    value["id"].as_u64().expect("repository id")
}
const WORKFLOW: &str = r#"on: { push: { branches: ['temper/**'] } }
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: |
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone "${{ github.server_url }}/${{ github.repository }}.git" .
          git checkout "${{ github.sha }}" && sh check.sh
"#;
