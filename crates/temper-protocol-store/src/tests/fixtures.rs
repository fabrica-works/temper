use skein_lib::{Duration, List, Rng};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_client as client;
pub const REPO: client::api::Repository = client::api::Repository { forge: 1, repository: 2 };

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

pub fn named_branch() -> top::Name {
    top::Name { forge: 1, repository: 2, what: top::What::Branch(Box::new([Box::from([0, 1, 255]), Box::from([1])])) }
}
pub fn adopted(protection: top::Protection) -> top::Stored {
    top::Stored::Repository(top::Repository {
        project: 3,
        home: true,
        provider: REPO,
        host: Box::from(b"forge".as_slice()),
        owner: Box::from(b"owner".as_slice()),
        name: Box::from(b"repo".as_slice()),
        prefix: Box::from(b"temper/".as_slice()),
        role: top::Role::Owned,
        ci: true,
        checks: Box::new([1, 2, 3, 4, 5, 6, 7, 8]),
        kinds: top::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection,
        settings: client::api::Settings {
            default_branch: Box::from(b"main".as_slice()),
            merge: true,
            squash: true,
            rebase: true,
        },
    })
}
fn initial_rows(name: &top::Name) -> [(&'static str, top::Stored); 8] {
    [
        ("repository unknown", adopted(top::Protection::Unknown)),
        ("repository absent", adopted(top::Protection::Absent)),
        (
            "repository rule",
            adopted(top::Protection::Rule(client::api::Protection {
                branch: Box::from(b"main".as_slice()),
                contexts: Box::new([Box::from(b"test".as_slice()), Box::from(b"build".as_slice())]),
                approvals: 2,
                dismiss_stale: true,
            })),
        ),
        (
            "names",
            top::Stored::Names {
                task: 1,
                resources: Box::new([
                    name.clone(),
                    top::Name { forge: 1, repository: 2, what: top::What::Repository },
                    top::Name { forge: 1, repository: 2, what: top::What::Pull(3) },
                    top::Name { forge: 1, repository: 2, what: top::What::Issue(4) },
                ]),
            },
        ),
        (
            "branch head",
            top::Stored::BranchHead(top::BranchHead { name: name.clone(), commit: [255; 32], owned: true }),
        ),
        (
            "pull state",
            top::Stored::PullState(top::PullState {
                name: name.clone(),
                head: [255; 32],
                ci: client::api::Ci::Passed,
                state: client::api::State::Closed,
            }),
        ),
        ("ci", top::Stored::Ci(top::CiState { repository: REPO, head: [255; 32], status: client::api::Ci::Failed })),
        ("landed", top::Stored::Landed { commit: [255; 32], task: u64::MAX }),
    ]
}
fn add_holds(name: &top::Name, rows: &mut List<(&'static str, top::Stored)>) {
    for (label, writer, change) in [
        (
            "run branch",
            top::Writer::Run { task: 1, attempt: 2 },
            top::DriftChange::Branch { expected: [0; 32], observed: Some([255; 32]) },
        ),
        ("lost pull", top::Writer::Lost { task: 1, attempt: 2 }, top::DriftChange::PullClosed { number: 3 }),
        (
            "entry retargeted",
            top::Writer::Entry(4),
            top::DriftChange::Retargeted {
                number: 3,
                before: Box::from(b"main".as_slice()),
                after: Box::from(b"next".as_slice()),
            },
        ),
    ] {
        rows.push((
            label,
            top::Stored::Hold(top::Hold {
                name: name.clone(),
                task: 1,
                writer: Some(writer),
                drift: Some(top::Drift { at: skein_lib::Wall::from_nanos(99), change }),
            }),
        ))
        .expect("fixture rows fit");
    }
}
fn add_subscriptions(_name: &top::Name, rows: &mut List<(&'static str, top::Stored)>) {
    for (label, topic) in [
        ("landings", top::Topic::Landings { repository: REPO, branch: Box::from(b"main".as_slice()) }),
        ("ci topic", top::Topic::Ci { repository: REPO, head: [255; 32] }),
        ("pull", top::Topic::Pull { repository: REPO, number: 3 }),
        ("participation", top::Topic::Participation { repository: REPO, number: 4 }),
    ] {
        rows.push((
            label,
            top::Stored::Subscription(top::Subscriber {
                task: 1,
                number: 2,
                topic,
                own_change: Some(3),
                goal_tasks: Some(Box::new([1, 2, 3, 4, 5, 6, 7, 8])),
                paths: Box::new([
                    Box::from(b"a".as_slice()),
                    Box::from(b"b".as_slice()),
                    Box::from(b"c".as_slice()),
                    Box::from(b"d".as_slice()),
                ]),
            }),
        ))
        .expect("fixture rows fit");
    }
}
fn add_releases(name: &top::Name, rows: &mut List<(&'static str, top::Stored)>) {
    for (label, ending) in [
        ("done", top::ReleaseEnding::Done),
        ("failed", top::ReleaseEnding::Failed),
        ("cancelled", top::ReleaseEnding::Cancelled),
    ] {
        rows.push((
            label,
            top::Stored::Release(top::ReleaseRow {
                task: 1,
                root: 2,
                ending,
                close_pull: Some(3),
                pending: Some((4, Some(name.clone()))),
                failed: true,
                effects_settled: true,
                releasing: true,
            }),
        ))
        .expect("fixture rows fit");
    }
}
pub fn names_holds_and_topics() -> Box<[(&'static str, top::Stored)]> {
    let name = named_branch();
    let mut rows = List::with_capacity(18);
    for row in initial_rows(&name) {
        rows.push(row).expect("fixture rows fit");
    }
    add_holds(&name, &mut rows);
    add_subscriptions(&name, &mut rows);
    add_releases(&name, &mut rows);
    rows.into_boxed()
}

pub fn entry(write: client::api::Write, condition: client::Condition) -> client::Entry {
    client::Entry {
        number: 3,
        task: 4,
        repository: REPO,
        effect: client::Effect { write, condition },
        start: Some(client::Position { at: skein_lib::Time::from_nanos(11), comment: 12, review: 13, review_page: 14 }),
        attempt: Some(client::Attempt {
            sent: skein_lib::Time::from_nanos(15),
            deadline: skein_lib::Time::from_nanos(16),
            wall: skein_lib::Wall::from_nanos(17),
            expires: skein_lib::Wall::from_nanos(18),
        }),
        failures: 19,
    }
}
pub fn writes() -> [(&'static str, client::api::Write); 13] {
    use client::api::{Check, Verdict, Write};
    [
        ("issue", Write::CreateIssue { key: Box::new([0, 1]), title: Box::new([2]), body: Box::new([3]) }),
        (
            "pull",
            Write::OpenPull { title: Box::new([1]), body: Box::new([2]), head: Box::new([3]), base: Box::new([4]) },
        ),
        ("post", Write::Post { number: 1, key: Box::new([2]), body: Box::new([3]) }),
        (
            "review",
            Write::Review { number: 1, key: Box::new([2]), verdict: Verdict::RequestChanges, body: Box::new([3]) },
        ),
        ("edit", Write::Edit { number: 1, title: Some(Box::new([2])), body: Some(Box::new([3])) }),
        ("reviewers", Write::SetReviewers { number: 1, reviewers: Box::new([2, 3]) }),
        ("close", Write::Close { number: 1 }),
        ("reopen", Write::Reopen { number: 2 }),
        ("merge", Write::Merge { number: 1, head: [3; 32] }),
        ("update", Write::Update { number: 1 }),
        ("status", Write::Status { commit: [1; 32], context: Box::new([2]), check: Check::Failed }),
        ("create branch", Write::CreateBranch { branch: Box::new([2]), commit: [3; 32] }),
        ("delete branch", Write::DeleteBranch { branch: Box::new([2]) }),
    ]
}
pub fn conditions() -> [client::Condition; 4] {
    [
        client::Condition::None,
        client::Condition::Merge { base: Box::new([0, 255]) },
        client::Condition::Update { head: [2; 32], base: [3; 32] },
        client::Condition::Review { head: [4; 32] },
    ]
}

pub fn errors() -> [(client::api::Error, u8); 18] {
    use client::api::Error;
    [
        (Error::Unavailable, 0),
        (Error::Timeout, 1),
        (Error::RateLimited { after: Duration::from_secs(1) }, 2),
        (Error::Forbidden, 3),
        (Error::Missing, 4),
        (Error::MissingJob, 5),
        (Error::TooLarge, 6),
        (Error::Empty, 7),
        (Error::Full, 8),
        (Error::Exists, 9),
        (Error::NothingToMerge, 10),
        (Error::Closed, 11),
        (Error::Stale, 12),
        (Error::Conflict, 13),
        (Error::Protected, 14),
        (Error::Refused, 15),
        (Error::InvalidAnswer, 16),
        (Error::Busy, 17),
    ]
}
pub fn named_call(answer: top::CallPayload) -> top::Stored {
    top::Stored::Call { key: top::NamedCall { task: 1, attempt: 2, completion: 3, position: 4 }, answer }
}
pub fn payloads() -> List<top::CallPayload> {
    use client::{Made, Outcome};
    let mut rows = List::with_capacity(80);
    for made in [
        Made::Created(1),
        Made::Commented(2),
        Made::Reviewed(3),
        Made::Merged([4; 32]),
        Made::Updated([5; 32]),
        Made::Branch([6; 32]),
        Made::Set,
    ] {
        for found in [false, true] {
            rows.push(payload(Some(Outcome::Made { made, found }))).expect("fixture room");
        }
        rows.push(payload(Some(Outcome::Raced { made, why: client::api::Error::Stale }))).expect("fixture room");
    }
    for outcome in [
        Some(Outcome::Failed(client::api::Error::Timeout)),
        Some(Outcome::Uncertain),
        Some(Outcome::Held),
        Some(Outcome::Withdrawn),
        None,
    ] {
        rows.push(payload(outcome)).expect("fixture room");
    }
    for (error, _) in errors() {
        rows.push(top::CallPayload::Refused(error)).expect("fixture room");
        rows.push(top::CallPayload::Read(Box::new(Err(error)))).expect("fixture room");
    }
    rows
}
fn payload(outcome: Option<client::Outcome>) -> top::CallPayload {
    top::CallPayload::Effect { entry: 8, deadline: skein_lib::Wall::from_nanos(123), outcome }
}
pub fn answers_without_item_pieces() -> [client::api::Answer; 17] {
    use client::api::{Answer, BranchCreation, JobAttempt, Permission, Protection, Settings};
    [
        Answer::Pull(pull()),
        Answer::Commit([9; 32]),
        Answer::Branches(Box::new([Box::new([0, 255]), Box::new([1])])),
        Answer::Job {
            attempt: JobAttempt { head: [10; 32], run: 11, job: 12, attempt: 13 },
            log: Box::new([14]),
            truncated: true,
        },
        Answer::File { head: [15; 32], path: Box::new([16]), bytes: Box::new([17]), truncated: true },
        Answer::Protection(Some(Protection {
            branch: Box::new([18]),
            contexts: Box::new([Box::new([19]), Box::new([])]),
            approvals: 20,
            dismiss_stale: true,
        })),
        Answer::Protection(None),
        Answer::Settings(Settings { default_branch: Box::new([21]), merge: true, squash: false, rebase: true }),
        Answer::Permission(Permission::None),
        Answer::Permission(Permission::Admin),
        Answer::Created(22),
        Answer::Commented(23),
        Answer::Reviewed(24),
        Answer::Merged([25; 32]),
        Answer::Branch(BranchCreation::Created),
        Answer::Branch(BranchCreation::Exists),
        Answer::Done,
    ]
}
pub fn encoded(value: &crate::v1::Error) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured error length"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

pub fn summary() -> client::api::Summary {
    client::api::Summary {
        labels: Box::new([Box::new([0, 255]), Box::new([1])]),
        number: 2,
        kind: client::api::Kind::Pull,
        state: client::api::State::Closed,
        author: 3,
        key: Some(Box::new([4])),
        title: Box::new([5]),
        body: Box::new([6]),
        updated: skein_lib::Time::from_nanos(7),
    }
}
pub fn comment() -> client::api::Comment {
    client::api::Comment {
        id: 1,
        provenance: client::api::Provenance::Revised,
        author: 2,
        created: skein_lib::Time::from_nanos(3),
        revision: 4,
        key: Some(Box::new([5])),
        body: Box::new([6]),
    }
}
pub fn review() -> client::api::Review {
    client::api::Review {
        id: 1,
        provenance: client::api::Provenance::Unknown,
        revision: 2,
        author: 3,
        verdict: client::api::Verdict::Comment,
        commit: [4; 32],
        key: Some(Box::new([5])),
        body: Box::new([6]),
        at: skein_lib::Time::from_nanos(7),
        official: true,
    }
}
pub fn item_answers() -> [client::api::Answer; 3] {
    use client::api::Answer;
    [
        Answer::Items { items: Box::new([summary(), summary()]), more: true, now: skein_lib::Time::from_nanos(9) },
        Answer::Item { item: summary(), comments: Box::new([comment(), comment()]), more: true },
        Answer::Reviews { reviews: Box::new([review(), review()]), more: true },
    ]
}
pub fn encoded_row(row: &crate::v1::Row) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).expect("measured row length"));
    row.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn item_piece_row(items: Box<[u8]>) -> crate::v1::Row {
    use crate::v1 as w;
    let answer = w::Answer::Items(
        w::ItemsAnswer::new(&w::CEILINGS, w::ItemsAnswerParts { items, more: false, now: 1 }).expect("piece ceiling"),
    );
    let result = w::ReadResult::Answer(Box::new(
        w::AnswerValue::new(&w::CEILINGS, w::AnswerValueParts { answer }).expect("answer ceiling"),
    ));
    let answer =
        w::CallPayload::Read(w::CallRead::new(&w::CEILINGS, w::CallReadParts { result }).expect("call ceiling"));
    let key = w::NamedCall::new(&w::CEILINGS, w::NamedCallParts { task: 1, attempt: 2, completion: 3, position: 4 })
        .expect("key ceiling");
    let value = w::Value::Call(w::Call::new(&w::CEILINGS, w::CallParts { key, answer }).expect("row ceiling"));
    w::Row::new(&w::CEILINGS, w::RowParts { value }).expect("row ceiling")
}

pub fn status() -> client::api::Status {
    client::api::Status {
        author: 1,
        at: skein_lib::Time::from_nanos(2),
        context: Box::new([3]),
        check: client::api::Check::Passed,
        description: Box::new([4]),
        url: Box::new([5]),
        job: Some(client::api::JobAttempt { head: [6; 32], run: 7, job: 8, attempt: 9 }),
    }
}
pub fn file() -> client::api::File {
    client::api::File { path: Box::new([1]), before: Some(Box::new([2])), after: Some(Box::new([3])) }
}
pub fn remaining_answers() -> [client::api::Answer; 6] {
    use client::api::{Answer, Ci, Collaborator, Permission, Remark};
    [
        Answer::Statuses { ci: Ci::Failed, statuses: Box::new([status(), status()]), more: true },
        Answer::Remarks {
            remarks: Box::new([Remark { id: 1, author: 2, path: Box::new([3]), line: 4, body: Box::new([5]) }]),
            more: true,
        },
        Answer::PullFiles {
            head: [6; 32],
            files: Box::new([
                file(),
                client::api::File { path: Box::new([7]), before: None, after: Some(Box::new([8])) },
            ]),
            more: true,
        },
        Answer::Compare {
            before: [9; 32],
            after: [10; 32],
            contains_before: true,
            files: Box::new([
                file(),
                client::api::File { path: Box::new([11]), before: Some(Box::new([12])), after: None },
            ]),
            commits: Box::new([[13; 32], [14; 32]]),
        },
        Answer::Checks(Box::new([status(), client::api::Status { job: None, ..status() }])),
        Answer::Collaborators {
            collaborators: Box::new([
                Collaborator { user: 15, permission: Permission::Read },
                Collaborator { user: 16, permission: Permission::Write },
            ]),
            more: true,
        },
    ]
}

pub fn pull() -> client::api::Pull {
    use client::api::{Ci, State};
    client::api::Pull {
        reviewers: Box::new([1, 2]),
        number: 3,
        state: State::Closed,
        head: Box::new([4]),
        base: Box::new([5]),
        commit: [6; 32],
        base_commit: Some([7; 32]),
        merged: Some([8; 32]),
        mergeable: true,
        ci: Ci::Passed,
    }
}

pub fn live(what: client::What) -> client::LiveRecord {
    client::LiveRecord {
        watch: client::Watch { resource: client::Resource { repository: REPO, what }, participating: true },
        position: entry(client::api::Write::Close { number: 1 }, client::Condition::None)
            .start
            .expect("fixture position"),
        listed: skein_lib::Time::from_nanos(2),
        comment_after: 3,
        comment_page: 4,
        comment_scanning: true,
        comment_repair: false,
        review_page: 5,
        comments: Box::new([client::Delivery { id: 6, revision: 7 }]),
        reviews: Box::new([client::Delivery { id: 8, revision: 9 }]),
        cached: Box::new(client::Cached { item: Some(summary()), pull: Some(pull()), tip: Some([10; 32]) }),
        pushed: Some([11; 32]),
        echoes: Box::new([client::Echo::Comment(12), client::Echo::Review(13)]),
    }
}
pub fn client_rows() -> List<top::Stored> {
    let mut rows = List::with_capacity(6);
    for what in [
        client::What::Repository,
        client::What::Branch(Box::new([0, 255])),
        client::What::Pull(1),
        client::What::Issue(2),
    ] {
        rows.push(top::Stored::Client(client::Stored::Live(live(what)))).expect("fixture room");
    }
    for mark in [None, Some(skein_lib::Time::from_nanos(1))] {
        rows.push(top::Stored::Client(client::Stored::Repository(client::RepositoryRecord {
            repository: REPO,
            mark,
            clock: skein_lib::Time::from_nanos(2),
        })))
        .expect("fixture room");
    }
    rows
}

pub fn change_row(state: temper_engine_domain_forge_change::State) -> top::ChangeRow {
    use temper_engine_domain_forge_change::{
        Change, Delegate, EffectResult, Freshness, Gate, GateKind, GateReport, Hold, Status,
    };
    top::ChangeRow {
        task: 1,
        repository: REPO,
        branch: Box::new([2]),
        base: Box::new([3]),
        title: Box::new([4]),
        body: Box::new([5]),
        priority: -6,
        change: Change {
            task: 1,
            state,
            gates: Box::new([Gate {
                number: 7,
                kind: GateKind::Check,
                blocking: true,
                freshness: Freshness::Clean,
                eager: true,
            }]),
            clean: Box::new([[8; 32], [9; 32]]),
            repairs: 10,
            resolutions: 11,
            updates: 12,
            since: skein_lib::Wall::from_nanos(13),
            ready_since: Some(skein_lib::Wall::from_nanos(14)),
            owns_turn: true,
            last_head: Some([15; 32]),
        },
        pull: Some(16),
        pending: Some(17),
        effect: EffectResult::Made,
        delegate: Some((18, Delegate::Gate { number: 19, head: [20; 32] })),
        delegate_status: Status::Passed,
        verdicts: Box::new([GateReport { number: 21, head: [22; 32], status: Status::Failed }]),
        gate_remarks: Box::new([top::GateRemark { number: 23, head: [24; 32], words: Box::new([25]) }]),
        drift: Some(Hold::BranchMoved),
        base_repair: true,
        queue_repair: Some(26),
        queue_repairs: 27,
    }
}
pub fn change_states() -> List<temper_engine_domain_forge_change::State> {
    use temper_engine_domain_forge_change::{Hold, Repair, State};
    let mut states = List::with_capacity(17);
    for state in [
        State::Producing { requested: true },
        State::Opening { requested: false },
        State::Recreating { head: [1; 32] },
        State::Reopening { requested: true, retarget: false },
        State::Checking { head: [2; 32] },
        State::Gating { head: [3; 32], asked: Box::new([4, 5]) },
        State::Queued { head: [6; 32], ready_since: skein_lib::Wall::from_nanos(7) },
        State::First { head: [8; 32], ready_since: skein_lib::Wall::from_nanos(9) },
        State::Updating { from: [10; 32], base: [11; 32], ready_since: skein_lib::Wall::from_nanos(12) },
        State::Resolving { requested: true, base: [13; 32] },
        State::Repairing { requested: true, why: Repair::Gate(14) },
        State::Landing { head: [15; 32] },
        State::Landed { merge: [16; 32] },
        State::Repairing { requested: false, why: Repair::Ci },
        State::Repairing { requested: true, why: Repair::Semantic },
    ] {
        states.push(state).expect("fixture room");
    }
    let held = State::Held { was: Box::new(State::Checking { head: [17; 32] }), why: Hold::Stalled };
    states.push(held.clone()).expect("fixture room");
    states.push(State::Held { was: Box::new(held), why: Hold::BranchMissing }).expect("fixture room");
    states
}
