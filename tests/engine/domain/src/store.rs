//! Temper record translation over jig's ordered store fake.
//! Root writes wait in the caller-owned protocol queue; durability belongs to
//! jig-fake-store. Paging selects the application's store-key families.
use std::collections::{BTreeMap, VecDeque};
use temper_engine_domain::{Deployment, JournalLimits, Key, Range, Record, Write};

pub const LIMITS: JournalLimits = JournalLimits {
    commits: 3,
    held: 12,
    writes: 4,
    deliveries: 4,
    transcript_bytes: 128,
    result_bytes: 128,
    run_bytes: 1,
};

pub const HEADER: Deployment = Deployment {
    id: [31; 16],
    tasks: 0,
    people: 0,
    sign_ins: 0,
    messages: 0,
    runs: 0,
    calls: 0,
    connector_rows: 0,
    commits: 0,
};

/// Only the domain's storage contract: numbered transactions apply whole in
/// order. A completion can be lost after applying; a fresh process sees it.
#[derive(Debug)]
pub struct Store {
    inner: jig_fake_store::Store<Key, Record>,
    pub pending: VecDeque<(u64, Box<[Write]>)>,
}

impl Store {
    #[must_use]
    pub fn new() -> Store {
        let mut inner = jig_fake_store::Store::new();
        inner.rows = BTreeMap::from([(
            Key::Core(jig_core::Key::Core(jig_core::CoreKey::Deployment)),
            Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Deployment(HEADER))),
        )]);
        Store { inner, pending: VecDeque::new() }
    }

    pub fn apply(&mut self) -> u64 {
        let (number, writes) = self.pending.pop_front().expect("queued transaction");
        let writes = writes
            .into_vec()
            .into_iter()
            .map(|write| match write {
                Write::Save(row) => jig_fake_store::Write::Save { key: row.key(), row },
                Write::Erase(key) => jig_fake_store::Write::Erase(key),
            })
            .collect();
        self.inner.submit(number, writes, 0);
        assert_eq!(self.inner.tick(false), Ok(Some(number)), "one ordered transaction applied");
        let Some(Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Deployment(header)))) =
            self.rows.get(&Key::Core(jig_core::Key::Core(jig_core::CoreKey::Deployment)))
        else {
            panic!("durable deployment");
        };
        assert_eq!(header.commits, number, "header and records applied atomically");
        number
    }

    #[must_use]
    pub fn header(&self) -> Deployment {
        let Some(Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Deployment(header)))) =
            self.rows.get(&Key::Core(jig_core::Key::Core(jig_core::CoreKey::Deployment)))
        else {
            panic!("deployment");
        };
        *header
    }

    /// Key-range paging, derived directly from the fake store's durable map.
    /// Each page is a separate request and has a cursor only when rows remain.
    #[must_use]
    pub fn page(&self, range: &Range, after: Option<&Key>, most: u32) -> (Box<[Record]>, Option<Key>) {
        let count = usize::try_from(most).expect("small page");
        let mut selected: Vec<Record> = self
            .rows
            .iter()
            .filter(|(key, _)| range.contains(key) && after.is_none_or(|old| *key > old))
            .take(count.saturating_add(1))
            .map(|(_, row)| row.clone())
            .collect();
        let more = selected.len() > count;
        if more {
            selected.pop();
        }
        let rows: Box<[Record]> = selected.into_boxed_slice();
        let next = if more { Some(rows.last().expect("positive page").key()) } else { None };
        (rows, next)
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for Store {
    type Target = jig_fake_store::Store<Key, Record>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl core::ops::DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
