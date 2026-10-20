//! The shell's adapter over the temper composition (jig's protocol/service.md, 7).
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use std::io::{self, Write};
use temper_service::{End, Service, Stop};

pub struct Host {
    pub service: Service,
    output: Box<dyn Write>,
    output_failed: bool,
}
impl Host {
    #[must_use]
    pub fn new(service: Service) -> Self {
        Self { service, output: Box::new(io::stdout()), output_failed: false }
    }
}
impl skein_shell::Host for Host {
    fn iterate(&mut self, now: Time, wall: Wall) {
        temper_service::iterate(&mut self.service, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.service.worst_case()
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
    fn exit(&self) -> Option<Exit> {
        if !self.service.is_empty() {
            return None;
        }
        if let Some(End::Stopped(why)) = self.service.end() {
            let lines = match why {
                Stop::Start(found) => crate::problems(&temper_service::startup::Problems::jig(
                    found.clone(),
                    &jig_protocol_config::Path::root(),
                )),
                Stop::CommitFailed { number } => format!("temper serve: commit {number} failed\n"),
                Stop::Hooks => "temper serve: webhooks listener failed\n".into(),
                Stop::Listener => "temper serve: client listener failed\n".into(),
                Stop::Forge { number, why } => {
                    use skein_forge_connection::{StartRead, Stopped};
                    match why {
                        Stopped::OtherUser { user } => {
                            format!("temper serve: forge {number}: start answered as user {user}\n")
                        }
                        Stopped::Refused { read, status } => {
                            let path = match read {
                                StartRead::User => "/api/v1/user",
                                StartRead::ApiSettings => "/api/v1/settings/api",
                            };
                            format!("temper serve: forge {number}: {path}: status {status}\n")
                        }
                    }
                }
            };
            eprint!("{lines}");
        }
        Some(Exit::Code(u8::from(self.output_failed || self.service.end() != Some(&End::Drained))))
    }
    fn drain(&mut self) {
        while let Some(link) = self.service.links().pop() {
            if self
                .output
                .write_all(&link)
                .and_then(|()| self.output.write_all(b"\n"))
                .and_then(|()| self.output.flush())
                .is_err()
            {
                self.output_failed = true;
                self.service.shutdown();
            }
        }
    }
}
