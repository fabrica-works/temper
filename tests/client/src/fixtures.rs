//! Named story values and one client profile, shared by every story.
use crate::{app::Temper, front::Joint};
use jig_client_world::harness::{Settings, World};
use skein_lib::{Duration, RequestLimits};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use temper_client_domain::{Limits, forge};
use temper_client_view as view;
pub const FORGE: u16 = 0;
pub const PARTY: u64 = 1;
pub const SEED: u64 = 1101;
pub const WITHIN: Duration = Duration::from_secs(1);
#[must_use]
pub fn domain_limits() -> Limits {
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
            projects: 4,
            roles: 2,
            charters: 4,
            text: 32,
            rows: 4,
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

type Books = (jig_view::words::Words, view::words::Words);
static BOOKS: OnceLock<Mutex<BTreeMap<u32, Books>>> = OnceLock::new();
pub fn phrasebooks(limits: &Limits) -> Books {
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books
        .entry(limits.jig.text)
        .or_insert_with(|| {
            (
                jig_view::words::Words {
                    book: jig_view::phrases::english::english(limits.jig.text).expect("jig English fits"),
                    language: 0,
                    offset_minutes: 0,
                },
                view::words::Words::english(FORGE, limits.jig.text).expect("forge English fits"),
            )
        })
        .clone()
}
#[must_use]
pub fn view_bounds(limits: &Limits, jig: &jig_view::words::Words, forge: &view::words::Words) -> view::Limits {
    // The world's storage profile is independent of the all-pages envelope.
    // Builder and the world heap checks enforce these practical capacities.
    let text = jig
        .book
        .jig_capacity()
        .expect("jig phrase envelope")
        .checked_mul(2)
        .expect("fixture phrase size fits")
        .checked_add(limits.jig.text.checked_mul(2).expect("fixture input size fits"))
        .expect("fixture text fits")
        .max(forge.capacity().expect("forge phrase envelope"));
    let bounds = view::Limits {
        tree: jig_view::Limits {
            nodes: 1024,
            depth: limits.jig.rows + 34,
            text_per_node: text,
            text_bytes: 1024 * text,
            drawing_bytes: 0,
            styles_per_node: 2,
            patches: 5120,
            bindings: 1024,
            slots: 256,
            markdown_depth: 3,
        },
    };
    view::worst_case(&bounds, limits, jig, forge).expect("world view bound fits");
    bounds
}
#[must_use]
pub fn inbox() -> temper_client_domain::Address {
    temper_client_domain::Address::Jig(jig_domain_client::address::Address::Inbox { project: None })
}
#[must_use]
pub fn world(seed: u64) -> World<Temper, Joint> {
    let mut world = World::new(
        Settings { seed, heap_bytes_per_client: 64 * 1024 * 1024, iterations: 4096, ..Settings::default() },
        Joint::new(seed, domain_limits()),
    );
    world.randomness(2, 0, 0);
    world
}
#[must_use]
pub fn codec() -> jig_protocol_transcript::Limits {
    jig_protocol_transcript::Limits {
        smith: smith_transcript::v3::CEILINGS,
        bytes: 32768,
        turn: jig_domain_client::contract::TurnLimits { messages: 32, blocks: 32, text: 8192 },
    }
}
