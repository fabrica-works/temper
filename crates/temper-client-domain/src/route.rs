//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Action, Address, Domain, Event, Fact, Limits, Place, Request, forge, owner::Owner};
use skein_lib::{Env, List, Queue};
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_client_view::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
    root.jig.step(env.now, env.wall, &root.client, event, &mut child);
    route_jig(root, env, &mut child, out);
}
pub(crate) fn forge(root: &mut Domain, env: &Env<Limits>, event: forge::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(forge::max_out(&env.limits.forge));
    forge::step(&mut root.forge, &Env { now: env.now, wall: env.wall, limits: env.limits.forge }, event, &mut child);
    route_forge(root, env, &mut child, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Start { address, kept, offset: _ } => {
            let jig_kept = List::with_capacity(0);
            let jig_address = match &address {
                Address::Jig(address) => *address,
                Address::Forge(_) => jig_client_view::address::Address::Inbox { project: None },
            };
            let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
            root.jig.step(
                env.now,
                env.wall,
                &root.client,
                jig_client_view::Event::Start {
                    place: jig_client_view::address::Place::Jig(jig_address),
                    records: jig_kept,
                    seed: root.salt,
                },
                &mut child,
            );
            if !root.unlinked {
                client(root, env, jig_client::Event::Start { kept }, out);
            }
            route_jig(root, env, &mut child, out);
            told(root, env, jig_client::Told::Link { link: root.client.link().state }, out);
            match address {
                Address::Jig(_) => {}
                Address::Forge(address) => went(root, env, Place::Address(Address::Forge(address)), out),
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Forge(act) => forge(root, env, forge::Event::Act { act }, out),
            Action::Jig(act) => jig(root, env, jig_client_view::Event::Act { action: act }, out),
        },
        Event::Client { event } => client(root, env, event, out),
        Event::Jig(event) => jig(root, env, event, out),
        Event::Forge(event) => forge(root, env, event, out),
        Event::SignedOut => client(root, env, jig_client::Event::Up { up: jig_client::Up::SignedOut }, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            forge(root, env, forge::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Jig(address) }, out);
        }
        Place::Address(Address::Forge(address)) => {
            jig(root, env, jig_client_view::Event::Left, out);
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
        let request = child.pop().expect("counted jig output");
        match request {
            jig_client_view::Request::Client { asked } => client(root, env, asked, out),
            jig_client_view::Request::Show { place, how } => match place {
                jig_client_view::address::Place::Jig(address) => out.push(Request::Show {
                    address: Address::Jig(address),
                    replace: how == jig_client_view::How::Replace,
                }),
                jig_client_view::address::Place::Elsewhere | jig_client_view::address::Place::Missing => {}
            },
            jig_client_view::Request::ShowPart { part, scope } => crate::parts::show(root, env, part, scope, out),
            jig_client_view::Request::PartsLeft => forge(root, env, forge::Event::PartsLeft, out),
            jig_client_view::Request::Save { .. } | jig_client_view::Request::Erase { .. } => {}
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
            forge::Request::Client { asked } => client(root, env, asked, out),
            forge::Request::Show { address } => {
                out.push(Request::Show { address: Address::Forge(address), replace: false });
            }
            forge::Request::Send { request, ask } => client(
                root,
                env,
                jig_client::Event::Send {
                    key: jig_client::Key { asker: Owner::Forge.asker(), number: request.raw() },
                    request: ask,
                },
                out,
            ),
            forge::Request::Fact { fact: forge::Fact::Full } => out.push(Request::Fact { fact: Fact::PartsFull }),
        }
    }
}
pub(crate) fn client(root: &mut Domain, env: &Env<Limits>, event: jig_client::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client::max_out(&env.limits.client));
    root.client.step(env.now, env.wall, event, &mut child);
    route_client(root, env, &mut child, out);
}
pub(crate) fn route_client(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client::Request>,
    out: &mut Queue<Request>,
) {
    while let Some(request) = child.pop() {
        match request {
            jig_client::Request::Told { told: value } => told(root, env, value, out),
            request @ (jig_client::Request::Save { .. }
            | jig_client::Request::Erase { .. }
            | jig_client::Request::Exchange { .. }
            | jig_client::Request::OpenStream { .. }
            | jig_client::Request::CloseStream
            | jig_client::Request::Down { .. }
            | jig_client::Request::SignOut) => out.push(Request::Client { request }),
        }
    }
    forge_served(root, env, out);
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: jig_client::Told, out: &mut Queue<Request>) {
    use jig_client::Told;
    let asker = match &told {
        Told::Link { .. } => {
            forge_served(root, env, out);
            forge_me(root, env, &told, out);
            Owner::Jig.asker()
        }
        Told::Dropped { .. } => Owner::Jig.asker(),
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    match Owner::of(asker) {
        Some(Owner::Jig) => {
            let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
            root.jig.step(env.now, env.wall, &root.client, jig_client_view::Event::Client { told }, &mut child);
            route_jig(root, env, &mut child, out);
        }
        Some(Owner::Forge) => {
            if forge_me(root, env, &told, out) {
                return;
            }
            if let Some(event) = forge::told(&root.client, &env.limits, told) {
                forge(root, env, event, out);
            }
        }
        None => out.push(Request::Fact { fact: Fact::UnknownAsker }),
    }
}

fn forge_served(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let served = if root.client.link().not_served.contains(&root.forge_number()) {
        forge::Served::OutOfDate
    } else {
        forge::Served::Yes
    };
    if root.forge.served() != served {
        forge(root, env, forge::Event::Served { served }, out);
    }
}

fn forge_me(root: &mut Domain, env: &Env<Limits>, told: &jig_client::Told, out: &mut Queue<Request>) -> bool {
    let mut child = Queue::with_capacity(forge::max_out(&env.limits.forge));
    let consumed = forge::me::told(&mut root.forge, &root.client, told, &mut child);
    route_forge(root, env, &mut child, out);
    consumed
}
