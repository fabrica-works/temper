use skein_ui::Role;
use temper_client_domain::{self as d, Address, Place};
use temper_client_world::{
    app::Root,
    fixtures,
    front::{Joint, account_acts as accounts},
};
fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(
        &mut view,
        root.person(),
        fixtures::NOW,
        &mut skein_lib::Queue::with_capacity(bounds.tree.patches),
    );
    view
}
fn text(root: &Root) -> String {
    let view = view(root);
    let text: Vec<u8> =
        view.tree().nodes().iter().flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    String::from_utf8(text).expect("view text")
}
fn go(front: &mut Joint, root: &mut Root, address: Address) {
    accounts::event(front, root, d::Event::Went { place: Place::Address(address) }.into());
}
fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], act: skein_ui::Act) {
    let mut view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("flow control")
        .id;
    let event = temper_client_view::event(&mut view, id, act).expect("flow binding");
    accounts::event(front, root, event.into());
}
#[test]
fn the_banner_key_way_opens_details_and_ready_removes_the_banner() {
    let (mut front, mut root) = accounts::empty_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::Chats);
    assert!(text(&root).contains("Agents need an LLM account to run."));
    press(&mut front, &mut root, Role::Link, b"Use an API key", skein_ui::Act::Press);
    assert_eq!(
        root.person().pages().address(),
        &Address::ConnectModel { way: Some(d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::Key }) }
    );
    assert!(text(&root).contains("Details"));
    press(
        &mut front,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        skein_ui::Act::Input { value: b"test-private-key".as_slice().into() },
    );
    press(&mut front, &mut root, Role::Button, b"Add key", skein_ui::Act::Press);
    accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    assert!(text(&root).contains("Added your Scripted API key"));
    assert!(!format!("{:?}", root.person().pages()).contains("test-private-key"));
    press(&mut front, &mut root, Role::Link, b"Start a chat", skein_ui::Act::Press);
    assert!(!text(&root).contains("Agents need an LLM account to run."));
    accounts::assert_no_carried(&front);
}
#[test]
fn declaring_a_program_login_shows_its_command_on_ready() {
    let (mut front, mut root) = accounts::program_fixture(fixtures::SEED);
    go(&mut front, &mut root, Address::ConnectModel { way: None });
    press(&mut front, &mut root, Role::Link, b"Use your Scripted subscription", skein_ui::Act::Press);
    press(
        &mut front,
        &mut root,
        Role::TextField,
        b"Program name",
        skein_ui::Act::Input { value: b"claude-alice".as_slice().into() },
    );
    press(&mut front, &mut root, Role::Button, b"Declare", skein_ui::Act::Press);
    assert!(text(&root).contains("Declared your Scripted subscription · claude-alice"));
    assert!(text(&root).contains("temper program-login claude-alice"));
    assert!(text(&root).contains("Claude Code asks you to sign in. temper never reads its login."));
}
#[test]
fn a_creation_key_is_absent_from_the_small_store() {
    use jig_client_world::harness::Until;
    use skein_fake_person::{Find, Step};
    let mut world = fixtures::model_world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, Address::ConnectModel { way: None });
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"Use an API key"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Use an API key") },
            Step::See { find: Find::exactly_named(Role::Button, b"Add key"), within: fixtures::WITHIN },
            Step::Type {
                find: Find::exactly_named(Role::TextField, b"Key (pasted once)"),
                words: b"test-private-key".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Add key") },
            Step::See { find: Find::text(b"Added your Scripted API key"), within: fixtures::WITHIN },
        ],
    );
    let mut kept = false;
    for _ in 0..4096 {
        world.run(Until::Steps(2));
        if world.front().engine.protocol.accounts.iter().any(|request| {
            matches!(request, jig_domain_accounts::Request::Signing(jig_domain_accounts::Signing::Keep { .. }))
        }) {
            accounts::kept(world.front());
            kept = true;
            break;
        }
    }
    assert!(kept, "the fake credential store received the carried key");
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(world.kept(person).is_empty(), "a creation key has no small-store record");
}
