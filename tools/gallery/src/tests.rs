use super::*;
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_scratch::Scratch;

#[test]
fn names_are_the_promised_ones() {
    let samples = samples::samples();
    let mut names = Vec::new();
    for sample in &samples {
        assert!(!names.contains(&sample.name), "duplicate {}", sample.name);
        names.push(sample.name);
    }
    let promised: Vec<&str> = include_str!("../names.txt").lines().collect();
    assert_eq!(names, promised);
}

#[test]
fn every_link_has_an_address() {
    for sample in samples::samples() {
        let mut view = view(&sample);
        let mut links = Vec::new();
        for node in view.tree().nodes() {
            if node.role == Role::Link {
                assert!(node.link.is_some() || node.external.is_some(), "{}: link {:?}", sample.name, node.name);
                links.push(node.id);
            }
        }
        for node in links {
            let address = temper_client_view::href(&mut view, node).expect("link has an href");
            assert!(!address.is_empty(), "{}: empty href for {node:?}", sample.name);
        }
    }
}

// Expected words are the shipped view's; available counts are the worlds'
// live/closed windows, combined in the named Your chats list.
const SOLO_LIVE: u32 = 2;
const SOLO_CLOSED: u32 = 1;
const REFERENCE_LIVE: u32 = 1;
const REFERENCE_CLOSED: u32 = 2;
const SOLO_PLAN_LIVE: u32 = 3;
struct Holds {
    name: &'static str,
    heading: &'static str,
    parts: &'static [Part],
}
enum Part {
    Named { role: Role, name: &'static str },
    Absent { role: Role, name: &'static str },
    Value { role: Role, name: &'static str, value: &'static str },
    Chats { live: u32, closed: u32 },
    Plan { live: u32, title: &'static str },
}
const HOLDS: [Holds; samples::N] = [
    Holds {
        name: "chats-solo",
        heading: "Chats",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Why is tally sum slow on big files?" },
            Part::Chats { live: SOLO_LIVE, closed: SOLO_CLOSED },
        ],
    },
    Holds {
        name: "chat-solo-T10",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Paragraph, name: "JSON output for tally report" },
        ],
    },
    Holds {
        name: "inbox-solo",
        heading: "Inbox",
        parts: &[
            Part::Named { role: Role::Button, name: "Answer" },
            Part::Named { role: Role::Paragraph, name: "JSON output for tally report" },
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Paragraph, name: "The old parser accepts" },
        ],
    },
    Holds {
        name: "task-solo-T3",
        heading: "CSV import",
        parts: &[
            Part::Named { role: Role::Status, name: "waiting on an answer from you" },
            Part::Plan { live: SOLO_PLAN_LIVE, title: "CSV import: the quoted-field parser" },
            Part::Named { role: Role::Region, name: "Budget and authority" },
            Part::Named { role: Role::Paragraph, name: "5 spent · 15 left of 20" },
            Part::Named { role: Role::Term, name: "Funded by" },
            Part::Named { role: Role::Description, name: "Your pool" },
        ],
    },
    Holds {
        name: "accounts-solo",
        heading: "Accounts",
        parts: &[
            Part::Named { role: Role::Heading { level: 3 }, name: "Claude" },
            Part::Named { role: Role::Item, name: "Source 1 · Order 1 · Program login claude-robin · Ready" },
            Part::Named { role: Role::Item, name: "Source 2 · Order 2 · API key robin-key · Ready" },
            Part::Named { role: Role::Paragraph, name: "laptop" },
        ],
    },
    Holds {
        name: "chats-first-run-1",
        heading: "What shall we change?",
        parts: &[
            Part::Named { role: Role::List, name: "Suggested requests" },
            Part::Named { role: Role::Button, name: "Add CSV export to widget" },
            Part::Named { role: Role::Button, name: "Add a dark theme to widget" },
            Part::Named { role: Role::Button, name: "Fix widget’s empty search state" },
        ],
    },
    Holds {
        name: "adoption",
        heading: "Adopt a repository into factory",
        parts: &[Part::Value { role: Role::TextField, name: "Owner", value: "" }],
    },
    Holds {
        name: "adoption-filled",
        heading: "Adopt a repository into factory",
        parts: &[
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Value { role: Role::TextField, name: "Owner", value: "acme" },
        ],
    },
    Holds {
        name: "adoption-refused",
        heading: "Adopt a repository into factory",
        parts: &[Part::Named { role: Role::Alert, name: "" }],
    },
    Holds {
        name: "chats-reference",
        heading: "Chats",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Director" },
            Part::Chats { live: REFERENCE_LIVE, closed: REFERENCE_CLOSED },
        ],
    },
    Holds {
        name: "chat-reference-T1",
        heading: "T1 · Director",
        parts: &[Part::Named { role: Role::Paragraph, name: "Check · two-hour timer" }],
    },
    Holds {
        name: "chat-solo-T6",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[Part::Named { role: Role::Paragraph, name: "T7 found it" }],
    },
    Holds {
        name: "chat-first-run-2-T1",
        heading: "T1 · Add CSV export to widget",
        parts: &[Part::Named { role: Role::Link, name: "Task 2" }],
    },
    Holds {
        name: "chat-first-run-3-T1",
        heading: "T1 · Add CSV export to widget",
        parts: &[Part::Named { role: Role::Paragraph, name: "CSV export is available" }],
    },
    Holds {
        name: "chat-first-run-4-T4",
        heading: "T4 · Add a dark theme to widget",
        parts: &[Part::Named { role: Role::Paragraph, name: "I’d give widget" }],
    },
    Holds { name: "inbox-reference", heading: "Inbox", parts: &[Part::Named { role: Role::Link, name: "Task 1" }] },
    Holds {
        name: "accounts-reference",
        heading: "Accounts",
        parts: &[Part::Named { role: Role::Heading { level: 3 }, name: "Claude" }],
    },
    Holds {
        name: "accounts-first-run-1",
        heading: "Accounts",
        parts: &[Part::Named { role: Role::Heading { level: 3 }, name: "Demo model" }],
    },
    Holds {
        name: "signed-out",
        heading: "",
        parts: &[Part::Named {
            role: Role::Paragraph,
            name: "Signed out. To sign in, open a sign-in link temper serve printed.",
        }],
    },
    Holds {
        name: "out-of-date",
        heading: "",
        parts: &[Part::Named {
            role: Role::Paragraph,
            name: "This temper is out of date: run the temper the engine runs.",
        }],
    },
    Holds {
        name: "chats-empty",
        heading: "Chats",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your chats" },
            Part::Chats { live: 0, closed: 0 },
        ],
    },
    Holds {
        name: "inbox-empty",
        heading: "Inbox",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you" },
            Part::Absent { role: Role::List, name: "" },
        ],
    },
    Holds {
        name: "chat-closing",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[Part::Named { role: Role::Dialog, name: "Close" }],
    },
    Holds {
        name: "chat-rejecting",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Paragraph, name: "JSON output for tally report" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds { name: "task-held", heading: "CSV import", parts: &[Part::Named { role: Role::Button, name: "Release" }] },
    Holds {
        name: "task-releasing",
        heading: "CSV import",
        parts: &[Part::Named { role: Role::Dialog, name: "Release" }],
    },
    Holds {
        name: "accounts-signin",
        heading: "Accounts",
        parts: &[Part::Named { role: Role::Link, name: "Open the sign-in page" }],
    },
    Holds { name: "accounts-refused", heading: "Accounts", parts: &[Part::Named { role: Role::Alert, name: "" }] },
];
fn in_main(role: Role, name: &str) -> Find {
    Find::named(role, name.as_bytes())
        .within(Box::new([skein_fake_person::Target::Role { role: Role::Main, name: Box::new([]) }]))
}
fn find(tree: &skein_ui::Tree, role: Role, name: &str, sample: &str) -> skein_ui::NodeId {
    let mut face = TreeFace::new(tree);
    match face.find(&Find::named(role, name.as_bytes())) {
        Found::Node(node) => node,
        Found::Absent | Found::Pending => panic!("{sample}: missing {role:?} {name:?}"),
    }
}
#[test]
fn each_state_holds_what_its_name_says() {
    for sample in samples::samples() {
        let mut row = None;
        for holds in &HOLDS {
            if holds.name == sample.name {
                assert!(row.is_none(), "duplicate holds row");
                row = Some(holds);
            }
        }
        let row = row.expect("every sample has a HOLDS row");
        let view = view(&sample);
        let tree = view.tree();
        let mut main = TreeFace::new(tree);
        let h1 = in_main(Role::Heading { level: 1 }, row.heading);
        if row.heading.is_empty() {
            assert_eq!(main.find(&h1), Found::Absent, "{} has no main heading", sample.name);
        } else {
            assert!(matches!(main.find(&h1), Found::Node(_)), "{}: heading {}", sample.name, row.heading);
        }
        for part in row.parts {
            match part {
                Part::Named { role, name } => {
                    find(tree, *role, name, sample.name);
                }
                Part::Absent { role, name } => {
                    assert_eq!(
                        main.find(&in_main(*role, name)),
                        Found::Absent,
                        "{}: absent {role:?} {name:?}",
                        sample.name
                    );
                }
                Part::Value { role, name, value } => {
                    let node = find(tree, *role, name, sample.name);
                    let value = value.as_bytes();
                    assert_eq!(
                        tree.find(node).expect("field").value.as_ref().expect("field value").text.as_ref(),
                        value,
                        "{}",
                        sample.name
                    );
                }
                Part::Chats { live, closed } => {
                    let available = (*live).min(driver::limits().client.domain.rows)
                        + (*closed).min(driver::limits().client.domain.rows);
                    if available == 0 {
                        assert_eq!(main.find(&in_main(Role::List, "Your chats")), Found::Absent, "{}", sample.name);
                        continue;
                    }
                    let list = find(tree, Role::List, "Your chats", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == list).expect("chats list");
                    let end = start + usize::try_from(nodes[start].size).expect("chats subtree");
                    let mut count = 0;
                    for node in &nodes[start..end] {
                        if node.role == Role::Link
                            && let Some(name) = &node.name
                            && name
                                .strip_prefix(b"T")
                                .is_some_and(|number| !number.is_empty() && number.iter().all(u8::is_ascii_digit))
                        {
                            count += 1;
                        }
                    }
                    assert_eq!(count, available, "{}: windowed chat rows", sample.name);
                }
                Part::Plan { live, title } => {
                    let plan = find(tree, Role::Tree, "Plan", sample.name);
                    let row = Find::named(Role::Paragraph, title.as_bytes()).within(Box::new([
                        skein_fake_person::Target::Role { role: Role::Tree, name: Box::from(b"Plan".as_slice()) },
                    ]));
                    assert!(matches!(main.find(&row), Found::Node(_)), "{}: delegate title", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == plan).expect("plan node");
                    let end = start + usize::try_from(nodes[start].size).expect("plan subtree");
                    let count = nodes[start..end].iter().filter(|node| node.role == Role::TreeItem).count();
                    let window = driver::limits().client.domain.rows;
                    assert_eq!(
                        count,
                        usize::try_from((*live).min(window)).expect("row count"),
                        "{}: plan window",
                        sample.name
                    );
                }
            }
        }
    }
}
#[test]
fn print_writes_every_page_and_the_bundle() {
    let scratch = Scratch::new("temper-client-gallery-print");
    let out = scratch.path().join("gallery");
    print(&out).expect("print gallery");
    let index = fs::read_to_string(out.join("index.html")).expect("index");
    let bundle = out.join("bundle");
    let mut digest = None;
    for entry in fs::read_dir(&bundle).expect("bundle") {
        let entry = entry.expect("bundle entry");
        if entry.file_type().expect("file type").is_dir() {
            assert!(digest.is_none(), "one digest");
            digest = Some(entry.file_name().into_string().expect("digest ASCII"));
        }
    }
    let digest = digest.expect("digest directory");
    assert!(bundle.join(&digest).join("files/menu.svg").is_file(), "bundled menu mask");
    for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
        assert!(bundle.join(&digest).join(file).is_file(), "bundled {file}");
    }
    let mut position = 0;
    for sample in samples::samples() {
        let link = format!("href=\"{}.html\"", sample.name);
        assert_eq!(index.matches(&link).count(), 1, "one link for {}", sample.name);
        let at = index.find(&link).expect("index link");
        assert!(at >= position, "table order");
        position = at + link.len();
        let page = fs::read_to_string(out.join(format!("{}.html", sample.name))).expect("sample page");
        assert!(page.contains("class=\"temper-menu\""), "{}: menu disclosure", sample.name);
        assert!(page.contains("<summary>Menu</summary>"), "{}: menu name", sample.name);
        let mut position = 0;
        for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
            let link = format!("href=\"bundle/{digest}/{file}\"");
            let at = page.find(&link).expect("stylesheet linked");
            assert!(at >= position, "stylesheet order");
            position = at + link.len();
        }
    }
    let signin = fs::read_to_string(out.join("accounts-signin.html")).expect("sign-in page");
    assert!(signin.contains("href=\"https://issuer.example/visit/gallery-sign-in\""), "external sign-in href");
    let closing = fs::read_to_string(out.join("chat-closing.html")).expect("confirmation page");
    assert!(closing.contains("<dialog open "), "static confirmation opens");
    let chats = fs::read_to_string(out.join("chats-solo.html")).expect("chats page");
    assert!(chats.contains("<title>Chats · temper</title>"), "shell heading title");
    let signed_out = fs::read_to_string(out.join("signed-out.html")).expect("signed-out page");
    assert!(signed_out.contains("<title>temper</title>"), "shell title without heading");
    // Reprinting replaces the whole owned directory, including stale files.
    let stale = out.join("stale.html");
    fs::write(&stale, b"obsolete").expect("stale page");
    print(&out).expect("replace gallery");
    assert!(!stale.exists(), "old output removed");
    let mut expected = Vec::new();
    for path in STYLESHEETS {
        expected.push(Path::new(path).file_name().expect("stylesheet basename").to_str().expect("ASCII"));
    }
    let mut actual = Vec::new();
    for line in include_str!("../../../scripts/bundle.sh").lines() {
        let line = line.trim();
        if line.starts_with("--stylesheet ") {
            let (_, path) = line.split_once("$shell/").expect("shell stylesheet");
            actual.push(path.split('"').next().expect("basename"));
        }
    }
    assert_eq!(expected, actual);
}
#[test]
fn print_refuses_a_directory_not_its_own() {
    let scratch = Scratch::new("temper-client-gallery-refusal");
    let file = scratch.path().join("precious");
    fs::write(&file, b"keep me").expect("foreign file");
    match print(scratch.path()) {
        Err(Refusal::NotGallery { out }) => assert_eq!(out, scratch.path()),
        other => panic!("expected NotGallery, got {other:?}"),
    }
    assert_eq!(fs::read(&file).expect("file untouched"), b"keep me");
    assert_eq!(fs::read_dir(scratch.path()).expect("directory untouched").count(), 1);
}

#[test]
fn times_read_against_each_mockup_worlds_fixed_now() {
    for (name, expected) in [
        ("chats-solo", "1 h 28 min ago"),
        ("chats-reference", "50 min ago"),
        ("inbox-solo", "Waiting 12 min"),
        ("inbox-solo", "Waiting 18 min"),
        ("chat-solo-T10", "waiting 12 min"),
        ("chat-reference-T1", "Today · Thursday 22 October"),
        ("chat-solo-T10", "Today · Tuesday 27 October"),
        ("chat-first-run-4-T4", "Ended 21:05"),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("named sample");
        let view = view(&sample);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .any(|node| node.text.iter().any(|run| String::from_utf8_lossy(&run.text).contains(expected))),
            "{name}: {expected}"
        );
    }
}

#[test]
fn gallery_chat_clocks_are_separate_leaves_under_the_speakers() {
    for (name, clock) in [("chat-reference-T1", "06:56"), ("chat-solo-T10", "09:16")] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sample");
        let view = view(&sample);
        let tree = view.tree();
        let who = tree
            .nodes()
            .iter()
            .enumerate()
            .find(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::WHO))
            .expect("speaker")
            .0;
        assert_eq!(tree.nodes()[who + 1].name.as_deref(), Some(b"You".as_slice()));
        let clock_at = who + 1 + usize::try_from(tree.nodes()[who + 1].size).expect("speaker size");
        assert_eq!(tree.nodes()[clock_at].role, Role::Time);
        assert_eq!(tree.nodes()[clock_at].name.as_deref(), Some(clock.as_bytes()));
    }
}
