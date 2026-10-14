use super::{Bounds, FITS, Undecodable, api, names, w};
use skein_lib::{Time, Wall};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_client as d;

fn encode_condition(value: &d::Condition) -> w::Condition {
    match value {
        d::Condition::None => w::Condition::None,
        d::Condition::Merge { base } => w::Condition::Merge(
            w::MergeCondition::new(&w::CEILINGS, w::MergeConditionParts { base: base.clone() }).expect(FITS),
        ),
        d::Condition::Update { head, base } => w::Condition::Update(
            w::UpdateCondition::new(&w::CEILINGS, w::UpdateConditionParts { head: *head, base: *base }).expect(FITS),
        ),
        d::Condition::Review { head } => {
            w::Condition::Review(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: *head }).expect(FITS))
        }
    }
}
fn decode_condition(value: w::Condition) -> d::Condition {
    match value {
        w::Condition::None => d::Condition::None,
        w::Condition::Merge(value) => d::Condition::Merge { base: value.into_parts().base },
        w::Condition::Update(value) => {
            let w::UpdateConditionParts { head, base } = value.into_parts();
            d::Condition::Update { head, base }
        }
        w::Condition::Review(value) => d::Condition::Review { head: *value.commit() },
    }
}
fn encode_effect(value: &d::Effect) -> w::Effect {
    let d::Effect { write, condition } = value;
    w::Effect::new(
        &w::CEILINGS,
        w::EffectParts { write: api::encode_write(write), condition: encode_condition(condition) },
    )
    .expect(FITS)
}
fn decode_effect(value: w::Effect) -> d::Effect {
    let w::EffectParts { write, condition } = value.into_parts();
    d::Effect { write: api::decode_write(write), condition: decode_condition(condition) }
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_entry(value: &d::Entry) -> w::Entry {
    let d::Entry { number, task, repository, effect, start, attempt, failures } = value;
    w::Entry::new(
        &w::CEILINGS,
        w::EntryParts {
            number: *number,
            task: *task,
            repository: names::encode_repository(*repository),
            effect: encode_effect(effect),
            start: match start {
                Some(value) => Some(encode_position(*value)),
                None => None,
            },
            attempt: match attempt {
                Some(value) => Some(encode_attempt(*value)),
                None => None,
            },
            failures: *failures,
        },
    )
    .expect(FITS)
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_entry(value: w::Entry) -> d::Entry {
    let w::EntryParts { number, task, repository, effect, start, attempt, failures } = value.into_parts();
    d::Entry {
        number,
        task,
        repository: names::decode_repository(repository),
        effect: decode_effect(effect),
        start: match start {
            Some(value) => Some(decode_position(value)),
            None => None,
        },
        attempt: match attempt {
            Some(value) => Some(decode_attempt(value)),
            None => None,
        },
        failures,
    }
}
fn encode_position(value: d::Position) -> w::Position {
    let d::Position { at, comment, review, review_page } = value;
    w::Position::new(&w::CEILINGS, w::PositionParts { at: at.as_nanos(), comment, review, review_page }).expect(FITS)
}
fn decode_position(value: w::Position) -> d::Position {
    let w::PositionParts { at, comment, review, review_page } = value.into_parts();
    d::Position { at: Time::from_nanos(at), comment, review, review_page }
}
fn encode_attempt(value: d::Attempt) -> w::Attempt {
    let d::Attempt { sent, deadline, wall, expires } = value;
    w::Attempt::new(
        &w::CEILINGS,
        w::AttemptParts {
            sent: sent.as_nanos(),
            deadline: deadline.as_nanos(),
            wall: wall.as_nanos(),
            expires: expires.as_nanos(),
        },
    )
    .expect(FITS)
}
fn decode_attempt(value: w::Attempt) -> d::Attempt {
    let w::AttemptParts { sent, deadline, wall, expires } = value.into_parts();
    d::Attempt {
        sent: Time::from_nanos(sent),
        deadline: Time::from_nanos(deadline),
        wall: Wall::from_nanos(wall),
        expires: Wall::from_nanos(expires),
    }
}
pub(super) fn encode_entry_row(entry: &d::Entry, held: bool) -> w::EntryRow {
    w::EntryRow::new(&w::CEILINGS, w::EntryRowParts { entry: encode_entry(entry), held }).expect(FITS)
}
pub(super) fn decode_entry_row(value: w::EntryRow) -> top::Stored {
    let w::EntryRowParts { entry, held } = value.into_parts();
    top::Stored::Entry { entry: decode_entry(entry), held }
}
pub(super) fn encode_proposed(value: &top::ProposedEffect) -> w::ProposedEffect {
    let top::ProposedEffect { number, resource, entry } = value;
    w::ProposedEffect::new(
        &w::CEILINGS,
        w::ProposedEffectParts { number: *number, resource: names::encode_what(resource), entry: encode_entry(entry) },
    )
    .expect(FITS)
}
pub(super) fn decode_proposed(value: w::ProposedEffect, bounds: &Bounds) -> Result<top::ProposedEffect, Undecodable> {
    let w::ProposedEffectParts { number, resource, entry } = value.into_parts();
    Ok(top::ProposedEffect { number, resource: names::decode_what(resource, bounds)?, entry: decode_entry(entry) })
}
