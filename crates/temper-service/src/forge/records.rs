//! Application records before forge IO and the root's Restart (protocol/forge.md, 2.1).
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_lib::{List, Map, Secret, Token};
use temper_protocol_records as records;
pub(crate) const LOAD: Token = Token::new(crate::tokens::FORGE | 1);
const FORGES: u32 = 65536;
/// The service retains values only for its configured forges. The codec's
/// general-purpose Rows is filled solely through this holder's checked push.
#[derive(Debug)]
pub(crate) struct Rows {
    values: records::Rows,
    configured: List<u16>,
    unconfigured: Map<u16, ()>,
}
impl Rows {
    fn new(forges: &[crate::settings::Forge]) -> Self {
        let mut configured = List::with_capacity(u32::try_from(forges.len()).expect("configured forges"));
        for forge in forges {
            configured.push(forge.number).expect("configured forges");
        }
        Self { values: records::Rows::new(), configured, unconfigured: Map::with_capacity(FORGES) }
    }
    pub(crate) fn push(&mut self, number: u64, bytes: Secret, value_bytes: u32) {
        if number >= records::CARRIED || number >> 48_u32 != u64::from(records::FAMILY_FORGE) {
            return;
        }
        let Ok(forge) = u16::try_from((number >> 16_u32) & 0xffff_ffff) else { return };
        if !self.configured.as_slice().contains(&forge) {
            self.unconfigured.insert(forge, ()).expect("all u16 forge numbers fit");
            return;
        }
        self.values.push(number, bytes, value_bytes);
    }
    pub(crate) fn forge(&self, forge: u16) -> Option<&records::ForgeRows> {
        self.values.forge(forge)
    }
    fn next_unconfigured(&self, after: u32, configured: &[u16]) -> Option<u16> {
        let after = u16::try_from(after).ok()?;
        for (number, ()) in self.unconfigured.range(after..) {
            if !configured.contains(number) {
                return Some(*number);
            }
        }
        None
    }
    pub(crate) fn worst_case(forges: u32, value_bytes: u32) -> Option<u64> {
        // The codec map allocates as it fills; push bounds it to configured forges.
        Map::<u16, records::ForgeRows>::worst_case(forges)?
            .checked_add(u64::from(forges).checked_mul(10)?.checked_mul(u64::from(value_bytes))?)?
            .checked_add(List::<u16>::worst_case(forges)?)?
            .checked_add(Map::<u16, ()>::worst_case(FORGES)?)
    }
}

#[derive(Debug)]
pub(crate) struct Loader {
    #[cfg(test)]
    pub(crate) inspected: List<u64>,
    pub(crate) rows: Rows,
    phase: Phase,
    pub(crate) bounds: skein_forge_setup::item::Bounds,
    value_bytes: u32,
}
#[derive(Debug)]
enum Phase {
    Load { after: Option<u64> },
    Connect { index: usize },
    Name { after: u32 },
    Done,
}
impl Loader {
    pub(crate) fn new(limits: &crate::limits::Limits, forges: &[crate::settings::Forge]) -> Self {
        Self {
            #[cfg(test)]
            inspected: List::with_capacity(128),
            rows: Rows::new(forges),
            phase: Phase::Load { after: None },
            value_bytes: limits.records.value_bytes,
            bounds: skein_forge_setup::item::Bounds {
                token_bytes: limits.forge_component.token_bytes,
                name_bytes: limits.forge.name_bytes,
                key_bytes: limits.forge_component.key_bytes,
                installations: limits.forges.installations,
            },
        }
    }
    pub(crate) fn loading(&self) -> bool {
        match self.phase {
            Phase::Load { .. } => true,
            Phase::Connect { .. } | Phase::Name { .. } | Phase::Done => false,
        }
    }
    pub(crate) fn done(&self) -> bool {
        matches_done(&self.phase)
    }
    pub(crate) fn loaded(&mut self, rows: Box<[jig_protocol_store::SecretRow]>, next: Option<u64>) {
        for row in rows {
            // Inspect the number alone. Carried rows belong to temper-13, including their bytes.
            if row.number >= records::CARRIED {
                self.phase = Phase::Connect { index: 0 };
                return;
            }
            #[cfg(test)]
            self.inspected.push(row.number).expect("test inspected records");
            self.rows.push(row.number, row.value, self.value_bytes);
        }
        self.phase = match next {
            Some(after) if after < records::CARRIED => Phase::Load { after: Some(after) },
            Some(_) | None => Phase::Connect { index: 0 },
        };
    }
}
fn matches_done(phase: &Phase) -> bool {
    match phase {
        Phase::Done => true,
        Phase::Load { .. } | Phase::Connect { .. } | Phase::Name { .. } => false,
    }
}
pub(crate) fn advance(engine: &mut crate::engine::Engine) {
    if engine.end.is_some() || engine.drain.is_some() {
        return;
    }
    match engine.forge_records.phase {
        Phase::Load { after } => {
            if !engine.store.opened || !engine.store.room() {
                return;
            }
            let request = Secrets::Load { owner: SecretOwner::Application, after };
            match engine.store.secrets(LOAD, request) {
                Some(Event::SecretsLoaded { owner, rows, next }) => {
                    assert!(owner == LOAD, "forge records token");
                    engine.forge_records.loaded(rows, next);
                }
                Some(_) | None => crate::stop::unavailable(engine, b"forge records"),
            }
        }
        Phase::Connect { index } => {
            let Some(forge) = engine.workers.forges.get(index) else {
                engine.forge_records.phase = Phase::Name { after: 0 };
                return;
            };
            if !engine.forge.room(2) || engine.notes.room() == 0 {
                return;
            }
            if forge.kind == crate::settings::Kind::GitHub {
                let kind = skein_forge_connection::settings::Kind::GitHub;
                let expected = records::origin(kind, &forge.api);
                let start = match engine.forge_records.rows.forge(forge.number) {
                    Some(rows) => records::access(kind, &expected, rows, &engine.forge_records.bounds, false),
                    None => Err(records::NotConnected::NoRecords),
                };
                match start {
                    Ok(start) => {
                        engine.forge.down(skein_forge_connection::Request::Connect {
                            forge: forge.number,
                            access: start.access,
                        });
                        engine.forge.down(skein_forge_connection::Request::Installations {
                            forge: forge.number,
                            accepted: start.accepted,
                        });
                        // temper-02 uses start.slug for the git identity's bot author.
                    }
                    Err(why) => engine.notes.push(crate::Note::ForgeNotConnected { number: forge.number, why }),
                }
            }
            // Forgejo keeps its files until temper-15; its grouped rows change nothing.
            engine.forge_records.phase = Phase::Connect { index: index.checked_add(1).expect("configured forges") };
        }
        Phase::Name { after } => {
            let rows = &engine.forge_records.rows;
            let found = rows.next_unconfigured(after, rows.configured.as_slice());
            match found {
                Some(number) => {
                    if engine.notes.room() == 0 {
                        return;
                    }
                    engine.notes.push(crate::Note::ForgeUnconfigured { number });
                    engine.forge_records.phase =
                        Phase::Name { after: u32::from(number).checked_add(1).expect("u16 number") };
                }
                None => {
                    engine.forge_records.phase = Phase::Done;
                }
            }
        }
        Phase::Done => {}
    }
}

/// Configured forge values, unconfigured numbers, diagnostics and one access/origin handoff.
pub(crate) fn worst_case(settings: &crate::settings::Engine) -> Option<u64> {
    let own = &settings.limits;
    let value = u64::from(own.records.value_bytes);
    let mut address = 64_u64; // Rust's longest socket Display, including an IPv6 scope.
    for forge in &settings.forges {
        if let jig_protocol_config::Address::Host { name, .. } = &forge.api {
            address = address.max(u64::try_from(name.len()).ok()?.checked_add(6)?);
        }
    }
    Rows::worst_case(u32::try_from(settings.forges.len()).ok()?, own.records.value_bytes)?
        .checked_add(u64::try_from(settings.forges.len()).ok()?.checked_mul(value)?)?
        .checked_add(address.checked_mul(2)?)?
        .checked_add(value)?
        .checked_add(u64::from(own.forge_component.key_bytes))?
        .checked_add(u64::from(own.forge.name_bytes))?
        .checked_add(u64::from(own.forges.installations).checked_mul(8)?)
}

#[cfg(test)]
#[test]
fn unconfigured_numbers_resume_at_the_cursor_without_retaining_values() {
    let settings = crate::tests::fixtures::engine();
    let mut rows = Rows::new(&settings.forges);
    for forge in [u16::MAX, 77, 0, 77] {
        for item in [records::Item::AppKey, records::Item::Origin] {
            rows.push(records::number(forge, item), Secret::new(Box::from([2, 255])), 4096);
        }
        assert!(rows.forge(forge).is_none());
    }
    // Unknown item codes still identify an unconfigured forge without reading bytes.
    rows.push((u64::from(records::FAMILY_FORGE) << 48) | (78 << 16) | 0x000b, Secret::new(Box::from([])), 4096);
    assert_eq!(rows.unconfigured.len(), 4);
    assert_eq!(rows.next_unconfigured(0, &[1]), Some(0));
    assert_eq!(rows.next_unconfigured(1, &[1]), Some(77));
    assert_eq!(rows.next_unconfigured(78, &[1]), Some(78));
    assert_eq!(rows.next_unconfigured(78, &[1, 78]), Some(u16::MAX));
    assert_eq!(rows.next_unconfigured(u32::from(u16::MAX), &[1]), Some(u16::MAX));
    assert_eq!(rows.next_unconfigured(FORGES, &[1]), None);
    assert_eq!(rows.next_unconfigured(u32::MAX, &[1]), None);
    rows.push(records::number(1, records::Item::User), Secret::new(Box::from([1, 9])), 4096);
    assert!(rows.forge(1).is_some());
    assert_eq!(rows.unconfigured.len(), 4);
}

#[cfg(test)]
#[test]
fn retained_values_are_priced_only_at_the_configured_forge_count() {
    for forges in [0, 1, 9, FORGES] {
        let small = Rows::worst_case(forges, 4096).unwrap();
        let large = Rows::worst_case(forges, 4097).unwrap();
        assert_eq!(large.checked_sub(small), u64::from(forges).checked_mul(10));
    }
}
