//! The first-run choice through the client's scripted demo peer.
use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Place, Step, pages::Act};
use temper_client_protocol_wire::v1 as w;
use temper_client_world::{
    app::{Root, Temper},
    fixtures as f,
    front::{Joint, account_acts as accounts},
};

fn pump(front: &mut Joint, root: &mut Root, event: d::Event) -> usize {
    let limits = f::domain_limits();
    let mut now = Time::ZERO.saturating_add(Duration::from_secs(1));
    let mut out = Queue::with_capacity(Temper::max_out(&limits));
    Temper::step(root, &Env { now, wall: Wall::EPOCH, limits }, event.into(), &mut out);
    let mut replaced = 0;
    for _ in 0..64 {
        Temper::take(root, &Env { now, wall: Wall::EPOCH, limits }, &mut out);
        while let Some(request) = out.pop() {
            if let d::Request::Show { address, replace } = &request {
                replaced += usize::from(*replace);
                Temper::step(
                    root,
                    &Env { now, wall: Wall::EPOCH, limits },
                    d::Event::Went { place: Place::Address(address.clone()) }.into(),
                    &mut out,
                );
            }
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(ClientId(0), now, Wall::EPOCH, request);
            }
        }
        now = now.saturating_add(Duration::from_millis(3));
        let callbacks = front.poll(now, Wall::EPOCH);
        if callbacks.is_empty() {
            return replaced;
        }
        for (_, event) in callbacks {
            front.received(ClientId(0), &event);
            Temper::step(root, &Env { now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    panic!("demo story did not settle");
}
fn go(front: &mut Joint, root: &mut Root, address: Address) -> usize {
    pump(front, root, d::Event::Went { place: Place::Address(address) })
}
fn view(root: &Root) -> temper_client_view::View {
    let limits = f::domain_limits();
    let words = f::phrasebooks(&limits);
    let bounds = f::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, f::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    view(root).tree().nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    let mut view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
        .expect("button")
        .id;
    let action = temper_client_view::act(&mut view, id, skein_ui::Act::Press).expect("bound start");
    pump(front, root, d::Event::Act { action });
}
#[test]
fn not_started_welcome_and_strip_start_the_playground_in_its_project() {
    for strip in [false, true] {
        let (mut front, mut root) = accounts::demo_fixture(f::SEED, &w::DemoState::NotStarted);
        assert!(has(&root, Role::Heading { level: 1 }, b"Welcome to temper"));
        assert!(has(&root, Role::Button, b"Try the playground"));
        assert!(!has(&root, Role::Region, b"Playground"));
        assert!(front.demo_starts.is_empty());
        if strip {
            go(&mut front, &mut root, Address::Chats);
            assert!(has(&root, Role::Region, b"Setup"));
        }
        press(&mut front, &mut root, b"Try the playground");
        assert_eq!(front.demo_starts.len(), 1);
        assert_eq!(root.person().pages().address(), &Address::Chats);
        assert_eq!(root.person().pages().project(), Some(1));
        assert!(has(&root, Role::Region, b"Playground"));
        assert!(!has(&root, Role::Region, b"Setup"));
        assert!(has(&root, Role::Status, b"Ready"));
        pump(&mut front, &mut root, d::Event::Act { action: d::Action::Pages(Act::Playground) });
        assert_eq!(front.demo_starts.len(), 1);
    }
}
#[test]
fn failed_start_survives_navigation_and_try_again_uses_a_new_key() {
    let (mut front, mut root) = accounts::demo_fixture(f::SEED, &w::DemoState::NotStarted);
    front.demo_outcomes.push_back(w::Outcome::StartFailed(
        w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account })
            .expect("account failure"),
    ));
    press(&mut front, &mut root, b"Try the playground");
    assert!(has(&root, Role::Status, b"Not started"));
    assert!(has(&root, Role::Alert, b"The playground did not start: the scripted model's account was refused."));
    go(&mut front, &mut root, Address::Accounts);
    go(&mut front, &mut root, Address::Chats);
    assert!(has(&root, Role::Button, b"Try again"));
    press(&mut front, &mut root, b"Try again");
    assert_eq!(front.demo_starts.len(), 2);
    assert_ne!(front.demo_starts[0], front.demo_starts[1]);
    assert!(has(&root, Role::Status, b"Ready"));
    assert!(!has(&root, Role::Button, b"Try again"));
}
#[test]
fn running_read_starts_once_silently_and_welcome_moves_once() {
    let (mut front, mut root) = accounts::demo_fixture(f::SEED, &w::DemoState::Running);
    assert_eq!(front.demo_starts.len(), 1);
    assert_eq!(root.person().pages().address(), &Address::Chats);
    assert!(!has(&root, Role::Status, b"Starting"));
    let replaces = go(&mut front, &mut root, Address::Welcome { step: Step::Welcome });
    assert_eq!(replaces, 1);
    assert_eq!(front.demo_starts.len(), 1);
    go(&mut front, &mut root, Address::Welcome { step: Step::Ready });
    assert!(!has(&root, Role::Region, b"Playground"));
    assert!(!has(&root, Role::Region, b"Setup"));
}
#[test]
fn absent_demo_still_has_the_setup_strip() {
    let (mut front, mut root) = accounts::forge_fixture(f::SEED);
    go(&mut front, &mut root, Address::Chats);
    assert!(has(&root, Role::Region, b"Setup"));
    assert!(!has(&root, Role::Region, b"Playground"));
}

fn add_model(front: &mut Joint, root: &mut Root) {
    go(
        front,
        root,
        Address::ConnectModel { way: Some(d::ModelWay { source: None, provider: accounts::PROVIDER + 1, kind: d::WayKind::Key }) },
    );
    pump(front, root, d::Event::AccountKey { value: skein_lib::Secret::new(b"test-private-key".as_slice().into()) });
    press(front, root, b"Add key");
    accounts::kept(front);
    accounts::settle(front, root);
}
fn connected(front: &mut Joint, root: &mut Root) {
    // The scripted engine half has durably ended before delivering the connection.
    front.demo = None;
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
    })));
    accounts::settle(front, root);
}
fn demo_account(front: &mut Joint, root: &mut Root) -> bool {
    let address = root.person().pages().address().clone();
    go(front, root, Address::Accounts);
    let found = root
        .person()
        .pages()
        .accounts()
        .expect("accounts page")
        .deployment(root.person().client())
        .expect("deployment delivered by the real engine")
        .accounts
        .iter()
        .any(|line| {
            line.provider == Some(accounts::PROVIDER)
                && line.holder == jig_client::contract::OwnerAccountHolder::Deployment
        });
    go(front, root, address);
    found
}
#[test]
fn a_model_connected_removes_the_scripted_account_and_leaves_the_playground_forge_banner() {
    let (mut front, mut root) = accounts::demo_fixture(f::SEED, &w::DemoState::Running);
    assert!(demo_account(&mut front, &mut root));
    assert!(root.person().demo().expect("running playground").1.model);
    add_model(&mut front, &mut root);
    assert!(!demo_account(&mut front, &mut root));
    assert_eq!(front.demo_removals.len(), 1);
    go(&mut front, &mut root, Address::Accounts);
    assert!(!has(&root, Role::Heading { level: 2 }, b"Scripted"));
    let halves = root.person().demo().expect("running playground").1;
    assert_eq!(halves, d::forge::demo::Halves { model: false, forge: true });
    go(&mut front, &mut root, Address::Chats);
    assert!(has(&root, Role::Region, b"Playground"));
    assert!(has(&root, Role::Paragraph, b"This is the playground: a playground forge."));
    assert!(!has(&root, Role::Button, b"Connect a model"));
    assert!(!has(&root, Role::Status, b"Ready"), "no demo start");
    go(&mut front, &mut root, Address::Forges);
    assert!(d::forge::listed::playground(root.person()).is_some());
    assert!(has(&root, Role::Heading { level: 2 }, b"The playground"));
    connected(&mut front, &mut root);
    assert_eq!(front.demo_removals.len(), 1, "forge end never repeats removal");
    assert!(root.person().forge().demo().is_none());
}
#[test]
fn a_forge_connected_clears_the_demo_everywhere_and_welcome_moves_when_a_model_is_usable() {
    let (mut front, mut root) = accounts::demo_fixture(f::SEED, &w::DemoState::Running);
    assert!(demo_account(&mut front, &mut root));
    go(&mut front, &mut root, Address::Forges);
    assert!(d::forge::listed::playground(root.person()).is_some());
    assert!(has(&root, Role::Heading { level: 2 }, b"The playground"));
    let reads = front.demo_reads;
    connected(&mut front, &mut root);
    assert_eq!(front.demo_reads, reads + 1);
    assert_eq!(front.demo_removals.len(), 1);
    assert!(!demo_account(&mut front, &mut root));
    assert!(root.person().forge().demo().is_none());
    assert!(d::forge::listed::playground(root.person()).is_none());
    assert!(!has(&root, Role::Region, b"Playground"));
    assert!(!has(&root, Role::Heading { level: 2 }, b"The playground"));
    go(&mut front, &mut root, Address::Chats);
    assert!(!has(&root, Role::Region, b"Playground"));
    assert!(!has(&root, Role::Button, b"Try the playground"));
    assert!(!has(&root, Role::Status, b"Ready"));
    // The fixture also has a usable configured model; connect the other fake
    // account before reopening Welcome, exercising both sources of usable facts.
    add_model(&mut front, &mut root);
    assert_eq!(go(&mut front, &mut root, Address::Welcome { step: Step::Welcome }), 1);
    assert_eq!(root.person().pages().address(), &Address::Chats);
    assert_eq!(front.demo_starts.len(), 1);
    assert_eq!(front.demo_removals.len(), 1);
}
