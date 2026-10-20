//! Route the shared client's tells to temper's owning child.
use crate::{Action, Address, Domain, Event, Fact, Limits, Place, Request, forge, owner::Owner};
use skein_lib::{Env, Queue};
pub(crate) fn forge(root: &mut Domain, env: &Env<Limits>, event: forge::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(forge::max_out(&env.limits.forge));
    forge::step(&mut root.forge, &Env { now: env.now, wall: env.wall, limits: env.limits.forge }, event, &mut child);
    route_forge(root, env, &mut child, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Start { address, kept } => {
            if !root.unlinked {
                client(root, env, jig_client::Event::Start { kept }, out);
            }
            told(root, env, jig_client::Told::Link { link: root.client.link().state }, out);
            pages(root, env, crate::pages::Event::Started, out);
            went(root, env, Place::Address(address), out);
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Pages(act) => pages(root, env, crate::pages::Event::Act { act }, out),
            Action::Forge(act) => forge(root, env, forge::Event::Act { act }, out),
        },
        Event::Client { event } => client(root, env, event, out),
        Event::Forge(event) => forge(root, env, event, out),
        Event::AccountKey { value } => pages(root, env, crate::pages::Event::AccountKey { value }, out),
        Event::SignedOut => client(root, env, jig_client::Event::Up { up: jig_client::Up::SignedOut }, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(address) => {
            pages(root, env, crate::pages::Event::Went { address: address.clone() }, out);
            if let Address::Adopt { project } = address {
                forge(root, env, forge::Event::Went { address: forge::Address::Adopt { project } }, out);
            } else {
                forge(root, env, forge::Event::Left, out);
            }
        }
        Place::Missing => {
            pages(root, env, crate::pages::Event::Left, out);
            forge(root, env, forge::Event::Left, out);
        }
    }
}
pub(crate) fn pages(root: &mut Domain, env: &Env<Limits>, event: crate::pages::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(crate::pages::max_out(&env.limits));
    crate::pages::step(&mut root.pages, env, &root.client, event, &mut child);
    for _ in 0..child.len() {
        match child.pop().expect("counted pages output") {
            crate::pages::Request::Client(event) => client(root, env, event, out),
            crate::pages::Request::Show { address } => out.push(Request::Show { address, replace: false }),
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
                let forge::Address::Adopt { project } = address;
                out.push(Request::Show { address: Address::Adopt { project }, replace: false });
            }
            forge::Request::Send { request, ask } => client(
                root,
                env,
                jig_client::Event::Send {
                    key: jig_client::Key { asker: Owner::Forge.asker(), number: request.raw() },
                    name: None,
                    request: ask,
                },
                out,
            ),
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
            | jig_client::Request::ServiceCredential { .. }
            | jig_client::Request::SignOut) => out.push(Request::Client { request }),
        }
    }
    forge_served(root, env, out);
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: jig_client::Told, out: &mut Queue<Request>) {
    use jig_client::Told;
    let asker = match &told {
        Told::Link { link } => {
            pages(root, env, crate::pages::Event::Link { link: *link }, out);
            forge_served(root, env, out);
            forge_me(root, env, &told, out);
            return;
        }
        Told::Dropped { count } => {
            out.push(Request::Dropped { count: *count });
            return;
        }
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    match Owner::of(asker) {
        Some(Owner::Pages) => pages(root, env, crate::pages::Event::Told { told }, out),
        Some(Owner::Forge) => {
            if forge_me(root, env, &told, out) {
                return;
            }
            if let Some(event) = forge::told(&root.client, &env.limits, told) {
                forge(root, env, event, out);
            }
        }
        Some(Owner::Runner) | None => out.push(Request::Fact { fact: Fact::UnknownAsker }),
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
