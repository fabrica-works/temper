//! Named story values and one client profile, shared by every story.
use crate::{app::Temper, front::Joint};
use jig_client_world::harness::FrontFault;
use jig_client_world::harness::{Settings, World};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, RequestLimits, Time, Token};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use temper_client_domain::{Limits, forge};
use temper_client_view as view;
use temper_engine_domain as engine;
use temper_engine_domain_world::direct::Driver;
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const FORGE_HOST: &[u8] = b"forge.example";
pub const REPOSITORIES: &[(&[u8], &[u8], u32)] = &[(b"acme", b"shop", 7), (b"acme", b"second", 8)];
pub const PARTY: u64 = 1;
pub const SEED: u64 = 1101;
pub const WITHIN: Duration = Duration::from_secs(1);
#[must_use]
pub fn domain_limits() -> Limits {
    {
        let jig = jig_client_view::Limits {
            segments: 9,
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
            text: 64,
            rows: 4,
            conversation_words: 2,
            navigation_entries: 8,
            updates_count: 99,
        };
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        Limits {
            jig,
            forge: forge::Limits { parts: 8, field_bytes: 256, checks: 8 },
            client: temper_client_domain::limits::client(
                &jig,
                requests,
                8,
                jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                Duration::from_millis(100),
            )
            .expect("client profile fits"),
        }
    }
}

type Books = (jig_client_view::words::Words, view::words::Words);
static BOOKS: OnceLock<Mutex<BTreeMap<u32, Books>>> = OnceLock::new();
pub fn phrasebooks(limits: &Limits) -> Books {
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books
        .entry(limits.jig.text)
        .or_insert_with(|| {
            (
                jig_client_view::words::Words {
                    book: jig_client_view::phrases::english::english(limits.jig.text).expect("jig English fits"),
                    language: 0,
                    offset_minutes: 0,
                },
                view::words::Words::english(FORGE, limits.jig.text).expect("forge English fits"),
            )
        })
        .clone()
}
#[must_use]
pub fn view_bounds(limits: &Limits, jig: &jig_client_view::words::Words, forge: &view::words::Words) -> view::Limits {
    // The world's storage profile is independent of the all-pages envelope.
    // Builder and the world heap checks enforce these practical capacities.
    let text = jig_client_view::phrases::jig_capacity(&jig.book)
        .expect("jig phrase envelope")
        .checked_mul(2)
        .expect("fixture phrase size fits")
        .checked_add(limits.jig.text.checked_mul(2).expect("fixture input size fits"))
        .expect("fixture text fits")
        .max(forge.capacity().expect("forge phrase envelope"));
    let bounds = view::Limits {
        tree: skein_ui::Limits {
            nodes: 1024,
            depth: limits.jig.rows + 34,
            text_per_node: text,
            text_bytes: 1024 * text,
            drawing_bytes: 0,
            scene_marks: 0,
            scene_bytes: 0,
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
    temper_client_domain::Address::Jig(jig_client_view::address::Address::Inbox { project: None })
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
        turn: jig_client::contract::TurnLimits { messages: 32, blocks: 32, text: 8192 },
    }
}

#[must_use]
pub fn conversations() -> temper_client_domain::Address {
    temper_client_domain::Address::Jig(jig_client_view::address::Address::Conversations { project: None })
}
#[must_use]
pub fn chats(driver: &Driver) -> u32 {
    u32::try_from(
        driver
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(
                    record,
                    engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                    if matches!(row.requester, tasks::Party::Person(_)) && row.tracked.is_none()
                )
            })
            .count(),
    )
    .expect("bounded chats")
}
#[must_use]
pub fn faults(seed: u64) -> Vec<(Time, FrontFault)> {
    let choices = [
        FrontFault::Restart { offline: Duration::from_millis(30) },
        FrontFault::NoAnswer,
        FrontFault::DropWatch,
        FrontFault::Busy,
        FrontFault::SignedOut,
    ];
    (0..4)
        .map(|index| {
            (
                Time::from_nanos(Duration::from_millis(20 * (index + 1)).as_nanos()),
                choices[usize::try_from((seed + index) % 5).expect("five faults")].clone(),
            )
        })
        .collect()
}
/// A chat made and held by two failed real worker attempts.
pub fn held_chat(front: &mut Joint) -> u64 {
    let driver = &mut front.engine;
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: skein_lib::ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask: jig_domain_people::Ask::StartChat { project: 1, words: b"Held chat".as_slice().into() },
    }));
    driver.settle();
    let first = driver.last_assignment();
    driver.answer(
        Token::new(7),
        first.task,
        first.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    driver.answer(
        Token::new(7),
        second.task,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    assert!(matches!(driver.task(first.task).expect("held chat").phase, tasks::Phase::Held { .. }));
    first.task
}

/// The stored ended task a client's one-shot task/result read asks for.
#[must_use]
pub fn read(
    store: &temper_engine_domain_world::store::Store,
    range: &jig_domain_core::reads::Range,
) -> jig_domain_core::reads::Loaded {
    use jig_domain_core::{Key, Record, reads};
    let (reads::Scope::Task { task } | reads::Scope::Result { task } | reads::Scope::PersonTask { task }) = range.scope
    else {
        return reads::Loaded::Gone;
    };
    let Some(engine::Record::Core(Record::Tasks(tasks::Stored::Ended(row)))) =
        store.rows.get(&engine::Key::Core(Key::Tasks(tasks::Key::Ended(task))))
    else {
        return reads::Loaded::Gone;
    };
    let value = reads::Row {
        value: Record::Tasks(tasks::Stored::Ended(row.clone())),
        ended_read: None,
        owner: Some(row.clone()),
        ancestors: Box::new([]),
        related: None,
        unjoined: None,
        linked: Box::new([]),
        inbox: None,
    };
    if range.after.is_some() {
        return reads::Loaded::Page { rows: Box::new([]), next: None };
    }
    if range.rows == 0 || value.owned_bytes().expect("finite ended row") > range.bytes {
        return reads::Loaded::Busy;
    }
    reads::Loaded::Page { rows: Box::new([value]), next: None }
}
#[must_use]
pub fn project() -> temper_client_domain::Address {
    temper_client_domain::Address::Jig(jig_client_view::address::Address::Project {
        project: 1,
        section: jig_client_view::address::ProjectSection::Overview,
    })
}
#[must_use]
pub fn adopt(owner: &[u8], name: &[u8]) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    vec![
        Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within: WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Adopt a repository") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: owner.into() },
        Step::Type { find: Find::named(Role::TextField, b"Name"), words: name.into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
    ]
}
#[must_use]
pub fn adopt_then_open_resources(
    owner: &[u8],
    name: &[u8],
) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    let mut steps = adopt(owner, name);
    steps.extend([
        Step::See { find: Find::text(b"Adopted acme/shop on forge 1 as owned."), within: WITHIN },
        Step::Press { find: Find::named(Role::Link, b"See the project's resources") },
        Step::See { find: Find::named(Role::Heading { level: 1 }, b"Project"), within: WITHIN },
    ]);
    steps
}
