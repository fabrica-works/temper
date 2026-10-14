//! The two newest git identity generations by grant number (credentials.md, 4).
use skein_lib::List;
use temper_worker_protocol_git::Identity;

#[derive(Debug)]
pub struct IdentityTable {
    identities: List<Generations>,
}
#[derive(Debug)]
struct Generations {
    newest: Option<Entry>,
    older: Option<Entry>,
}
#[derive(Debug)]
struct Entry {
    generation: u64,
    value: Identity,
}
/// A grant's number outside the configured table.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unfit;

impl IdentityTable {
    /// Room for numbers `1..=identities`; values have already passed decoding.
    #[must_use]
    pub fn new(identities: u32) -> Self {
        let mut slots = List::with_capacity(identities);
        for _ in 0..identities {
            slots.push(Generations { newest: None, older: None }).expect("one slot per configured identity");
        }
        Self { identities: slots }
    }
    /// Keep the two newest generations, replacing a repeated generation.
    pub fn granted(&mut self, number: u32, generation: u64, identity: Identity) -> Result<(), Unfit> {
        let slot = number.checked_sub(1).ok_or(Unfit)?;
        let slots = self.identities.get_mut(slot).ok_or(Unfit)?;
        let entry = Entry { generation, value: identity };
        match &slots.newest {
            Some(newest) if generation < newest.generation => match &slots.older {
                Some(older) if generation < older.generation => (),
                Some(_) | None => slots.older = Some(entry),
            },
            Some(newest) if generation == newest.generation => slots.newest = Some(entry),
            Some(_) | None => {
                slots.older = slots.newest.take();
                slots.newest = Some(entry);
            }
        }
        Ok(())
    }
    /// The value handed to every operation naming this identity.
    #[must_use]
    pub fn newest(&self, number: u32) -> Option<&Identity> {
        let slot = number.checked_sub(1)?;
        Some(&self.identities.get(slot)?.newest.as_ref()?.value)
    }
}
/// Fixed slots and at most two decoded values within each grant's byte bound.
#[must_use]
pub fn identities_worst_case(identities: u32, value_bytes: u32) -> Option<u64> {
    List::<Generations>::worst_case(identities)?
        .checked_add(u64::from(identities).checked_mul(2)?.checked_mul(u64::from(value_bytes))?)
}
