use super::fixtures;
use crate::render;
use skein_ui::{Role, State};

#[test]
fn each_adoption_form_state_prints_as_its_golden_file() {
    use temper_client_domain::forge;
    let words = fixtures::page_words();
    let empty = fixtures::adoption_form(false);
    let mut invalid = empty.clone();
    for field in [forge::Text::Owner, forge::Text::Name, forge::Text::Landing] {
        invalid.texts[field.index()].broken = Some(forge::Broken::Required);
    }
    let filled = fixtures::adoption_form(true);
    let mut pending = filled.clone();
    pending.pending = Some((skein_lib::Token::new(1), skein_lib::RequestProgress::Parked));
    let mut adopted = filled.clone();
    adopted.outcome = Some(forge::Answer::Adopted(fixtures::adopted(forge::Protection::Unknown)));
    let mut not_found = filled.clone();
    not_found.outcome = Some(forge::Answer::ForgeRefused(forge::Reason::NotFound));
    let mut core = filled.clone();
    core.outcome = Some(forge::Answer::CoreRefused(fixtures::role_refusal()));
    let mut out_of_date = filled;
    out_of_date.served = forge::Served::OutOfDate;
    for (name, form) in [
        ("adopt_empty", empty),
        ("adopt_invalid", invalid),
        ("adopt_pending", pending),
        ("adopt_adopted", adopted),
        ("adopt_refused_not_found", not_found),
        ("adopt_refused_core", core),
        ("adopt_out_of_date", out_of_date),
    ] {
        let tree = fixtures::adoption_page(&form, &words);
        let print = render::print(&tree, &fixtures::frame(), &fixtures::bounds());
        let golden = std::fs::read(fixtures::golden(name)).expect("adoption golden file");
        assert_eq!(print.text.as_ref(), golden, "page {name}");
    }
}

#[test]
fn each_fixture_page_prints_as_its_golden_file() {
    for (name, tree) in [
        ("inbox", fixtures::inbox_page()),
        ("task", fixtures::task_page()),
        ("project", fixtures::project_page()),
        ("deployment", fixtures::deployment_page()),
        ("signed_out", fixtures::signed_out_page()),
        ("every_role", fixtures::every_role()),
        ("control_text", fixtures::control_text()),
    ] {
        let print = render::print(&tree, &fixtures::frame(), &fixtures::bounds());
        let golden = std::fs::read(fixtures::golden(name)).expect("golden file");
        assert_eq!(String::from_utf8_lossy(&print.text), String::from_utf8_lossy(&golden), "page {name}");
    }
}
#[test]
fn numbers_go_to_enabled_bound_controls_in_preorder() {
    let tree = fixtures::every_role();
    let print = render::print(&tree, &fixtures::frame(), &fixtures::bounds());
    for (number, name) in [
        (1, b"Button".as_slice()),
        (2, b"Tab"),
        (3, b"Link"),
        (4, b"Field"),
        (5, b"Area"),
        (6, b"Choice"),
        (7, b"Check"),
        (8, b"Chosen"),
        (9, b"Checked"),
        (10, b"Nested"),
    ] {
        let node = tree.find(print.numbers.node(number).expect("printed number")).expect("visible node");
        let index = tree.nodes().iter().position(|candidate| candidate.id == node.id).expect("preorder node");
        assert_eq!(
            node.name.as_deref().unwrap_or_else(|| tree.nodes()[index + 1]
                .text
                .get(0)
                .expect("label run")
                .text
                .as_ref()),
            name,
            "number {number}"
        );
    }
    assert_eq!(print.numbers.count(), 10);
    assert_eq!(print.numbers.node(0), None);
    assert_eq!(print.numbers.node(11), None);
}
#[test]
fn a_disabled_or_unbound_control_prints_a_dash() {
    let tree = fixtures::every_role();
    let print = render::print(&tree, &fixtures::frame(), &fixtures::bounds());
    let text = String::from_utf8(print.text.into_vec()).expect("printable UTF-8");
    for name in ["All states", "Unbound"] {
        assert!(text.contains(&format!("[-] {name}")), "control {name}");
    }
    for node in tree.nodes() {
        if node.role == Role::Button
            && (node.states.as_slice().contains(&State::Disabled) || tree.bound(node.id).is_none())
        {
            assert!(
                (1..=print.numbers.count()).all(|number| print.numbers.node(number) != Some(node.id)),
                "node {:?}",
                node.id
            );
        }
    }
}
#[test]
fn the_first_line_is_the_title_and_the_link_state() {
    for (title, expected) in [(Some(b"Title".as_slice()), "== Title == Live"), (None, "== temper == Live")] {
        let print = render::print(&fixtures::header_page(title, false), &fixtures::frame(), &fixtures::bounds());
        let text = String::from_utf8(print.text.into_vec()).expect("printable UTF-8");
        assert_eq!(text.lines().next(), Some(expected), "title {title:?}");
        assert_eq!(text.matches("Live").count(), 1, "link appears only in header");
        assert_eq!(text.matches("Title").count(), usize::from(title.is_some()), "title appears only in header");
    }
}
#[test]
fn a_behind_frame_adds_its_sentence_to_the_first_line() {
    let print = render::print(&fixtures::header_page(Some(b"Title"), true), &fixtures::frame(), &fixtures::bounds());
    let text = String::from_utf8(print.text.into_vec()).expect("printable UTF-8");
    assert_eq!(text.lines().next(), Some("== Title == Live; Behind: waiting for current facts"));
    assert_eq!(text.matches("Behind: waiting for current facts").count(), 1);
    assert!(text.contains("* Signed in as Pat"), "identity is separate");
}
#[test]
fn the_largest_fixture_tree_prints_within_its_bounds() {
    let bounds = fixtures::bounds();
    for (name, tree) in [
        ("roles", fixtures::every_role()),
        ("task", fixtures::task_page()),
        ("project", fixtures::project_page()),
        ("deployment", fixtures::deployment_page()),
    ] {
        let print = render::print(&tree, &fixtures::frame(), &bounds);
        assert!(print.text.len() <= usize::try_from(bounds.bytes).expect("bytes fit"), "{name}: bytes");
        assert!(skein_lib::bytes::count(&print.text, b"\n", u32::MAX) <= bounds.lines, "{name}: lines");
        assert!(print.numbers.count() <= bounds.numbers, "{name}: numbers");
        assert!(render::worst_case(&bounds).is_some(), "{name}: memory bound");
    }
    let native = crate::limits::view(&crate::limits::native()).expect("native view bounds").0;
    assert!(render::bounds(&native.tree).is_some(), "native view has a print envelope: {:?}", native.tree);
}
