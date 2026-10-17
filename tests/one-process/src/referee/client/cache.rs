//! Failure-only metadata of the native client's real private-store operations.
use jig_client_protocol_wire::v8 as wire;
use skein_io::kernel::{Complete, Done, Fd, Op};
use skein_lib::Reader;
use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

pub(crate) struct Table {
    key: [u8; 16],
    scope: u64,
    first_sent: bool,
    version: u16,
}
pub(crate) enum Event {
    Write { table: Vec<Table> },
    Sync { directory: bool, table: Vec<Table> },
    Rename { table: Vec<Table> },
    Restored { table: Vec<Table> },
}
impl fmt::Debug for Table {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Table")
            .field("key", &self.key)
            .field("scope", &self.scope)
            .field("first_sent", &self.first_sent)
            .field("version", &self.version)
            .finish()
    }
}
impl fmt::Debug for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write { table } => formatter.debug_tuple("Write").field(table).finish(),
            Self::Sync { directory, table } => {
                formatter.debug_struct("Sync").field("directory", directory).field("table", table).finish()
            }
            Self::Rename { table } => formatter.debug_tuple("Rename").field(table).finish(),
            Self::Restored { table } => formatter.debug_tuple("Restored").field(table).finish(),
        }
    }
}
pub(crate) type History = Rc<RefCell<Vec<(u32, Event)>>>;
pub(crate) fn saved(history: &History) -> std::collections::BTreeSet<[u8; 16]> {
    let mut saved = std::collections::BTreeSet::new();
    for (_, event) in history.borrow().iter() {
        if let Event::Sync { directory: true, table } = event {
            saved = table.iter().map(|entry| entry.key).collect();
        }
    }
    saved
}
pub(crate) struct Cache {
    names: BTreeMap<i32, Vec<u8>>,
    files: BTreeMap<Vec<u8>, Vec<u8>>,
    history: History,
    life: u32,
    limits: wire::Limits,
}
impl Cache {
    pub(crate) fn new(root: Fd, life: u32, history: History) -> Self {
        Self {
            names: BTreeMap::from([(root.raw(), Vec::new())]),
            files: BTreeMap::new(),
            history,
            life,
            limits: jig_client_protocol_link::Limits::new(
                temper_client_shell_terminal::limits::native().client.client.domain,
                temper_client_shell_terminal::limits::native().client.client.exchanges,
            )
            .expect("native link limits")
            .wire,
        }
    }
    fn path(&self, fd: Fd) -> Option<&Vec<u8>> {
        self.names.get(&fd.raw())
    }
    fn table(&self, path: &[u8]) -> Vec<Table> {
        self.files.get(path).and_then(|bytes| tables(bytes, &self.limits)).unwrap_or_default()
    }
    fn record(&self, event: Event) {
        self.history.borrow_mut().push((self.life, event));
    }
    pub(crate) fn complete(&mut self, complete: &Complete) {
        match (&complete.kind, &complete.result) {
            (Op::Open { root, path, .. }, Ok(Done::Fd(fd))) => {
                let Some(parent) = self.path(*root) else { return };
                let mut name = parent.clone();
                if !name.is_empty() && path.as_ref() != b"." {
                    name.push(b'/');
                }
                if path.as_ref() != b"." {
                    name.extend_from_slice(path);
                }
                self.names.insert(fd.raw(), name);
            }
            (Op::Write { fd, bytes, from, at }, Ok(Done::Count(count))) => {
                let Some(path) = self.path(*fd).cloned() else { return };
                self.bytes(&path, *at, &bytes[*from as usize..(*from + *count) as usize]);
                self.record(Event::Write { table: self.table(&path) });
            }
            (Op::Read { fd, buf, at }, Ok(Done::Count(count))) => {
                let Some(path) = self.path(*fd).cloned() else { return };
                self.bytes(&path, *at, &buf[..*count as usize]);
                if path == b"private/store" {
                    self.record(Event::Restored { table: self.table(&path) });
                }
            }
            (Op::Sync { fd }, Ok(Done::Nothing)) => {
                let Some(path) = self.path(*fd) else { return };
                let directory = path == b"private";
                let table = self.table(if directory { b"private/store" } else { path });
                self.record(Event::Sync { directory, table });
            }
            (Op::Rename { from_dir, from, to_dir, to }, Ok(Done::Nothing)) => {
                let (Some(source), Some(destination)) = (self.path(*from_dir), self.path(*to_dir)) else { return };
                let source = [source.as_slice(), b"/", from].concat();
                let destination = [destination.as_slice(), b"/", to].concat();
                if let Some(bytes) = self.files.remove(&source) {
                    self.files.insert(destination.clone(), bytes);
                }
                if destination == b"private/store" {
                    self.record(Event::Rename { table: self.table(&destination) });
                }
            }
            (Op::Close { fd }, _) => {
                self.names.remove(&fd.raw());
            }
            _ => {}
        }
    }
    fn bytes(&mut self, path: &[u8], at: u64, bytes: &[u8]) {
        let at = usize::try_from(at).expect("private fixture offset");
        let file = self.files.entry(path.to_vec()).or_default();
        file.resize(file.len().max(at + bytes.len()), 0);
        file[at..at + bytes.len()].copy_from_slice(bytes);
    }
}
fn tables(bytes: &[u8], limits: &wire::Limits) -> Option<Vec<Table>> {
    let mut reader = Reader::new(bytes);
    let count = reader.u32()?;
    let mut table = Vec::new();
    for _ in 0..count {
        let length = reader.u32()?;
        let key = reader.bytes(length)?;
        let length = reader.u32()?;
        let record = reader.bytes(length)?;
        let kept = wire::Kept::decode(limits, &mut Reader::new(record)).ok()?;
        let entry = wire::TableEntry::decode(limits, &mut Reader::new(kept.record())).ok()?;
        if key != entry.key().as_slice() {
            return None;
        }
        table.push(Table {
            key: *entry.key(),
            scope: entry.scope(),
            first_sent: entry.first_sent().is_some(),
            version: kept.store_version(),
        });
    }
    reader.is_empty().then_some(table)
}
