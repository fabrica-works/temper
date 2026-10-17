//! The process tier's isolated Forgejo, credentials and landing scripts.
use super::{Forge, Scratch, owned, write_private};
use serde_json::{Value, json};
use std::{
    fs,
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
};
use temper_throwaway_forgejo::{Account, Forgejo, Init, Instance, Made, Outgoing, Settings, Token};
pub type BranchHead = (Box<[u8]>, Box<[u8]>);

pub struct Throwaway {
    instance: Instance,
    forgejo: PathBuf,
    acme: Account,
    bot: Account,
    widget: Made,
}
impl Throwaway {
    #[must_use]
    pub fn start(scratch: &Scratch) -> Self {
        let binary = Forgejo::from_env().expect("TEMPER_FORGEJO must name the pinned Forgejo");
        let port = TcpListener::bind("127.0.0.1:0").expect("free port").local_addr().expect("address").port();
        owned::configuration(
            scratch.path(),
            &owned::Names {
                forgejo: binary.binary(),
                runner: None,
                port,
                bot: "temper-bot",
                person: "ada",
                orgs: &["workshop"],
            },
        );
        let prepared = owned::prepare(scratch.path(), 1);
        assert_eq!(prepared.status.code(), Some(0), "{}", String::from_utf8_lossy(&prepared.stderr));
        let log = fs::File::create(scratch.path().join("server.log")).expect("server log");
        let child = owned::run(scratch.path(), "forgejo")
            .stdout(log.try_clone().expect("log clone"))
            .stderr(log)
            .spawn()
            .expect("owned Forgejo starts");
        let token = fs::read_to_string(scratch.path().join("forge-1/admin_token")).expect("admin token");
        let mut instance =
            Instance::attach(child, port, scratch.path(), Token::new(token.trim().into()), &Settings::small())
                .expect("owned Forgejo answers");
        let acme = instance.create_user("acme").expect("repository owner");
        let bot = instance.create_user("temper-bot").expect("forge user");
        let widget = instance.create_repository(&acme, "widget", Init::Readme).expect("initialized repository");
        instance.add_collaborator(&acme, "widget", &bot, "write").expect("bot may write");
        Self { instance, forgejo: binary.binary().into(), acme, bot, widget }
    }
    #[must_use]
    pub fn forge(&self) -> Forge<'_> {
        Forge::Throwaway {
            forgejo: &self.forgejo,
            api: SocketAddr::from(([127, 0, 0, 1], self.instance.port())),
            git: self.instance.base(),
            token: self.bot.token.value(),
            user: self.bot.id,
        }
    }
    fn get(&self, suffix: &str) -> temper_throwaway_forgejo::Exchange {
        self.instance
            .exchange(&Outgoing::get(&format!("/api/v1/repos/acme/widget/{suffix}"), Some(&self.acme.token)))
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
        self.instance.stop().expect("owned Forgejo stopped");
    }
    #[must_use]
    pub fn repository(&self) -> u64 {
        self.widget.id
    }
}

pub fn configuration(directory: &Path, forge: &Forge<'_>, value: &mut Value) {
    let Forge::Throwaway { api, git: _, token, user, forgejo } = forge else { return };
    owned::document(
        value,
        &owned::Names {
            forgejo,
            runner: None,
            port: api.port(),
            bot: "temper-bot",
            person: "ada",
            orgs: &["workshop"],
        },
    );
    write_private(&directory.join("forge-1/api_token"), token.as_bytes());
    write_private(&directory.join("forge-1/git_token"), token.as_bytes());
    write_private(&directory.join("forge-1/user"), format!("{user}\n").as_bytes());
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
    value["application"]["landing"]["projects"] = json!([{"project":1,"rules":[{"connector":1,"kind":4,"enforce":true,"pattern":{"segments":["forge",api.to_string(),"acme","widget","branch"],"last":"exact","value":"main"},"ci":false,"up_to_date":false,"gates":[],"approvals":[]}]}]);
    value["application"]["limits"]["overrides"]["root.brief.gather_ms"] = 5000.into();
    let limits = value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides");
    for (name, size) in [
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
        ("authority.requirements", 1),
        ("notes.scopes", 15),
        ("brief.briefs", 2),
        ("brief.sections", 4),
        ("fleet.attempts", 2),
        ("people.inbox_tasks", 16),
        ("people.pending", 4),
        ("people.waiters", 4),
        ("people.amendment_bytes", 1024),
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
