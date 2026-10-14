use crate::{Limits, forge};
use jig_domain_client::contract::{
    self, Delivery, DocumentText, Frame, FrameParty, HeldRole, Needs, PartyKind, WatchEvent,
};
use skein_lib::{Duration, Env, List, RequestLimits, Time, Wall};
pub const FORGE: u16 = 1;
pub const OTHER: u16 = 2;
pub const PROJECT: u32 = 1;
pub const SEED: u64 = 11;
pub fn limits() -> Limits {
    Limits {
        jig: jig_domain_client::Limits {
            watches: 8,
            notices: 2,
            notice_life: Duration::from_secs(5),
            // A finite fixture keeps the same before/after-linger behavior at a short scale.
            linger: Duration::from_millis(250),
            save_delay: Duration::from_millis(10),
            intents: 2,
            fields: 4,
            providers: 1,
            objects: 3,
            sources: 8,
            reads: 8,
            parts: 8,
            delegates: 2,
            document_bytes: 65536,
            records: 2,
            kept: 4,
            drafts: 1,
            projects: 2,
            roles: 2,
            charters: 2,
            text: 32,
            rows: 2,
            conversation_words: 2,
            navigation_entries: 8,
            updates_count: 99,
            backoff: jig_domain_client::limits::Backoff {
                first: Duration::from_millis(10),
                most: Duration::from_millis(80),
            },
            heartbeat: Duration::from_millis(100),
        },
        forge: forge::Limits { parts: 8 },
        table: RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        },
    }
}
pub fn env(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}
pub fn connector() -> contract::Part {
    contract::Part::Connector { connector: FORGE }
}
pub fn resource() -> contract::Part {
    let mut path = List::with_capacity(2);
    path.push(Box::from(*b"acme")).expect("fixture path fits");
    path.push(Box::from(*b"shop")).expect("fixture path fits");
    contract::Part::Resource { connector: FORGE, path }
}
pub fn frame(party: u64) -> WatchEvent {
    WatchEvent::Delivered(Delivery::Frame(Box::new(Frame {
        party: FrameParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(contract::Text::new(Box::from(*b"fixture-login"), 32).expect("login fits")),
            name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits")),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: contract::Count { value: 0, limited: false },
        results_seen: 0,
        charters: List::with_capacity(0),
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    })))
}
