use super::*;
pub const LONG_LIVE: Duration = Duration::from_secs(31); // Exceeds the ordinary link deadline, below Hello's 60 seconds.
pub const BEFORE_DEADLINE: Duration = Duration::from_secs(29); // Proves the stopping wait cannot finish one second early.
impl Frame {
    pub fn wait(words: &[&str]) -> Self {
        let mut frame = Self::read(words);
        frame.sign_in_then_deliver_me();
        frame
    }
    pub fn advance_live(&mut self, span: Duration) {
        let end = self.now.saturating_add(span);
        while self.now < end {
            self.sockets.get_mut(&self.stream.expect("stream")).expect("stream socket").reply.extend(b": alive\n\n");
            self.now = self.now.saturating_add(crate::limits::DEADLINE).min(end);
            self.run();
        }
    }
    pub fn engine_stopping(&mut self) {
        let event = w::StreamEvent::Ended(
            w::Ended::new(
                &w::CEILINGS,
                w::EndedParts { watch: self.wait_task.expect("wait watch"), why: w::WatchEnd::Stopping },
            )
            .expect("ended"),
        );
        self.sockets.get_mut(&self.stream.expect("open stream")).expect("stream socket").reply.extend(sse(&event));
        self.stopping = Some(());
        self.run();
        self.sockets.get_mut(&self.stream.expect("stream")).expect("stream socket").eof = Some(());
        self.run();
    }
    pub fn change_task(&mut self) {
        self.deliver_task(true);
        self.run();
    }
    pub fn refuse_wait(&mut self) {
        let event = w::StreamEvent::Ended(
            w::Ended::new(
                &w::CEILINGS,
                w::EndedParts { watch: self.wait_task.expect("wait watch"), why: w::WatchEnd::NotAllowed },
            )
            .expect("ended"),
        );
        self.sockets.get_mut(&self.stream.expect("open stream")).expect("stream socket").reply.extend(sse(&event));
        self.run();
    }
    pub(super) fn deliver_task(&mut self, changed: bool) {
        let mut parts = jig_client_test_records::task_record().into_parts();
        if changed {
            parts.project += 1;
        }
        let task = w::Task::new(&w::CEILINGS, parts).expect("task");
        let value = w::DeliveryValue::Task(Box::new(task));
        let document = w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: encode!(value) },
        )
        .expect("task delivery");
        let event = w::StreamEvent::Delivery(
            w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: self.wait_task.expect("wait watch"), document })
                .expect("delivery"),
        );
        self.sockets.get_mut(&self.stream.expect("open stream")).expect("stream socket").reply.extend(sse(&event));
    }
    pub fn no_directory(&self) -> bool {
        self.opened.is_empty() && self.sockets.is_empty() && self.locks == 0
    }
}
