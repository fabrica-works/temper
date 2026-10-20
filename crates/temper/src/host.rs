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
            };
            eprint!("{lines}");
        }
        Some(Exit::Code(u8::from(self.output_failed || self.service.end() != Some(&End::Drained))))
    }
    fn drain(&mut self) {
        while let Some(note) = self.service.notes().pop() {
            let lines = note_line(note);
            eprint!("{lines}");
        }
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

fn note_line(note: temper_service::Note) -> String {
    match note {
        temper_service::Note::ForgeLacking { number, names } => {
            format!("temper serve: forge {number}: App lacks {}\n", String::from_utf8_lossy(&names))
        }
        temper_service::Note::ForgeStopped { number, why } => stopped_line(number, why),
        temper_service::Note::ForgeNotConnected { number, why } => {
            format!("temper serve: forge {number}: not connected: {why}\n")
        }
        temper_service::Note::ForgeUnconfigured { number } => {
            format!("temper serve: forge {number}: its records are not configured\n")
        }
        temper_service::Note::ForgeUnkept { number, installation } => {
            format!("temper serve: forge {number}: installation {installation}: not kept\n")
        }
    }
}
fn stopped_line(number: u16, why: skein_forge_connection::Stopped) -> String {
    use skein_forge_connection::{StartRead, Stopped, Value};
    match why {
        Stopped::Invalid { value } => {
            let member = match value {
                Value::Credential | Value::Token => "api_token",
                Value::User => "user",
                Value::Key => "app_key",
                Value::Secret => "webhook_secret",
            };
            format!("temper serve: forge {number}: {member}: invalid\n")
        }
        Stopped::KeyRefused => format!("temper serve: forge {number}: app_key: refused\n"),
        Stopped::TokenRefused { installation, status } => {
            format!("temper serve: forge {number}: /app/installations/{installation}/access_tokens: status {status}\n")
        }
        Stopped::OtherUser { user } => {
            format!("temper serve: forge {number}: start answered as user {user}\n")
        }
        Stopped::Refused { read, status } => {
            let path = match read {
                StartRead::Installations => "/app/installations",
                StartRead::Mint => "/app/installations/{id}/access_tokens",
                StartRead::Viewer => "/graphql viewer",
                StartRead::User => "/api/v1/user",
                StartRead::ApiSettings => "/api/v1/settings/api",
            };
            format!("temper serve: forge {number}: {path}: status {status}\n")
        }
    }
}

#[test]
fn github_start_refusals_name_their_paths() {
    use skein_forge_connection::{StartRead, Stopped};
    for (why, words) in [
        (Stopped::KeyRefused, "app_key: refused"),
        (Stopped::TokenRefused { installation: 7, status: 403 }, "/app/installations/7/access_tokens: status 403"),
        (Stopped::Refused { read: StartRead::Installations, status: 401 }, "/app/installations: status 401"),
        (Stopped::Refused { read: StartRead::Mint, status: 401 }, "/app/installations/{id}/access_tokens: status 401"),
        (Stopped::Refused { read: StartRead::Viewer, status: 401 }, "/graphql viewer: status 401"),
    ] {
        assert_eq!(stopped_line(1, why), format!("temper serve: forge 1: {words}\n"));
    }
}

#[test]
fn records_and_installation_notes_name_the_forge_and_reason() {
    use temper_service::{NotConnected, Note};
    assert_eq!(
        note_line(Note::ForgeNotConnected { number: 7, why: NotConnected::Unreadable }),
        "temper serve: forge 7: not connected: its records are unreadable\n"
    );
    assert_eq!(
        note_line(Note::ForgeNotConnected {
            number: 7,
            why: NotConnected::OtherForge {
                kind: skein_forge_connection::settings::Kind::Forgejo,
                api: Box::from("fake.test:443")
            }
        }),
        "temper serve: forge 7: not connected: its records were made for forgejo at fake.test:443\n"
    );
    assert_eq!(
        note_line(Note::ForgeUnconfigured { number: 8 }),
        "temper serve: forge 8: its records are not configured\n"
    );
    assert_eq!(
        note_line(Note::ForgeUnkept { number: 7, installation: 93 }),
        "temper serve: forge 7: installation 93: not kept\n"
    );
}
