use super::resume_fixture::root;
use crate::{Domain, Event, Limits, Request, StoreInput};
use skein_lib::{Env, List, Queue, Time, Wall};

// More than the service's two empty release passes, at one injected instant.
const RESUMES: u32 = 4;
// Initial instant, then an independent change of each injected clock.
const CLOCK_PAIRS: u32 = 3;
const WATCHER: skein_lib::Token = skein_lib::Token::new(30);
const SIGN_IN: u64 = 10;

fn resume(domain: &mut Domain, env: &Env<Limits>) {
    for _ in 0..RESUMES {
        crate::try_step(domain, env, Event::Resume).unwrap();
    }
}

#[test]
fn settled_resumes_decide_once_at_each_clock_pair() {
    let (mut domain, mut env) = root();
    let mut out = Queue::with_capacity(crate::max_out(&env.limits));
    let mut counts = List::with_capacity(CLOCK_PAIRS);
    for (now, wall) in [(0, 0), (1, 0), (1, 1)] {
        env.now = Time::from_nanos(now);
        env.wall = Wall::from_nanos(wall);
        resume(&mut domain, &env);
        crate::release(&mut domain, &env, &mut out);
        counts.push(domain.empty_resumes).unwrap();
    }
    assert_eq!(counts.as_slice(), [1, 2, 3]);
    assert!(out.is_empty());
    assert_eq!(domain.deployment().commits, 0);
}

#[test]
fn inputs_fire_and_reclaim_each_allow_a_new_empty_pass() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    // An input, fire and reclaim each invalidate the previous quiet pass.
    let mut counts = List::with_capacity(3);
    crate::try_step(&mut domain, &env, Event::Party(crate::PartyInput::Unwatch { watcher: skein_lib::Token::new(1) }))
        .unwrap();
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    crate::fire(&mut domain, &env);
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    domain.reclaim();
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    assert_eq!(counts.as_slice(), [2, 3, 4]);
    assert_eq!(domain.deployment().commits, 0);
}

#[test]
fn an_automatic_header_commit_cannot_mark_the_pass_quiet() {
    let (mut domain, env) = root();
    crate::fresh(&mut domain.core.counters, crate::Family::Message).unwrap();
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    assert_eq!(domain.empty_resumes, 0);
    assert_eq!(domain.settled, None);
    assert_eq!(domain.deployment().commits, 1);
}

#[test]
fn a_commit_completion_invalidates_a_settled_pass() {
    let (mut domain, env) = root();
    crate::fresh(&mut domain.core.counters, crate::Family::Message).unwrap();
    resume(&mut domain, &env);
    let mut out = Queue::with_capacity(crate::max_out(&env.limits));
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, .. })) = out.pop() else { panic!("header commit") };
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    resume(&mut domain, &env);
    assert_eq!(domain.empty_resumes, 2);
    assert_eq!(domain.deployment().commits, 1);
}

#[test]
fn released_callbacks_and_restart_invalidate_the_quiet_mark() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    let released = crate::Released(crate::Delivery::Fleet(jig_domain_fleet::Event::Acknowledge {
        run: skein_lib::Token::new(1),
        attempt: skein_lib::Token::new(1),
    }));
    crate::try_step(&mut domain, &env, Event::Released(released)).unwrap();
    resume(&mut domain, &env);
    let after_callback = domain.empty_resumes;
    crate::try_step(&mut domain, &env, Event::Restart).unwrap();
    resume(&mut domain, &env);
    assert_eq!((after_callback, domain.empty_resumes), (2, 3));
}

#[test]
fn finishing_frees_retained_text_without_any_watcher() {
    let (mut domain, env) = root();
    let run = skein_lib::Token::new(1);
    let mut out = Queue::with_capacity(1);
    jig_domain_views::step(
        &mut domain.core.views,
        &Env { now: env.now, wall: env.wall, limits: crate::VIEWS },
        jig_domain_views::Event::Text {
            run,
            turn: jig_domain_views::TurnName { attempt: 1, turn: 1, retry: 0 },
            offset: 0,
            text: b"provisional".as_slice().into(),
            call: None,
        },
        &mut out,
    );
    let retained = domain.core.views.retained_runs().len();
    resume(&mut domain, &env);
    assert_eq!((retained, domain.core.views.retained_runs().len(), domain.empty_resumes), (1, 0, 1));
    assert!(out.is_empty());
}

fn watched_root() -> (Domain, Env<Limits>) {
    use jig_domain_people as people;
    let (mut domain, env) = root();
    let mut out = Queue::with_capacity(people::max_out(&env.limits.people));
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1);
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
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
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
    crate::try_step(
        &mut domain,
        &env,
        Event::Party(crate::PartyInput::Watch {
            watcher: WATCHER,
            sign_in: SIGN_IN,
            subject: jig_domain_core::documents::Watched::Frame,
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        }),
    )
    .unwrap();
    let mut released = Queue::with_capacity(crate::max_out(&env.limits));
    crate::release(&mut domain, &env, &mut released);
    crate::release(&mut domain, &env, &mut released);
    crate::try_step(&mut domain, &env, Event::Party(crate::PartyInput::Delivered { watcher: WATCHER })).unwrap();
    resume(&mut domain, &env);
    (domain, env)
}

#[test]
fn a_projection_feed_then_committed_resume_delivers_the_watch() {
    let (mut domain, env) = watched_root();
    // Restore a changed projection through jig's existing row seam. The next
    // ordinary input finishes its feed before the deferred watch pass.
    crate::fresh(&mut domain.core.counters, crate::Family::Task).unwrap();
    let restored = domain.core.restore_core(
        jig_domain_core::CoreRecord::Projection(jig_domain_core::ProjectionRecord::Goal {
            goal: jig_domain_core::ProjectionGoal {
                number: 1,
                project: 1,
                priority: 1,
                words: b"goal".as_slice().into(),
                phase: jig_domain_tasks::Phase::Waiting,
            },
            pending: Box::new([]),
            changed: true,
            fed_at: None,
        }),
        &crate::limits::core_limits(&env.limits),
    );
    crate::try_step(
        &mut domain,
        &env,
        Event::Party(crate::PartyInput::Period { project: 1, period: 1, budget: 100, next: None }),
    )
    .unwrap();
    let mut out = Queue::with_capacity(crate::max_out(&env.limits));
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, writes })) = out.pop() else {
        panic!("projection and period commit")
    };
    let mut projection_written = false;
    for write in &writes {
        projection_written |= write.key()
            == crate::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Projection(
                jig_domain_core::ProjectionKey::Goal(1),
            )));
    }
    let before_committed = out.len();
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Party(crate::PartyRequest::Deliver(crate::Delivery::Document { watcher, .. }))) = out.pop()
    else {
        panic!("deferred watch document: {out:?}")
    };
    assert_eq!(restored, jig_domain_core::Restored::Live);
    assert!(projection_written);
    assert_eq!((before_committed, watcher, domain.empty_resumes), (0, WATCHER, 1));
    assert_eq!(domain.deployment().commits, 1);
    assert!(out.is_empty());
}

#[test]
fn advancing_wall_time_ends_a_settled_watch() {
    let (mut domain, mut env) = watched_root();
    env.wall = Wall::from_nanos(100);
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    let mut out = Queue::with_capacity(crate::max_out(&env.limits));
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Party(crate::PartyRequest::Deliver(crate::Delivery::WatchEnded {
            watcher: WATCHER,
            why: jig_domain_views::End::SignedOut,
        })))
    );
    assert!(out.is_empty());
}

#[test]
fn stop_from_settled_still_releases_the_drain_terminal() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    crate::try_step(&mut domain, &env, Event::Stop).unwrap();
    let mut out = Queue::with_capacity(crate::max_out(&env.limits));
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Drained));
    assert!(out.is_empty());
    assert_eq!(domain.settled, None);
}
