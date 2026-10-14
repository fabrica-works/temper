//! Total owner routing; tokens pass unchanged (client.md, 3.3 and 6.2).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use jig_domain_client::contract::VersionRange;
use jig_protocol_client_link as link;
use skein_lib::{RequestRecord, Secret};
use temper_client_domain as client;
pub mod family;

/// Each link output has exactly one destination.
#[derive(Debug)]
pub enum Up {
    Root(client::Event),
    Shell(link::Request),
    Kept(client::store::Record),
    Notice(Notice),
}
/// Values shown outside the domain, including one-time credentials.
#[derive(Debug)]
pub enum Notice {
    Credential { party: u64, credential: Secret },
    Dropped { keeper: link::Keeper },
    Unread { keeper: link::Keeper },
    Served { family: u16, versions: VersionRange },
    Unasked,
    Fact(link::Fact),
}
/// Each root output either steps the link or tells its shell.
#[derive(Debug)]
pub enum Down {
    Link(link::Event),
    Duty(Duty),
    Nothing,
}
/// Navigation, authentication and facts stay in the host.
#[derive(Debug)]
pub enum Duty {
    Show { address: client::Address, replace: bool },
    SignIn { provider: u16, back: client::Address },
    SignOut,
    Fact(client::Fact),
}
#[must_use]
pub fn up(request: link::Request, family: &mut family::Family) -> Up {
    match request {
        transport @ (link::Request::Write { .. }
        | link::Request::Remove { .. }
        | link::Request::Exchange { .. }
        | link::Request::Abandon { .. }
        | link::Request::OpenStream
        | link::Request::CloseStream) => Up::Shell(transport),
        link::Request::Kept { record } => {
            let keeper = match &record {
                link::Saved::Jig(_) => link::Keeper::Jig,
                link::Saved::Table { .. } => link::Keeper::Table,
                link::Saved::Family { family, .. } => link::Keeper::Family(*family),
            };
            let dropped = match &record {
                link::Saved::Table {
                    record: skein_lib::RequestRecord { request: link::Ask::Family(record), .. },
                    ..
                } => {
                    record.family == temper_protocol_client_wire::FORGE
                        && record.version != temper_protocol_client_wire::CLIENT_VERSION
                }
                link::Saved::Jig(_) | link::Saved::Table { .. } | link::Saved::Family { .. } => false,
            };
            if dropped {
                return Up::Notice(Notice::Dropped {
                    keeper: link::Keeper::Family(temper_protocol_client_wire::FORGE),
                });
            }
            match kept(record, family) {
                Some(record) => Up::Kept(record),
                None => Up::Notice(Notice::Unread { keeper }),
            }
        }
        link::Request::Answer { attempt, envelope, answer } => {
            // Every Answer ends its attempt, even when the keyed request retries.
            let forge = family.attempts.answered(attempt, true);
            let answer = match answer {
                Some(link::Answered::Jig(jig_domain_client::contract::Answer::Refused(refusal))) if forge => {
                    match jig_domain_client::contract::EntranceRefusal::new(refusal) {
                        Ok(refusal) => Some(client::Answered::Forge(client::forge::Answer::EntranceRefused(refusal))),
                        Err(_) => None,
                    }
                }
                Some(link::Answered::Jig(answer)) => Some(client::Answered::Jig(answer)),
                Some(link::Answered::Family(record)) => {
                    family::answer(&record, &family.wire, &family.link).map(client::Answered::Forge)
                }
                None => None,
            };
            let envelope = match &answer {
                None => match envelope {
                    skein_lib::RequestEnvelope::Final => skein_lib::RequestEnvelope::Lost,
                    skein_lib::RequestEnvelope::Again
                    | skein_lib::RequestEnvelope::Lost
                    | skein_lib::RequestEnvelope::SignedOut => envelope,
                },
                Some(_) => envelope,
            };
            Up::Root(client::Event::Answer { attempt, envelope, answer })
        }
        link::Request::Watch { watch, event } => {
            Up::Root(client::Event::Jig(jig_domain_client::Event::Watch { watch, event }))
        }
        link::Request::Ended { watch, end } => {
            Up::Root(client::Event::Jig(jig_domain_client::Event::Ended { watch, end }))
        }
        link::Request::Read { read, result } => match result {
            link::Readout::Jig(result) => Up::Root(client::Event::Jig(jig_domain_client::Event::Read { read, result })),
            link::Readout::Family { .. } => Up::Notice(Notice::Unasked),
        },
        link::Request::Hello { family: None, versions } => {
            Up::Root(client::Event::Jig(jig_domain_client::Event::Hello { versions }))
        }
        link::Request::Hello { family: Some(number), versions } => {
            if number == temper_protocol_client_wire::FORGE {
                Up::Root(client::Event::Forge(client::forge::Event::Served { versions }))
            } else {
                Up::Notice(Notice::Served { family: number, versions })
            }
        }
        link::Request::ServiceCredential { party, credential } => Up::Notice(Notice::Credential { party, credential }),
        link::Request::Dropped { keeper } => Up::Notice(Notice::Dropped { keeper }),
        link::Request::FamilyDelivery { .. } => Up::Notice(Notice::Unasked),
        link::Request::Fact { fact } => Up::Notice(Notice::Fact(fact)),
    }
}
#[must_use]
pub fn down(request: client::Request, family: &mut family::Family) -> Down {
    match request {
        client::Request::Save { record } => match saved(record, family) {
            Some(record) => Down::Link(link::Event::Save { record }),
            None => Down::Duty(Duty::Fact(client::Fact::InvalidRecord)),
        },
        client::Request::Erase { key } => Down::Link(link::Event::Erase { key: erased(key) }),
        client::Request::Send { attempt, key, ask: client::Ask::Jig(ask) } => {
            Down::Link(link::Event::Send { attempt, key, ask: link::Ask::Jig(ask) })
        }
        client::Request::Send { attempt, key, ask: client::Ask::Forge(ask) } => {
            match family::request(&ask, &family.wire) {
                Some(record) => {
                    if family.attempts.sent(attempt).is_err() {
                        Down::Duty(Duty::Fact(client::Fact::TableFull))
                    } else {
                        Down::Link(link::Event::Send { attempt, key, ask: link::Ask::Family(record) })
                    }
                }
                None => Down::Duty(Duty::Fact(client::Fact::InvalidRecord)),
            }
        }
        client::Request::Jig(request) => match request {
            client::JigOut::Read { read, what } => {
                Down::Link(link::Event::Read { read, what: link::Readable::Jig(what) })
            }
            client::JigOut::Open { watch, subject } => {
                Down::Link(link::Event::Open { watch, subject: link::Watchable::Jig(subject) })
            }
            client::JigOut::Continue { watch, subject } => Down::Link(link::Event::Continue { watch, subject }),
            client::JigOut::Close { watch } => Down::Link(link::Event::Close { watch }),
            client::JigOut::Answered { .. } | client::JigOut::Unknown { .. } => Down::Nothing,
        },
        client::Request::Show { address, replace } => Down::Duty(Duty::Show { address, replace }),
        client::Request::SignIn { provider, back } => Down::Duty(Duty::SignIn { provider, back }),
        client::Request::SignOut => Down::Duty(Duty::SignOut),
        client::Request::Fact { fact } => Down::Duty(Duty::Fact(fact)),
    }
}
#[must_use]
pub fn saved(record: client::store::Record, family: &family::Family) -> Option<link::Saved> {
    Some(match record {
        client::store::Record::Jig(record) => link::Saved::Jig(record),
        client::store::Record::Table { version, record } => link::Saved::Table {
            version,
            record: RequestRecord {
                key: record.key,
                scope: record.scope,
                first_sent: record.first_sent,
                request: match record.request {
                    client::Ask::Jig(ask) => link::Ask::Jig(ask),
                    client::Ask::Forge(ask) => link::Ask::Family(family::request(&ask, &family.wire)?),
                },
            },
        },
    })
}
#[must_use]
pub fn kept(saved: link::Saved, family: &family::Family) -> Option<client::store::Record> {
    match saved {
        link::Saved::Jig(record) => Some(client::store::Record::Jig(record)),
        link::Saved::Table { version, record } => {
            let request = match record.request {
                link::Ask::Jig(ask) => client::Ask::Jig(ask),
                link::Ask::Family(record) => client::Ask::Forge(family::restored(&record, &family.wire)?),
            };
            Some(client::store::Record::Table {
                version,
                record: RequestRecord { key: record.key, scope: record.scope, first_sent: record.first_sent, request },
            })
        }
        link::Saved::Family { .. } => None,
    }
}
#[must_use]
pub fn erased(key: client::store::Key) -> link::Erased {
    match key {
        client::store::Key::Jig(key) => link::Erased::Jig(key),
        client::store::Key::Table(key) => link::Erased::Table(key),
    }
}
#[must_use]
pub fn versions() -> [link::StoreVersion; 2] {
    [
        link::StoreVersion { keeper: link::Keeper::Jig, version: jig_domain_client::store::STORE_VERSION },
        link::StoreVersion { keeper: link::Keeper::Table, version: client::store::VERSION },
    ]
}
