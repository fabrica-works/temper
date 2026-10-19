//! Observe the hosted shell's kernel lock without changing its requests.
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
pub(crate) struct Recorded {
    pub(crate) host: temper_client_shell_cli::host::OneShot,
    pub(crate) call: usize,
    pub(crate) locked: Option<Time>,
    pub(crate) released: Option<Time>,
    pub(crate) busy: u32,
    lock: Option<Token>,
    descriptor: Option<Fd>,
}
impl Recorded {
    pub(crate) fn new(host: temper_client_shell_cli::host::OneShot, call: usize) -> Self {
        Self { host, call, locked: None, released: None, busy: 0, lock: None, descriptor: None }
    }
}
impl Host for Recorded {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for complete in self.host.completions().iter() {
            if let Op::Lock { fd } = complete.kind {
                assert_eq!(self.lock.take(), Some(complete.op), "submitted lock");
                match complete.result {
                    Ok(Done::Nothing) => {
                        self.locked = Some(now);
                        self.descriptor = Some(fd);
                    }
                    Err(Error::Busy) => self.busy += 1,
                    _ => panic!("lock completion {:?}", complete.result),
                }
            } else if matches!(complete.kind, Op::Close { fd } if Some(fd) == self.descriptor) {
                assert_eq!(complete.result, Ok(Done::Nothing), "locked descriptor closed");
                self.released = Some(now);
                self.descriptor = None;
            }
        }
        self.host.iterate(now, wall);
    }
    fn drain(&mut self) {
        self.host.drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        for submit in self.host.submissions().iter() {
            if matches!(submit.kind, Op::Lock { .. }) {
                self.lock = Some(submit.op);
            }
        }
        self.host.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host.exit()
    }
    fn worst_case(&self) -> u64 {
        self.host.worst_case()
    }
    fn operations(&self) -> u32 {
        self.host.operations()
    }
}
