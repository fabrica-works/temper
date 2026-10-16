use super::fixtures;
use crate::forge::{
    bindings::{self, Bindings},
    page,
};
use skein_ui::{Act, Role, State};
use temper_client_domain::{Action, forge};
#[test]
fn the_page_names_every_field_a_person_acts_on() {
    let mut view = fixtures::view();
    let root = fixtures::adoption_root();
    let tree = fixtures::build(&mut view, &root);
    let rows: [(Role, &[u8]); 13] = [
        (Role::TextField, b"Forge number"),
        (Role::TextField, b"Owner"),
        (Role::TextField, b"Name"),
        (Role::ChoiceGroup, b"Role"),
        (Role::Choice, b"Owned: temper pushes, opens pull requests and lands"),
        (Role::Choice, b"Fork: temper contributes through a fork"),
        (Role::Choice, b"Context: temper only reads"),
        (Role::Check, b"Make it the project's home"),
        (Role::TextField, b"Branch prefix"),
        (Role::TextField, b"Landing branch"),
        (Role::Check, b"It runs CI"),
        (Role::TextField, b"Checks: gate numbers, separated by spaces or commas"),
        (Role::Button, b"Adopt"),
    ];
    for (role, name) in rows {
        assert!(fixtures::find(tree, role, name).is_some(), "node {role:?}: {name:?}");
    }
    assert!(has_text(tree, b"Adopt a repository into Project: Shop"), "heading uses the frame's name");
}
#[test]
fn the_navigation_slot_links_the_frames_project_or_holds_nothing() {
    let mut view = fixtures::view();
    let mut root = fixtures::root_in_project(Some(1));
    let tree = fixtures::build(&mut view, &root);
    let link = fixtures::find(tree, Role::Link, b"Adopt a repository").expect("the link for project 1");
    assert_eq!(
        crate::act(&mut view, link, Act::Press),
        Some(Action::Forge(forge::Act::Go { address: forge::Address::Adopt { project: 1 } }))
    );
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Jig(
                jig_domain_client::address::Address::Inbox { project: None },
            )),
        },
    );
    let tree = fixtures::build(&mut view, &root);
    assert_eq!(fixtures::find(tree, Role::Link, b"Adopt a repository"), None, "no project, no link");
    assert_eq!(crate::act(&mut view, link, Act::Press), None, "old link is gone");
}
#[test]
fn a_check_takes_a_toggle_and_refuses_a_press() {
    let mut view = fixtures::view();
    fixtures::build(&mut view, &fixtures::adoption_root());
    for (name, expected) in
        [(b"Make it the project's home".as_slice(), forge::Act::Home), (b"It runs CI".as_slice(), forge::Act::Ci)]
    {
        let node = fixtures::find(view.tree(), Role::Check, name).expect("check");
        assert_eq!(crate::act(&mut view, node, Act::Press), None, "press {name:?}");
        assert_eq!(crate::act(&mut view, node, Act::Toggle), Some(Action::Forge(expected)), "toggle {name:?}");
    }
}
fn form() -> forge::Form {
    let mut form = forge::form(1);
    form.confirmed = true;
    form.texts[forge::Text::Owner.index()].text = Box::from(*b"acme");
    form.texts[forge::Text::Name.index()].text = Box::from(*b"shop");
    form
}
fn tree(form: &forge::Form) -> skein_ui::Tree {
    let words = fixtures::words();
    let jig = fixtures::jig_words();
    page_tree(form, &words, &jig)
}
fn page_tree(form: &forge::Form, words: &crate::words::Words, jig: &jig_view::words::Words) -> skein_ui::Tree {
    let mut limits = fixtures::tree_limits();
    limits.text_per_node = 2048;
    limits.text_bytes = 262_144;
    let mut builder = skein_ui::Builder::new(&limits);
    let mut bindings = Bindings::new(limits.bindings, 8, 1);
    page::page(&mut builder, form, b"Shop", words, jig, &mut bindings);
    builder.finish()
}
fn adopted() -> forge::Answer {
    forge::Answer::Adopted(forge::Adopted {
        forge: 1,
        repository: 7,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        role: forge::Role::Owned,
        home: true,
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: forge::Protection::Unknown,
    })
}
#[test]
fn each_outcome_reads_as_the_designs_sentence() {
    let words = fixtures::words();
    let jig = fixtures::jig_words();
    let rows: [(forge::Answer, &[u8], bool); 10] = [
        (adopted(), b"Adopted acme/shop on forge 1 as owned.It is the project's home.temper may: read; push; open pull requests; land; review; set statuses; comment; create issues; create and delete branches.Branch protection could not be read.See the project's resources", false),
        (forge::Answer::ForgeRefused(forge::Reason::NoSuchForge), b"No forge 1 is configured.", true),
        (forge::Answer::ForgeRefused(forge::Reason::NotFound), b"No repository acme/shop on forge 1 that temper's forge user can see.", true),
        (forge::Answer::ForgeRefused(forge::Reason::NoAccess), b"temper's forge user may not read acme/shop on forge 1.", true),
        (forge::Answer::ForgeRefused(forge::Reason::Refused), b"The forge connector refused it: another project writes acme/shop, the project has a home already, or branches under temper/ are not this deployment's.", true),
        (forge::Answer::ForgeRefused(forge::Reason::InFlight), b"acme/shop is being adopted already; try again when that ends.", true),
        (forge::Answer::ForgeRefused(forge::Reason::TooLarge), b"A field is past the forge's bounds, or the forge's repositories are full.", true),
        (forge::Answer::ForgeRefused(forge::Reason::Unreachable), b"Forge 1 did not answer; nothing was adopted. Try again.", true),
        (forge::Answer::ForgeRefused(forge::Reason::InvalidAnswer), b"Forge 1 answered what temper did not expect; nothing was adopted.", true),
        (forge::Answer::Unknown, b"It is not known whether acme/shop was adopted: see the project's resources.", false),
    ];
    for (answer, said, alert) in rows {
        let mut form = form();
        form.outcome = Some(answer.clone());
        let rendered = page_tree(&form, &words, &jig);
        let text = fixtures::text(&rendered);
        assert_eq!(&text[b"Adopt a repository into Shop".len()..], said, "outcome {answer:?}");
        assert_eq!(has_role(&rendered, Role::Alert), alert, "refusal alert {answer:?}");
    }
}
#[test]
fn each_kind_of_jig_refusal_uses_jigs_words() {
    let refusal = jig_domain_client::contract::Refusal::Role { held: None };
    for answer in [
        forge::Answer::CoreRefused(refusal.clone()),
        forge::Answer::EntranceRefused(
            jig_domain_client::contract::EntranceRefusal::new(refusal.clone()).expect("entrance refusal"),
        ),
    ] {
        let mut form = form();
        form.outcome = Some(answer.clone());
        let rendered = tree(&form);
        assert_eq!(
            &fixtures::text(&rendered)[b"Adopt a repository into Shop".len()..],
            b"The held role does not permit this actionNo held role was supplied",
            "outcome {answer:?}"
        );
        assert!(has_role(&rendered, Role::Alert), "alert {answer:?}");
    }
}
#[test]
fn typed_fields_reject_invalid_numbers_and_duplicate_checks() {
    for (text, expected) in [
        (b"1".as_slice(), Some(1)),
        (b"65535", Some(65535)),
        (b"0", None),
        (b"65536", None),
        (b"-1", None),
        (b"", None),
    ] {
        assert_eq!(bindings::forge_number(text), expected, "forge {text:?}");
    }
    let rows: [(&[u8], Option<&[u32]>); 7] = [
        (b"", Some(&[])),
        (b"1, 2 3", Some(&[1, 2, 3])),
        (b"1, 1", None),
        (b"0", None),
        (b"x", None),
        (b"4294967296", None),
        (b"1 2 3 4", None),
    ];
    for (text, expected) in rows {
        assert_eq!(bindings::checks(text, 3).as_deref(), expected, "checks {text:?}");
    }
}
#[test]
fn a_field_input_is_typed_and_older_binding_tokens_decode_nothing() {
    let mut bindings = Bindings::new(1, 2, 1);
    let field = bindings.insert(bindings::Meaning::Field(forge::Text::Checks));
    assert_eq!(
        bindings::decode(&bindings, field.token, Act::Input { value: Box::from(*b"1,2") }),
        Some(Action::Forge(forge::Act::Write {
            field: forge::Text::Checks,
            text: Box::from(*b"1,2"),
            typed: forge::Typed::Checks(Some(Box::new([1, 2])))
        }))
    );
    bindings.begin();
    let _new = bindings.insert(bindings::Meaning::Adopt);
    assert_eq!(bindings::decode(&bindings, field.token, Act::Press), None);
}
#[test]
fn adopt_is_disabled_until_a_party_is_confirmed() {
    let mut view = fixtures::view();
    let mut root = fixtures::adoption_root();
    for (party, disabled) in [(None, true), (Some(73), false), (None, true)] {
        fixtures::root_event(&mut root, temper_client_domain::Event::Forge(forge::Event::Confirmed { party }));
        let rendered = fixtures::build(&mut view, &root);
        let node = fixtures::find(rendered, Role::Button, b"Adopt").expect("Adopt button");
        for button in rendered.nodes() {
            if button.id == node {
                assert_eq!(button.states.as_slice().contains(&State::Disabled), disabled, "party {party:?}");
            }
        }
    }
}
#[test]
fn pending_or_out_of_date_disables_adopt() {
    for (pending, served, said) in [
        (
            Some((skein_lib::Token::new(1), skein_lib::RequestProgress::Parked)),
            forge::Served::Yes,
            b"Being decided: waiting for the engine connection or your sign-in".as_slice(),
        ),
        (
            None,
            forge::Served::OutOfDate,
            b"This temper is out of date for adopting repositories: run the temper the engine runs.".as_slice(),
        ),
    ] {
        let mut form = form();
        form.pending = pending;
        form.served = served;
        let rendered = tree(&form);
        let button = node_of_role(&rendered, Role::Button);
        assert!(button.states.as_slice().contains(&State::Disabled), "{served:?} {pending:?}");
        assert!(fixtures::find(&rendered, Role::Status, said).is_some(), "status {served:?} {pending:?}");
    }
}
#[test]
fn the_outcomes_link_opens_the_projects_resources() {
    let words = fixtures::words();
    let jig = fixtures::jig_words();
    let mut builder =
        skein_ui::Builder::new(&crate::Limits::of(&fixtures::domain_limits(), &jig, &words).expect("limits").tree);
    let mut bindings = Bindings::new(32, 8, 1);
    let mut form = form();
    form.outcome = Some(adopted());
    page::page(&mut builder, &form, b"Shop", &words, &jig, &mut bindings);
    let tree = builder.finish();
    let link = node_of_role(&tree, Role::Link);
    assert_eq!(
        bindings::decode(&bindings, link.link.expect("link bound").token, Act::Press),
        Some(Action::Jig(jig_domain_client::action::Act::Go {
            address: jig_domain_client::address::Address::Project {
                project: 1,
                section: jig_domain_client::address::ProjectSection::Resources
            }
        }))
    );
}
fn has_text(tree: &skein_ui::Tree, expected: &[u8]) -> bool {
    for text in fixtures::text(tree).windows(expected.len()) {
        if text == expected {
            return true;
        }
    }
    false
}
fn has_role(tree: &skein_ui::Tree, role: Role) -> bool {
    for node in tree.nodes() {
        if node.role == role {
            return true;
        }
    }
    false
}
fn node_of_role(tree: &skein_ui::Tree, role: Role) -> &skein_ui::Node {
    for node in tree.nodes() {
        if node.role == role {
            return node;
        }
    }
    panic!("missing {role:?}")
}
#[test]
fn the_form_fields_carry_the_domains_values_and_write_counts() {
    let rendered = tree(&form());
    let rows: [(&[u8], &[u8]); 6] = [
        (b"Forge number", b"1"),
        (b"Owner", b"acme"),
        (b"Name", b"shop"),
        (b"Branch prefix", b"temper/"),
        (b"Landing branch", b""),
        (b"Checks: gate numbers, separated by spaces or commas", b""),
    ];
    for (name, expected) in rows {
        let id = fixtures::find(&rendered, Role::TextField, name).expect("field");
        for node in rendered.nodes() {
            if node.id == id && node.name.as_deref() == Some(name) {
                let value = node.value.as_ref().expect("field value");
                assert_eq!((&*value.text, value.written), (expected, 0), "field {name:?}");
            }
        }
    }
}
#[test]
fn each_broken_rule_is_an_invalid_field_with_its_status() {
    let words = fixtures::words();
    let jig = fixtures::jig_words();
    let rows: [(forge::Broken, &[u8]); 6] = [
        (forge::Broken::Required, b"Required."),
        (forge::Broken::ForgeNumber, b"A forge number is from 1 to 65535."),
        (forge::Broken::Checks, b"Checks are numbers from 1, each once."),
        (forge::Broken::ContextHome, b"A context repository cannot be the project's home."),
        (forge::Broken::NoCheck, b"Without CI, name at least one check."),
        (forge::Broken::Long { bound: 256 }, b"At most 256 bytes."),
    ];
    for (broken, said) in rows {
        let mut form = form();
        form.texts[forge::Text::Owner.index()].broken = Some(broken);
        let rendered = page_tree(&form, &words, &jig);
        assert!(fixtures::find(&rendered, Role::Status, said).is_some(), "status {broken:?}");
        for node in rendered.nodes() {
            if node.role == Role::TextField && node.name.as_deref() == Some(b"Owner") {
                assert!(node.states.as_slice().contains(&State::Invalid), "invalid {broken:?}");
            }
        }
    }
}
#[test]
fn all_roles_take_a_press_or_a_toggle() {
    let mut bindings = Bindings::new(3, 8, 1);
    for role in [forge::Role::Owned, forge::Role::Fork, forge::Role::Context] {
        let binding = bindings.insert(bindings::Meaning::Role(role));
        for (name, act) in [("press", Act::Press), ("toggle", Act::Toggle)] {
            assert_eq!(
                bindings::decode(&bindings, binding.token, act),
                Some(Action::Forge(forge::Act::Choose { role })),
                "{role:?} {name}"
            );
        }
    }
}
#[test]
fn full_bound_repository_names_fit_the_outcome_phrase() {
    let mut form = form();
    let forge::Answer::Adopted(mut adopted) = adopted() else { panic!("adopted fixture") };
    adopted.owner = Box::new([b'o'; 256]);
    adopted.name = Box::new([b'n'; 256]);
    form.outcome = Some(forge::Answer::Adopted(adopted));
    let rendered = tree(&form);
    assert!(has_text(&rendered, b" on forge 1 as owned."));
}
