//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{
    Action, Address, Ask, Domain, Event, Fact, JigOut, Limits, Place, Request, forge, owner::Owner, store::Record,
};
use skein_lib::{Env, List, Map, Queue, RequestProgress};
pub(crate) fn jig_env(env: &Env<Limits>) -> Env<jig_client_view::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.jig }
}
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_client_view::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
    jig_client_view::step(&mut root.jig, &jig_env(env), event, &mut child);
    route_jig(root, env, &mut child, out);
}
pub(crate) fn forge(root: &mut Domain, env: &Env<Limits>, event: forge::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(forge::max_out(&env.limits.forge));
    forge::step(&mut root.forge, &Env { now: env.now, wall: env.wall, limits: env.limits.forge }, event, &mut child);
    route_forge(root, env, &mut child, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Start { address, kept, offset } => {
            let mut scopes = Map::<u64, u32>::with_capacity(env.limits.table.requests);
            let jig_kept = List::with_capacity(0);
            for record in kept.into_boxed() {
                match record {
                    Record::Table { version, record } => {
                        let size = crate::domain::ask_size(&env.limits, &record.request.request);
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
            let jig_address = match &address {
                Address::Jig(address) => *address,
                Address::Forge(_) => jig_client_view::address::Address::Inbox { project: None },
            };
            jig(root, env, jig_client_view::Event::Start { address: jig_address, kept: jig_kept, offset }, out);
            match address {
                Address::Jig(_) => {}
                Address::Forge(address) => went(root, env, Place::Address(Address::Forge(address)), out),
            }
            for (scope, requests) in &scopes {
                jig(root, env, jig_client_view::Event::Kept { scope: *scope, requests: *requests }, out);
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Forge(act) => forge(root, env, forge::Event::Act { act }, out),
            Action::Jig(act) => jig(root, env, jig_client_view::Event::Act { act }, out),
        },
        Event::Answer { attempt, envelope, answer } => {
            crate::domain::answered(root, env, attempt, envelope, answer, out);
        }
        Event::Jig(event) => jig(root, env, event, out),
        Event::Forge(event) => forge(root, env, event, out),
        Event::SignedOut => jig(root, env, jig_client_view::Event::SignedOut, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            forge(root, env, forge::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Jig(address) }, out);
        }
        Place::Address(Address::Forge(address)) => {
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Elsewhere }, out);
            forge(root, env, forge::Event::Went { address }, out);
        }
        Place::Missing => {
            forge(root, env, forge::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Missing }, out);
        }
    }
}
pub(crate) fn route_jig(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client_view::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted jig output") {
            jig_client_view::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Jig(request), scope, Ask::Jig(ask), out);
            }
            jig_client_view::Request::Read { read, what } => out.push(Request::Jig(JigOut::Read { read, what })),
            jig_client_view::Request::Open { watch, subject } => {
                out.push(Request::Jig(JigOut::Open { watch, subject }));
            }
            jig_client_view::Request::Continue { watch, subject } => {
                out.push(Request::Jig(JigOut::Continue { watch, subject }));
            }
            jig_client_view::Request::Close { watch } => out.push(Request::Jig(JigOut::Close { watch })),
            jig_client_view::Request::Show { address, replace } => {
                out.push(Request::Show { address: Address::Jig(address), replace });
            }
            jig_client_view::Request::Save { .. } | jig_client_view::Request::Erase { .. } => {}
            jig_client_view::Request::SignOut => out.push(Request::SignOut),
            jig_client_view::Request::SignIn { provider, back } => {
                out.push(Request::SignIn { provider, back: Address::Jig(back) });
            }
            jig_client_view::Request::ShowPart { part, generation, scope } => {
                crate::parts::show(root, env, part, generation, scope, out);
            }
            jig_client_view::Request::PartsLeft { generation } => {
                forge(root, env, forge::Event::PartsLeft { generation }, out);
            }
            jig_client_view::Request::Link { link } => {
                root.table.link(match link {
                    jig_client_view::link::Link::Live => true,
                    jig_client_view::link::Link::Starting | jig_client_view::link::Link::Offline => false,
                });
            }
            jig_client_view::Request::Confirmed { party } => {
                forge(root, env, forge::Event::Confirmed { party }, out);
                root.table.confirmed(party);
            }
            jig_client_view::Request::Retention { retention } => root.table.retention(retention),
            jig_client_view::Request::FrameIs { .. } => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            jig_client_view::Request::Answered { asked, answer } => {
                out.push(Request::Jig(JigOut::Answered { asked, answer }));
            }
            jig_client_view::Request::Unknown { asked } => out.push(Request::Jig(JigOut::Unknown { asked })),
            jig_client_view::Request::Fact { fact } => out.push(Request::Fact { fact: Fact::Jig(fact) }),
        }
    }
}
pub(crate) fn route_forge(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<forge::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted forge output") {
            forge::Request::Show { address } => {
                out.push(Request::Show { address: Address::Forge(address), replace: false });
            }
            forge::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Forge(request), scope, Ask::Forge(ask), out);
            }
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
        Owner::Forge(request) => forge(root, env, forge::Event::Progress { request, progress }, out),
        Owner::Jig(request) => {
            let progress = match progress {
                RequestProgress::InFlight => jig_client_view::intents::Progress::InFlight,
                RequestProgress::Retrying => jig_client_view::intents::Progress::Retrying,
                RequestProgress::Parked => jig_client_view::intents::Progress::Parked,
                RequestProgress::Unknown => jig_client_view::intents::Progress::Unknown,
            };
            jig(root, env, jig_client_view::Event::Progress { request, progress }, out);
        }
        Owner::Kept { scope } => match progress {
            RequestProgress::Unknown => jig(root, env, jig_client_view::Event::KeptGone { scope }, out),
            RequestProgress::InFlight | RequestProgress::Retrying | RequestProgress::Parked => {}
        },
    }
}
