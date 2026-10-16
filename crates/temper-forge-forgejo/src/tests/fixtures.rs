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
    let mut pump = Pump::new(body, limits);
    let mut rows = List::with_capacity(64);
    let mut demand = client::first(&decoder);
    loop {
        let event = pump.next(demand);
        match client::step(&mut decoder, limits, &event) {
            Step::Demand(next) => demand = next,
            Step::Row(row, next) => {
                rows.push(row).expect("test rows");
                demand = if cut_at == Some(rows.len()) { client::cut(&mut decoder) } else { next };
            }
            Step::Done(value) => return Decoded { rows, document: Ok(value) },
            Step::Malformed(error) => return Decoded { rows, document: Err(error) },
        }
    }
}
pub fn deliver(header: &[u8], body: &[u8], limits: &Limits) -> Result<crate::delivery::Delivery, Malformed> {
    use crate::client::delivery::{self, Step};
    let mut decoder = delivery::decoder(header, limits).expect("known delivery");
    let mut pump = Pump::new(body, limits);
    let mut demand = delivery::first(&decoder);
    loop {
        let event = pump.next(demand);
        match delivery::step(&mut decoder, limits, &event) {
            Step::Demand(next) => demand = next,
            Step::Done(value) => return Ok(value),
            Step::Malformed(error) => return Err(error),
        }
    }
}
struct Pump {
    env: Env<tokenizer::Limits>,
    tokenizer: tokenizer::Tokenizer,
    above: Queue<tokenizer::Event>,
    below: Queue<Down>,
    intake: Intake,
    remaining: usize,
}
impl Pump {
    fn new(body: &[u8], limits: &Limits) -> Self {
        let mut intake = Intake::with_capacity(
            u32::try_from(body.len()).expect("test bytes").max(tokenizer::largest_demand(&limits.json)),
        );
        intake.append(body).expect("test input");
        Self {
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.json },
            tokenizer: tokenizer::Tokenizer::new(&limits.json),
            above: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
            intake,
            remaining: body.len().saturating_mul(4).saturating_add(32),
        }
    }
    fn next(&mut self, demand: tokenizer::Request) -> tokenizer::Event {
        tokenizer::down(&mut self.tokenizer, &self.env, demand, &mut self.above, &mut self.below);
        while self.remaining > 0 {
            self.remaining = self.remaining.saturating_sub(1);
            if let Some(event) = self.above.pop() {
                return event;
            }
            match self.below.pop().expect("tokenizer demand") {
                Down::Demand { read, room: 0 } => {
                    let up = match self.intake.meet(read) {
                        Some(bytes) => Up::Bytes(bytes),
                        None => Up::End,
                    };
                    tokenizer::up(&mut self.tokenizer, &self.env, up, &mut self.above, &mut self.below);
                }
                other @ (Down::Demand { .. } | Down::Send(_) | Down::Finish) => {
                    panic!("unexpected stream demand {other:?}")
                }
            }
        }
        panic!("decoder exhausted the input step bound")
    }
}
