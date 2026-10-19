//! The released caller; hosted calls are added by the tools increment.
use skein_io::kernel::{Complete, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
pub enum Step {}
pub(crate) struct Caller {
    pub(crate) released: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Caller {
    pub(crate) fn new(steps: Vec<Step>) -> Self {
        assert!(steps.is_empty(), "no hosted calls before the tools increment");
        drop(steps);
        Self { released: false, completions: Queue::with_capacity(1), submissions: Queue::with_capacity(1) }
    }
}
impl Host for Caller {
    fn iterate(&mut self, _now: Time, _wall: Wall) {}
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        false
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.released
    }
    fn worst_case(&self) -> u64 {
        u64::try_from(std::mem::size_of::<Complete>() + std::mem::size_of::<Submit>()).expect("queue sizes")
    }
    fn operations(&self) -> u32 {
        self.submissions.capacity()
    }
}
