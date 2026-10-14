//! Native forms, LLM scripts and independent forge observations for repository stories.
use crate::{fixtures, world::Story};
use jig_fake_person::{Find, Step};
use jig_view::Role;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::Duration;
use std::collections::BTreeMap;
use temper_engine_domain as root;
use temper_fake_checkout::git::Remote;
thread_local! { static STORY: std::cell::Cell<Story> = const { std::cell::Cell::new(Story::Chat { words: b"Hello" }) }; }
pub(crate) fn set(story: Story) {
    STORY.with(|slot| slot.set(story));
}

pub(crate) fn person() -> Vec<Step<temper_client_domain::Address>> {
    let mut steps = vec![
        Step::Go {
            address: temper_client_domain::Address::Jig(jig_domain_client::address::Address::Project {
                project: 1,
                section: jig_domain_client::address::ProjectSection::Overview,
            }),
        },
        see(Find::named(Role::Link, b"Adopt a repository")),
        press(Role::Link, b"Adopt a repository"),
        see(Find::named(Role::TextField, b"Owner")),
        ty(Role::TextField, b"Owner", b"acme"),
        ty(Role::TextField, b"Name", b"app"),
        ty(Role::TextField, b"Landing branch", b"main"),
        press(Role::Check, b"Make it the project's home"),
        press(Role::Button, b"Adopt"),
        see(Find::text(b"Adopted acme/app on forge 1 as owned.")),
        Step::Go {
            address: temper_client_domain::Address::Jig(jig_domain_client::address::Address::Project {
                project: 1,
                section: jig_domain_client::address::ProjectSection::Goals,
            }),
        },
        see(Find::named(Role::TextArea, b"Goal specification")),
        ty(Role::TextArea, b"Goal specification", b"@plan Fix the adopted repository"),
        press(Role::Button, b"Charter 1"),
        ty(Role::TextField, b"Goal budget", b"100"),
        press(Role::Button, b"Set goal"),
        see(Find::named(Role::Button, b"Confirm")),
        press(Role::Button, b"Confirm"),
    ];
    steps.push(see(Find::text(b"@plan Fix the adopted repository")));
    steps
}
fn see(find: Find) -> Step<temper_client_domain::Address> {
    Step::See { find, within: Duration::from_secs(5) }
}
fn press(role: Role, name: &[u8]) -> Step<temper_client_domain::Address> {
    Step::Press { find: Find::named(role, name) }
}
fn ty(role: Role, name: &[u8], words: &[u8]) -> Step<temper_client_domain::Address> {
    Step::Type { find: Find::named(role, name), words: words.into() }
}
fn call(name: &[u8], arguments: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    }
}
pub(crate) fn provider(story: Story) -> skein_fake_peers::llm::Peer {
    if matches!(story, Story::Chat { .. }) {
        return fixtures::provider();
    }
    let grant = |kind| serde_json::json!({"connector":1,"kind":kind,"segments":["forge","127.0.0.1:3000"],"terminal":"open","last":"acme"});
    let delegate = serde_json::to_vec(&serde_json::json!({"batch":[{
        "executor":{"kind":"procedure","connector":1,"code":2},
        "spec":{"words":"@edit Fix","parameters":[{"name":1,"kind":"resource","connector":1,"resource":2},{"name":2,"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":1,"change_kind":1,"words":64},
        "authority":{"tools":3,"grants":[grant(3),grant(11)],"delegation":{"kinds":[{"kind":"agent","number":1}],"tasks":2,"depth":1},"budget":{"spend":40},"notes":0}
    }]})).expect("delegate JSON");
    let mut delegate: serde_json::Value = serde_json::from_slice(&delegate).expect("delegate");
    if matches!(story, Story::Merge) {
        delegate["batch"][0]["spec"]["parameters"]
            .as_array_mut()
            .expect("parameters")
            .push(serde_json::json!({"name":3,"kind":"bytes","value":std::str::from_utf8(fixtures::MERGE_BRANCH).expect("fixture branch")}));
    }
    let delegate = serde_json::to_vec(&delegate).expect("delegate JSON");
    let finish = call(
        b"finish",
        br#"{"form":"change","text":"","fields":{"title":"Fix the file","body":"The file is fixed."}}"#,
    );
    let edits = if matches!(story, Story::Merge) {
        vec![finish.clone()]
    } else {
        vec![call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish.clone()]
    };
    smith_agent_process_world::fake::configured(
        Box::new([
            Script {
                cue: b"@plan".as_slice().into(),
                turns: Box::new([call(b"delegate", &delegate), call(b"wait", b"{}"), call(b"wait", b"{}")]),
            },
            Script { cue: b"@edit".as_slice().into(), turns: edits.into_boxed_slice() },
            Script {
                cue: b"Resolve the merge conflict".as_slice().into(),
                turns: Box::new([call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish]),
            },
        ]),
        skein_llm::Credential {
            access_token: jig_channel_world::fixtures::TOKEN.into(),
            account_id: b"acc".as_slice().into(),
        },
        Duration::ZERO,
    )
}
pub(crate) fn forge(story: Story) -> temper_fake_git::Transport {
    let config = temper_fake_forge_domain::Config {
        limits: temper_fake_forge_domain::Limits { statuses: 8, ..temper_fake_git::fixtures::CONFIG.limits },
        ..temper_fake_git::fixtures::CONFIG
    };
    let mut domain = temper_fake_forge_domain::Domain::new(&config, fixtures::SEED);
    let base = temper_fake_forge_domain::repository(
        &mut domain,
        &config,
        temper_fake_forge_domain::api::Setup {
            id: 2,
            name: b"acme/app".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([temper_fake_forge_domain::api::File {
                path: b"conflict.txt".as_slice().into(),
                content: b"base\n".as_slice().into(),
            }]),
            labels: Box::new([]),
            checks: temper_fake_forge_domain::api::Checks {
                contexts: Box::new([b"check".as_slice().into()]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        },
    );
    temper_fake_forge_domain::grant(&mut domain, b"acme/app", 1, temper_fake_forge_domain::api::Permission::Write);
    let forge = std::rc::Rc::new(std::cell::RefCell::new(temper_fake_git::Forge::new(domain, config)));
    let mut transport = temper_fake_git::Transport::new(forge, &temper_fake_git::fixtures::table(b"acme/app", None));
    let identity = temper_fake_git::fixtures::identity();
    transport.authenticate(Some(&(identity.login, identity.token)));
    assert_eq!(
        transport.commit_tree(base, None, BTreeMap::from([(b"conflict.txt".to_vec(), b"topic\n".to_vec())]), b"topic"),
        fixtures::BRANCH_HEAD
    );

    transport
        .push(b"http://127.0.0.1:3000/acme/app", b"topic", fixtures::BRANCH_HEAD, None)
        .expect("the fixture's existing topic is already on the forge");
    if matches!(story, Story::Merge) {
        let mut forge = transport.forge.borrow_mut();
        let config = forge.config;
        let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::from_nanos(0), limits: config };
        let moved = temper_fake_forge_domain::advance(
            &mut forge.domain,
            &env,
            b"acme/app",
            b"main",
            b"conflict.txt",
            b"moved\n",
            2,
        )
        .expect("another party moves the base");
        assert_eq!(moved, fixtures::MOVED_BASE, "counted moved base");
    }
    while transport.forge.borrow_mut().domain.pop_observation().is_some() {}
    transport
}
pub(crate) fn prepare_merge_branch(transport: &temper_fake_git::Transport) {
    let mut forge = transport.forge.borrow_mut();
    let env = skein_lib::Env { now: forge.now, wall: skein_lib::Wall::EPOCH, limits: forge.config };
    let mut replies = skein_lib::Queue::with_capacity(temper_fake_forge_domain::MAX_OUT);
    let token = skein_lib::Token::new(1_u64 << 60);
    temper_fake_forge_domain::step(
        &mut forge.domain,
        &env,
        temper_fake_forge_domain::Event::Call {
            reply_to: skein_lib::ReplyTo::new(token),
            user: 1,
            repository: b"acme/app".as_slice().into(),
            op: temper_fake_forge_domain::api::Op::Git(temper_fake_forge_domain::api::Git::Create {
                branch: fixtures::MERGE_BRANCH.into(),
                commit: fixtures::BRANCH_HEAD,
            }),
        },
        &mut replies,
    );
    let mut created = false;
    for _ in 0..32 {
        while let Some(reply) = replies.pop() {
            match reply {
                temper_fake_forge_domain::Request::Reply { to, result } => {
                    let destination = to.into_token();
                    if destination != token {
                        forge.pending.push(temper_fake_forge_domain::Request::Reply {
                            to: skein_lib::ReplyTo::new(destination),
                            result,
                        });
                        continue;
                    }
                    assert_eq!(
                        result,
                        Ok(temper_fake_forge_domain::api::Answer::Branch(
                            temper_fake_forge_domain::api::Created::Created
                        )),
                        "the existing private merge branch"
                    );
                    created = true;
                }
                reply @ temper_fake_forge_domain::Request::Hook { .. } => forge.pending.push(reply),
            }
        }
        if created {
            break;
        }
        temper_fake_forge_domain::fire(&mut forge.domain, &env, &mut replies);
    }
    forge.domain.reclaim();
    assert!(created, "the existing private merge branch answered");
}
pub(crate) fn adopted(rows: &BTreeMap<root::Key, root::Record>) -> u32 {
    u32::try_from(rows.values().filter(|row| matches!(row,
        root::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Repository(_))
    )).count()).expect("adopted repositories")
}
pub(crate) fn configure(value: &mut serde_json::Value) {
    if STORY.with(|slot| matches!(slot.get(), Story::Chat { .. })) {
        return;
    }
    value["workers"][0]["slots"] = fixtures::AGENTS.into();
    value["workers"][0]["limits"]["overrides"]["hub.slots"] = fixtures::AGENTS.into();
    value["application"]["limits"]["overrides"]["checkout.conflicts"] = 1.into();
    value["application"]["limits"]["overrides"]["checkout.path_bytes"] = 64.into();
    let grants: Vec<_> = [3, 11]
        .into_iter()
        .map(|kind| {
            serde_json::json!({
                "connector":1,"kind":kind,"pattern":{"segments":["forge","127.0.0.1:3000"],"last":"open","value":"acme"}
            })
        })
        .collect();
    let kinds = serde_json::json!([{"kind":"charter","number":1},{"kind":"procedure","number":2}]);
    let mut ceiling = value["engine"]["rules"]["ceiling"].clone();
    ceiling["tools"] = 3.into();
    ceiling["grants"] = grants.into();
    ceiling["delegation"]["kinds"] = kinds;
    ceiling["delegation"]["depth"] = 3.into();
    value["engine"]["rules"]["ceiling"] = ceiling.clone();
    value["engine"]["rules"]["implies"] = [1, 2]
        .into_iter()
        .map(|implies| serde_json::json!({"connector":1,"kind":3,"implies":implies}))
        .collect::<Vec<_>>()
        .into();
    value["engine"]["rules"]["maximum_run_spend"] = 40.into();
    value["engine"]["projects"][0]["policy"]["ceiling"] = ceiling.clone();
    value["engine"]["projects"][0]["policy"]["roles"][0]["authority"] = ceiling;
    for name in ["inspect", "modify", "shell"] {
        value["engine"]["charters"][0][name] = true.into();
    }
    value["engine"]["charters"][0]["turns"] = 8.into();
    value["engine"]["charters"][0]["spend"] = 40.into();
    for (name, count) in [
        ("tasks.tasks", 4),
        ("tasks.project_tasks", 4),
        ("tasks.tree_tasks", 4),
        ("people.people", 4),
        ("people.inbox_tasks", 8),
        ("fleet.attempts", 4),
        ("fleet.turns", 16),
        ("brief.sections", 8),
        ("brief.brief_bytes", 4096),
        ("tasks.result_bytes", 128),
        ("tasks.parameters", 4),
        ("tasks.executor_kinds", 2),
        ("tasks.authority_grants", 4),
        ("tasks.authority_segments", 4),
        ("tasks.authority_bytes", 128),
        ("core.policy_bytes", 4096),
        ("authority.segments", 8),
        ("core.call_answer_bytes", 2048),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    let overrides = value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides");
    for name in [
        "tasks.funders",
        "tasks.dependencies",
        "tasks.charters",
        "people.amendment_bytes",
        "people.projects",
        "tasks.hold_bytes",
        "notes.entries_per_scope",
        "notes.recalled",
        "views.runs",
        "people.holdings",
        "people.waiters",
        "people.goals",
        "people.pending",
    ] {
        overrides.remove(name);
    }
}

/// The scripted shell's echo redirection acts on the inherited machine files.
pub(crate) struct Shell {
    jobs: std::collections::VecDeque<skein_io::kernel::Op>,
    owned: std::collections::BTreeSet<skein_io::kernel::Fd>,
    content: Box<[u8]>,
    pub(crate) target: Vec<u8>,
    pub(crate) written: Vec<u8>,
    code: u8,
    completions: skein_lib::Queue<skein_io::kernel::Complete>,
    submissions: skein_lib::Queue<skein_io::kernel::Submit>,
    pending: bool,
    finished: bool,
}
impl Shell {
    pub(crate) fn new(spawn: &skein_io::kernel::Spawn, inherited: &skein_world::Inherited) -> Self {
        use skein_io::kernel::{Op, OpenHow};
        assert_eq!(spawn.args[0].as_ref(), b"-c", "shell command invocation");
        let command = std::str::from_utf8(&spawn.args[1]).ok();
        let edit = command.and_then(|command| command.split_once('>')).and_then(|(echo, target)| {
            let text = echo.trim().strip_prefix("echo ")?;
            let target = target.trim().as_bytes();
            (!target.is_empty() && !target.contains(&b'/') && target != b"..").then_some((text, target))
        });
        let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("shell worker root").1;
        let mut jobs = std::collections::VecDeque::new();
        let (content, target, code) = if let Some((text, target)) = edit {
            let path = [spawn.dir.as_ref(), b"/", target].concat();
            jobs.push_back(Op::Open { root, path: path.into(), how: OpenHow::WriteNoFollow });
            ([text.as_bytes(), b"\n"].concat().into(), target.to_vec(), 0)
        } else {
            let error = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("shell stderr").1;
            jobs.push_back(Op::PipeWrite {
                fd: error,
                bytes: b"/bin/sh: command not found\n".as_slice().into(),
                from: 0,
            });
            (Box::new([]) as Box<[u8]>, Vec::new(), 127)
        };
        let mut owned: std::collections::BTreeSet<_> = inherited.roots.iter().map(|(_, fd)| *fd).collect();
        owned.extend(inherited.pipes.iter().map(|(_, fd)| *fd));
        owned.insert(inherited.signal);
        Self {
            jobs,
            owned,
            content,
            target,
            written: Vec::new(),
            code,
            completions: skein_lib::Queue::with_capacity(1),
            submissions: skein_lib::Queue::with_capacity(1),
            pending: false,
            finished: false,
        }
    }
}
impl skein_shell::Host for Shell {
    fn iterate(&mut self, _now: skein_lib::Time, _wall: skein_lib::Wall) {
        use skein_io::kernel::{Done, Op};
        while let Some(complete) = self.completions.pop() {
            self.pending = false;
            match (complete.kind, complete.result.expect("shell file operation")) {
                (Op::Open { .. }, Done::Fd(fd)) => {
                    self.owned.insert(fd);
                    self.jobs.push_back(Op::Truncate { fd, length: 0 });
                }
                (Op::Truncate { fd, .. }, Done::Nothing) => {
                    self.written.clear();
                    self.jobs.push_back(Op::Write { fd, bytes: self.content.clone(), from: 0, at: 0 });
                }
                (Op::Write { fd, bytes, from, .. }, Done::Count(count)) => {
                    self.written.extend_from_slice(&bytes[from as usize..(from + count) as usize]);
                    let from = from + count;
                    if (from as usize) < bytes.len() {
                        self.jobs.push_back(Op::Write { fd, bytes, from, at: u64::from(from) });
                    }
                }
                (Op::PipeWrite { fd, bytes, from }, Done::Count(count)) => {
                    if (from + count) as usize != bytes.len() {
                        self.jobs.push_back(Op::PipeWrite { fd, bytes, from: from + count });
                    }
                }
                (Op::Close { fd }, Done::Nothing) => {
                    self.owned.remove(&fd);
                }
                (op, done) => panic!("shell terminal {op:?}: {done:?}"),
            }
        }
        if !self.pending {
            if let Some(kind) = self.jobs.pop_front().or_else(|| self.owned.first().map(|fd| Op::Close { fd: *fd })) {
                self.pending = true;
                self.submissions.push(skein_io::kernel::Submit { op: skein_lib::Token::new(1), kind });
            } else {
                self.finished = true;
            }
        }
    }
    fn completions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: skein_lib::Time) -> bool {
        !self.completions.is_empty() || (!self.finished && !self.pending)
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<skein_lib::Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.finished
    }
    fn exit(&self) -> Option<skein_io::kernel::Exit> {
        self.finished.then_some(skein_io::kernel::Exit::Code(self.code))
    }
    fn worst_case(&self) -> u64 {
        8192
    }
    fn operations(&self) -> u32 {
        1
    }
}
