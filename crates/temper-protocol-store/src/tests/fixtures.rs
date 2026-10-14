use skein_lib::{Duration, List, Rng};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_client as client;

const CLIENT: client::Limits = client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    facts: 16,
};
pub const LIMITS: top::Limits = top::Limits {
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    collaborators: 8,
    landings: 8,
    judge_projects: 2,
    judge_criteria: 8,
    brief_sections: 32,
    brief_bytes: 4096,
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        repairs: 3,
        resolutions: 3,
        updates: 3,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
};

pub fn branch(segments: Box<[Box<[u8]>]>) -> top::Key {
    top::Key::Hold(top::Name { forge: 1, repository: 2, what: top::What::Branch(segments) })
}
fn random_bytes(random: &mut Rng) -> Box<[u8]> {
    let mut bytes = List::with_capacity(2);
    for _ in 0..random.below(3) {
        let byte = match random.below(3) {
            0 => 0,
            1 => 1,
            _ => 255,
        };
        bytes.push(byte).expect("bounded bytes");
    }
    bytes.into_boxed()
}
fn repository(random: &mut Rng) -> client::api::Repository {
    client::api::Repository {
        forge: u16::try_from(random.below(65_536)).expect("u16"),
        repository: u32::try_from(random.below(4_294_967_296)).expect("u32"),
    }
}
fn commit(random: &mut Rng) -> client::api::Commit {
    let mut commit = [0; 32];
    for byte in &mut commit {
        *byte = u8::try_from(random.below(256)).expect("byte");
    }
    commit
}
fn name(random: &mut Rng) -> top::Name {
    let repo = repository(random);
    let what = match random.below(4) {
        0 => top::What::Repository,
        1 => {
            let mut segments = List::with_capacity(2);
            for _ in 0..random.below(3) {
                segments.push(random_bytes(random)).expect("bounded segments");
            }
            top::What::Branch(segments.into_boxed())
        }
        2 => top::What::Pull(random.next_u64()),
        _ => top::What::Issue(random.next_u64()),
    };
    top::Name { forge: repo.forge, repository: repo.repository, what }
}
fn topic(random: &mut Rng) -> top::Topic {
    let repository = repository(random);
    match random.below(4) {
        0 => top::Topic::Landings { repository, branch: random_bytes(random) },
        1 => top::Topic::Ci { repository, head: commit(random) },
        2 => top::Topic::Pull { repository, number: random.next_u64() },
        _ => top::Topic::Participation { repository, number: random.next_u64() },
    }
}
fn client_key(random: &mut Rng) -> client::Key {
    let repository = repository(random);
    if random.below(2) == 0 {
        return client::Key::Repository(repository);
    }
    let what = match random.below(4) {
        0 => client::What::Repository,
        1 => client::What::Branch(random_bytes(random)),
        2 => client::What::Pull(random.next_u64()),
        _ => client::What::Issue(random.next_u64()),
    };
    client::Key::Live(client::Resource { repository, what })
}
pub fn key(random: &mut Rng) -> top::Key {
    let number = random.next_u64();
    match random.below(15) {
        0 => top::Key::Call(top::NamedCall {
            task: number,
            attempt: random.next_u64(),
            completion: u32::try_from(random.below(4_294_967_296)).expect("u32"),
            position: u32::try_from(random.below(4_294_967_296)).expect("u32"),
        }),
        1 => top::Key::ProposedEffect(number),
        2 => top::Key::Repository(repository(random)),
        3 => top::Key::Hold(name(random)),
        4 => top::Key::Names(number),
        5 => top::Key::Subscription { task: number, topic: topic(random) },
        6 => top::Key::BranchHead(name(random)),
        7 => top::Key::PullState(name(random)),
        8 => top::Key::Ci { repository: repository(random), head: commit(random) },
        9 => top::Key::Landed(commit(random)),
        10 => top::Key::Entry(number),
        11 => top::Key::Client(client_key(random)),
        12 => top::Key::Change(number),
        13 => top::Key::Issue(number),
        _ => top::Key::Release(number),
    }
}
pub fn core_keys() -> [jig_domain_core::Key; 4] {
    [
        jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment),
        jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(1)),
        jig_domain_core::Key::People(jig_domain_people::Key::Person(1)),
        jig_domain_core::Key::Notes(jig_domain_notes::Key::Entry { name: 1 }),
    ]
}
pub fn within(interval: &skein_kv::Range, tagged: &[u8]) -> bool {
    let start = jig_protocol_store::keys::application(&interval.start, u32::MAX).expect("tagged start");
    let end = match &interval.end {
        Some(end) => jig_protocol_store::keys::application(end, u32::MAX).expect("tagged end"),
        None => Box::from([3]),
    };
    tagged >= start.as_ref() && tagged < end.as_ref()
}
