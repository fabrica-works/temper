//! Application records before forge IO and the root's Restart (protocol/forge.md, 2.1).
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_lib::{List, Token};
use temper_protocol_records::{self as records, Rows};
pub(crate) const LOAD: Token = Token::new(crate::tokens::FORGE | 1);
#[derive(Debug)]
pub(crate) struct Loader {
    #[cfg(test)]
    pub(crate) inspected: List<u64>,
    rows: Rows,
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
    pub(crate) fn new(limits: &crate::limits::Limits) -> Self {
        Self {
            #[cfg(test)]
            inspected: List::with_capacity(128),
            rows: Rows::new(),
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
            let mut configured = List::with_capacity(u32::try_from(engine.workers.forges.len()).expect("forges"));
            for forge in &engine.workers.forges {
                configured.push(forge.number).expect("configured forges");
            }
            let mut found = None;
            for number in engine.forge_records.rows.unconfigured(configured.as_slice()) {
                if u32::from(number) >= after {
                    found = Some(number);
                    break;
                }
            }
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
                    engine.forge_records.rows = Rows::new();
                    engine.forge_records.phase = Phase::Done;
                }
            }
        }
        Phase::Done => {}
    }
}

/// All u16 forge groups, diagnostics and a single access/origin handoff.
pub(crate) fn worst_case(settings: &crate::settings::Engine) -> Option<u64> {
    let own = &settings.limits;
    let value = u64::from(own.records.value_bytes);
    let mut address = 64_u64; // Rust's longest socket Display, including an IPv6 scope.
    for forge in &settings.forges {
        if let jig_protocol_config::Address::Host { name, .. } = &forge.api {
            address = address.max(u64::try_from(name.len()).ok()?.checked_add(6)?);
        }
    }
    Rows::worst_case(own.records.value_bytes)?
        .checked_add(u64::try_from(settings.forges.len()).ok()?.checked_mul(value)?)?
        .checked_add(address.checked_mul(2)?)?
        .checked_add(value)?
        .checked_add(u64::from(own.forge_component.key_bytes))?
        .checked_add(u64::from(own.forge.name_bytes))?
        .checked_add(u64::from(own.forges.installations).checked_mul(8)?)
}
