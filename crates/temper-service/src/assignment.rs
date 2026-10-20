//! Root assignments as the hub's records, with application items beside them.
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_protocol_engine_workers as kit;
use skein_lib::{Decimal, List, Time, Token, Writer};
use temper_engine_domain as root;
use temper_protocol_worker_wire as wire;

/// The checked channel limits, endpoint names and deployment identity.
#[derive(Debug)]
pub struct Encoding {
    pub limits: wire::v1::Limits,
    pub endpoints: smith_protocol_channel::Endpoints,
    pub deployment: [u8; 16],
}

/// An assignment and its application parts, or none when it cannot encode.
#[must_use]
pub fn down(
    assignment: root::Assignment,
    forges: &[crate::settings::Forge],
    encoding: &Encoding,
    values: &crate::values::Values,
    channel: Token,
    now: Time,
) -> Option<kit::Event> {
    let items = crate::translate::items(&assignment, forges, &encoding.limits).ok()?;
    let charter = smith_protocol_channel::encode_charter(
        &temper_engine_smith::charter(&assignment),
        &smith_charter::CEILINGS,
        &encoding.endpoints,
    )
    .ok()?;
    let identity = identity(&encoding.deployment, assignment.task, assignment.conversation);
    let mut parts = List::with_capacity(1);
    let workspace = match items {
        Some(items) => {
            let token = Token::new(0);
            parts
                .push(jig_protocol_worker_wire::Part { token, bytes: wire::items(&items) })
                .expect("one workspace part");
            Some(host::Workspace { workstream: assignment.workstream, items: token })
        }
        None => None,
    };
    let grants = grants(&assignment.grants);
    let filled = values.grants(&grants, now);
    Some(kit::Event::Down {
        channel,
        down: host::LinkDown::Assign {
            assignment: host::Assignment {
                messages: messages(&assignment.messages),
                assignment: host::RunAssignment {
                    identity,
                    run: Token::new(assignment.task),
                    attempt: Token::new(assignment.attempt),
                    workspace,
                    save: false,
                    charter,
                    grants,
                },
                turns: assignment.transcript,
                answered: answered(&assignment.settled),
            },
        },
        parts: parts.into_boxed(),
        values: filled,
    })
}
fn identity(deployment: &[u8; 16], task: u64, conversation: u64) -> [u8; 16] {
    let mut source = Writer::new(32);
    source.put(deployment).expect("deployment identity");
    source.put(&task.to_be_bytes()).expect("task identity");
    source.put(&conversation.to_be_bytes()).expect("conversation identity");
    let digest = skein_io::digest::digest(&source.finish());
    let mut identity = Writer::new(16);
    for word in digest.0.get(..2).expect("first sixteen digest bytes") {
        identity.put(&word.to_be_bytes()).expect("two identity words");
    }
    identity.finish().as_ref().try_into().expect("sixteen identity bytes")
}
fn messages(words: &[tasks::Word]) -> Box<[host::StartMessage]> {
    let mut messages = List::with_capacity(u32::try_from(words.len()).expect("bounded messages"));
    for word in words {
        let sender = match word.from {
            tasks::Party::Task(number) | tasks::Party::Person(number) => Decimal::of(number).as_bytes().into(),
            tasks::Party::Deployment { .. } => b"deployment".as_slice().into(),
        };
        messages
            .push(host::StartMessage { name: Token::new(word.number), sender, words: word.words.clone() })
            .expect("bounded messages");
    }
    messages.into_boxed()
}
fn grants(grants: &[accounts::Grant]) -> Box<[host::Grant]> {
    let mut filled = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
    for grant in grants {
        let form = match grant.form {
            accounts::Form::Credential(scheme) => host::Form::Credential(match scheme {
                accounts::Scheme::Bearer => host::Scheme::Bearer,
                accounts::Scheme::Key => host::Scheme::Key,
            }),
            accounts::Form::Withdrawn => host::Form::Withdrawn,
            accounts::Form::Application => host::Form::Application,
        };
        filled
            .push(host::Grant { form, account: grant.account, generation: grant.generation, valid: grant.valid })
            .expect("bounded grants");
    }
    filled.into_boxed()
}
fn answered(calls: &[core::SettledCall]) -> Box<[host::AnsweredCall]> {
    let mut answered = List::with_capacity(u32::try_from(calls.len()).expect("bounded answered calls"));
    for call in calls {
        let answer = match &call.answer {
            core::SettledAnswer::Host { error, body } => {
                host::SettledAnswer::Host { error: *error, body: body.clone() }
            }
            core::SettledAnswer::Delivery { outcome, evidence } => host::SettledAnswer::Delivery {
                outcome: match outcome {
                    core::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
                    core::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
                    core::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
                    core::DeliveryOutcome::Refused | core::DeliveryOutcome::Invalid => host::DeliveryOutcome::Refused,
                    core::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
                },
                evidence: evidence.clone(),
            },
        };
        answered
            .push(host::AnsweredCall { name: call.name.clone(), tool: call.tool.clone(), answer })
            .expect("bounded answered calls");
    }
    answered.into_boxed()
}
