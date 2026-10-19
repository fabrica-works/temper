//! A local store listing exposes operation and party, never its key or name.
use super::{Text, document, outcome::Operation};
use crate::jig::runner::Printed;
use alloc::boxed::Box;
use jig_client::Held;
use skein_json::writer::{Encoder, Limits};
use skein_lib::RequestRecord;

#[must_use]
pub fn pending(rows: &[RequestRecord<Held>], unread: u32, json: bool) -> Printed {
    let stdout = if json {
        let limits = Limits { depth: 3, length: u32::MAX };
        let mut measured = Encoder::measure(&limits);
        encode(rows, unread, &mut measured);
        let mut out = Encoder::write(measured.measured().expect("bounded pending rows"), &limits);
        encode(rows, unread, &mut out);
        out.finish()
    } else {
        let mut measured = Text::measure();
        text(rows, unread, &mut measured);
        let mut out = Text::writing_exact(measured);
        text(rows, unread, &mut out);
        out.finish()
    };
    Printed { status: 0, stdout, stderr: Box::default() }
}
fn text(rows: &[RequestRecord<Held>], unread: u32, out: &mut Text) {
    for row in rows {
        out.put(Operation::of(&row.request.request).word);
        out.put(b", party ");
        out.number(row.scope);
        if let Some(wall) = row.first_sent {
            out.put(b", first sent ");
            document::utc(wall, out);
            out.put(b" UTC\n");
        } else {
            out.put(b", not sent\n");
        }
    }
    if unread > 0 {
        out.number(u64::from(unread));
        out.put(b" kept records these tools cannot read; a call that sends drops them\n");
    }
}
fn encode(rows: &[RequestRecord<Held>], unread: u32, out: &mut Encoder) {
    out.object_start();
    out.key(b"pending");
    out.array_start();
    for row in rows {
        out.object_start();
        out.key(b"operation");
        out.string(Operation::of(&row.request.request).word);
        out.key(b"party");
        out.unsigned(row.scope);
        out.key(b"first_sent");
        if let Some(wall) = row.first_sent {
            out.unsigned(wall.as_nanos());
        } else {
            out.null();
        }
        out.object_end();
    }
    out.array_end();
    out.key(b"unread");
    out.unsigned(u64::from(unread));
    out.object_end();
}
