//! Inspect the native store with the same decoding checks as the client.
use jig_client::{Ask, Held, Record};
use jig_client_protocol_link::{Limits, translate};
use jig_client_protocol_wire as w;
use skein_lib::{Reader, RequestRecord};
use temper_client_tools::jig::{Printed, render};

pub(crate) fn print(records: Vec<Record>, json: bool) -> Printed {
    let native = crate::limits::native();
    let limits =
        Limits::new(native.root.domain.client.domain, native.root.domain.client.exchanges).expect("link limits");
    let mut rows = Vec::new();
    let mut unread = 0;
    for record in records {
        if record.key.as_ref() == jig_client::STREAM_KEY {
            continue;
        }
        if let Some(row) = restore(&record, &limits) {
            rows.push(row);
        } else {
            unread += 1;
        }
    }
    render::pending::pending(&rows, unread, json)
}
fn restore(entry: &Record, limits: &Limits) -> Option<RequestRecord<Held>> {
    if entry.key.len() != 16 || entry.bytes.len() as u64 > limits.client.document_bytes {
        return None;
    }
    let mut reader = Reader::new(&entry.bytes);
    let kept = w::Kept::decode(&limits.wire, &mut reader).ok()?;
    if reader.remaining() != 0 || kept.store_version() != jig_client_domain::TABLE_VERSION {
        return None;
    }
    let mut reader = Reader::new(kept.record());
    let row = translate::store::restore_table(w::TableEntry::decode(&limits.wire, &mut reader).ok()?, limits).ok()?;
    if reader.remaining() != 0 || row.key.as_slice() != entry.key.as_ref() {
        return None;
    }
    if let Ask::Family { record, .. } = &row.request.request
        && !limits.client.owners.iter().any(|owner| owner.family == record.family && owner.version == record.version)
    {
        return None;
    }
    Some(row)
}
