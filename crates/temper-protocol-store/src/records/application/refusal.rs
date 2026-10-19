//! Jig's complete refusal facts at its pinned people-store version.
use jig_domain_people as p;
use jig_protocol_store::people_v4 as w;
use skein_lib::Wall;
pub(super) fn encode_failure_class(value: p::FailureClass, _l: &w::Limits) -> w::FailureClass {
    match value {
        p::FailureClass::Transient => w::FailureClass::Transient,
        p::FailureClass::Permanent => w::FailureClass::Permanent,
        p::FailureClass::Run => w::FailureClass::Run,
        p::FailureClass::Agent => w::FailureClass::Agent,
        p::FailureClass::Lost => w::FailureClass::Lost,
        p::FailureClass::Invalid => w::FailureClass::Invalid,
    }
}
pub(super) fn decode_failure_class(value: w::FailureClass) -> p::FailureClass {
    match value {
        w::FailureClass::Transient => p::FailureClass::Transient,
        w::FailureClass::Permanent => p::FailureClass::Permanent,
        w::FailureClass::Run => p::FailureClass::Run,
        w::FailureClass::Agent => p::FailureClass::Agent,
        w::FailureClass::Lost => p::FailureClass::Lost,
        w::FailureClass::Invalid => p::FailureClass::Invalid,
    }
}
pub(super) fn encode_hold_reason(value: &p::HoldReason, l: &w::Limits) -> Result<w::HoldReason, w::Problem> {
    Ok(match value {
        p::HoldReason::Failures { class } => w::HoldReason::Failures(w::HoldReasonFailures::new(
            l,
            w::HoldReasonFailuresParts { class: encode_failure_class(*class, l) },
        )?),
        p::HoldReason::Stalled => w::HoldReason::Stalled,
        p::HoldReason::Dependency { task } => {
            w::HoldReason::Dependency(w::HoldReasonDependency::new(l, w::HoldReasonDependencyParts { task: *task })?)
        }
        p::HoldReason::Stopped => w::HoldReason::Stopped,
        p::HoldReason::StoppedBy { party } => {
            w::HoldReason::StoppedBy(w::HoldReasonStoppedBy::new(l, w::HoldReasonStoppedByParts { party: *party })?)
        }
        p::HoldReason::Drift => w::HoldReason::Drift,
        p::HoldReason::Effects => w::HoldReason::Effects,
        p::HoldReason::EffectFailed => w::HoldReason::EffectFailed,
        p::HoldReason::Uncertain { entry } => {
            w::HoldReason::Uncertain(w::HoldReasonUncertain::new(l, w::HoldReasonUncertainParts { entry: *entry })?)
        }
        p::HoldReason::Budget => w::HoldReason::Budget,
        p::HoldReason::Deadline => w::HoldReason::Deadline,
        p::HoldReason::HoldsWaited => w::HoldReason::HoldsWaited,
        p::HoldReason::ResourceUnavailable => w::HoldReason::ResourceUnavailable,
        p::HoldReason::ResourceAccess => w::HoldReason::ResourceAccess,
        p::HoldReason::PoolLost { pool } => {
            w::HoldReason::PoolLost(w::HoldReasonPoolLost::new(l, w::HoldReasonPoolLostParts { pool: *pool })?)
        }
        p::HoldReason::Procedure => w::HoldReason::Procedure,
    })
}
pub(super) fn decode_hold_reason(value: w::HoldReason) -> p::HoldReason {
    match value {
        w::HoldReason::Failures(value) => {
            p::HoldReason::Failures { class: decode_failure_class(value.class().clone()) }
        }
        w::HoldReason::Stalled => p::HoldReason::Stalled,
        w::HoldReason::Dependency(value) => p::HoldReason::Dependency { task: value.task() },
        w::HoldReason::Stopped => p::HoldReason::Stopped,
        w::HoldReason::StoppedBy(value) => p::HoldReason::StoppedBy { party: value.party() },
        w::HoldReason::Drift => p::HoldReason::Drift,
        w::HoldReason::Effects => p::HoldReason::Effects,
        w::HoldReason::EffectFailed => p::HoldReason::EffectFailed,
        w::HoldReason::Uncertain(value) => p::HoldReason::Uncertain { entry: value.entry() },
        w::HoldReason::Budget => p::HoldReason::Budget,
        w::HoldReason::Deadline => p::HoldReason::Deadline,
        w::HoldReason::HoldsWaited => p::HoldReason::HoldsWaited,
        w::HoldReason::ResourceUnavailable => p::HoldReason::ResourceUnavailable,
        w::HoldReason::ResourceAccess => p::HoldReason::ResourceAccess,
        w::HoldReason::PoolLost(value) => p::HoldReason::PoolLost { pool: value.pool() },
        w::HoldReason::Procedure => p::HoldReason::Procedure,
    }
}
pub(super) fn encode_moved(value: &p::Moved, l: &w::Limits) -> Result<w::Moved, w::Problem> {
    Ok(match value {
        p::Moved::Hold { revision, reason } => w::Moved::Hold(w::MovedHold::new(
            l,
            w::MovedHoldParts { revision: *revision, reason: encode_hold_reason(reason, l)? },
        )?),
        p::Moved::Spec { revision } => w::Moved::Spec(w::MovedSpec::new(l, w::MovedSpecParts { revision: *revision })?),
        p::Moved::Person { revision } => {
            w::Moved::Person(w::MovedPerson::new(l, w::MovedPersonParts { revision: *revision })?)
        }
        p::Moved::Note { revision } => w::Moved::Note(w::MovedNote::new(l, w::MovedNoteParts { revision: *revision })?),
    })
}
pub(super) fn decode_moved(value: w::Moved) -> p::Moved {
    match value {
        w::Moved::Hold(value) => {
            p::Moved::Hold { revision: value.revision(), reason: decode_hold_reason(value.reason().clone()) }
        }
        w::Moved::Spec(value) => p::Moved::Spec { revision: value.revision() },
        w::Moved::Person(value) => p::Moved::Person { revision: value.revision() },
        w::Moved::Note(value) => p::Moved::Note { revision: value.revision() },
    }
}
pub(super) fn encode_party(value: &p::Party, l: &w::Limits) -> Result<w::Party, w::Problem> {
    Ok(match value {
        p::Party::Person(value) => w::Party::Person(w::PartyPerson::new(l, w::PartyPersonParts { value: *value })?),
        p::Party::Task(value) => w::Party::Task(w::PartyTask::new(l, w::PartyTaskParts { value: *value })?),
        p::Party::Deployment { project } => {
            w::Party::Deployment(w::PartyDeployment::new(l, w::PartyDeploymentParts { project: *project })?)
        }
    })
}
pub(super) fn decode_party(value: w::Party) -> p::Party {
    match value {
        w::Party::Person(value) => p::Party::Person(value.value()),
        w::Party::Task(value) => p::Party::Task(value.value()),
        w::Party::Deployment(value) => p::Party::Deployment { project: value.project() },
    }
}
pub(super) fn encode_ending(value: &p::Ending, l: &w::Limits) -> Result<w::Ending, w::Problem> {
    Ok(match value {
        p::Ending::Done => w::Ending::Done,
        p::Ending::Failed => w::Ending::Failed,
        p::Ending::Cancelled { by, at } => w::Ending::Cancelled(w::EndingCancelled::new(
            l,
            w::EndingCancelledParts { by: encode_party(by, l)?, at: at.as_nanos() },
        )?),
        p::Ending::Decided { by, at } => {
            w::Ending::Decided(w::EndingDecided::new(l, w::EndingDecidedParts { by: *by, at: at.as_nanos() })?)
        }
        p::Ending::Deleted => w::Ending::Deleted,
    })
}
pub(super) fn decode_ending(value: w::Ending) -> p::Ending {
    match value {
        w::Ending::Done => p::Ending::Done,
        w::Ending::Failed => p::Ending::Failed,
        w::Ending::Cancelled(value) => {
            p::Ending::Cancelled { by: decode_party(value.by().clone()), at: Wall::from_nanos(value.at()) }
        }
        w::Ending::Decided(value) => p::Ending::Decided { by: value.by(), at: Wall::from_nanos(value.at()) },
        w::Ending::Deleted => p::Ending::Deleted,
    }
}
pub(super) fn encode_lack(value: &p::Lack, l: &w::Limits) -> Result<w::Lack, w::Problem> {
    Ok(match value {
        p::Lack::Approver => w::Lack::Approver,
        p::Lack::Budget { needs, left } => {
            w::Lack::Budget(w::LackBudget::new(l, w::LackBudgetParts { needs: *needs, left: *left })?)
        }
        p::Lack::Grant { kind } => w::Lack::Grant(w::LackGrant::new(l, w::LackGrantParts { kind: *kind })?),
        p::Lack::Funding => w::Lack::Funding,
        p::Lack::TaskKind { executor } => w::Lack::TaskKind(w::LackTaskKind::new(
            l,
            w::LackTaskKindParts {
                executor: match executor {
                    Some(executor) => Some(encode_executor(*executor, l)?),
                    None => None,
                },
            },
        )?),
        p::Lack::Depth { most } => w::Lack::Depth(w::LackDepth::new(l, w::LackDepthParts { most: *most })?),
        p::Lack::Deadline => w::Lack::Deadline,
        p::Lack::Tasks { left } => w::Lack::Tasks(w::LackTasks::new(l, w::LackTasksParts { left: *left })?),
        p::Lack::Tool => w::Lack::Tool,
        p::Lack::Scope => w::Lack::Scope,
        p::Lack::Reference => w::Lack::Reference,
        p::Lack::ResourceAccess => w::Lack::ResourceAccess,
        p::Lack::ResourceUnavailable => w::Lack::ResourceUnavailable,
        p::Lack::Shape => w::Lack::Shape,
        p::Lack::Again { required } => w::Lack::Again(w::LackAgain::new(l, w::LackAgainParts { required: *required })?),
        p::Lack::Account => w::Lack::Account,
        p::Lack::Writer => w::Lack::Writer,
        p::Lack::Required { connector, kind } => {
            w::Lack::Required(w::LackRequired::new(l, w::LackRequiredParts { connector: *connector, kind: *kind })?)
        }
        p::Lack::Failed { connector, kind } => {
            w::Lack::Failed(w::LackFailed::new(l, w::LackFailedParts { connector: *connector, kind: *kind })?)
        }
        p::Lack::Unguarded { connector, kind } => {
            w::Lack::Unguarded(w::LackUnguarded::new(l, w::LackUnguardedParts { connector: *connector, kind: *kind })?)
        }
    })
}
pub(super) fn decode_lack(value: w::Lack) -> p::Lack {
    match value {
        w::Lack::Approver => p::Lack::Approver,
        w::Lack::Budget(value) => p::Lack::Budget { needs: value.needs(), left: value.left() },
        w::Lack::Grant(value) => p::Lack::Grant { kind: value.kind() },
        w::Lack::Funding => p::Lack::Funding,
        w::Lack::TaskKind(value) => match value.into_parts().executor {
            Some(executor) => p::Lack::TaskKind { executor: Some(decode_executor(executor)) },
            None => p::Lack::TaskKind { executor: None },
        },
        w::Lack::Depth(value) => p::Lack::Depth { most: value.most() },
        w::Lack::Deadline => p::Lack::Deadline,
        w::Lack::Tasks(value) => p::Lack::Tasks { left: value.left() },
        w::Lack::Tool => p::Lack::Tool,
        w::Lack::Scope => p::Lack::Scope,
        w::Lack::Reference => p::Lack::Reference,
        w::Lack::ResourceAccess => p::Lack::ResourceAccess,
        w::Lack::ResourceUnavailable => p::Lack::ResourceUnavailable,
        w::Lack::Shape => p::Lack::Shape,
        w::Lack::Again(value) => p::Lack::Again { required: value.required() },
        w::Lack::Account => p::Lack::Account,
        w::Lack::Writer => p::Lack::Writer,
        w::Lack::Required(value) => p::Lack::Required { connector: value.connector(), kind: value.kind() },
        w::Lack::Failed(value) => p::Lack::Failed { connector: value.connector(), kind: value.kind() },
        w::Lack::Unguarded(value) => p::Lack::Unguarded { connector: value.connector(), kind: value.kind() },
    }
}
pub(super) fn encode_limit_kind(value: p::LimitKind, _l: &w::Limits) -> w::LimitKind {
    match value {
        p::LimitKind::Words => w::LimitKind::Words,
        p::LimitKind::Identity => w::LimitKind::Identity,
        p::LimitKind::Roles => w::LimitKind::Roles,
        p::LimitKind::Arithmetic => w::LimitKind::Arithmetic,
        p::LimitKind::Duplicate => w::LimitKind::Duplicate,
        p::LimitKind::Empty => w::LimitKind::Empty,
        p::LimitKind::Batch => w::LimitKind::Batch,
        p::LimitKind::ProjectLiveTasks => w::LimitKind::ProjectLiveTasks,
        p::LimitKind::LiveTasks => w::LimitKind::LiveTasks,
        p::LimitKind::Tree => w::LimitKind::Tree,
        p::LimitKind::Depth => w::LimitKind::Depth,
        p::LimitKind::Delegates => w::LimitKind::Delegates,
        p::LimitKind::Subscription => w::LimitKind::Subscription,
        p::LimitKind::Dependencies => w::LimitKind::Dependencies,
        p::LimitKind::Cycle => w::LimitKind::Cycle,
        p::LimitKind::Spec => w::LimitKind::Spec,
        p::LimitKind::Contract => w::LimitKind::Contract,
        p::LimitKind::Authority => w::LimitKind::Authority,
        p::LimitKind::Inputs => w::LimitKind::Inputs,
        p::LimitKind::Attempts => w::LimitKind::Attempts,
        p::LimitKind::LiveDelegates => w::LimitKind::LiveDelegates,
        p::LimitKind::Restore => w::LimitKind::Restore,
        p::LimitKind::Read => w::LimitKind::Read,
        p::LimitKind::Turn => w::LimitKind::Turn,
        p::LimitKind::HoldKind => w::LimitKind::HoldKind,
        p::LimitKind::HoldTaken => w::LimitKind::HoldTaken,
        p::LimitKind::Holds => w::LimitKind::Holds,
        p::LimitKind::InboxFull => w::LimitKind::InboxFull,
        p::LimitKind::Notes => w::LimitKind::Notes,
        p::LimitKind::Revision => w::LimitKind::Revision,
        p::LimitKind::Question => w::LimitKind::Question,
        p::LimitKind::Priority => w::LimitKind::Priority,
        p::LimitKind::Policy => w::LimitKind::Policy,
        p::LimitKind::Resource => w::LimitKind::Resource,
        p::LimitKind::Service => w::LimitKind::Service,
        p::LimitKind::Operations => w::LimitKind::Operations,
        p::LimitKind::Grace => w::LimitKind::Grace,
        p::LimitKind::Goals => w::LimitKind::Goals,
    }
}
pub(super) fn decode_limit_kind(value: w::LimitKind) -> p::LimitKind {
    match value {
        w::LimitKind::Words => p::LimitKind::Words,
        w::LimitKind::Identity => p::LimitKind::Identity,
        w::LimitKind::Roles => p::LimitKind::Roles,
        w::LimitKind::Arithmetic => p::LimitKind::Arithmetic,
        w::LimitKind::Duplicate => p::LimitKind::Duplicate,
        w::LimitKind::Empty => p::LimitKind::Empty,
        w::LimitKind::Batch => p::LimitKind::Batch,
        w::LimitKind::ProjectLiveTasks => p::LimitKind::ProjectLiveTasks,
        w::LimitKind::LiveTasks => p::LimitKind::LiveTasks,
        w::LimitKind::Tree => p::LimitKind::Tree,
        w::LimitKind::Depth => p::LimitKind::Depth,
        w::LimitKind::Delegates => p::LimitKind::Delegates,
        w::LimitKind::Subscription => p::LimitKind::Subscription,
        w::LimitKind::Dependencies => p::LimitKind::Dependencies,
        w::LimitKind::Cycle => p::LimitKind::Cycle,
        w::LimitKind::Spec => p::LimitKind::Spec,
        w::LimitKind::Contract => p::LimitKind::Contract,
        w::LimitKind::Authority => p::LimitKind::Authority,
        w::LimitKind::Inputs => p::LimitKind::Inputs,
        w::LimitKind::Attempts => p::LimitKind::Attempts,
        w::LimitKind::LiveDelegates => p::LimitKind::LiveDelegates,
        w::LimitKind::Restore => p::LimitKind::Restore,
        w::LimitKind::Read => p::LimitKind::Read,
        w::LimitKind::Turn => p::LimitKind::Turn,
        w::LimitKind::HoldKind => p::LimitKind::HoldKind,
        w::LimitKind::HoldTaken => p::LimitKind::HoldTaken,
        w::LimitKind::Holds => p::LimitKind::Holds,
        w::LimitKind::InboxFull => p::LimitKind::InboxFull,
        w::LimitKind::Notes => p::LimitKind::Notes,
        w::LimitKind::Revision => p::LimitKind::Revision,
        w::LimitKind::Question => p::LimitKind::Question,
        w::LimitKind::Priority => p::LimitKind::Priority,
        w::LimitKind::Policy => p::LimitKind::Policy,
        w::LimitKind::Resource => p::LimitKind::Resource,
        w::LimitKind::Service => p::LimitKind::Service,
        w::LimitKind::Operations => p::LimitKind::Operations,
        w::LimitKind::Grace => p::LimitKind::Grace,
        w::LimitKind::Goals => p::LimitKind::Goals,
    }
}
pub(super) fn encode_refusal(value: &p::Refusal, l: &w::Limits) -> Result<w::Refusal, w::Problem> {
    Ok(match value {
        p::Refusal::NotWellFormed { at, why } => w::Refusal::NotWellFormed(w::RefusalNotWellFormed::new(
            l,
            w::RefusalNotWellFormedParts {
                at: *at,
                why: match why {
                    p::Malformed::ProposedAsAllowed => w::Malformed::ProposedAsAllowed,
                    p::Malformed::Cycle => w::Malformed::Cycle,
                    p::Malformed::AsAllowedTwice => w::Malformed::AsAllowedTwice,
                    p::Malformed::Conversation => w::Malformed::Conversation,
                },
            },
        )?),
        p::Refusal::Role { held } => w::Refusal::Role(w::RefusalRole::new(
            l,
            w::RefusalRoleParts {
                held: match held {
                    Some(value) => Some(encode_role(*value, l)?),
                    None => None,
                },
            },
        )?),
        p::Refusal::Authority { lacking, proposable } => w::Refusal::Authority(w::RefusalAuthority::new(
            l,
            w::RefusalAuthorityParts {
                lacking: encode_lack(lacking, l)?,
                proposable: match proposable {
                    None => None,
                    Some(p::Accepter::Nobody) => Some(w::Accepter::Nobody),
                    Some(p::Accepter::Role(role)) => {
                        Some(w::Accepter::Role(w::AccepterRole::new(l, w::AccepterRoleParts { role: *role })?))
                    }
                },
            },
        )?),
        p::Refusal::Limit { which } => {
            w::Refusal::Limit(w::RefusalLimit::new(l, w::RefusalLimitParts { which: encode_limit_kind(*which, l) })?)
        }
        p::Refusal::Moved { now } => {
            w::Refusal::Moved(w::RefusalMoved::new(l, w::RefusalMovedParts { now: encode_moved(now, l)? })?)
        }
        p::Refusal::Ended { how } => {
            w::Refusal::Ended(w::RefusalEnded::new(l, w::RefusalEndedParts { how: encode_ending(how, l)? })?)
        }
        p::Refusal::Taken { by } => w::Refusal::Taken(w::RefusalTaken::new(l, w::RefusalTakenParts { by: *by })?),
        p::Refusal::NoEscalationHolder => w::Refusal::NoEscalationHolder,
        p::Refusal::NotWaiting => w::Refusal::NotWaiting,
        p::Refusal::NoFurther => w::Refusal::NoFurther,
        p::Refusal::Unknown => w::Refusal::Unknown,
        p::Refusal::NotOffered => w::Refusal::NotOffered,
        p::Refusal::KeyConflict => w::Refusal::KeyConflict,
    })
}
pub(super) fn decode_refusal(value: w::Refusal) -> p::Refusal {
    match value {
        w::Refusal::NotWellFormed(value) => p::Refusal::NotWellFormed {
            at: value.at(),
            why: match value.why() {
                w::Malformed::ProposedAsAllowed => p::Malformed::ProposedAsAllowed,
                w::Malformed::Cycle => p::Malformed::Cycle,
                w::Malformed::AsAllowedTwice => p::Malformed::AsAllowedTwice,
                w::Malformed::Conversation => p::Malformed::Conversation,
            },
        },
        w::Refusal::Role(value) => p::Refusal::Role {
            held: match value.into_parts().held {
                Some(value) => {
                    let result = decode_role(value);
                    Some(result)
                }
                None => None,
            },
        },
        w::Refusal::Authority(value) => p::Refusal::Authority {
            lacking: decode_lack(value.lacking().clone()),
            proposable: match value.proposable() {
                None => None,
                Some(w::Accepter::Nobody) => Some(p::Accepter::Nobody),
                Some(w::Accepter::Role(role)) => Some(p::Accepter::Role(role.role())),
            },
        },
        w::Refusal::Limit(value) => p::Refusal::Limit { which: decode_limit_kind(value.which().clone()) },
        w::Refusal::Moved(value) => p::Refusal::Moved { now: decode_moved(value.now().clone()) },
        w::Refusal::Ended(value) => p::Refusal::Ended { how: decode_ending(value.how().clone()) },
        w::Refusal::Taken(value) => p::Refusal::Taken { by: value.by() },
        w::Refusal::NoEscalationHolder => p::Refusal::NoEscalationHolder,
        w::Refusal::NotWaiting => p::Refusal::NotWaiting,
        w::Refusal::NoFurther => p::Refusal::NoFurther,
        w::Refusal::Unknown => p::Refusal::Unknown,
        w::Refusal::NotOffered => p::Refusal::NotOffered,
        w::Refusal::KeyConflict => p::Refusal::KeyConflict,
    }
}

fn encode_role(v: p::Role, l: &w::Limits) -> Result<w::Role, w::Problem> {
    Ok(match v {
        p::Role::Owner => w::Role::Owner,
        p::Role::Maintainer => w::Role::Maintainer,
        p::Role::Member => w::Role::Member,
        p::Role::Observer => w::Role::Observer,
        p::Role::Policy { role } => w::Role::Policy(w::RolePolicy::new(l, w::RolePolicyParts { role })?),
    })
}
fn decode_role(v: w::Role) -> p::Role {
    match v {
        w::Role::Owner => p::Role::Owner,
        w::Role::Maintainer => p::Role::Maintainer,
        w::Role::Member => p::Role::Member,
        w::Role::Observer => p::Role::Observer,
        w::Role::Policy(v) => p::Role::Policy { role: v.role() },
    }
}

fn encode_executor(value: p::TaskExecutor, limits: &w::Limits) -> Result<w::TaskExecutor, w::Problem> {
    Ok(match value {
        p::TaskExecutor::Agent { charter } => {
            w::TaskExecutor::Agent(w::TaskExecutorAgent::new(limits, w::TaskExecutorAgentParts { charter })?)
        }
        p::TaskExecutor::Procedure { connector, code } => w::TaskExecutor::Procedure(w::TaskExecutorProcedure::new(
            limits,
            w::TaskExecutorProcedureParts { connector, code },
        )?),
        p::TaskExecutor::Person(address) => w::TaskExecutor::Person(w::TaskExecutorPerson::new(
            limits,
            w::TaskExecutorPersonParts { address: encode_person_address(&address, limits)? },
        )?),
    })
}
fn decode_executor(value: w::TaskExecutor) -> p::TaskExecutor {
    match value {
        w::TaskExecutor::Agent(value) => p::TaskExecutor::Agent { charter: value.into_parts().charter },
        w::TaskExecutor::Procedure(value) => {
            let w::TaskExecutorProcedureParts { connector, code } = value.into_parts();
            p::TaskExecutor::Procedure { connector, code }
        }
        w::TaskExecutor::Person(value) => p::TaskExecutor::Person(decode_person_address(value.into_parts().address)),
    }
}

fn encode_person_address(value: &p::PersonAddress, limits: &w::Limits) -> Result<w::PersonAddress, w::Problem> {
    Ok(match value {
        p::PersonAddress::Person(value) => w::PersonAddress::Person(w::PersonAddressPerson::new(
            limits,
            w::PersonAddressPersonParts { value: *value },
        )?),
        p::PersonAddress::Role(value) => {
            w::PersonAddress::Role(w::PersonAddressRole::new(limits, w::PersonAddressRoleParts { value: *value })?)
        }
    })
}
fn decode_person_address(value: w::PersonAddress) -> p::PersonAddress {
    match value {
        w::PersonAddress::Person(value) => {
            let w::PersonAddressPersonParts { value } = value.into_parts();
            p::PersonAddress::Person(value)
        }
        w::PersonAddress::Role(value) => {
            let w::PersonAddressRoleParts { value } = value.into_parts();
            p::PersonAddress::Role(value)
        }
    }
}
