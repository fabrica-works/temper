//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{
    Action, Address, Ask, Domain, Event, Fact, JigOut, Limits, Place, Request, forge,
    owner::Owner,
    store::{Key, Record},
};
use skein_lib::{Env, List, Map, Queue, RequestProgress};
pub(crate) fn jig_env(env: &Env<Limits>) -> Env<jig_domain_client::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.jig }
}
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_domain_client::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_domain_client::max_out(&env.limits.jig));
    jig_domain_client::step(&mut root.jig, &jig_env(env), event, &mut child);
    route_jig(root, env, &mut child, out);
}
pub(crate) fn forge(root: &mut Domain, env: &Env<Limits>, event: forge::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(forge::max_out(&env.limits.forge));
    forge::step(&mut root.forge, &Env { now: env.now, wall: env.wall, limits: env.limits.forge }, event, &mut child);
    route_forge(&mut child, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Start { address, kept, offset } => {
            let mut scopes = Map::<u64, u32>::with_capacity(env.limits.table.requests);
            let mut jig_kept = List::with_capacity(env.limits.jig.records);
            for record in kept.into_boxed() {
                match record {
                    Record::Jig(record) => {
                        if jig_kept.push(record).is_err() {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        }
                    }
                    Record::Table { version, record } => {
                        let size = crate::domain::ask_size(&env.limits, &record.request);
                        let scope = record.scope;
                        if version != crate::store::VERSION
                            || root.table.restore(Owner::Kept { scope }, size, record).is_err()
                        {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        } else {
                            let requests = scopes
                                .get(&scope)
                                .copied()
                                .unwrap_or(0)
                                .checked_add(1)
                                .expect("restored request count is bounded");
                            scopes.insert(scope, requests).expect("one scope per admitted restored request");
                        }
                    }
                }
            }
            let Address::Jig(jig_address) = address;
            jig(root, env, jig_domain_client::Event::Start { address: jig_address, kept: jig_kept, offset }, out);
            for (scope, requests) in &scopes {
                jig(root, env, jig_domain_client::Event::Kept { scope: *scope, requests: *requests }, out);
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Jig(act) => jig(root, env, jig_domain_client::Event::Act { act }, out),
        },
        Event::Answer { attempt, envelope, answer } => {
            crate::domain::answered(root, env, attempt, envelope, answer, out);
        }
        Event::Jig(event) => jig(root, env, event, out),
        Event::SignedOut => jig(root, env, jig_domain_client::Event::SignedOut, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            jig(
                root,
                env,
                jig_domain_client::Event::Went { place: jig_domain_client::address::Place::Jig(address) },
                out,
            );
        }
        Place::Missing => {
            jig(root, env, jig_domain_client::Event::Went { place: jig_domain_client::address::Place::Missing }, out);
        }
    }
}
pub(crate) fn route_jig(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_domain_client::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted jig output") {
            jig_domain_client::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Jig(request), scope, Ask::Jig(ask), out);
            }
            jig_domain_client::Request::Read { read, what } => out.push(Request::Jig(JigOut::Read { read, what })),
            jig_domain_client::Request::Open { watch, subject } => {
                out.push(Request::Jig(JigOut::Open { watch, subject }));
            }
            jig_domain_client::Request::Continue { watch, subject } => {
                out.push(Request::Jig(JigOut::Continue { watch, subject }));
            }
            jig_domain_client::Request::Close { watch } => out.push(Request::Jig(JigOut::Close { watch })),
            jig_domain_client::Request::Show { address, replace } => {
                out.push(Request::Show { address: Address::Jig(address), replace });
            }
            jig_domain_client::Request::Save { record } => out.push(Request::Save { record: Record::Jig(record) }),
            jig_domain_client::Request::Erase { key } => out.push(Request::Erase { key: Key::Jig(key) }),
            jig_domain_client::Request::SignOut => out.push(Request::SignOut),
            jig_domain_client::Request::SignIn { provider, back } => {
                out.push(Request::SignIn { provider, back: Address::Jig(back) });
            }
            jig_domain_client::Request::ShowPart { part, generation, scope } => {
                crate::parts::show(root, env, part, generation, scope, out);
            }
            jig_domain_client::Request::PartsLeft { generation } => {
                forge(root, env, forge::Event::PartsLeft { generation }, out);
            }
            jig_domain_client::Request::Link { link } => {
                root.table.link(match link {
                    jig_domain_client::link::Link::Live => true,
                    jig_domain_client::link::Link::Starting | jig_domain_client::link::Link::Offline => false,
                });
            }
            jig_domain_client::Request::Confirmed { party } => {
                root.table.confirmed(party);
            }
            jig_domain_client::Request::Retention { retention } => root.table.retention(retention),
            jig_domain_client::Request::FrameIs { .. } => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            jig_domain_client::Request::Answered { asked, answer } => {
                out.push(Request::Jig(JigOut::Answered { asked, answer }));
            }
            jig_domain_client::Request::Unknown { asked } => out.push(Request::Jig(JigOut::Unknown { asked })),
            jig_domain_client::Request::Fact { fact } => out.push(Request::Fact { fact: Fact::Jig(fact) }),
        }
    }
}
pub(crate) fn route_forge(child: &mut Queue<forge::Request>, out: &mut Queue<Request>) {
    for _ in 0..child.len() {
        match child.pop().expect("counted forge output") {
            forge::Request::Fact { fact: forge::Fact::Full } => out.push(Request::Fact { fact: Fact::PartsFull }),
        }
    }
}
pub(crate) fn progress(
    root: &mut Domain,
    env: &Env<Limits>,
    owner: Owner,
    progress: RequestProgress,
    out: &mut Queue<Request>,
) {
    match owner {
        Owner::Jig(request) => {
            let progress = match progress {
                RequestProgress::InFlight => jig_domain_client::intents::Progress::InFlight,
                RequestProgress::Retrying => jig_domain_client::intents::Progress::Retrying,
                RequestProgress::Parked => jig_domain_client::intents::Progress::Parked,
                RequestProgress::Unknown => jig_domain_client::intents::Progress::Unknown,
            };
            jig(root, env, jig_domain_client::Event::Progress { request, progress }, out);
        }
        Owner::Kept { scope } => match progress {
            RequestProgress::Unknown => jig(root, env, jig_domain_client::Event::KeptGone { scope }, out),
            RequestProgress::InFlight | RequestProgress::Retrying | RequestProgress::Parked => {}
        },
    }
}
