//! The check's evidence and lines, without io (protocol/setup.md, 8.4).
use alloc::boxed::Box;
use skein_fake_forge_domain::promises::{Allowance, Bounds, Breach, Decision, Profile, Promise, Verdict, Why};
use skein_fake_forge_protocol::routes;
use skein_forge::api::{Commit, Repository, Write};
use skein_forge::forgejo::id::{ObjectFormat, ObjectId};
use skein_json::writer::{Encoder, Limits};
use skein_lib::{List, Map, Set, Writer};
use temper_engine_domain_forge as d;
use temper_protocol_store::records;

/// The judge's capacity: repository slots and each slot's item, comment,
/// branch, key, label and evidence tables (skein's fake-forge.md, 2.5).
/// The judge reserves them all at once, so memory grows with OBJECTS squared.
/// Sized for the runs the check judges (a first run or the QA loop: one
/// repository and tens of items, comments, branches and keys), not the fake's
/// limits, under which a run's history has no bound. A run past this capacity
/// is undecided: `judge full`.
pub const OBJECTS: u32 = 256;

/// Temper's client profile and independent merge decisions from its store.
#[derive(Debug)]
pub struct Evidence {
    pub profile: Profile,
    pub decisions: Box<[Decision]>,
    pub complete: bool,
}

/// Why the check cannot decide, including reasons from the promises' judge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    ObservationsLost,
    ObservationsMissing,
    JudgeFull,
    ObservationsUnforwarded,
    StoreUnreadable,
    ClientUnknown,
    DecisionsIncomplete,
}
impl Reason {
    /// The check's words in protocol/setup.md, 8.4.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::ObservationsLost => "observations lost",
            Self::ObservationsMissing => "observations missing",
            Self::JudgeFull => "judge full",
            Self::ObservationsUnforwarded => "observations unforwarded",
            Self::StoreUnreadable => "store unreadable",
            Self::ClientUnknown => "client unknown",
            Self::DecisionsIncomplete => "decisions incomplete",
        }
    }
}
/// `Truncated` and `NoDecisions` cannot arise when the binary feeds decisions
/// first and cuts the stream at a checkpoint (protocol/setup.md, "Its verdict").
impl From<Why> for Reason {
    fn from(why: Why) -> Self {
        reason(&why)
    }
}

fn reason(why: &Why) -> Reason {
    match why {
        Why::Lost(_) => Reason::ObservationsLost,
        Why::Truncated => Reason::ObservationsMissing,
        Why::Full => Reason::JudgeFull,
        Why::NoDecisions => Reason::DecisionsIncomplete,
        Why::Unforwarded => Reason::ObservationsUnforwarded,
    }
}

/// The word temper's client writes with (temper-service's forge.rs:12;
/// protocol/forge.md, section 4), framed as `skein_forge::marker` frames it.
const WORD: &[u8] = b"temper";

/// Read every row in temper's interval, with keys relative to jig's application
/// tag, as `temper_protocol_store::interval()` and `records::row` require.
#[expect(clippy::type_complexity, reason = "the reader supplies raw KV key/value pairs")]
pub fn evidence(forge: u16, rows: &[(Box<[u8]>, Box<[u8]>)], limits: &d::Limits) -> Result<Evidence, Reason> {
    let bounds = temper_protocol_store::bounds(limits).ok_or(Reason::StoreUnreadable)?;
    let count = u32::try_from(rows.len()).or(Err(Reason::StoreUnreadable))?;
    let mut repositories = Map::with_capacity(count);
    let mut changes = Map::with_capacity(count);
    let mut merges = List::with_capacity(count);
    let mut landed = List::with_capacity(count);
    let mut prefixes = Set::with_capacity(count);
    let mut client = None;
    let mut review = false;
    for (key, value) in rows {
        match records::row(key, value, &bounds).or(Err(Reason::StoreUnreadable))? {
            d::Stored::Writer(writer) => {
                if writer.forge == forge {
                    if let Some(author) = client
                        && author != writer.author
                    {
                        return Err(Reason::StoreUnreadable);
                    }
                    client = Some(writer.author);
                }
            }
            d::Stored::Repository(repository) => {
                if repository.provider.forge == forge {
                    review |= repository.kinds.review;
                    prefixes.insert(repository.prefix.clone()).expect("at most one prefix per input row");
                    drop(
                        repositories
                            .insert(repository.provider, repository)
                            .expect("at most one repository per input row"),
                    );
                }
            }
            d::Stored::Change(change) => {
                drop(changes.insert(change.task, change).expect("at most one change per input row"));
            }
            d::Stored::Landed { task, .. } => landed.push(task).expect("at most one landing per input row"),
            d::Stored::Entry { entry, .. } => {
                if entry.repository.forge == forge {
                    match entry.effect.write {
                        Write::Merge { number, head } => {
                            merges.push((entry.repository, number, head)).expect("at most one merge per input row");
                        }
                        Write::CreateIssue { .. }
                        | Write::OpenPull { .. }
                        | Write::Post { .. }
                        | Write::Review { .. }
                        | Write::Edit { .. }
                        | Write::SetReviewers { .. }
                        | Write::Close { .. }
                        | Write::Reopen { .. }
                        | Write::Update { .. }
                        | Write::Status { .. }
                        | Write::CreateBranch { .. }
                        | Write::DeleteBranch { .. } => {}
                    }
                }
            }
            d::Stored::Call { .. }
            | d::Stored::ProposedEffect(_)
            | d::Stored::Hold(_)
            | d::Stored::Names { .. }
            | d::Stored::Subscription(_)
            | d::Stored::BranchHead(_)
            | d::Stored::PullState(_)
            | d::Stored::Ci(_)
            | d::Stored::Client(_)
            | d::Stored::Issue(_)
            | d::Stored::Release(_)
            | d::Stored::Application { .. } => {}
        }
    }
    let client = client.ok_or(Reason::ClientUnknown)?;
    let mut complete = landed.len() < limits.landings;
    let mut decisions = Set::with_capacity(count);
    for (repository, number, head) in &merges {
        complete &= decision(*repository, *number, head, &repositories, &mut decisions)?;
    }
    for task in &landed {
        let change = changes.get(task).ok_or(Reason::StoreUnreadable)?;
        if change.repository.forge == forge {
            let number = change.pull.ok_or(Reason::StoreUnreadable)?;
            let head = change.change.last_head.ok_or(Reason::StoreUnreadable)?;
            complete &= decision(change.repository, number, &head, &repositories, &mut decisions)?;
        }
    }
    let mut out = List::with_capacity(decisions.len());
    for (repository, number, head) in &decisions {
        out.push(Decision { repository: repository.clone(), number: *number, head: *head }).expect("decisions counted");
    }
    Ok(Evidence { profile: profile(client, &prefixes, review, out.len()), decisions: out.into_boxed(), complete })
}

fn profile(client: u64, prefixes: &Set<Box<[u8]>>, review: bool, decisions: u32) -> Profile {
    let mut owned_branches = List::with_capacity(prefixes.len());
    for prefix in prefixes {
        owned_branches.push(prefix.clone()).expect("distinct prefixes counted");
    }
    let mut allowed = List::with_capacity(5);
    for allowance in [Allowance::Comment, Allowance::Merge, Allowance::Close] {
        allowed.push(allowance).expect("five allowances at most");
    }
    if review {
        allowed.push(Allowance::Review).expect("five allowances at most");
        allowed.push(Allowance::Request).expect("five allowances at most");
    }
    let mut marker = Writer::new(WORD.len().checked_add(10).expect("constant marker length"));
    marker.put(b"<!-- ").expect("sized marker");
    marker.put(WORD).expect("sized marker");
    marker.put(b":key ").expect("sized marker");
    Profile {
        client,
        marker: (marker.finish(), skein_forge::marker::TAIL.into()),
        owned_branches: owned_branches.into_boxed(),
        allowed: allowed.into_boxed(),
        bounds: Bounds { objects: OBJECTS, decisions },
    }
}

fn decision(
    repository: Repository,
    number: u64,
    head: &Commit,
    repositories: &Map<Repository, d::Repository>,
    decisions: &mut Set<(Box<[u8]>, u64, u64)>,
) -> Result<bool, Reason> {
    let head = routes::commit(&ObjectId { format: ObjectFormat::Sha1, bytes: *head }).ok_or(Reason::StoreUnreadable)?;
    let Some(repository) = repositories.get(&repository) else { return Ok(false) };
    let size =
        repository.owner.len().checked_add(1).expect("decoded owner fits usize").checked_add(repository.name.len());
    let mut name = Writer::new(size.expect("decoded repository name fits usize"));
    name.put(&repository.owner).expect("sized repository name");
    name.put(b"/").expect("sized repository name");
    name.put(&repository.name).expect("sized repository name");
    decisions.insert((name.finish(), number, head)).expect("at most one decision per entry or landing row");
    Ok(true)
}

/// Exit status and the check's JSON lines, in judge order.
#[derive(Debug)]
pub struct Output {
    pub status: u8,
    pub stdout: Box<[u8]>,
}

/// Hold back order breaches if decisions may be missing; other breaches stand.
#[must_use]
pub fn verdict(verdict: &Verdict, complete: bool) -> Output {
    let (breaches, mut reason): (&[Breach], Option<Reason>) = match verdict {
        Verdict::Pass => (&[], None),
        Verdict::Fail(breaches) => (breaches, None),
        Verdict::Undecided(why, breaches) => (breaches, Some(reason(why))),
    };
    let count = u32::try_from(breaches.len()).expect("judge bounds breaches").checked_add(1).expect("room for reason");
    let mut lines = List::with_capacity(count);
    let mut held = false;
    for breach in breaches {
        if !complete && breach.promise == Promise::Order {
            held = true;
        } else {
            let summary = match breach.promise {
                Promise::Once => "a marked key created twice",
                Promise::Forced => "a move that does not descend",
                Promise::Foreign => "a write to what the client did not make",
                Promise::Order => "a merge with no decision at its head",
            };
            lines.push(line(Some(breach), summary)).expect("one line per breach");
        }
    }
    let mut status = u8::from(!lines.is_empty());
    if held && reason.is_none() {
        reason = Some(Reason::DecisionsIncomplete);
    }
    if let Some(reason) = reason {
        lines.push(line(None, reason.text())).expect("room for one reason");
        if status == 0 {
            status = 2;
        }
    }
    let mut size = 0usize;
    for line in &lines {
        size = size.checked_add(line.len()).expect("output fits address space");
    }
    let mut stdout = Writer::new(size);
    for line in &lines {
        stdout.put(line).expect("sized output");
    }
    Output { status, stdout: stdout.finish() }
}

/// One undecided line, without running the judge.
#[must_use]
pub fn undecided(reason: Reason) -> Output {
    Output { status: 2, stdout: line(None, reason.text()) }
}

fn line(breach: Option<&Breach>, text: &str) -> Box<[u8]> {
    let limits = Limits { depth: 1, length: u32::MAX };
    let mut measure = Encoder::measure(&limits);
    encode(breach, text, &mut measure);
    let mut write = Encoder::write(measure.measured().expect("judge lines are ASCII and fit the JSON length"), &limits);
    encode(breach, text, &mut write);
    let bytes = write.finish();
    let mut out = Writer::new(bytes.len().checked_add(1).expect("room for newline"));
    out.put(&bytes).expect("sized line");
    out.put(b"\n").expect("sized line");
    out.finish()
}
fn encode(breach: Option<&Breach>, text: &str, out: &mut Encoder) {
    out.object_start();
    if let Some(breach) = breach {
        out.key(b"fail");
        out.string(&breach.line());
        out.key(b"summary");
    } else {
        out.key(b"undecided");
    }
    out.string(text.as_bytes());
    out.object_end();
}
