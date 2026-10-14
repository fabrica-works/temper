use super::{Bounds, FITS, Undecodable, answers, api, entries, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Time};
use temper_engine_domain_forge_client as d;

fn encode_what(value: &d::What) -> w::LiveWhat {
    match value {
        d::What::Repository => w::LiveWhat::Repository,
        d::What::Branch(branch) => w::LiveWhat::Branch(
            w::LiveBranch::new(&w::CEILINGS, w::LiveBranchParts { branch: branch.clone() }).expect(FITS),
        ),
        d::What::Pull(number) => {
            w::LiveWhat::Pull(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::What::Issue(number) => {
            w::LiveWhat::Issue(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
    }
}
fn decode_what(value: w::LiveWhat) -> d::What {
    match value {
        w::LiveWhat::Repository => d::What::Repository,
        w::LiveWhat::Branch(value) => d::What::Branch(value.into_parts().branch),
        w::LiveWhat::Pull(value) => d::What::Pull(value.number()),
        w::LiveWhat::Issue(value) => d::What::Issue(value.number()),
    }
}
fn encode_watch(value: &d::Watch) -> w::Watch {
    let d::Watch { resource: d::Resource { repository, what }, participating } = value;
    let resource = w::LiveResource::new(
        &w::CEILINGS,
        w::LiveResourceParts { repository: names::encode_repository(*repository), what: encode_what(what) },
    )
    .expect(FITS);
    w::Watch::new(&w::CEILINGS, w::WatchParts { resource, participating: *participating }).expect(FITS)
}
fn decode_watch(value: w::Watch) -> d::Watch {
    let w::WatchParts { resource, participating } = value.into_parts();
    let w::LiveResourceParts { repository, what } = resource.into_parts();
    d::Watch {
        resource: d::Resource { repository: names::decode_repository(repository), what: decode_what(what) },
        participating,
    }
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_cached(value: &d::Cached) -> w::Cached {
    let d::Cached { item, pull, tip } = value;
    w::Cached::new(
        &w::CEILINGS,
        w::CachedParts {
            item: match item {
                Some(value) => Some(api::encode_summary(value)),
                None => None,
            },
            pull: match pull {
                Some(value) => Some(answers::encode_pull(value)),
                None => None,
            },
            tip: *tip,
        },
    )
    .expect(FITS)
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_cached(value: w::Cached, bounds: &Bounds) -> Result<d::Cached, Undecodable> {
    let w::CachedParts { item, pull, tip } = value.into_parts();
    Ok(d::Cached {
        item: match item {
            Some(value) => Some(api::decode_summary(value, bounds)?),
            None => None,
        },
        pull: match pull {
            Some(value) => Some(answers::decode_pull(value)),
            None => None,
        },
        tip,
    })
}
fn encode_live(value: &d::LiveRecord) -> w::LiveRecord {
    let d::LiveRecord {
        watch,
        position,
        listed,
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments,
        reviews,
        cached,
        pushed,
        echoes,
    } = value;
    w::LiveRecord::new(
        &w::CEILINGS,
        w::LiveRecordParts {
            watch: encode_watch(watch),
            position: entries::encode_position(*position),
            listed: listed.as_nanos(),
            comment_after: *comment_after,
            comment_page: *comment_page,
            comment_scanning: *comment_scanning,
            comment_repair: *comment_repair,
            review_page: *review_page,
            comments: encode_deliveries(comments),
            reviews: encode_deliveries(reviews),
            cached: encode_cached(cached),
            pushed: *pushed,
            echoes: encode_echoes(echoes),
        },
    )
    .expect(FITS)
}
fn decode_live(value: w::LiveRecord, bounds: &Bounds) -> Result<d::LiveRecord, Undecodable> {
    let w::LiveRecordParts {
        watch,
        position,
        listed,
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments,
        reviews,
        cached,
        pushed,
        echoes,
    } = value.into_parts();
    Ok(d::LiveRecord {
        watch: decode_watch(watch),
        position: entries::decode_position(position),
        listed: Time::from_nanos(listed),
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments: decode_deliveries(comments),
        reviews: decode_deliveries(reviews),
        cached: Box::new(decode_cached(cached, bounds)?),
        pushed,
        echoes: decode_echoes(echoes),
    })
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_repository(value: d::RepositoryRecord) -> w::RepositoryRecord {
    let d::RepositoryRecord { repository, mark, clock } = value;
    w::RepositoryRecord::new(
        &w::CEILINGS,
        w::RepositoryRecordParts {
            repository: names::encode_repository(repository),
            mark: match mark {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            clock: clock.as_nanos(),
        },
    )
    .expect(FITS)
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_repository(value: w::RepositoryRecord) -> d::RepositoryRecord {
    let w::RepositoryRecordParts { repository, mark, clock } = value.into_parts();
    d::RepositoryRecord {
        repository: names::decode_repository(repository),
        mark: match mark {
            Some(value) => Some(Time::from_nanos(value)),
            None => None,
        },
        clock: Time::from_nanos(clock),
    }
}
pub(super) fn encode(value: &d::Stored) -> w::ClientRow {
    let value = match value {
        d::Stored::Live(value) => w::ClientValue::Live(Box::new(encode_live(value))),
        d::Stored::Repository(value) => w::ClientValue::Repository(encode_repository(*value)),
    };
    w::ClientRow::new(&w::CEILINGS, w::ClientRowParts { value }).expect(FITS)
}
pub(super) fn decode(value: w::ClientRow, bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    Ok(match value.into_parts().value {
        w::ClientValue::Live(value) => d::Stored::Live(decode_live(*value, bounds)?),
        w::ClientValue::Repository(value) => d::Stored::Repository(decode_repository(value)),
    })
}
fn encode_deliveries(values: &[d::Delivery]) -> List<w::Delivery> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let d::Delivery { id, revision } = *value;
        let item = w::Delivery::new(&w::CEILINGS, w::DeliveryParts { id, revision }).expect(FITS);
        result.push(item).expect("same count");
    }
    result
}
fn decode_deliveries(values: List<w::Delivery>) -> Box<[d::Delivery]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let w::DeliveryParts { id, revision } = value.into_parts();
        result.push(d::Delivery { id, revision }).expect("same count");
    }
    result.into_boxed()
}
fn encode_echoes(values: &[d::Echo]) -> List<w::Echo> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = match value {
            d::Echo::Comment(number) => {
                w::Echo::Comment(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
            }
            d::Echo::Review(number) => {
                w::Echo::Review(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
            }
        };
        result.push(item).expect("same count");
    }
    result
}
fn decode_echoes(values: List<w::Echo>) -> Box<[d::Echo]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = match value {
            w::Echo::Comment(value) => d::Echo::Comment(value.number()),
            w::Echo::Review(value) => d::Echo::Review(value.number()),
        };
        result.push(item).expect("same count");
    }
    result.into_boxed()
}
