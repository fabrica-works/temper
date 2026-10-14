//! Outside conformance over actual bytes and independently decoded durable records.
pub(crate) mod durable {
    //! Durability observations from real file completions, independent of the service.
    use jig_domain_core as core;
    use jig_protocol_store::{codec, keys, transcripts};
    use skein_io::kernel::{Complete, Done, Fd, Op};
    use skein_lib::Reader;
    use std::collections::{BTreeMap, VecDeque};
    use temper_engine_domain as root;

    #[derive(Debug)]
    pub struct Commit {
        pub number: u64,
        pub writes: Vec<root::Write>,
    }
    #[derive(Default)]
    pub struct Store {
        pub rows: BTreeMap<root::Key, root::Record>,
        pub pending: VecDeque<Commit>,
        pub applied: u64,
        pub recovered: std::collections::BTreeSet<u64>,
        files: BTreeMap<Vec<u8>, Vec<u8>>,
        lengths: BTreeMap<u64, u64>,
    }
    #[derive(Debug)]
    pub enum Observed {
        Bytes { path: Vec<u8>, at: usize, bytes: Vec<u8> },
        Sync(Vec<u8>),
        Recovered(Vec<u8>),
        Pending(Vec<u8>),
    }
    #[derive(Default)]
    pub struct Capture {
        names: BTreeMap<i32, Vec<u8>>,
        pub events: Vec<Observed>,
    }
    impl Capture {
        pub fn root(&mut self, fd: Fd) {
            self.names.insert(fd.raw(), Vec::new());
        }
        fn path(&self, fd: Fd) -> Option<&[u8]> {
            self.names.get(&fd.raw()).map(Vec::as_slice)
        }
        pub fn complete(&mut self, complete: &Complete) {
            match (&complete.kind, &complete.result) {
                (Op::Open { root, path, .. }, Ok(Done::Fd(fd))) => {
                    let Some(parent) = self.path(*root) else { return };
                    let mut name = parent.to_vec();
                    if !name.is_empty() {
                        name.push(b'/');
                    }
                    if path.as_ref() != b"." {
                        name.extend_from_slice(path);
                    }
                    self.names.insert(fd.raw(), name);
                }
                (Op::Write { fd, bytes, from, at }, Ok(Done::Count(count))) => {
                    if let Some(path) = self.path(*fd) {
                        let from = *from as usize;
                        self.events.push(Observed::Bytes {
                            path: path.to_vec(),
                            at: usize::try_from(*at).expect("fixture file offset"),
                            bytes: bytes[from..from + *count as usize].to_vec(),
                        });
                    }
                }
                (Op::Read { fd, buf, at }, Ok(Done::Count(count))) => {
                    if let Some(path) = self.path(*fd).map(<[u8]>::to_vec) {
                        self.events.push(Observed::Bytes {
                            path: path.clone(),
                            at: usize::try_from(*at).expect("fixture file offset"),
                            bytes: buf[..*count as usize].to_vec(),
                        });
                        if path.starts_with(b"store/") {
                            self.events.push(Observed::Recovered(path.clone()));
                        }
                    }
                }
                (Op::Sync { fd }, Ok(Done::Nothing)) => {
                    if let Some(path) = self.path(*fd) {
                        self.events.push(Observed::Sync(path.to_vec()));
                    }
                }
                (Op::Close { fd }, _) => {
                    self.names.remove(&fd.raw());
                }
                _ => {}
            }
        }
        pub fn pending(&mut self, op: &Op) {
            if let Op::Write { fd, bytes, from: 0, .. } = op
                && self.path(*fd).is_some_and(|path| path.starts_with(b"store/"))
            {
                let frames = frames(bytes);
                let turn = frames.iter().any(|(_, ops)| {
                    ops.iter().any(|op| match op {
                        skein_kv::Op::Put { key, .. } => {
                            matches!(keys::decode_own(key), Some(keys::Own::Transcript { .. }))
                        }
                        skein_kv::Op::Erase { .. } => false,
                    })
                });
                self.events.push(Observed::Pending(bytes.to_vec()));
                let _ = turn;
            }
        }
    }
    impl Store {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }
        pub fn observe(
            &mut self,
            events: &mut Vec<Observed>,
            limits: &codec::Limits,
            connector: &temper_protocol_store::Bounds,
        ) -> Vec<Commit> {
            let mut committed = Vec::new();
            for event in events.drain(..) {
                match event {
                    Observed::Bytes { path, at, bytes } => {
                        let file = self.files.entry(path).or_default();
                        file.resize(file.len().max(at + bytes.len()), 0);
                        file[at..at + bytes.len()].copy_from_slice(&bytes);
                    }
                    Observed::Pending(bytes) => {
                        self.pending = frames(&bytes)
                            .into_iter()
                            .filter(|(number, _)| *number > self.applied)
                            .map(|(number, ops)| self.commit(number, ops, limits, connector))
                            .collect();
                    }
                    Observed::Recovered(path) => {
                        if let Some(bytes) = self.files.get(&path) {
                            self.recovered.extend(frames(bytes).into_iter().map(|(number, _)| number));
                        }
                        committed.extend(self.durable(&path, limits, connector));
                    }
                    Observed::Sync(path) if path.starts_with(b"store/") => {
                        committed.extend(self.durable(&path, limits, connector));
                    }
                    Observed::Sync(_) => {}
                }
            }
            committed
        }
        fn durable(
            &mut self,
            path: &[u8],
            limits: &codec::Limits,
            connector: &temper_protocol_store::Bounds,
        ) -> Vec<Commit> {
            let mut committed = Vec::new();
            let Some(bytes) = self.files.get(path) else { return committed };
            let frames = frames(bytes);
            for (number, ops) in frames {
                if number <= self.applied {
                    continue;
                }
                let commit = self.commit(number, ops, limits, connector);
                for write in &commit.writes {
                    match write {
                        root::Write::Save(row) => {
                            self.rows.insert(row.key(), row.clone());
                            if let root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) = row {
                                let len = transcripts::frame::encode(turn, limits).expect("observed frame bounds").len()
                                    as u64;
                                *self.lengths.entry(turn.task).or_default() += len;
                            }
                        }
                        root::Write::Erase(key) => {
                            self.rows.remove(key);
                        }
                    }
                }
                self.applied = number;
                self.pending.retain(|pending| pending.number > number);
                committed.push(commit);
            }
            committed
        }
        fn commit(
            &self,
            number: u64,
            ops: Box<[skein_kv::Op]>,
            limits: &codec::Limits,
            connector: &temper_protocol_store::Bounds,
        ) -> Commit {
            let mut writes = Vec::new();
            for op in ops {
                match op {
                    skein_kv::Op::Put { key, value } => {
                        if let Some(key) = keys::decode(&key) {
                            let row = codec::decode(&key, &value, limits).expect("real stored row decodes");
                            writes.push(root::Write::Save(root::Record::Core(row)));
                        } else if let Some(keys::Own::Transcript { task }) = keys::decode_own(&key) {
                            let place = codec::component::Transcript::decode(&value).expect("real transcript place");
                            let name = format!("transcripts/{:016x}/{task:016x}", task >> 12).into_bytes();
                            let bytes = self.files.get(&name).expect("transcript write precedes map commit");
                            let mut at = usize::try_from(self.lengths.get(&task).copied().unwrap_or(0))
                                .expect("fixture transcript offset");
                            while at < usize::try_from(place.length).expect("fixture transcript length") {
                                let len =
                                    u32::from_be_bytes(bytes[at..at + 4].try_into().expect("frame length")) as usize;
                                let turn = transcripts::frame::decode(&bytes[at..at + len], limits)
                                    .expect("whole real transcript frame");
                                writes.push(root::Write::Save(root::Record::Core(core::Record::Core(
                                    core::CoreRecord::Turn(turn),
                                ))));
                                at += len;
                            }
                        } else if key.first() == Some(&keys::APPLICATION) {
                            let row = temper_protocol_store::records::decode(&value, connector)
                                .expect("real application row");
                            writes.push(root::Write::Save(root::Record::Forge { row: Box::new(row) }));
                        }
                    }
                    skein_kv::Op::Erase { key } => {
                        if let Some(key) = keys::decode(&key) {
                            writes.push(root::Write::Erase(root::Key::Core(key)));
                        } else if key.first() == Some(&keys::APPLICATION) {
                            let key = temper_protocol_store::keys::decode(&key[1..], connector.forge())
                                .expect("real application key");
                            writes.push(root::Write::Erase(root::Key::Forge(key)));
                        }
                    }
                }
            }
            Commit { number, writes }
        }
    }
    fn frames(bytes: &[u8]) -> Vec<(u64, Box<[skein_kv::Op]>)> {
        let mut reader = Reader::new(bytes);
        let mut frames = Vec::new();
        while reader.remaining() >= 26 {
            let start = bytes.len() - reader.remaining() as usize;
            assert_eq!(reader.bytes(4), Some(b"SKVC".as_slice()), "real KV frame magic");
            assert_eq!(reader.u16(), Some(1), "real KV frame version");
            let number = reader.u64().expect("frame number");
            let count = reader.u32().expect("frame writes");
            let body = reader.u32().expect("frame body length");
            if reader.remaining() < body + 4 {
                break;
            }
            let end = (bytes.len() - reader.remaining() as usize) + body as usize;
            let mut ops = Vec::new();
            for _ in 0..count {
                let kind = reader.u8().expect("write kind");
                let len = reader.u32().expect("key length");
                let key = reader.bytes(len).expect("key").into();
                ops.push(match kind {
                    1 => {
                        let len = reader.u32().expect("value length");
                        skein_kv::Op::Put { key, value: reader.bytes(len).expect("value").into() }
                    }
                    2 => skein_kv::Op::Erase { key },
                    _ => panic!("real KV write kind"),
                });
            }
            assert_eq!((bytes.len() - reader.remaining() as usize), end, "whole frame operations");
            let expected = transcripts::frame::crc32c(&bytes[start..end]);
            assert_eq!(reader.u32(), Some(expected), "real KV frame checksum");
            frames.push((number, ops.into_boxed_slice()));
        }
        frames
    }
}
pub(crate) use durable::{Capture, Store};
pub(crate) mod client;
mod agent {
    //! The worker's actual agent pipe bytes, decoded independently of its root.
    use jig_conformance::referee::{Observed, Receipt};
    use skein_io::kernel::{Complete, Done, Op};
    use skein_lib::Reader;
    use std::collections::BTreeMap;

    #[derive(Default)]
    pub(crate) struct Wire {
        pipes: BTreeMap<i32, (bool, Vec<u8>)>,
        attempt: u64,
        pub events: Vec<Observed>,
    }
    impl Wire {
        pub fn complete(&mut self, complete: &Complete) {
            match (&complete.kind, &complete.result) {
                (Op::Spawn { spawn }, Ok(Done::Spawned { .. }))
                    if spawn.program.as_ref() == b"/opt/temper/bin/temper" =>
                {
                    for pipe in spawn.pipes.iter().filter(|pipe| pipe.child < 2) {
                        self.pipes.insert(pipe.parent.expect("real agent pipe").raw(), (pipe.child == 0, Vec::new()));
                    }
                }
                (Op::Write { fd, bytes, from, .. } | Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(count))) => {
                    self.bytes(fd.raw(), &bytes[*from as usize..(*from + *count) as usize]);
                }
                (Op::Read { fd, buf, .. } | Op::PipeRead { fd, buf }, Ok(Done::Count(count))) => {
                    self.bytes(fd.raw(), &buf[..*count as usize]);
                }
                (Op::Close { fd }, _) => {
                    self.pipes.remove(&fd.raw());
                }
                _ => {}
            }
        }
        fn bytes(&mut self, fd: i32, bytes: &[u8]) {
            let Some((down, buffer)) = self.pipes.get_mut(&fd) else { return };
            buffer.extend_from_slice(bytes);
            let mut frames = Vec::new();
            while buffer.len() >= 8 {
                let kind = u16::from_be_bytes(buffer[..2].try_into().expect("frame kind"));
                assert_eq!(&buffer[2..4], &[0, 0], "agent frame flags");
                let length = u32::from_be_bytes(buffer[4..8].try_into().expect("frame length")) as usize;
                if buffer.len() < 8 + length {
                    break;
                }
                frames.push((*down, kind, buffer[8..8 + length].to_vec()));
                buffer.drain(..8 + length);
            }
            for (down, kind, body) in frames {
                self.frame(down, kind, &body);
            }
        }
        fn frame(&mut self, down: bool, kind: u16, bytes: &[u8]) {
            let limits = crate::fixtures::agent_limits().channel.bodies;
            if down && kind == 0x0100 {
                let start = smith_channel::Start::decode(&limits, &mut Reader::new(bytes)).expect("agent Start bytes");
                assert_eq!(*start.identity(), Some(RUN_ID), "logical chat identity");
                let charter =
                    smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(start.charter()))
                        .expect("agent charter bytes");
                self.attempt = start.activation();
                self.events.push(Observed::Assigned {
                    task: 1,
                    attempt: self.attempt,
                    conversation: 1,
                    resumed: !start.transcript().is_empty(),
                    budget: charter.budget().spend(),
                });
            } else if !down && kind == 0x0109 {
                let turn = smith_channel::Turn::decode(&limits, &mut Reader::new(bytes)).expect("agent Turn bytes");
                self.events.push(Observed::Completion { task: 1, attempt: self.attempt, cumulative: turn.spent() });
            } else if down && kind == 0x0103 {
                let ack =
                    smith_channel::Acknowledge::decode(&limits, &mut Reader::new(bytes)).expect("agent ACK bytes");
                self.events.push(Observed::Released { requires: vec![Receipt::Turn(1, self.attempt, ack.number())] });
            }
        }
    }

    const RUN_ID: [u8; 16] = [245, 11, 125, 102, 96, 250, 0, 253, 157, 194, 14, 99, 169, 218, 74, 33];
}
use crate::world::Proc;
pub(crate) use agent::Wire;
use jig_conformance::referee as rules;
use jig_domain_core as core;
use skein_lib::Time;
use skein_shell::Host;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
use temper_engine_domain as root;
pub(crate) struct Referee {
    controls: skein_world::Controls,
    pub(crate) store: Rc<RefCell<Store>>,
    rules: rules::Referee,
    applied: u64,
    agent_seen: usize,
    client_seen: usize,
    party: Option<u64>,
    handed: bool,
    signed_in: bool,
    released: bool,
    done: Option<()>,
    drained: Option<()>,
}
impl Referee {
    pub(crate) fn new(controls: skein_world::Controls, store: Rc<RefCell<Store>>) -> Self {
        let scope = rules::Scope {
            budget: 100,
            executors: BTreeSet::from([(0, 1)]),
            descendants: 8,
            depth: 8,
            ..rules::Scope::default()
        };
        let policy = rules::Policy {
            deployment: scope.clone(),
            projects: BTreeMap::from([(1, scope.clone())]),
            people: BTreeMap::from([((1, 1), scope)]),
            projections: BTreeMap::new(),
            effect_accepters: BTreeSet::new(),
            requirements: Vec::new(),
            implications: BTreeSet::new(),
            kinds: BTreeMap::new(),
            owned: Vec::new(),
            participating: BTreeSet::new(),
            mechanics: BTreeSet::new(),
            delivery_bound: 10_000_000_000,
            uncertainty_bound: 10_000_000_000,
            story_steps: 100_000,
        };
        Self {
            controls,
            store,
            rules: rules::Referee::new(policy),
            applied: 0,
            agent_seen: 0,
            client_seen: 0,
            party: None,
            handed: false,
            signed_in: false,
            released: false,
            done: None,
            drained: None,
        }
    }
    fn say(&mut self, now: Time, event: rules::Observed) {
        self.rules.observe(now.as_nanos(), event).expect("one-process conformance over bytes and durable rows");
    }
}
pub(crate) fn terminals(store: &Store) -> u32 {
    u32::try_from(store.rows.values().filter(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal))) if matches!(terminal.end, jig_domain_tasks::End::Finished { .. }))).count()).expect("fixture terminals")
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the chat's conformance receipts come from relevant durable core rows"
)]
fn snapshot(store: &Store) -> rules::Snapshot {
    let mut snapshot = rules::Snapshot::default();
    for row in store.rows.values() {
        match row {
            root::Record::Core(core::Record::Tasks(
                jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
            )) => {
                snapshot.tasks.insert(task.number, jig_core_world::observations::task(task));
                snapshot.receipts.insert(rules::Receipt::Task(task.number));
                for word in &task.inbox {
                    snapshot.receipts.insert(rules::Receipt::Word(task.number, word.number));
                }
            }
            root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) => {
                snapshot.receipts.insert(rules::Receipt::Claim(proof.task, proof.attempt));
            }
            root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) => {
                snapshot.receipts.insert(rules::Receipt::Turn(turn.task, turn.attempt, turn.turn));
            }
            root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(end))) => {
                snapshot.receipts.insert(rules::Receipt::Terminal(end.task, end.attempt));
            }
            root::Record::Core(core::Record::People(jig_domain_people::Stored::SignIn { number, .. })) => {
                snapshot.receipts.insert(rules::Receipt::SignIn(*number));
            }
            root::Record::Core(core::Record::People(jig_domain_people::Stored::Answer { key, .. })) => {
                snapshot.receipts.insert(rules::Receipt::Party(key.person, key.key));
            }
            _ => {}
        }
    }
    snapshot
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let (service, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut service[0] else { panic!("service first") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            for proc in other.iter_mut() {
                if let Proc::Client(person) = proc {
                    person.sign_in(&link);
                }
            }
            self.handed = true;
        }
        let done = other.iter().all(|proc| match proc {
            Proc::Client(person) => person.is_empty(),
            Proc::Service(_) | Proc::Agent(_) | Proc::Peer(_) | Proc::Git(_) | Proc::Forge(_) => true,
        });
        if self.done.is_none() && done && terminals(&self.store.borrow()) > 0 {
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.done = Some(());
        }
        if self.done.is_some() {
            for proc in other {
                match proc {
                    Proc::Peer(peer) => peer.shutdown(),
                    Proc::Forge(forge) => forge.shutdown(),
                    Proc::Service(_) | Proc::Agent(_) | Proc::Git(_) | Proc::Client(_) => {}
                }
            }
        }
    }
    fn observe(&mut self, now: Time, procs: &[Proc]) {
        let Proc::Service(service) = &procs[0] else { panic!("service first") };
        let store = self.store.borrow();
        for commit in &service.commits {
            if commit.number <= self.applied {
                continue;
            }
            let mut view = Store::new();
            view.rows = service.rows_at(commit.number);
            self.rules
                .observe(now.as_nanos(), rules::Observed::Durable { number: commit.number, snapshot: snapshot(&view) })
                .expect("durable conformance");
            self.applied = commit.number;
        }
        drop(store);
        for event in &service.wire.events[self.agent_seen..] {
            self.say(now, event.clone());
        }
        self.agent_seen = service.wire.events.len();
        for proc in procs {
            if let Proc::Client(person) = proc {
                for event in &person.wire.received[self.client_seen..] {
                    if let client::Received::SignedIn(party) = event {
                        self.party = Some(*party);
                        self.signed_in = true;
                    }
                    let requires = client_receipts(event, &self.store.borrow(), self.party);
                    self.released |= requires.iter().any(|receipt| matches!(receipt, rules::Receipt::Turn(..)));
                    if !requires.is_empty() {
                        self.say(now, rules::Observed::Released { requires });
                    }
                }
                self.client_seen = person.wire.received.len();
            }
        }
        if self.done.is_some() && self.drained.is_none() && service.is_empty() {
            assert!(self.store.borrow().pending.is_empty(), "drain settles commits");
            self.say(now, rules::Observed::Drained { at: self.applied });
            self.drained = Some(());
        }
        self.say(now, rules::Observed::Tick);
    }
    fn next_deadline(&self) -> Option<Time> {
        self.done.is_none().then_some(Time::from_nanos(10_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.done.is_none() && now >= Time::from_nanos(10_000_000_000)).then(|| {
            format!(
                "chat not complete: link {}, signed in {}, terminals {}",
                self.handed,
                self.signed_in,
                terminals(&self.store.borrow())
            )
        })
    }
    fn passed(&self) -> bool {
        self.done.is_some() && self.released && self.signed_in
    }
}
#[expect(clippy::wildcard_enum_match_arm, reason = "the conformance receipts select committed client documents")]
fn client_receipts(event: &client::Received, store: &Store, party: Option<u64>) -> Vec<rules::Receipt> {
    use jig_domain_client::contract as c;
    match event {
        client::Received::SignedIn(party) => {
            let receipts: Vec<_> = store
                .rows
                .values()
                .filter_map(|row| match row {
                    root::Record::Core(core::Record::People(jig_domain_people::Stored::SignIn {
                        number,
                        person,
                        ..
                    })) if person == party => Some(rules::Receipt::SignIn(*number)),
                    _ => None,
                })
                .collect();
            assert!(!receipts.is_empty(), "sign-in bytes follow a durable sign-in");
            receipts
        }
        client::Received::Answer { key, answer } => {
            let mut receipts = vec![rules::Receipt::Party(party.expect("signed-in request"), *key)];
            if let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Started { task })) = answer {
                receipts.push(rules::Receipt::Task(*task));
            }
            receipts
        }
        client::Received::Delivery(c::Delivery::RunState(run)) if run.committed_turn > 0 => {
            vec![rules::Receipt::Turn(run.task, run.attempt.expect("committed attempt"), run.committed_turn)]
        }
        client::Received::Read(c::ReadValue::Activity(page)) => page
            .rows
            .iter()
            .filter_map(|row| {
                if let c::Activity::Owner { source, .. } = &row.event {
                    row.event.committed_turn().map(|turn| rules::Receipt::Turn(source.task, turn.attempt, turn.number))
                } else {
                    None
                }
            })
            .collect(),
        client::Received::Delivery(_) | client::Received::Read(_) => Vec::new(),
    }
}
