//! The root's ranges, expressed in jig's store vocabulary.
use super::route::Flight;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_store as store;
use temper_engine_domain::{CoreRange, Key, Range};

pub(super) fn cursor(range: &Range, key: &Key) -> Option<store::Cursor> {
    let after = match key {
        Key::Core(key) => store::keys::encode(key, u32::MAX)?,
        Key::Forge(key) => store::keys::application(&temper_protocol_store::keys::encode(key, u32::MAX)?, u32::MAX)?,
    };
    let interval = match (range, key) {
        (Range::Census(census), Key::Core(key)) => census.interval(key)?,
        (Range::Core(CoreRange::Tasks), Key::Core(core::Key::Tasks(_))) => 1,
        (Range::Core(_) | Range::Census(_) | Range::Forge, Key::Core(_) | Key::Forge(_)) => 0,
    };
    Some(store::Cursor { interval, after })
}

pub(super) fn request(owner: skein_lib::Token, flight: &Flight) -> Option<store::Request> {
    let after = if let Some(cursor) = &flight.cursor {
        Some(cursor.clone())
    } else {
        match &flight.after {
            Some(key) => Some(cursor(&flight.range, key)?),
            None => None,
        }
    };
    let rows = flight.most;
    let bytes = u64::from(flight.bytes);
    let range = match &flight.range {
        Range::Forge => store::Restore::Application(Box::new([temper_protocol_store::interval()])),
        Range::Census(census) => store::Restore::Jig(Box::new([core::RestoreRange::Census(census.clone())])),
        Range::Core(range) => match range {
            CoreRange::Deployment => store::Restore::Jig(Box::new([core::RestoreRange::Deployment])),
            CoreRange::Accounts => store::Restore::Jig(Box::new([core::RestoreRange::Accounts])),
            CoreRange::People => store::Restore::Jig(Box::new([core::RestoreRange::People])),
            CoreRange::Tasks => store::Restore::Jig(Box::new([core::RestoreRange::Core, core::RestoreRange::Tasks])),
            CoreRange::Calls | CoreRange::RunProofs => store::Restore::Jig(Box::new([core::RestoreRange::Core])),
            CoreRange::EndedRead { task, party } => {
                return Some(store::Request::Get {
                    owner,
                    key: store::Get::Jig(core::Key::People(jig_domain_people::Key::EndedRead {
                        task: *task,
                        party: *party,
                    })),
                });
            }
            CoreRange::TaskResult { task } => {
                return Some(store::Request::Get {
                    owner,
                    key: store::Get::Jig(core::Key::Tasks(jig_domain_tasks::Key::Ended(*task))),
                });
            }
            CoreRange::TaskProposal { task, .. } => {
                return Some(store::Request::Get {
                    owner,
                    key: store::Get::Jig(core::Key::Tasks(jig_domain_tasks::Key::Live(*task))),
                });
            }
            CoreRange::ProposalDecision { proposer, proposal } => {
                return Some(store::Request::Get {
                    owner,
                    key: store::Get::Jig(core::Key::Core(core::CoreKey::ProposalDecision {
                        proposer: *proposer,
                        proposal: *proposal,
                    })),
                });
            }
            CoreRange::EscalationDecision { task, revision } => {
                return Some(store::Request::Get {
                    owner,
                    key: store::Get::Jig(core::Key::Core(core::CoreKey::EscalationDecision {
                        task: *task,
                        revision: *revision,
                    })),
                });
            }
            CoreRange::Notes(range) => {
                return Some(store::Request::Load {
                    owner,
                    range: store::Load::Notes(range.clone()),
                    after,
                    rows,
                    bytes,
                });
            }
            CoreRange::Turns { task, .. } => {
                return Some(store::Request::Transcript {
                    owner,
                    read: store::TranscriptRead::Page { task: *task, backwards: false, after: flight.offset },
                    rows,
                    bytes,
                });
            }
            CoreRange::TaskTranscript { task, from: (attempt, turn) } => {
                return Some(store::Request::Transcript {
                    owner,
                    read: store::TranscriptRead::Window {
                        task: *task,
                        attempt: *attempt,
                        turn: *turn,
                        after: flight.offset,
                    },
                    rows,
                    bytes,
                });
            }
        },
    };
    Some(store::Request::Restore { owner, range, after, rows, bytes })
}
