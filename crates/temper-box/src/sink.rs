//! One queued line is one write grant; bytes from different pipes stay apart.
use alloc::boxed::Box;
use skein_io::Request;
use skein_lib::{
    Queue, Token,
    stream::{Down, Read, Up},
};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Idle,
    Waiting,
    Failed,
    Closing,
    Closed,
}
#[derive(Debug)]
pub struct Sink {
    pub(crate) token: Token,
    queued: Queue<Box<[u8]>>,
    phase: Phase,
}
impl Sink {
    #[must_use]
    pub fn new(token: Token, entries: u32) -> Self {
        Self { token, queued: Queue::with_capacity(entries), phase: Phase::Idle }
    }
    #[must_use]
    pub fn room(&self) -> bool {
        self.phase == Phase::Idle || self.phase == Phase::Waiting
    }
    #[must_use]
    pub fn takes(&self) -> bool {
        self.room() && self.queued.room() > 0
    }
    pub fn push(&mut self, bytes: Box<[u8]>) -> Result<(), Box<[u8]>> {
        if !self.room() {
            return Err(bytes);
        }
        self.queued.try_push(bytes)
    }
    /// Reports a broken output once.
    pub fn up(&mut self, up: Up, out: &mut Queue<Request>) -> bool {
        match up {
            Up::Room => {
                if self.phase == Phase::Waiting {
                    let bytes = self.queued.pop().expect("demanded front line");
                    out.push(Request::Stream { stream: self.token, down: Down::Send(bytes) });
                    self.phase = Phase::Idle;
                }
            }
            Up::Failed(_) => {
                if self.phase != Phase::Failed && self.phase != Phase::Closing && self.phase != Phase::Closed {
                    self.phase = Phase::Failed;
                    for _ in 0..self.queued.len() {
                        drop(self.queued.pop());
                    }
                    return true;
                }
            }
            Up::Bytes(_) | Up::End => {}
        }
        false
    }
    pub fn tick(&mut self, finish: bool, out: &mut Queue<Request>) {
        if self.phase == Phase::Idle {
            if let Some(front) = self.queued.iter().next() {
                let room = u32::try_from(front.len()).expect("bounded line");
                out.push(Request::Stream { stream: self.token, down: Down::Demand { read: Read::Nothing, room } });
                self.phase = Phase::Waiting;
            } else if finish {
                out.push(Request::Close { entity: self.token });
                self.phase = Phase::Closing;
            }
        } else if self.phase == Phase::Failed {
            out.push(Request::Close { entity: self.token });
            self.phase = Phase::Closing;
        }
    }
    pub fn closed(&mut self) {
        self.phase = Phase::Closed;
    }
    pub(crate) fn worst_case(entries: u32, bytes: u32) -> u64 {
        Queue::<Box<[u8]>>::worst_case(entries)
            .expect("sink queue")
            .saturating_add(u64::from(entries).saturating_mul(u64::from(bytes)))
    }
    #[must_use]
    pub fn pending(&self) -> bool {
        self.phase == Phase::Idle && !self.queued.is_empty() || self.phase == Phase::Failed
    }
}
