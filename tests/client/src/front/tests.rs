use super::*;
use jig_client_world::harness::{App, Duty};
use skein_lib::{Env, Queue};

const CLIENT: ClientId = ClientId(0);
const CALLBACK_DELAY: Duration = Duration::from_millis(3);
const WALL: Wall = Wall::from_nanos(0);

#[test]
fn an_authenticated_reload_routes_the_restored_person_frame() {
    let limits = fixtures::domain_limits();
    let mut front = Joint::new(fixtures::SEED, limits);
    front.bind(CLIENT, fixtures::PARTY);
    let (mut root, _) = Temper::new(fixtures::SEED);
    settle_root(&mut front, &mut root, &limits, Vec::new());
    assert_eq!(root.client().link().state, jig_client::Link::Live);
    let kept = pending_adoption(&mut front, &mut root, &limits);
    front.reload(CLIENT, front.now);
    let (mut restored, mut view) = Temper::new(1102);
    settle_root(&mut front, &mut restored, &limits, kept);
    assert_eq!(restored.client().link().state, jig_client::Link::Live);
    assert_eq!(restored.client().link().party, Some(fixtures::PARTY));
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, &restored, &mut patches);
    assert!(Temper::tree(&view).nodes().iter().any(|node| node.name.as_deref() == Some(b"Person")));
}

fn settle_root(
    front: &mut Joint,
    root: &mut client::Domain,
    limits: &client::Limits,
    kept: Vec<skein_lib::RequestRecord<jig_client::Held>>,
) {
    let mut env = Env { now: front.now, wall: WALL, limits: *limits };
    let mut out = Queue::with_capacity(Temper::max_out(limits));
    Temper::step(root, &env, Temper::start(fixtures::project(), kept, 0), &mut out);
    assert_eq!(flush(front, root, &env, &mut out), 1, "startup shares the own Me with the UI");
    for _ in 0..8 {
        let callbacks = front.poll(env.now, env.wall);
        if callbacks.is_empty() {
            return;
        }
        for (who, event) in callbacks {
            assert_eq!(who, CLIENT);
            env.now = env.now.saturating_add(CALLBACK_DELAY);
            front.received(CLIENT, &event);
            Temper::step(root, &env, event, &mut out);
            flush(front, root, &env, &mut out);
        }
    }
    panic!("root/front did not settle");
}

fn flush(front: &mut Joint, root: &mut client::Domain, env: &Env<client::Limits>, out: &mut Queue<Request>) -> u32 {
    let mut frames = 0;
    Temper::take(root, env, out);
    while let Some(request) = out.pop() {
        if matches!(
            &request,
            Request::Client {
                request: jig_client::Request::Down {
                    down: jig_client::Down::Open {
                        subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
                        ..
                    }
                }
            }
        ) {
            frames += 1;
        }
        if let Duty::Wire(request) = Temper::duty(request) {
            front.wire(CLIENT, env.now, env.wall, request);
        }
    }
    frames
}

fn pending_adoption(
    front: &mut Joint,
    root: &mut client::Domain,
    limits: &client::Limits,
) -> Vec<skein_lib::RequestRecord<jig_client::Held>> {
    use client::forge::{Act, Text, Typed};
    let env = Env { now: front.now, wall: WALL, limits: *limits };
    let mut out = Queue::with_capacity(Temper::max_out(limits));
    Temper::step(
        root,
        &env,
        RootEvent::Went { place: client::Place::Address(client::Address::Adopt { project: 1 }) }.into(),
        &mut out,
    );
    for (field, text) in [(Text::Owner, b"acme".as_slice()), (Text::Name, b"shop"), (Text::Landing, b"main")] {
        Temper::step(
            root,
            &env,
            RootEvent::Act {
                action: client::Action::Forge(Act::Write { field, text: text.into(), typed: Typed::Bytes }),
            }
            .into(),
            &mut out,
        );
    }
    front.fault(env.now, FrontFault::NoAnswer);
    Temper::step(root, &env, RootEvent::Act { action: client::Action::Forge(Act::Adopt) }.into(), &mut out);
    let mut kept = Vec::new();
    while let Some(request) = out.pop() {
        match Temper::duty(request) {
            Duty::Store(jig_client_world::harness::StoreWrite::Save(record)) => kept.push(record),
            Duty::Wire(request) => front.wire(CLIENT, env.now, env.wall, request),
            Duty::Store(_) | Duty::Show { .. } | Duty::SignIn { .. } | Duty::SignOut | Duty::Fact => {}
        }
    }
    assert_eq!(kept.len(), 1);
    kept
}

#[test]
fn a_real_sign_out_records_the_engine_receipt_before_the_callback() {
    let mut front = Joint::new(fixtures::SEED, fixtures::domain_limits());
    front.bind(CLIENT, fixtures::PARTY);
    front.poll(Time::ZERO, WALL);
    front.sign_out(CLIENT, Time::ZERO, WALL);
    let callbacks = front.poll(Time::ZERO, WALL);
    assert!(callbacks.iter().any(|(who, event)| *who == CLIENT
        && matches!(
            event.root(),
            Some(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } })
        )));
    assert!(
        front.observed().into_iter().any(|observation| matches!(
            observation, Observed::SignInEnded { client } if client == CLIENT
        )),
        "the real engine's sign-out answer must precede the callback"
    );
}

#[test]
fn the_referee_reads_live_from_tempers_link_region() {
    let limits = fixtures::domain_limits();
    let mut front = Joint::new(fixtures::SEED, limits);
    front.bind(CLIENT, fixtures::PARTY);
    let (mut root, mut view) = Temper::new(fixtures::SEED);
    settle_root(&mut front, &mut root, &limits, Vec::new());
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, &root, &mut patches);
    assert!(Temper::seen(Temper::tree(&view)).contains(&jig_client_world::harness::Seen::Link { live: true }));
}
