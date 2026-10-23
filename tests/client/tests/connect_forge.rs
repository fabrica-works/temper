use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Choice, Step};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::{Joint, account_acts as accounts},
};
use temper_engine_domain_forge::{self as forge, Connection};
const OWNED: u16 = 1;
const CLIENT: ClientId = ClientId(0);
const PUMP_STEPS: usize = 64;
fn pump(front: &mut Joint, root: &mut Root, now: &mut Time, event: d::Event) -> bool {
    let limits = fixtures::domain_limits();
    let mut out = Queue::with_capacity(Temper::max_out(&limits));
    Temper::step(root, &Env { now: *now, wall: Wall::EPOCH, limits }, event.into(), &mut out);
    let mut model = false;
    for _ in 0..PUMP_STEPS {
        while let Some(request) = out.pop() {
            if let d::Request::Show { address, .. } = &request {
                model |= *address == Address::Welcome { step: Step::Model { way: None } };
                Temper::step(
                    root,
                    &Env { now: *now, wall: Wall::EPOCH, limits },
                    d::Event::Went { place: d::Place::Address(address.clone()) }.into(),
                    &mut out,
                );
            }
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(CLIENT, *now, Wall::EPOCH, request);
            }
        }
        *now = now.saturating_add(Duration::from_millis(3));
        let callbacks = front.poll(*now, Wall::EPOCH);
        if callbacks.is_empty() {
            return model;
        }
        for (_, event) in callbacks {
            front.received(CLIENT, &event);
            Temper::step(root, &Env { now: *now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    panic!("forge story did not settle");
}
fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
fn node(view: &temper_client_view::View, role: Role, name: &[u8]) -> skein_ui::NodeId {
    view.tree()
        .nodes()
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
        .id
}
fn connection(front: &mut Joint, state: Connection) {
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::Connection { forge: OWNED, state })));
    front.engine.settle();
}
#[test]
fn setting_up_forgejo_here_from_the_welcome() {
    let (mut front, mut root) = accounts::forge_fixture(fixtures::SEED);
    let mut now = Time::ZERO.saturating_add(Duration::from_secs(1));
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: None } }) },
    );
    let mut choose = view(&root);
    let heading = node(&choose, Role::Region, b"Forgejo on this machine");
    let index = choose.tree().nodes().iter().position(|n| n.id == heading).expect("choice region");
    let size = usize::try_from(choose.tree().nodes()[index].size).expect("region size fits");
    let region = &choose.tree().nodes()[index..][..size];
    let link =
        region.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Choose")).expect("choice link").id;
    let action = temper_client_view::act(&mut choose, link, skein_ui::Act::Press).expect("bound choice");
    pump(&mut front, &mut root, &mut now, d::Event::Act { action });
    assert_eq!(
        root.person().pages().address(),
        &Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } }
    );
    let mut details = view(&root);
    let button = node(&details, Role::Button, b"Set up a Forgejo here");
    let action = temper_client_view::act(&mut details, button, skein_ui::Act::Press).expect("bound set up");
    assert!(!pump(&mut front, &mut root, &mut now, d::Event::Act { action }), "the box has not answered yet");
    assert_eq!(front.set_ups().iter().map(|(forge, _)| *forge).collect::<Vec<_>>(), [OWNED]);
    let (forge, request) = *front.set_ups().first().expect("engine asked the box");
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge,
        request,
        answer: forge::SetUpAnswer::SettingUp,
    })));
    front.engine.settle();
    assert!(
        pump(
            &mut front,
            &mut root,
            &mut now,
            d::Event::Went {
                place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } })
            }
        ),
        "accepted setup moves to Model"
    );
    connection(&mut front, Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let model = view(&root);
    node(&model, Role::Paragraph, "Forgejo on this machine · Setting up: preparing".as_bytes());
    node(&model, Role::Disclosure, b"Each step");
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(Address::Chats) });
    let strip = view(&root);
    node(&strip, Role::Region, b"Setup");
    node(&strip, Role::Link, b"View");
    node(&strip, Role::Paragraph, "Forgejo on this machine · Setting up: preparing".as_bytes());
    assert!(strip.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&temper_client_view::look::NAV_LIMITED)));
    connection(&mut front, Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::ConnectForge { forge: Some(Choice::Forge(OWNED)) }) },
    );
    let making = view(&root);
    node(&making, Role::Heading { level: 2 }, b"Setting up Forgejo on this machine");
    node(&making, Role::Paragraph, b"work/hello, into Project 1");
    let before = front.applications_admitted();
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::FirstMade { forge: OWNED, id: 42 })));
    front.engine.settle();
    // Creation is observed and adopted while the person is on another page.
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(Address::Chats) });
    assert_eq!(front.applications_admitted(), before + 1, "one adoption admitted");
    assert!(front.engine_adopted(OWNED, 42));
    for _ in 0_u32..2 {
        pump(
            &mut front,
            &mut root,
            &mut now,
            d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) },
        );
        let ready = view(&root);
        node(&ready, Role::Heading { level: 2 }, b"work/hello is ready");
        node(&ready, Role::Link, b"Open Project 1");
        assert_eq!(front.applications_admitted(), before + 1, "revisiting Ready does not adopt again");
    }
}

#[test]
fn a_connected_forge_and_usable_model_remove_the_setup_strip_and_dot() {
    use d::pages::connect_model::Act as A;
    let (mut front, mut root) = accounts::forge_fixture(fixtures::SEED);
    let mut now = Time::ZERO.saturating_add(Duration::from_secs(1));
    connection(&mut front, Connection::Connecting);
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(Address::Chats) });
    let strip = view(&root);
    node(&strip, Role::Region, b"Setup");
    node(&strip, Role::Link, b"View");
    let way = d::ModelWay { provider: accounts::PROVIDER, kind: d::WayKind::Key };
    for event in [
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: Some(way) } }) },
        d::Event::AccountKey { value: skein_lib::Secret::new(b"test-private-key".as_slice().into()) },
        d::Event::Act { action: d::Action::Pages(d::pages::Act::ConnectModel(A::Submit { name: [41; 16] })) },
    ] {
        pump(&mut front, &mut root, &mut now, event);
    }
    accounts::kept(&mut front);
    connection(&mut front, Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(Address::Chats) });
    let ready = view(&root);
    assert!(!ready.tree().nodes().iter().any(|n| n.role == Role::Region && n.name.as_deref() == Some(b"Setup")));
    assert!(!ready.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&temper_client_view::look::NAV_LIMITED)
        || n.styles.as_slice().contains(&temper_client_view::look::NAV_NEEDS)));
}
