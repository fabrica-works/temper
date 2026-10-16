use crate::{
    client,
    decode::{Malformed, Step},
    document::{Document, Row},
    limits::Limits,
};
use skein_json::tokenizer;
use skein_lib::{
    Env, Intake, List, Queue, Time, Wall,
    stream::{Down, Up},
};
pub fn bot() -> crate::document::User {
    crate::document::User { id: 1, login: Box::from(*b"fixture"), is_admin: true, full_name: Box::from(*b"temper:bot") }
}
pub fn user(id: u64, login: &[u8]) -> crate::document::User {
    crate::document::User { id, login: Box::from(login), is_admin: false, full_name: Box::from([]) }
}
pub fn limits() -> Limits {
    Limits {
        json: tokenizer::Limits { depth: 16, string: 65_536, number: 32, chunk: 4_096, length: 4_194_304 },
        writer: skein_json::writer::Limits { depth: 16, length: 4_194_304 },
        string_bytes: 1024,
        body_bytes: 65_536,
        inner: 64,
        target_bytes: 4_096,
    }
}
pub fn small() -> Limits {
    Limits { string_bytes: 8, body_bytes: 16, ..limits() }
}
pub fn capped(body: &[u8], cap: u32, limits: &Limits) -> tokenizer::Event {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.json };
    let mut tokenizer = tokenizer::Tokenizer::new(&limits.json);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut intake =
        Intake::with_capacity(u32::try_from(body.len()).unwrap().max(tokenizer::largest_demand(&limits.json)));
    intake.append(body).unwrap();
    tokenizer::down(&mut tokenizer, &env, tokenizer::Request::Text(cap), &mut above, &mut below);
    for _ in 0..body.len().saturating_mul(4).saturating_add(32) {
        if let Some(event) = above.pop() {
            return event;
        }
        match below.pop().expect("tokenizer demand") {
            Down::Demand { read, room: 0 } => {
                let up = match intake.meet(read) {
                    Some(bytes) => Up::Bytes(bytes),
                    None => Up::End,
                };
                tokenizer::up(&mut tokenizer, &env, up, &mut above, &mut below);
            }
            other @ (Down::Demand { .. } | Down::Send(_) | Down::Finish) => panic!("unexpected demand {other:?}"),
        }
    }
    panic!("capped text exhausted the input step bound")
}
pub struct Decoded {
    pub rows: List<Row>,
    pub document: Result<Document, Malformed>,
}
pub fn decode(decoder: client::Decoder, body: &[u8], limits: &Limits) -> Decoded {
    drive(decoder, body, limits, None)
}
pub fn decode_cut(decoder: client::Decoder, body: &[u8], limits: &Limits, after_rows: u32) -> Decoded {
    drive(decoder, body, limits, Some(after_rows))
}
fn drive(mut decoder: client::Decoder, body: &[u8], limits: &Limits, cut_at: Option<u32>) -> Decoded {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.json };
    let mut tokenizer = tokenizer::Tokenizer::new(&limits.json);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut intake = Intake::with_capacity(
        u32::try_from(body.len()).expect("test bytes").max(tokenizer::largest_demand(&limits.json)),
    );
    intake.append(body).expect("test input");
    let mut rows = List::with_capacity(64);
    let mut demand = client::first(&decoder);
    let mut read = None;
    for _ in 0..body.len().saturating_mul(4).saturating_add(32) {
        match read.take() {
            Some(read) => {
                let event = match intake.meet(read) {
                    Some(bytes) => Up::Bytes(bytes),
                    None => Up::End,
                };
                tokenizer::up(&mut tokenizer, &env, event, &mut above, &mut below);
            }
            None => tokenizer::down(&mut tokenizer, &env, demand, &mut above, &mut below),
        }
        match below.pop() {
            Some(Down::Demand { read: r, room: 0 }) => read = Some(r),
            None => {}
            Some(other) => panic!("unexpected stream demand {other:?}"),
        }
        if let Some(event) = above.pop() {
            match client::step(&mut decoder, limits, &event) {
                Step::Demand(next) => demand = next,
                Step::Row(row, next) => {
                    rows.push(row).expect("test rows");
                    demand = if cut_at == Some(rows.len()) { client::cut(&mut decoder) } else { next };
                }
                Step::Done(document) => return Decoded { rows, document: Ok(document) },
                Step::Malformed(error) => return Decoded { rows, document: Err(error) },
            }
        }
    }
    panic!("decoder exhausted the input step bound")
}
