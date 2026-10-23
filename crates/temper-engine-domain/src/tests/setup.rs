//! Keyed setup and the deployment document through the real core (domain/root.md, 5.1).
use super::resume_fixture::{config, root_with_config};
use crate::{Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request, StoreInput, StoreRequest};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

const SIGN_IN: u64 = 10;
const WATCHER: Token = Token::new(30);

fn signed_in(role: Option<people::Role>) -> (Domain, Env<Limits>) {
    let mut config = config(1);
    config.forges =
        Box::new([forge::Configured { number: 1, kind: forge::Kind::Forgejo, mode: forge::Mode::Owned, asked: true }]);
    config.under_box = true;
    let (mut domain, env) = root_with_config(config);
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1);
    let mut out = Queue::with_capacity(people::max_out(&env.limits.people));
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"owner".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: SIGN_IN, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project: 0,
            holdings: match role {
                Some(role) => Box::new([people::Holding { person: 1, role }]),
                None => Box::new([]),
            },
        },
    ] {
        people::step(
            &mut domain.core.people,
            &Env { now: env.now, wall: env.wall, limits: env.limits.people },
            people::Event::Restore { record },
            &mut out,
        );
    }
    people::step(
        &mut domain.core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::Restored,
        &mut out,
    );
    (domain, env)
}

// Drive the root's existing journal seam; only its store acknowledgements are scripted.
fn settle(domain: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    let mut out = Queue::with_capacity(crate::max_out(&env.limits).checked_mul(2).unwrap());
    let mut requests = List::with_capacity(crate::max_out(&env.limits));
    for _ in 0_u32..256 {
        crate::release(domain, env, &mut out);
        crate::release(domain, env, &mut out);
        while let Some(request) = out.pop() {
            match request {
                Request::Store(StoreRequest::Commit { number, .. }) => {
                    crate::try_step(domain, env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
                }
                Request::ToChild(released) => crate::try_step(domain, env, Event::Released(released)).unwrap(),
                request @ (Request::SetUpGroup { .. }
                | Request::SetUp { .. }
                | Request::Setup { .. }
                | Request::Store(_)
                | Request::Worker(_)
                | Request::Party(_)
                | Request::Forge { .. }
                | Request::Account(_)
                | Request::Busy(_)
                | Request::Drained
                | Request::Stop) => requests.push(request).unwrap(),
            }
        }
        crate::try_step(domain, env, Event::Resume).unwrap();
    }
    requests.into_boxed()
}
fn setup(key: u8) -> Event {
    Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Deployment,
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in: SIGN_IN,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: forge::Ask::SetUp {
            forge: 1,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: forge::SetUpProject::Existing(1),
        },
    })
}
fn connection(domain: &mut Domain, env: &Env<Limits>, state: forge::Connection) {
    crate::try_step(domain, env, Event::Forge(Box::new(forge::Event::Connection { forge: 1, state }))).unwrap();
}
#[test]
fn a_keyed_setup_answers_the_client_and_the_same_key_replays_without_asking_the_box() {
    let (mut domain, env) = signed_in(Some(people::Role::Owner));
    connection(&mut domain, &env, forge::Connection::NotSetUp);
    crate::try_step(&mut domain, &env, setup(1)).unwrap();
    let requests = settle(&mut domain, &env);
    let [Request::SetUp { forge: 1, request, org, repository }] = requests.as_ref() else {
        panic!("one setup ask: {requests:?}")
    };
    assert_eq!((org.as_ref(), repository.as_ref()), (b"work".as_slice(), b"hello".as_slice()));
    crate::try_step(
        &mut domain,
        &env,
        Event::Forge(Box::new(forge::Event::SetUpAnswered {
            forge: 1,
            request: *request,
            answer: forge::SetUpAnswer::SettingUp,
        })),
    )
    .unwrap();
    let expected = Request::Party(PartyRequest::Deliver(Delivery::Application {
        to: ReplyTo::new(Token::new(90)),
        outcome: Some(forge::Answered::SetUp(forge::SetUpAnswer::SettingUp)),
    }));
    assert_eq!(settle(&mut domain, &env).as_ref(), [expected]);
    crate::try_step(&mut domain, &env, setup(1)).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Application {
            to: ReplyTo::new(Token::new(90)),
            outcome: Some(forge::Answered::SetUp(forge::SetUpAnswer::SettingUp)),
        }))]
    );
}
fn watch() -> Event {
    Event::Party(PartyInput::Watch {
        watcher: WATCHER,
        sign_in: SIGN_IN,
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Deployment,
            connector: 0,
            subject: Token::new(0),
        },
        window: jig_domain_core::documents::Window { rows: 1, after: None },
    })
}
#[test]
fn the_deployment_owner_watches_forges_again_after_a_connection_changes() {
    let (mut domain, env) = signed_in(Some(people::Role::Owner));
    connection(&mut domain, &env, forge::Connection::NotSetUp);
    crate::try_step(&mut domain, &env, watch()).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Forges {
            watcher: WATCHER,
            forges: forge::Forges { rows: Box::new([row(forge::Connection::NotSetUp)]) },
        }))]
    );
    crate::try_step(&mut domain, &env, Event::Party(PartyInput::Delivered { watcher: WATCHER })).unwrap();
    connection(&mut domain, &env, forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Forges {
            watcher: WATCHER,
            forges: forge::Forges {
                rows: Box::new([row(forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) })])
            },
        }))]
    );
    crate::try_step(&mut domain, &env, Event::Party(PartyInput::Delivered { watcher: WATCHER })).unwrap();
    crate::try_step(&mut domain, &env, Event::Forge(Box::new(forge::Event::BoxClosed))).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Forges {
            watcher: WATCHER,
            forges: forge::Forges {
                rows: Box::new([forge::Row {
                    first: None,
                    number: 1,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Owned,
                    connection: forge::Connection::SetUpFailed { why: b"the box closed the pipe".as_slice().into() },
                    offer: forge::Offer::NotUnderTheBox,
                }])
            },
        }))]
    );
}
#[test]
fn a_party_without_a_deployment_role_cannot_watch_forges_or_ask_the_box() {
    let (mut domain, env) = signed_in(None);
    crate::try_step(&mut domain, &env, watch()).unwrap();
    let requests = settle(&mut domain, &env);
    let [Request::Party(PartyRequest::WatchRefused { watcher: WATCHER, .. })] = requests.as_ref() else {
        panic!("deployment watch refused: {requests:?}")
    };
    crate::try_step(&mut domain, &env, setup(1)).unwrap();
    for request in settle(&mut domain, &env) {
        if let Request::SetUpGroup { .. } | Request::SetUp { .. } = request {
            panic!("a party without a deployment role never reaches the box")
        }
    }
}

fn row(connection: forge::Connection) -> forge::Row {
    forge::Row {
        first: None,
        number: 1,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Owned,
        connection,
        offer: forge::Offer::Offered,
    }
}
