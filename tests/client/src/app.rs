//! The shell adapter observes requests and trees, never a domain's decisions.
use jig_client_world::{
    harness::{App, CardKey, Duty, Receipt, Seen, Sent, StoreWrite},
    observe,
};
use jig_domain_client::{
    boundary::Offset,
    contract::{Answer, Made, Outcome, ReadNone, ReadResult, WatchEnd},
};
use jig_view::{Act, NodeId, Patch, Tree};
use skein_lib::{Env, List, Queue, RequestEnvelope, Time};
use temper_client_domain::{self as domain, Action, Address, Event, JigOut, Request};
use temper_client_view as view;

use crate::fixtures;

/// temper's root and view on jig's common client world kit.
pub struct Temper;
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
    type Record = domain::store::Record;
    type Key = domain::store::Key;
    type Address = Address;
    fn limits() -> Self::Limits {
        limits()
    }
    fn new(seed: u64) -> (Self::Root, Self::View) {
        let limits = limits();
        let (jig, forge) = fixtures::phrasebooks(&limits);
        let bounds = fixtures::view_bounds(&limits, &jig, &forge);
        let mut root = domain::new(&limits, seed, domain::forge::Config { connector: fixtures::FORGE, version: 1 });
        domain::configure_navigation(&mut root, domain::navigation(&limits)).expect("navigation fits");
        (root, view::View::new(&bounds, &limits, jig, forge, fixtures::FORGE))
    }
    fn start(address: Address, kept: Vec<Self::Record>, offset: i32) -> Event {
        let mut records = List::with_capacity(u32::try_from(kept.len()).expect("small store count"));
        for record in kept {
            records.push(record).expect("kept count measured");
        }
        Event::Start { address, kept: records, offset: Offset { minutes: offset } }
    }
    fn went(address: Address) -> Event {
        Event::Went { place: domain::Place::Address(address) }
    }
    fn step(root: &mut Self::Root, env: &Env<Self::Limits>, event: Event, out: &mut Queue<Request>) {
        domain::step(root, env, event, out);
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
        view::act(view, node, act).map(|action| Event::Act { action })
    }
    fn duty(request: Request) -> Duty<Self> {
        match request {
            Request::Save { record } => Duty::Store(StoreWrite::Save(record)),
            Request::Erase { key } => Duty::Store(StoreWrite::Erase(key)),
            Request::Show { address, replace } => Duty::Show { address, replace },
            Request::SignIn { provider, back } => Duty::SignIn { provider, back },
            Request::SignOut => Duty::SignOut,
            Request::Send { .. }
            | Request::Jig(
                JigOut::Read { .. } | JigOut::Open { .. } | JigOut::Continue { .. } | JigOut::Close { .. },
            ) => Duty::Wire(request),
            Request::Fact { .. } | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. }) => Duty::Fact,
        }
    }
    fn record_key(record: &Self::Record) -> Self::Key {
        match record {
            domain::store::Record::Jig(record) => domain::store::Key::Jig(record.key()),
            domain::store::Record::Table { record, .. } => domain::store::Key::Table(record.key),
        }
    }
    fn unsent(request: Request) -> Option<Event> {
        match request {
            Request::Send { attempt, .. } => {
                Some(Event::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None })
            }
            Request::Jig(JigOut::Read { read, .. }) => {
                Some(Event::Jig(jig_domain_client::Event::Read { read, result: ReadResult::None(ReadNone::Busy) }))
            }
            Request::Jig(JigOut::Open { watch, .. } | JigOut::Continue { watch, .. } | JigOut::Close { watch }) => {
                Some(Event::Jig(jig_domain_client::Event::Ended { watch, end: WatchEnd::Dropped }))
            }
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::SignIn { .. }
            | Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. }) => None,
        }
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
        match event {
            Event::Act { action: Action::Jig(act) } | Event::Jig(jig_domain_client::Event::Act { act }) => {
                observe::jig_window_changed(act)
            }
            Event::Start { .. }
            | Event::Went { .. }
            | Event::Answer { .. }
            | Event::Jig(_)
            | Event::Forge(_)
            | Event::SignedOut => false,
        }
    }
    fn seen(tree: &Tree) -> Vec<Seen> {
        observe::jig_seen(tree)
    }
    fn sent(request: &Request) -> Option<Sent> {
        let Request::Send { attempt, key, ask } = request else { return None };
        match ask {
            domain::Ask::Forge(_) => None,
            domain::Ask::Jig(ask) => Some(observe::jig_sent(*attempt, *key, ask)),
        }
    }
    fn received(event: &Event) -> Vec<Receipt> {
        match event {
            Event::Jig(event) => observe::jig_received(event),
            Event::Answer { attempt, envelope, answer } => {
                if *envelope == RequestEnvelope::SignedOut {
                    return vec![Receipt::Parked];
                }
                let words = match answer {
                    Some(domain::Answered::Jig(Answer::Done(Outcome::Made {
                        what: Made::Words { task, message },
                    }))) => Some((*task, *message)),
                    Some(domain::Answered::Jig(_) | domain::Answered::Forge(_)) | None => None,
                };
                vec![Receipt::Answer { attempt: *attempt, final_answer: *envelope == RequestEnvelope::Final, words }]
            }
            Event::SignedOut => vec![Receipt::Parked],
            Event::Start { .. } | Event::Went { .. } | Event::Act { .. } | Event::Forge(_) => vec![],
        }
    }
    fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
        observe::jig_card(tree, node)
    }
}
