//! The shell adapter observes requests and trees, never a domain's decisions.
use jig_client_view::boundary::Offset;
use jig_client_world::{
    harness::{App, CardKey, Duty, Receipt, Seen, Sent, StoreWrite},
    observe,
};
use skein_lib::{Env, List, Queue, RequestEnvelope, Time};
use skein_ui::{Act, NodeId, Patch, Tree};
use temper_client_domain::{self as domain, Action, Address, JigOut, Request};
use temper_client_view as view;

use crate::fixtures;

/// temper's root and view on jig's common client world kit.
pub struct Temper;
/// The shell world's Start carries its unlinked client's own typed records.
#[derive(Debug)]
pub enum Event {
    Root(domain::Event),
    Start { address: Address, kept: List<skein_lib::RequestRecord<jig_client::Held>>, offset: Offset },
}
impl From<domain::Event> for Event {
    fn from(event: domain::Event) -> Self {
        Self::Root(event)
    }
}
impl Event {
    #[must_use]
    pub fn root(&self) -> Option<&domain::Event> {
        match self {
            Self::Root(event) => Some(event),
            Self::Start { .. } => None,
        }
    }
}

/// The same small story capacities as ops's jig profile.
#[must_use]
pub fn limits() -> domain::Limits {
    fixtures::domain_limits()
}
impl App for Temper {
    type Root = domain::Domain;
    type View = view::View;
    type Limits = domain::Limits;
    type Event = Event;
    type Request = Request;
    type Record = skein_lib::RequestRecord<jig_client::Held>;
    type Key = [u8; 16];
    type Address = Address;
    fn limits() -> Self::Limits {
        limits()
    }
    fn new(seed: u64) -> (Self::Root, Self::View) {
        let limits = limits();
        let (jig, forge) = fixtures::phrasebooks(&limits);
        let bounds = fixtures::view_bounds(&limits, &jig, &forge);
        let mut root = domain::unlinked(&limits, seed);
        domain::configure_navigation(&mut root, domain::navigation(&limits)).expect("navigation fits");
        (root, view::View::new(&bounds, &limits, jig, forge, fixtures::FORGE))
    }
    fn view_limits(view: &Self::View) -> skein_ui::Limits {
        view.limits().tree
    }
    fn start(address: Address, kept: Vec<Self::Record>, offset: i32) -> Event {
        let mut records = List::with_capacity(u32::try_from(kept.len()).expect("small store count"));
        for record in kept {
            records.push(record).expect("kept count measured");
        }
        Event::Start { address, kept: records, offset: Offset { minutes: offset } }
    }
    fn went(address: Address) -> Event {
        domain::Event::Went { place: domain::Place::Address(address) }.into()
    }
    fn step(root: &mut Self::Root, env: &Env<Self::Limits>, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Root(event) => domain::step(root, env, event, out),
            Event::Start { address, kept, offset } => {
                domain::step(
                    root,
                    env,
                    domain::Event::Client { event: jig_client::Event::StartUnlinked { kept } },
                    out,
                );
                domain::step(
                    root,
                    env,
                    domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } },
                    out,
                );
                domain::step(root, env, domain::Event::Start { address, kept: List::with_capacity(0), offset }, out);
            }
        }
    }
    fn fire(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Request>) {
        domain::fire(root, env, out);
    }
    fn take(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Request>) {
        domain::take(root, env, out);
    }
    fn next_deadline(root: &Self::Root) -> Option<Time> {
        domain::next_deadline(root)
    }
    fn build(view: &mut Self::View, root: &Self::Root, patches: &mut Queue<Patch>) {
        view::build(view, root, patches);
    }
    fn tree(view: &Self::View) -> &Tree {
        view.tree()
    }
    fn act(view: &mut Self::View, node: NodeId, act: Act) -> Option<Event> {
        view::act(view, node, act).map(|action| domain::Event::Act { action }.into())
    }
    fn duty(request: Request) -> Duty<Self> {
        match request {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Save { record } } } => {
                Duty::Store(StoreWrite::Save(record))
            }
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Erase { key } } } => {
                Duty::Store(StoreWrite::Erase(key))
            }
            Request::Client { request: jig_client::Request::SignOut } | Request::SignOut => Duty::SignOut,
            Request::Client { .. } => Duty::Wire(request),
            Request::Show { address, replace } => Duty::Show { address, replace },
            Request::SignIn { provider, back } => Duty::SignIn { provider, back },
            Request::Fact { .. } | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. }) => Duty::Fact,
        }
    }
    fn record_key(record: &Self::Record) -> Self::Key {
        record.key
    }
    fn unsent(request: Request) -> Option<Event> {
        let event = match request {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                jig_client::Event::Up {
                    up: jig_client::Up::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None, why: None },
                }
            }
            Request::Client { request: jig_client::Request::Exchange { exchange, .. } } => {
                jig_client::Event::Lost { exchange }
            }
            Request::Client { request: jig_client::Request::OpenStream { .. } } => jig_client::Event::StreamGone,
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            jig_client::Down::Open { watch, .. }
                            | jig_client::Down::Close { watch }
                            | jig_client::Down::Continue { watch, .. },
                    },
            } => jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Read { read, .. } } } => {
                jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Limit } }
            }
            Request::Client {
                request:
                    jig_client::Request::Save { .. }
                    | jig_client::Request::Erase { .. }
                    | jig_client::Request::CloseStream
                    | jig_client::Request::Told { .. }
                    | jig_client::Request::SignOut
                    | jig_client::Request::Down { down: jig_client::Down::Save { .. } | jig_client::Down::Erase { .. } },
            }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut
            | Request::Fact { .. }
            | Request::Jig(_) => return None,
        };
        Some(domain::Event::Client { event }.into())
    }
    fn max_out(limits: &Self::Limits) -> u32 {
        domain::max_out(limits)
    }
    fn patch_limit(view: &Self::View) -> u32 {
        view.limits().tree.patches
    }
    fn worst_case(limits: &Self::Limits) -> u64 {
        let (jig, forge) = fixtures::phrasebooks(limits);
        let bounds = fixtures::view_bounds(limits, &jig, &forge);
        domain::worst_case(limits).expect("root bound")
            + 2 * view::worst_case(&bounds, limits, &jig, &forge).expect("shell model projection bound")
    }
    fn window_changed(event: &Event) -> bool {
        match event.root() {
            Some(
                domain::Event::Act { action: Action::Jig(act) }
                | domain::Event::Jig(jig_client_view::Event::Act { act }),
            ) => observe::jig_window_changed(act),
            Some(domain::Event::Act { action: Action::Forge(_) }) => true,
            Some(
                domain::Event::Start { .. }
                | domain::Event::Went { .. }
                | domain::Event::Client { .. }
                | domain::Event::Jig(_)
                | domain::Event::Forge(_)
                | domain::Event::SignedOut,
            )
            | None => false,
        }
    }
    fn seen(tree: &Tree) -> Vec<Seen> {
        let mut seen = observe::jig_seen(tree);
        if adoption_page(tree) {
            let mut state = jig_client_world::harness::CardSeen::Waiting;
            for node in tree.nodes() {
                if node.role == skein_ui::Role::Button
                    && node.name.as_deref() == Some(b"Adopt")
                    && node.states.as_slice().contains(&skein_ui::State::Disabled)
                {
                    state = jig_client_world::harness::CardSeen::Deciding;
                }
                if node.role == skein_ui::Role::Alert
                    || node.role == skein_ui::Role::Status
                        && node.name.as_deref().is_some_and(|name| {
                            name.starts_with(b"Adopted ") || name.starts_with(b"It is not known whether ")
                        })
                {
                    state = jig_client_world::harness::CardSeen::Done;
                }
            }
            seen.push(Seen::Card { key: adoption_card(), state, revision: None });
        }
        seen
    }
    fn sent(request: &Request) -> Option<Sent> {
        let Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, key, request } },
        } = request
        else {
            return None;
        };
        Some(match request {
            jig_client::Ask::Family(_) => {
                Sent { attempt: *attempt, key: *key, card: Some(adoption_card()), revision: None, words: None }
            }
            jig_client::Ask::Jig(ask) => observe::jig_sent(*attempt, *key, ask),
        })
    }
    fn received(event: &Event) -> Vec<Receipt> {
        let Some(event) = event.root() else { return vec![Receipt::Started] };
        match event {
            domain::Event::Jig(event) => observe::jig_received(event),
            domain::Event::Client { event: jig_client::Event::Up { up } } => {
                let jig = match up {
                    jig_client::Up::Served { served, .. } => Some(jig_client_view::Event::Hello { served: *served }),
                    jig_client::Up::Watched { watch, event } => {
                        Some(jig_client_view::Event::Watch { watch: *watch, event: event.clone() })
                    }
                    jig_client::Up::WatchEnded { watch, end } => {
                        Some(jig_client_view::Event::Ended { watch: *watch, end: *end })
                    }
                    jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(result) } => {
                        Some(jig_client_view::Event::Read { read: *read, result: result.clone() })
                    }
                    jig_client::Up::SignedOut => return vec![Receipt::Parked],
                    jig_client::Up::SignedIn
                    | jig_client::Up::NotKept { .. }
                    | jig_client::Up::FamilyServed { .. }
                    | jig_client::Up::Answer { .. }
                    | jig_client::Up::FamilyDelivered { .. }
                    | jig_client::Up::ReadDone { .. } => None,
                };
                if let Some(event) = jig {
                    return observe::jig_received(&event);
                }
                observe::received(&jig_client_test_domain::Event::Client {
                    event: jig_client::Event::Up { up: up.clone() },
                })
            }
            domain::Event::SignedOut => vec![Receipt::Parked],
            domain::Event::Start { .. }
            | domain::Event::Went { .. }
            | domain::Event::Act { .. }
            | domain::Event::Forge(_)
            | domain::Event::Client { .. } => vec![],
        }
    }
    fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
        if adoption_page(tree)
            && tree.nodes().iter().any(|value| {
                value.id == node
                    && value.binding.is_some_and(|binding| {
                        binding.owner == (jig_client_view::Owner::Application { number: fixtures::FORGE }).opaque()
                    })
            })
        {
            Some(adoption_card())
        } else {
            observe::jig_card(tree, node)
        }
    }
}
fn adoption_card() -> CardKey {
    CardKey(b"forge/adoption".to_vec())
}
fn adoption_page(tree: &Tree) -> bool {
    tree.nodes().iter().any(|node| {
        node.role == skein_ui::Role::Heading { level: 1 }
            && node.text.as_slice().iter().any(|run| run.text.starts_with(b"Adopt a repository into "))
    })
}
