//! Deferred standalone preparation with its original admission envelope.
mod cue;
mod watch;
use crate::fixtures;
pub use cue::Cue;
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{
    net::{SocketAddr, TcpListener},
    path::PathBuf,
};

pub struct ServiceHost {
    pub(crate) service: Option<temper_service::Service>,
    pub(crate) address: Option<SocketAddr>,
    pub(crate) link: Vec<u8>,
    watch: watch::Watch,
    cues: Vec<Cue>,
    links: u32,
    path: PathBuf,
    signals: Fd,
    forge: Option<TcpListener>,
    envelope: (u64, u32),
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl ServiceHost {
    pub(crate) fn new(path: PathBuf, signals: Fd) -> Self {
        let forge = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("held forge listener");
        let address = forge.local_addr().expect("held forge address");
        fixtures::write(&path, address, address).expect("stand-in standalone documents");
        let config = std::fs::read(path.join("config.json")).expect("standalone document");
        let envelope = temper::envelope_in(&path, &config).expect("standalone admission envelope");
        Self {
            service: None,
            address: None,
            link: Vec::new(),
            watch: watch::Watch::default(),
            cues: Vec::with_capacity(cue::CUES),
            links: 0,
            path,
            signals,
            forge: Some(forge),
            envelope,
            completions: Queue::with_capacity(1),
            submissions: Queue::with_capacity(1),
        }
    }
}
impl ServiceHost {
    /// Links printed by this service start, independent of the scratch's old file.
    #[must_use]
    pub fn links(&self) -> u32 {
        self.links
    }
    pub(crate) fn arm_watch(&mut self) {
        self.watch.armed = true;
    }
    pub fn arm(&mut self, key: skein_lib::RequestKey) {
        assert!(self.cues.len() < cue::CUES, "bounded held cues");
        self.cues.push(Cue::new(key));
    }
    #[must_use]
    pub fn held(&self) -> bool {
        !self.cues.is_empty() && self.cues.iter().all(Cue::held)
    }
    pub(crate) fn held_at(&self, index: usize) -> bool {
        self.cues.get(index).is_some_and(Cue::held)
    }
    pub(crate) fn watching(&self) -> bool {
        self.watch.armed
    }
    pub(crate) fn delivered(&self) -> bool {
        self.watch.delivered
    }
}
impl Host for ServiceHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.service.is_none()
            && let Some(address) = self.address
        {
            let forge = self.forge.as_ref().expect("held forge").local_addr().expect("forge address");
            fixtures::write(&self.path, address, forge).expect("bound standalone documents");
            let mut service = temper::prepare_in(&self.path).expect("standalone preparation");
            assert_eq!((service.worst_case(), service.operations()), self.envelope, "preparation preserves admission");
            service.adopt_signals(self.signals).expect("service signal pipe");
            self.service = Some(service);
        }
        if let Some(service) = &mut self.service {
            for complete in service.completions().iter() {
                self.watch.complete(complete);
                for cue in &mut self.cues {
                    cue.complete(complete);
                }
            }
            temper_service::iterate(service, now, wall);
            while let Some(link) = service.links().pop() {
                self.links += 1;
                self.link.clear();
                self.link.extend_from_slice(&link);
                self.link.push(b'\n');
                fixtures::private(&self.path.join("link"), &self.link).expect("private printed link");
            }
            if service.is_empty() {
                self.forge = None;
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        match &mut self.service {
            Some(service) => service.completions(),
            None => &mut self.completions,
        }
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        match &mut self.service {
            Some(service) => service.submissions(),
            None => &mut self.submissions,
        }
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.as_ref().map_or(self.address.is_some(), |service| service.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        self.service.as_ref().and_then(temper_service::Service::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.as_ref().and_then(temper_service::Service::next_policy_deadline)
    }
    fn is_empty(&self) -> bool {
        self.service.as_ref().is_some_and(temper_service::Service::is_empty)
    }
    fn worst_case(&self) -> u64 {
        self.envelope.0 + watch::Watch::memory() + cue::CUES as u64 * Cue::memory()
    }
    fn operations(&self) -> u32 {
        self.envelope.1
    }
}
