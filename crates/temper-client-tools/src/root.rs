//! The tools' shared client and exhaustive owner routing (tools.md, 4).
use crate::jig::{self, runner};
use alloc::boxed::Box;
use jig_client::{Client, Told};
use skein_lib::{Env, Queue, Time};
use temper_client_domain::owner::Owner;
pub const RUNNER: Owner = Owner::Runner;
#[derive(Debug)]
pub struct Limits {
    pub domain: temper_client_domain::Limits,
    pub tools: jig::Limits,
}
#[derive(Debug)]
pub struct Root {
    client: Client,
    runner: runner::Runner,
    adopter: crate::forge::Adopter,
    next: Option<u32>,
}
#[derive(Debug)]
pub enum Event {
    Client { event: jig_client::Event },
    Call { number: u32, call: jig::call::Call, json: bool },
}
#[derive(Debug)]
pub enum Request {
    Client { request: jig_client::Request },
    Ready,
    Kept { line: Box<[u8]> },
    Printed { number: u32, printed: jig::Printed },
    Fact { fact: temper_client_domain::Fact },
}
impl Root {
    /// # Errors
    /// Refuses client limits that cannot be represented.
    pub fn new(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable> {
        Ok(Self::with_client(Client::new(&limits.domain.client, seed)?))
    }
    pub(crate) fn with_client(client: Client) -> Self {
        let mut runner = runner::Runner::new(RUNNER.asker(), crate::key::TOOLS);
        runner.parts(owner_part);
        Self { client, runner, adopter: crate::forge::Adopter::new(), next: Some(1) }
    }
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }
    #[must_use]
    pub fn kept(&self) -> u32 {
        self.runner.kept(&self.client)
    }
    pub fn kept_json(&mut self, json: bool) {
        self.runner.kept_json(json);
        self.adopter.kept_json(json);
    }
}
pub fn step(root: &mut Root, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let mut client = Queue::with_capacity(jig_client::max_out(&env.limits.domain.client));
    let mut child = Queue::with_capacity(env.limits.tools.rows.checked_mul(2).expect("tools outputs fit"));
    match event {
        Event::Client { event } => {
            if let jig_client::Event::Start { .. } | jig_client::Event::StartUnlinked { .. } = &event {
                root.runner.start(&mut child);
            }
            root.client.step(env.now, env.wall, event, &mut client);
        }
        Event::Call { number, call, json } => {
            if let Some(call_number) = root.next {
                root.next = call_number.checked_add(1);
                let forge = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name
                    == crate::adopt::ROW.name;
                let called = runner::Called { number, call_number, call, json };
                if forge {
                    crate::forge::call(&mut root.adopter, &root.client, &env.limits, called, &mut child);
                } else {
                    runner::call(&mut root.runner, &root.client, &tools_env(env), called, &mut child);
                }
            } else {
                out.push(Request::Printed { number, printed: jig::render::failure(b"call number exhausted", json, 1) });
            }
        }
    }
    drain(root, env, &mut client, &mut child, out);
}
fn tools_env(env: &Env<Limits>) -> Env<jig::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.tools }
}
fn drain(
    root: &mut Root,
    env: &Env<Limits>,
    client: &mut Queue<jig_client::Request>,
    child: &mut Queue<runner::Out>,
    out: &mut Queue<Request>,
) {
    let bound = jig_client::max_out(&env.limits.domain.client)
        .checked_mul(env.limits.tools.rows.checked_add(1).expect("rows fit"))
        .expect("routing bound fits");
    for _ in 0..bound {
        if let Some(request) = client.pop() {
            match request {
                jig_client::Request::Told { told } => route(root, env, told, child, out),
                request @ (jig_client::Request::Exchange { .. }
                | jig_client::Request::OpenStream { .. }
                | jig_client::Request::CloseStream
                | jig_client::Request::Save { .. }
                | jig_client::Request::Erase { .. }
                | jig_client::Request::Down { .. }
                | jig_client::Request::SignOut
                | jig_client::Request::ServiceCredential { .. }) => out.push(Request::Client { request }),
            }
        } else if let Some(request) = child.pop() {
            match request {
                runner::Out::Client { event } => root.client.step(env.now, env.wall, event, client),
                runner::Out::Ready => out.push(Request::Ready),
                runner::Out::Kept { line } => out.push(Request::Kept { line }),
                runner::Out::Printed { number, printed } => out.push(Request::Printed { number, printed }),
            }
        } else {
            return;
        }
    }
    assert!(client.is_empty() && child.is_empty(), "bounded routing completes");
}
fn route(root: &mut Root, env: &Env<Limits>, told: Told, child: &mut Queue<runner::Out>, out: &mut Queue<Request>) {
    if let Told::Kept { key, .. } = &told {
        let prefix = u32::try_from(key.number >> 32u32).expect("high half fits");
        if let Some(next) = root.next
            && prefix >= next
        {
            root.next = prefix.checked_add(1);
        }
    }
    let asker = match &told {
        Told::Link { .. } | Told::Dropped { .. } => RUNNER.asker(),
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    let me_changed = match &told {
        Told::Changed { handle } => handle.asker == RUNNER.asker() && handle.number == 0,
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => false,
    };
    match Owner::of(asker) {
        Some(Owner::Runner) => runner::told(&mut root.runner, &root.client, &tools_env(env), told, child),
        Some(Owner::Forge) => {
            match &told {
                Told::Kept { key, request } => runner::told(
                    &mut root.runner,
                    &root.client,
                    &tools_env(env),
                    Told::Kept { key: *key, request: request.clone() },
                    child,
                ),
                Told::Answered { key, .. } => root.runner.forget_kept(*key),
                Told::Link { .. }
                | Told::Pending { .. }
                | Told::Dropped { .. }
                | Told::Changed { .. }
                | Told::Text { .. }
                | Told::Ended { .. }
                | Told::Page { .. } => {}
            }
            crate::forge::told(&mut root.adopter, &env.limits, told, child);
        }
        Some(Owner::Pages) | None => out.push(Request::Fact { fact: temper_client_domain::Fact::UnknownAsker }),
    }
    if me_changed {
        crate::forge::ready(&mut root.adopter, &root.client, &env.limits, child);
    }
}
pub fn fire(root: &mut Root, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut client = Queue::with_capacity(jig_client::max_out(&env.limits.domain.client));
    let mut child = Queue::with_capacity(env.limits.tools.rows.checked_mul(2).expect("tools outputs fit"));
    root.client.fire(env.now, env.wall, &mut client);
    drain(root, env, &mut client, &mut child, out);
    jig::wait::fire(&mut root.runner, &root.client, &tools_env(env), &mut child);
    drain(root, env, &mut client, &mut child, out);
}
#[must_use]
pub fn next_deadline(root: &Root) -> Option<Time> {
    root.client.next_deadline().into_iter().chain(jig::wait::next_deadline(&root.runner)).min()
}

#[cfg(test)]
#[path = "tests/root.rs"]
mod tests;

/// Forge parts use the domain's shared words and whole resource paths.
#[derive(Debug)]
pub struct Parts;
impl jig::render::Parts for Parts {
    fn owner_part(&self, part: &jig_client::contract::OwnerPart, out: &mut jig::render::Text) {
        crate::forge::part(part, out);
    }
}
fn owner_part(part: &jig_client::contract::OwnerPart, out: &mut jig::render::Text) {
    use jig::render::Parts as _;
    Parts.owner_part(part, out);
}

#[cfg(test)]
#[path = "tests/fixtures/root_acts.rs"]
pub(crate) mod act_fixture;
