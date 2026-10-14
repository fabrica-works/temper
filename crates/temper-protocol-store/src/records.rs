//! Total translations between durable domain rows and generated records.
use crate::{Bounds, Undecodable, v1 as w};
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};
use temper_engine_domain_forge as d;
mod names;
const FITS: &str = "admitted fields fit the schema";

/// Encode a supported family admitted by the domain; later families return none.
#[must_use]
pub fn encode(value: &d::Stored) -> Option<Box<[u8]>> {
    let value = match value {
        d::Stored::Repository(value) => w::Value::Repository(names::encode_adopted(value)),
        d::Stored::Hold(value) => w::Value::Hold(names::encode_hold(value)),
        d::Stored::Call { .. }
        | d::Stored::ProposedEffect(_)
        | d::Stored::Entry { .. }
        | d::Stored::Client(_)
        | d::Stored::Change(_)
        | d::Stored::Issue(_)
        | d::Stored::Names { .. }
        | d::Stored::Subscription(_)
        | d::Stored::BranchHead(_)
        | d::Stored::PullState(_)
        | d::Stored::Ci(_)
        | d::Stored::Landed { .. }
        | d::Stored::Release(_) => return None,
    };
    let row = w::Row::new(&w::CEILINGS, w::RowParts { value }).expect("admitted row fits the schema");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("row length fits"));
    row.encode(&mut writer).expect("measured room");
    Some(writer.finish())
}

/// Decode one supported row with the checked startup bounds.
pub fn decode(bytes: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = w::Row::decode(bounds.limits(), &mut Reader::new(bytes)).or(Err(Undecodable))?;
    Ok(match row.into_parts().value {
        w::Value::Repository(value) => d::Stored::Repository(names::decode_adopted(value, bounds)?),
        w::Value::Hold(value) => d::Stored::Hold(names::decode_hold(value, bounds)?),
    })
}
