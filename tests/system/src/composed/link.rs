//! The protocol layers between the engine root and the worker root, as the
//! composed world plays them (testing-strategy.md, 4). jig's link records
//! cross as they are, both roots speaking the hub's vocabulary; what differs
//! is translated here by small total functions: an assignment, which the
//! engine gives typed and the hub takes with smith's charter encoded and
//! temper's workspace items beside it; an answer's parts, which the worker
//! gives in its own detail and the engine takes typed (jig's
//! `protocol/channel.md`, 6.2); and the engine's hand-outs of host calls and
//! results, decoded and rendered with smith's codecs and temper's charter
//! translation.

use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, Reader, ReplyTo, Token};
use smith_domain as smith;
use smith_domain_run as run;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_worker_domain::wire;
use temper_worker_domain_world::protocol as worker_protocol;

/// The hub's assignment for the engine's, with temper's workspace items
/// beside it. The engine names no
/// saved-work branch to save to (nor did `main`'s), so the hub saves nothing.
#[must_use]
pub fn assignment(assignment: &engine::Assignment) -> (host::Assignment, wire::Workspace) {
    let workspace = if assignment.workspace.repositories.is_empty() {
        None
    } else {
        // The root names the items itself (domain/worker.md, 4.1).
        Some(host::Workspace { workstream: assignment.workstream, items: Token::new(0) })
    };
    let mut repositories = Vec::with_capacity(assignment.workspace.repositories.len());
    for repository in &assignment.workspace.repositories {
        repositories.push(self::repository(repository));
    }
    let hub = host::Assignment {
        messages: messages(&assignment.messages),
        assignment: host::RunAssignment {
            identity: identity(assignment.task, assignment.conversation),
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            workspace,
            save: false,
            charter: charter(assignment),
            grants: grants(&assignment.grants),
        },
        turns: assignment.transcript.clone(),
        answered: answered(&assignment.settled),
    };
    (hub, wire::Workspace { repositories: repositories.into_boxed_slice() })
}

/// temper's parts beside a worker's answer, as the engine takes them: the
/// heads its pushes landed, and, when it saved, the repositories with saved
/// work after it: those whose save landed, and those that started from saved
/// work and whose save left that branch as it was. A permanent preparation
/// names the repository by tag and what the forge lacks or refused.
#[must_use]
pub fn parts(work: wire::Work, preparation: Option<wire::Preparation>) -> engine::Parts {
    let mut pushed = Vec::with_capacity(work.landed.len());
    for landed in &work.landed {
        pushed.push(forge::Pushed { tag: landed.tag, commit: landed.commit });
    }
    let saved = match work.saved {
        Some(repositories) => {
            let mut tags = Vec::with_capacity(repositories.len());
            for repository in &repositories {
                if saved(&repository.landing, repository.from_saved) {
                    tags.push(repository.tag);
                }
            }
            Some(tags.into_boxed_slice())
        }
        None => None,
    };
    let preparation = match preparation {
        Some(wire::Preparation::Missing { tag, missing }) => {
            let missing = match missing {
                wire::Missing::Repository => engine::Missing::Repository,
                wire::Missing::Branch => engine::Missing::Branch,
                wire::Missing::Commit => engine::Missing::Commit,
            };
            Some(engine::Preparation::Missing { tag, missing })
        }
        Some(wire::Preparation::Refused { tag }) => Some(engine::Preparation::Refused { tag }),
        Some(wire::Preparation::Transient) | None => None,
    };
    engine::Parts { saved, pushed: pushed.into_boxed_slice(), preparation }
}

/// Whether a repository has saved work after a save that ended in `landing`.
const fn saved(landing: &wire::Landing, from_saved: bool) -> bool {
    match landing {
        wire::Landing::Landed { .. } => true,
        wire::Landing::Unchanged
        | wire::Landing::Conflicted { .. }
        | wire::Landing::Explained { .. }
        | wire::Landing::Moved
        | wire::Landing::Failed
        | wire::Landing::Refused => from_saved,
    }
}

/// A relayed call decoded into the engine's tool vocabulary: smith's call
/// name by smith's channel codec, its input by temper's charter translation.
#[must_use]
pub fn decode(call: &fleet::Call) -> engine::Call {
    let mut reader = Reader::new(&call.name);
    let name = smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader).expect("smith's call name");
    assert!(reader.is_empty(), "one whole call name");
    let name =
        run::CallName { activation: name.activation(), completion: name.completion(), position: name.position() };
    let input = run::HostInput::attested(call.input.clone()).expect("bounded host input");
    temper_engine_smith::call(name, &call.tool, &input).expect("the run calls the declared host schema")
}

/// A decided call's answer rendered as smith takes it.
#[must_use]
pub fn render(answer: &engine::CallAnswer) -> core::SettledAnswer {
    let (body, error) = temper_engine_smith::answer(answer).into_parts();
    core::SettledAnswer::Host { error, body }
}

/// An ended answer's outcome read back as the engine's task ending: smith's
/// result codec, then temper's translation of an accepted result.
#[must_use]
pub fn end(outcome: &[u8]) -> tasks::End {
    let declared = smith_protocol_channel::decode_declared(outcome, &smith_charter::CEILINGS).expect("smith's result");
    let spent = run::Spend {
        turns: 0,
        input: None,
        output: None,
        cache_read: None,
        cache_write: None,
        reasoning: None,
        units: 0,
    };
    let answer = run::Answer::Accepted { outcome: declared, spent, turns: 0 };
    temper_engine_smith::result(answer, None).expect("a result the engine takes").end
}

fn identity(task: u64, conversation: u64) -> [u8; 16] {
    let mut identity = [0; 16];
    let (first, second) = identity.split_at_mut(8);
    first.copy_from_slice(&task.to_be_bytes());
    second.copy_from_slice(&conversation.to_be_bytes());
    identity
}

/// The engine's charter, smith's charter encoded: temper's translation, its
/// conversation resumed when the charter resumes and turns are kept.
fn charter(assignment: &engine::Assignment) -> Box<[u8]> {
    let window = smith::Window { turns: 0, bytes: 0 };
    let start = temper_engine_smith::start(assignment, None, None, ReplyTo::new(Token::new(0)), Token::new(0), window);
    let smith::Event::Start { mut charter, .. } = start else { unreachable!("temper's translation starts a run") };
    charter.resume = assignment.run.policy.resume && !assignment.transcript.is_empty();
    smith_protocol_channel::encode_charter(&charter, &smith_charter::CEILINGS, &worker_protocol::endpoints())
        .expect("smith encodes the engine's charter")
}

fn messages(words: &[tasks::Word]) -> Box<[host::StartMessage]> {
    let mut messages = Vec::with_capacity(words.len());
    for word in words {
        messages.push(host::StartMessage {
            name: Token::new(word.number),
            sender: sender(word.from),
            words: word.words.clone(),
        });
    }
    messages.into_boxed_slice()
}

fn sender(from: tasks::Party) -> Box<[u8]> {
    match from {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
}

fn grants(grants: &[accounts::Grant]) -> Box<[host::Grant]> {
    let mut translated = Vec::with_capacity(grants.len());
    for grant in grants {
        let table = match grant.table {
            accounts::GrantTable::Accounts => host::GrantTable::Accounts,
            accounts::GrantTable::Application => host::GrantTable::Application,
        };
        translated.push(host::Grant {
            table,
            account: grant.account,
            generation: grant.generation,
            valid: grant.valid,
        });
    }
    translated.into_boxed_slice()
}

fn answered(settled: &[core::SettledCall]) -> Box<[host::AnsweredCall]> {
    let mut answered = Vec::with_capacity(settled.len());
    for call in settled {
        let answer = match &call.answer {
            core::SettledAnswer::Host { error, body } => {
                host::SettledAnswer::Host { error: *error, body: body.clone() }
            }
            core::SettledAnswer::Delivery { outcome, evidence } => {
                host::SettledAnswer::Delivery { outcome: delivery(*outcome), evidence: evidence.clone() }
            }
        };
        answered.push(host::AnsweredCall { name: call.name.clone(), tool: call.tool.clone(), answer });
    }
    answered.into_boxed_slice()
}

/// As jig's protocol layer names a delivery's outcome: an invalid one was
/// refused (`testing/jig-test-service`).
const fn delivery(outcome: core::DeliveryOutcome) -> host::DeliveryOutcome {
    match outcome {
        core::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
        core::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
        core::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
        core::DeliveryOutcome::Refused | core::DeliveryOutcome::Invalid => host::DeliveryOutcome::Refused,
        core::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
    }
}

/// A forge checkout as the worker's workspace takes it. The engine names no
/// expected head for a writable branch, nor did `main`'s.
fn repository(repository: &engine::ForgeRepository) -> wire::Repository {
    let start = match &repository.start {
        engine::ForgeStart::Base(branch) => wire::Start::Base { branch: branch.clone() },
        engine::ForgeStart::Branch(branch) => wire::Start::Branch { branch: branch.clone() },
        engine::ForgeStart::Saved(branch) => wire::Start::Saved { branch: branch.clone() },
        engine::ForgeStart::Merge { branch, base } => wire::Start::Merge { branch: branch.clone(), base: *base },
    };
    let access = match &repository.push {
        Some(push) => wire::Access::Writable { push: push.clone(), expected: None },
        None => wire::Access::ReadOnly,
    };
    wire::Repository {
        tag: repository.tag,
        name: repository.name.clone(),
        remote: [repository.owner.as_ref(), b"/", repository.name.as_ref()].concat().into_boxed_slice(),
        start,
        access,
        identity: repository.identity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heads_a_run_landed_reach_the_engine_as_its_pushed_heads() {
        let work = wire::Work { landed: Box::new([wire::Landed { tag: 3, commit: [7; 32] }]), saved: None };
        let parts = parts(work, None);
        assert_eq!(
            parts,
            engine::Parts {
                saved: None,
                pushed: Box::new([forge::Pushed { tag: 3, commit: [7; 32] }]),
                preparation: None
            }
        );
    }

    #[test]
    fn a_save_names_each_repository_with_saved_work_after_it() {
        let saved = Box::new([
            wire::SavedRepository { tag: 1, from_saved: false, landing: wire::Landing::Landed { commit: [9; 32] } },
            wire::SavedRepository { tag: 2, from_saved: true, landing: wire::Landing::Unchanged },
            wire::SavedRepository { tag: 3, from_saved: true, landing: wire::Landing::Failed },
            wire::SavedRepository { tag: 4, from_saved: false, landing: wire::Landing::Unchanged },
        ]);
        let work = wire::Work { landed: Box::new([]), saved: Some(saved) };
        let parts = parts(work, None);
        assert_eq!(parts, engine::Parts { saved: Some(Box::new([1, 2, 3])), pushed: Box::new([]), preparation: None });
    }

    #[test]
    fn a_permanent_preparation_reaches_the_engine_by_its_tag() {
        let rows = [
            (
                wire::Preparation::Missing { tag: 4, missing: wire::Missing::Commit },
                engine::Preparation::Missing { tag: 4, missing: engine::Missing::Commit },
            ),
            (
                wire::Preparation::Missing { tag: 5, missing: wire::Missing::Branch },
                engine::Preparation::Missing { tag: 5, missing: engine::Missing::Branch },
            ),
            (
                wire::Preparation::Missing { tag: 6, missing: wire::Missing::Repository },
                engine::Preparation::Missing { tag: 6, missing: engine::Missing::Repository },
            ),
            (wire::Preparation::Refused { tag: 7 }, engine::Preparation::Refused { tag: 7 }),
        ];
        for (preparation, expected) in rows {
            let work = wire::Work { landed: Box::new([]), saved: None };
            let parts = parts(work, Some(preparation));
            assert_eq!(
                parts,
                engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(expected) },
                "{preparation:?}"
            );
        }
    }

    #[test]
    fn a_transient_preparation_reaches_the_engine_as_no_part() {
        let work = wire::Work { landed: Box::new([]), saved: None };
        assert_eq!(parts(work, Some(wire::Preparation::Transient)), engine::Parts::none());
    }
}
