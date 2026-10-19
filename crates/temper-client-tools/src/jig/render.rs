//! Bounded terminal bytes and integer JSON (tools contract, 6).
pub mod acts;
pub mod document;
use crate::jig::{Limits, runner::Printed};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Latest, Readout,
    contract::{self, Delivery, ReadNone, ReadResult, ReadValue},
};
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Writer;
/// Application vocabulary for owner parts, borrowed for this step alone.
pub trait Parts {
    fn owner_part(&self, part: &contract::OwnerPart, out: &mut Text);
}
/// A bounded text result; only free text is cut, never identities or authority.
#[derive(Debug)]
pub struct Text {
    writer: Option<Writer>,
    room: usize,
    written: usize,
    left: usize,
}
impl Text {
    pub(crate) fn measure() -> Self {
        Self { writer: None, room: 0, written: 0, left: 0 }
    }
    pub(crate) fn structural(&self) -> usize {
        self.written
    }
    pub(crate) fn bounded(limits: &Limits, structural: usize) -> Self {
        let limit = usize::try_from(limits.text).expect("text bound fits");
        let room = limit.saturating_sub(structural).saturating_sub(64);
        Self { writer: None, room, written: 0, left: 0 }
    }
    pub(crate) fn writing(mut self, limits: &Limits, structural: usize) -> Self {
        self.cut_line();
        let mut output = Self::bounded(limits, structural);
        output.writer = Some(Writer::new(self.written));
        output
    }
    pub fn put(&mut self, bytes: &[u8]) {
        self.written = self.written.checked_add(bytes.len()).expect("document bound");
        if let Some(writer) = &mut self.writer {
            writer.put(bytes).expect("measured document capacity");
        }
    }
    pub fn number(&mut self, mut number: u64) {
        let mut bytes = [0u8; 20];
        let mut start = bytes.len();
        for _ in 0..20u8 {
            start = start.checked_sub(1).expect("decimal digit slot");
            *bytes.get_mut(start).expect("decimal digit slot") =
                b'0'.checked_add(u8::try_from(number % 10).expect("digit fits")).expect("ASCII digit");
            number /= 10;
            if number == 0 {
                break;
            }
        }
        self.put(bytes.get(start..).expect("decimal range"));
    }
    pub fn words(&mut self, bytes: &[u8]) {
        let end = text_end(bytes, self.room);
        self.room = self.room.checked_sub(end).expect("within text budget");
        self.put(bytes.get(..end).expect("text range"));
        self.left = self
            .left
            .checked_add(bytes.len().checked_sub(end).expect("text cut within value"))
            .expect("document bound");
    }
    #[must_use]
    pub fn finish(mut self) -> Box<[u8]> {
        self.cut_line();
        self.writer.expect("only written output is finished").finish()
    }
    fn cut_line(&mut self) {
        if self.left > 0 {
            self.put(b"\n");
            self.number(u64::try_from(self.left).expect("byte count fits"));
            self.put(b" bytes left\n");
        }
    }
}
pub(crate) fn text_end(bytes: &[u8], most: usize) -> usize {
    let mut end = bytes.len().min(most);
    for _ in 0..4u8 {
        match bytes.get(end) {
            Some(byte) if byte & 0xc0 == 0x80 => end = end.checked_sub(1).expect("continuation follows a leading byte"),
            _ => break,
        }
    }
    end
}
#[must_use]
pub fn failure(reason: &[u8], json: bool, status: u8) -> Printed {
    let stderr = if json {
        let limits = JsonLimits { depth: 2, length: 4096 };
        let mut measure = Encoder::measure(&limits);
        error(&mut measure, reason);
        let mut write = Encoder::write(measure.measured().expect("bounded error"), &limits);
        error(&mut write, reason);
        write.finish()
    } else {
        let mut out = Writer::new(reason.len().checked_add(1).expect("bounded error"));
        out.put(reason).expect("measured reason");
        out.put(b"\n").expect("measured newline");
        out.finish()
    };
    Printed { status, stdout: Box::default(), stderr }
}
fn error(out: &mut Encoder, reason: &[u8]) {
    out.object_start();
    out.key(b"error");
    out.string(reason);
    out.object_end();
}
#[must_use]
pub fn me(me: &contract::Me, json: bool, limits: &Limits, owner_part: fn(&contract::OwnerPart, &mut Text)) -> Printed {
    document::print(document::Document::Me(me), json, limits, owner_part)
}
#[must_use]
pub fn read(
    page: &Readout,
    client: &Client,
    me: Handle,
    json: bool,
    limits: &Limits,
    owner_part: fn(&contract::OwnerPart, &mut Text),
) -> Printed {
    match page {
        Readout::Jig(ReadResult::Value(ReadValue::Task(task))) => {
            let party = match client.document(me) {
                Some(document) => match document.latest {
                    Some(Latest::Jig(Delivery::Me(me))) => Some(me.as_ref()),
                    _ => None,
                },
                None => None,
            };
            document::print(document::Document::Task(task, party), json, limits, owner_part)
        }
        Readout::Jig(ReadResult::None(why)) => {
            let (reason, status): (&[u8], u8) = match why {
                ReadNone::Refused => (b"refused", 2),
                ReadNone::Unknown => (b"unknown", 2),
                ReadNone::Gone => (b"gone", 2),
                ReadNone::Busy => (b"busy", 1),
                ReadNone::SignedOut => (b"signed out", 1),
                ReadNone::Unreachable => (b"unreachable", 1),
                ReadNone::Malformed => (b"malformed document", 1),
                ReadNone::TooLarge { .. } => (b"document limit", 1),
            };
            failure(reason, json, status)
        }
        Readout::Limit => failure(b"read table limit", json, 1),
        Readout::NotServed { .. } => failure(b"version not served", json, 1),
        Readout::Family { .. } | Readout::Jig(ReadResult::Value(_)) => {
            failure(b"tools' bug; unexpected document; report it as a finding", json, 1)
        }
    }
}
