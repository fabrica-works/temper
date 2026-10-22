//! The forge family's records at the clients' boundary (client/README.md, 8).
use alloc::boxed::Box;
use jig_protocol_engine_clients as kit;
use skein_lib::{Decimal, List, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
mod refusals;

// client/README.md, 4.6: connector documents and their wire share every bound.
const _: () = {
    assert!(forge::PANEL_AHEAD == wire::TASKS_AHEAD, "panel predecessor bound");
    assert!(forge::PANEL_GATES == wire::GATES, "panel gate bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_owner, "panel owner bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_name, "panel repository bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_branch, "panel branch bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_landing, "panel landing branch bound");
};

/// Decode the keyed request, preserving its admission envelope and supplied digest.
#[expect(clippy::too_many_lines, reason = "one exhaustive match translates the family's typed requests")]
pub fn to_root(
    exchange: kit::ToApplication,
    forges: &[crate::settings::Forge],
    demo: Option<&crate::settings::Demo>,
    limits: &w::Limits,
) -> Result<root::Event, Box<kit::Event>> {
    match exchange {
        kit::ToApplication::Request { reply_to, sign_in, key, family, version, record, digest, scope, carried } => {
            let request = match wire::owner(family) {
                Some(wire::Owner::Forge) if version == wire::CLIENT_VERSION => wire::decode_requesting(&record, limits),
                Some(wire::Owner::Forge | wire::Owner::Pages | wire::Owner::Runner) | None => None,
            };
            let request = match request {
                Some(request) => request.into_parts().request,
                None => {
                    return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })));
                }
            };
            let request = match request {
                w::Request::SetUp(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: forge::Ask::SetUp { forge: value.forge() },
                    }));
                }
                w::Request::Adopt(value) => value,
            };
            let a = request.into_parts();
            let mut host = Box::default();
            for forge in forges {
                if forge.number == a.forge {
                    host = self::host(&forge.git);
                    break;
                }
            }
            Ok(root::Event::Party(root::PartyInput::Application {
                scope,
                reply_to,
                sign_in,
                key,
                digest,
                request: forge::Ask::Adopt(forge::Adoption {
                    project: a.project,
                    home: a.home,
                    forge: a.forge,
                    host,
                    owner: a.owner,
                    name: a.name,
                    prefix: a.prefix,
                    role: role(a.role),
                    landing: a.landing,
                    ci: a.ci,
                    checks: a.checks.into_boxed(),
                }),
                carried,
            }))
        }
        kit::ToApplication::Read { reply, family, version, record, .. } => {
            let mut answer = None;
            if family == wire::FORGE
                && version == wire::CLIENT_VERSION
                && let Some(reading) = wire::decode_reading(&record, limits)
                && *reading.read() == w::Read::Demo
                && let Some(demo) = demo
            {
                let mut cues = List::with_capacity(u32::try_from(demo.cues.len()).expect("bounded demo cues"));
                for cue in &demo.cues {
                    cues.push(cue.clone()).expect("bounded demo cues");
                }
                let record = w::DemoAnswer::new(
                    limits,
                    w::DemoAnswerParts {
                        project: demo.project,
                        forge: demo.forge,
                        provider: demo.provider,
                        owner: demo.owner.clone(),
                        name: demo.name.clone(),
                        cues,
                    },
                )
                .expect("demo checked at startup");
                answer = Some(wire::demo_answer(&record));
            }
            Err(Box::new(kit::Event::Application(kit::FromApplication::Read { reply, family, record: answer })))
        }
        kit::ToApplication::Watch { watcher, sign_in, family, version, record, scope, window } => {
            if family == wire::FORGE
                && version == wire::CLIENT_VERSION
                && let Some(watching) = wire::decode_watching(&record, limits)
            {
                let subject = match watching.into_parts().watch {
                    w::Watch::Forges => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Deployment,
                        connector: wire::FORGE,
                        subject: Token::new(0),
                    },
                    w::Watch::Change(subject) if subject.task() > 0 => {
                        jig_domain_core::documents::Watched::Application {
                            scope,
                            connector: wire::FORGE,
                            subject: Token::new(subject.task()),
                        }
                    }
                    w::Watch::Change(_) => {
                        return Err(Box::new(kit::Event::WatchRefused {
                            watcher,
                            why: jig_domain_core::WatchRefusal::Unknown,
                        }));
                    }
                };
                return Ok(root::Event::Party(root::PartyInput::Watch { watcher, sign_in, subject, window }));
            }
            Err(Box::new(kit::Event::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::Unknown }))
        }
    }
}

/// Encode the whole retained panel, deriving its public address from configuration
/// (client/forge.md, 9.2). No domain record carries a deployment's web address.
#[must_use]
pub fn panel(value: forge::Panel, forges: &[crate::settings::Forge], limits: &w::Limits) -> Option<Box<[u8]>> {
    let pull = match value.pull {
        Some(number) => {
            Some(w::Pull::new(limits, w::PullParts { number, address: address(&value, number, forges)? }).ok()?)
        }
        None => None,
    };
    let mut gates = List::with_capacity(forge::PANEL_GATES);
    for gate in value.gates {
        gates
            .push(
                w::Gate::new(
                    limits,
                    w::GateParts {
                        number: gate.number,
                        kind: match gate.kind {
                            change::GateKind::Agent => w::GateKind::Review,
                            change::GateKind::Person => w::GateKind::Approval,
                            change::GateKind::Check => w::GateKind::Check,
                        },
                        blocks: gate.blocking,
                        verdict: match gate.verdict {
                            change::Status::Unknown => w::Verdict::None,
                            change::Status::Pending => w::Verdict::Pending,
                            change::Status::Passed => w::Verdict::Approved,
                            change::Status::Failed => w::Verdict::Changes,
                        },
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let record = w::ChangePanel::new(
        limits,
        w::ChangePanelParts {
            task: value.task,
            forge: value.forge,
            owner: value.owner,
            name: value.name,
            branch: value.branch,
            landing: value.landing,
            head: value.head,
            pull,
            step: panel_step(value.step, limits)?,
            hold: panel_hold(value.hold),
            runs: value.runs,
            ci: match value.ci {
                change::Status::Unknown => w::Ci::Unknown,
                change::Status::Pending => w::Ci::Pending,
                change::Status::Passed => w::Ci::Passed,
                change::Status::Failed => w::Ci::Failed,
            },
            gates,
            repairs: used(value.repairs, limits)?,
            resolutions: used(value.resolutions, limits)?,
            updates: used(value.updates, limits)?,
        },
    )
    .ok()?;
    Some(wire::change_panel(&record))
}

fn used(value: forge::PanelUsed, limits: &w::Limits) -> Option<w::Used> {
    w::Used::new(limits, w::UsedParts { used: value.used, limit: value.limit }).ok()
}

fn panel_step(value: forge::PanelStep, limits: &w::Limits) -> Option<w::Step> {
    Some(match value {
        forge::PanelStep::Producing => w::Step::Producing,
        forge::PanelStep::Opening => w::Step::Opening,
        forge::PanelStep::Recreating => w::Step::Recreating,
        forge::PanelStep::Reopening => w::Step::Reopening,
        forge::PanelStep::Checking => w::Step::Checking,
        forge::PanelStep::Gating => w::Step::Gating,
        forge::PanelStep::Queued { place, ahead } => {
            let mut tasks = List::with_capacity(forge::PANEL_AHEAD);
            for task in ahead {
                tasks.push(task).ok()?;
            }
            w::Step::Queued(w::Queued::new(limits, w::QueuedParts { place, ahead: tasks }).ok()?)
        }
        forge::PanelStep::First => w::Step::First,
        forge::PanelStep::Updating => w::Step::Updating,
        forge::PanelStep::Resolving => w::Step::Resolving,
        forge::PanelStep::Repairing(reason) => w::Step::Repairing(
            w::Repairing::new(
                limits,
                w::RepairingParts {
                    reason: match reason {
                        change::Repair::Ci => w::Reason::Ci,
                        change::Repair::Gate(gate) => {
                            w::Reason::Gate(w::GateReason::new(limits, w::GateReasonParts { gate }).ok()?)
                        }
                        change::Repair::Semantic => w::Reason::Semantic,
                    },
                },
            )
            .ok()?,
        ),
        forge::PanelStep::Landing => w::Step::Landing,
        forge::PanelStep::Landed { merge } => w::Step::Landed(w::Landed::new(limits, w::LandedParts { merge }).ok()?),
    })
}

fn panel_hold(value: Option<change::Hold>) -> Option<w::Hold> {
    Some(match value? {
        change::Hold::Stalled => w::Hold::Stalled,
        change::Hold::Repairs => w::Hold::Repairs,
        change::Hold::Resolutions => w::Hold::Resolutions,
        change::Hold::Updates => w::Hold::Updates,
        change::Hold::BranchMoved => w::Hold::Moved,
        change::Hold::BranchMissing => w::Hold::Missing,
        change::Hold::PullClosed => w::Hold::Closed,
        change::Hold::Retargeted => w::Hold::Retargeted,
        change::Hold::Failed => w::Hold::Failed,
        change::Hold::Rejected => w::Hold::Rejected,
    })
}

fn address(panel: &forge::Panel, number: u64, forges: &[crate::settings::Forge]) -> Option<Box<[u8]>> {
    for forge in forges {
        if forge.number != panel.forge {
            continue;
        }
        let Some(web) = &forge.web else { return Some(Box::default()) };
        let mut web = web.as_ref();
        while web.last() == Some(&b'/') {
            web = web.get(..web.len().checked_sub(1)?).expect("trailing slash");
        }
        let route = match forge.kind {
            crate::settings::Kind::GitHub => b"/pull/".as_slice(),
            crate::settings::Kind::Forgejo => b"/pulls/".as_slice(),
        };
        let digits = Decimal::of(number);
        let length = web
            .len()
            .checked_add(1)?
            .checked_add(panel.owner.len())?
            .checked_add(1)?
            .checked_add(panel.name.len())?
            .checked_add(route.len())?
            .checked_add(digits.as_bytes().len())?;
        // An address past its bound is left out, as with no `web`, so one long
        // owner and name never fail the delivery (client/forge.md, 9.2).
        if length > usize::try_from(wire::ADDRESS_BYTES).expect("u32 fits usize") {
            return Some(Box::default());
        }
        let mut writer = Writer::new(length);
        for part in [web, b"/", &panel.owner, b"/", &panel.name, route, digits.as_bytes()] {
            writer.put(part).ok()?;
        }
        return Some(writer.finish());
    }
    Some(Box::default())
}
/// Keep the authority of an already checked git URL, including its explicit port.
#[must_use]
pub fn host(git: &[u8]) -> Box<[u8]> {
    let Some(start) = skein_lib::bytes::find(git, b"://") else { return Box::default() };
    let rest = git.get(start.checked_add(3).expect("URL separator")..).expect("separator within URL");
    let mut first = 0;
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' || *byte == b'?' || *byte == b'#' {
            end = index;
            break;
        }
        if *byte == b'@' {
            first = index.checked_add(1).expect("URL authority separator");
        }
    }
    rest.get(first..end).expect("host within URL authority").into()
}
fn role(role: w::Role) -> forge::Role {
    match role {
        w::Role::Owned => forge::Role::Owned,
        w::Role::Adopted => forge::Role::Adopted,
        w::Role::Fork => forge::Role::Fork,
        w::Role::Context => forge::Role::Context,
    }
}
fn encoded_role(role: forge::Role) -> w::Role {
    match role {
        forge::Role::Owned => w::Role::Owned,
        forge::Role::Adopted => w::Role::Adopted,
        forge::Role::Fork => w::Role::Fork,
        forge::Role::Context => w::Role::Context,
    }
}
/// Encode one durable terminal, or the unknown terminal past its retention.
#[must_use]
pub fn answer(
    outcome: Option<forge::Answered>,
    limits: &w::Limits,
    jig: &jig_client_protocol_wire::v12::Limits,
) -> Option<Box<[u8]>> {
    let outcome = match outcome {
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(a))) => w::Outcome::Adopted(
            w::Adopted::new(
                limits,
                w::AdoptedParts {
                    forge: a.repository.forge,
                    repository: a.repository.repository,
                    owner: a.owner,
                    name: a.name,
                    role: encoded_role(a.role),
                    home: a.home,
                    kinds: w::Kinds::new(
                        limits,
                        w::KindsParts {
                            skein_bools: [
                                a.kinds.read,
                                a.kinds.push,
                                a.kinds.open,
                                a.kinds.land,
                                a.kinds.review,
                                a.kinds.status,
                                a.kinds.comment,
                                a.kinds.issue,
                                a.kinds.branch,
                            ],
                        },
                    )
                    .ok()?,
                    protection: protection(a.protection, limits)?,
                },
            )
            .ok()?,
        ),
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::CoreRefused(value))) => {
            let value = refusals::refusal(value, jig).ok()?;
            let mut bytes = Writer::new(usize::try_from(value.measure()).ok()?);
            value.encode(&mut bytes).ok()?;
            w::Outcome::CoreRefused(w::CoreRefused::new(limits, w::CoreRefusedParts { refusal: bytes.finish() }).ok()?)
        }
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::ForgeRefused(reason))) => w::Outcome::ForgeRefused(
            w::ForgeRefused::new(
                limits,
                w::ForgeRefusedParts {
                    reason: match reason {
                        forge::AdoptionRefusal::NoSuchForge => w::ForgeReason::NoSuchForge,
                        forge::AdoptionRefusal::NotFound => w::ForgeReason::NotFound,
                        // temper-07 adds not_installed to the client wire.
                        forge::AdoptionRefusal::NotInstalled | forge::AdoptionRefusal::NoAccess => {
                            w::ForgeReason::NoAccess
                        }
                        forge::AdoptionRefusal::Refused => w::ForgeReason::Refused,
                        forge::AdoptionRefusal::InFlight => w::ForgeReason::InFlight,
                        forge::AdoptionRefusal::TooLarge => w::ForgeReason::TooLarge,
                        forge::AdoptionRefusal::Unreachable => w::ForgeReason::Unreachable,
                        forge::AdoptionRefusal::InvalidAnswer => w::ForgeReason::InvalidAnswer,
                    },
                },
            )
            .ok()?,
        ),
        Some(forge::Answered::SetUp(forge::SetUpAnswer::SettingUp)) => w::Outcome::SettingUp,
        Some(forge::Answered::SetUp(forge::SetUpAnswer::Refused(why))) => w::Outcome::SetUpRefused(
            w::SetUpRefused::new(
                limits,
                w::SetUpRefusedParts {
                    why: match why {
                        forge::SetUpRefusal::NotOffered => w::SetUpRefusal::NotOffered,
                        forge::SetUpRefusal::NotUnderTheBox => w::SetUpRefusal::NotUnderTheBox,
                        forge::SetUpRefusal::Busy => w::SetUpRefusal::Busy,
                    },
                },
            )
            .ok()?,
        ),
        None => w::Outcome::Unknown,
    };
    let answer = w::Answer::new(limits, w::AnswerParts { outcome }).ok()?;
    Some(wire::answer(&answer))
}
fn protection(value: forge::Protection, limits: &w::Limits) -> Option<w::Protection> {
    Some(match value {
        forge::Protection::Unknown => w::Protection::Unknown,
        forge::Protection::Absent => w::Protection::Absent,
        forge::Protection::Rule(rule) => {
            let mut contexts = List::with_capacity(limits.rule_contexts);
            for context in rule.contexts {
                contexts.push(context).ok()?;
            }
            w::Protection::Rule(
                w::Rule::new(
                    limits,
                    w::RuleParts {
                        branch: rule.branch,
                        contexts,
                        approvals: rule.approvals,
                        dismiss_stale: rule.dismiss_stale,
                    },
                )
                .ok()?,
            )
        }
    })
}

/// Encode the connector's whole document without adding policy from settings.
#[must_use]
pub fn forges(value: forge::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut forges = List::with_capacity(limits.forges_forges);
    for row in value.rows {
        forges
            .push(
                w::ForgeRow::new(
                    limits,
                    w::ForgeRowParts {
                        number: row.number,
                        kind: match row.kind {
                            forge::Kind::Forgejo => w::Kind::Forgejo,
                            forge::Kind::GitHub => w::Kind::Github,
                        },
                        mode: match row.mode {
                            forge::Mode::Connected => w::Mode::Connected,
                            forge::Mode::Managed => w::Mode::Managed,
                            forge::Mode::Owned => w::Mode::Owned,
                            forge::Mode::Fake => w::Mode::Fake,
                        },
                        connection: connection(row.connection, limits)?,
                        offer: match row.offer {
                            forge::Offer::NotOffered => w::Offer::NotOffered,
                            forge::Offer::NotUnderTheBox => w::Offer::NotUnderTheBox,
                            forge::Offer::Offered => w::Offer::Offered,
                        },
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(wire::forges(&w::Forges::new(limits, w::ForgesParts { forges }).ok()?))
}
fn connection(value: forge::Connection, limits: &w::Limits) -> Option<w::Connection> {
    Some(match value {
        forge::Connection::NotConnected => w::Connection::NotConnected,
        forge::Connection::NotSetUp => w::Connection::NotSetUp,
        forge::Connection::SettingUp { step } => w::Connection::SettingUp(
            w::SettingUp::new(
                limits,
                w::SettingUpParts {
                    step: match step {
                        None => None,
                        Some(forge::SetUpStep::Fetch) => Some(w::SetUpStep::Fetch),
                        Some(forge::SetUpStep::Prepare) => Some(w::SetUpStep::Prepare),
                        Some(forge::SetUpStep::Run) => Some(w::SetUpStep::Run),
                    },
                },
            )
            .ok()?,
        ),
        forge::Connection::SetUpFailed { why } => {
            w::Connection::SetUpFailed(w::SetUpFailed::new(limits, w::SetUpFailedParts { why }).ok()?)
        }
        forge::Connection::Connecting => w::Connection::Connecting,
        forge::Connection::Connected { id, login } => {
            w::Connection::Connected(w::Connected::new(limits, w::ConnectedParts { id, login }).ok()?)
        }
        forge::Connection::Stopped { reason } => {
            w::Connection::Stopped(w::Stopped::new(limits, w::StoppedParts { reason }).ok()?)
        }
    })
}
