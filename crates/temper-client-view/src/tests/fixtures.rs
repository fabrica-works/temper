use jig_domain_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Wall, Writer};
use temper_client_domain::{Limits, forge};
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
pub fn words() -> crate::words::Words {
    crate::words::Words::english(FORGE, 64).expect("forge English fits")
}
pub fn jig_words() -> jig_view::words::Words {
    jig_view::words::Words {
        book: jig_view::phrases::english::english(32).expect("jig English fits"),
        language: 0,
        offset_minutes: 0,
    }
}
pub fn tree_limits() -> jig_view::Limits {
    jig_view::Limits {
        nodes: 128,
        depth: 32,
        text_per_node: 512,
        text_bytes: 65536,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 640,
        bindings: 128,
        slots: 32,
        markdown_depth: 3,
    }
}
pub fn domain_limits() -> Limits {
    limits()
}
pub fn path(segments: &[&[u8]]) -> List<Box<[u8]>> {
    let mut path = List::with_capacity(u32::try_from(segments.len()).expect("fixture segments fit"));
    for segment in segments {
        path.push(skein_lib::bytes::copy_of(segment)).expect("fixture path fits");
    }
    path
}
pub fn text(tree: &jig_view::Tree) -> Box<[u8]> {
    let mut bytes = 0_usize;
    for node in tree.nodes() {
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).expect("fixture text fits");
        }
    }
    let mut writer = Writer::new(bytes);
    for node in tree.nodes() {
        for run in &node.text {
            writer.put(&run.text).expect("text counted");
        }
    }
    writer.finish()
}
pub fn owner_part_text(words: &crate::words::Words, segments: &[&[u8]]) -> Box<[u8]> {
    let mut builder = jig_view::Builder::new(&tree_limits());
    crate::forge::owner_part(
        &mut builder,
        &OwnerPart { connector: FORGE, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path(segments)) },
        words,
    );
    text(&builder.finish())
}
pub fn part_tree(words: &crate::words::Words, part: Part, held: bool) -> jig_view::Tree {
    let limits = forge::Limits { parts: 1 };
    let mut side = forge::Side::new(&limits);
    if held {
        forge::step(
            &mut side,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            forge::Event::Show { part: part.clone(), generation: 1 },
            &mut Queue::with_capacity(1),
        );
    }
    let mut builder = jig_view::Builder::new(&tree_limits());
    crate::forge::part(&mut builder, &side, &part, 1, words);
    builder.finish()
}
pub fn part_text(words: &crate::words::Words, part: Part, held: bool) -> Box<[u8]> {
    text(&part_tree(words, part, held))
}
pub fn built(root_forge: u16) -> crate::View {
    let domain = domain_limits();
    let root = temper_client_domain::new(&domain, SEED, root_forge);
    let jig = jig_words();
    let forge = words();
    let envelope = crate::Limits::of(&domain, &jig, &forge).expect("composed envelope fits");
    assert!(envelope.tree.nodes >= 128);
    let limits = crate::Limits { tree: tree_limits() };
    let mut view = crate::View::new(&limits, &domain, jig, forge, FORGE);
    crate::build(&mut view, &root, &mut Queue::with_capacity(limits.tree.patches));
    view
}
