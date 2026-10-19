//! One client host: temper's root and view, stepped by the page's platform facts.
use crate::limits;
use skein_lib::{Env, List, Queue, Secret, Time, Wall};
use skein_ui::{Act, NodeId, Patch};
use temper_client_domain::{self as root, Address, Event, Place, Request, address::Filter};
use temper_client_view as view;

pub struct Host {
    root: root::Domain,
    view: view::View,
    pub bounds: view::Limits,
    env: Env<root::Limits>,
    out: Queue<Request>,
    address: Address,
}
impl Host {
    pub fn new(seed: u64, now: Time, wall: Wall) -> Self {
        let client = limits::client();
        let (bounds, words) = limits::view(&client).expect("web view capacities are checked");
        Self {
            root: root::new(&client, seed),
            view: view::View::new(&bounds, &client, words, temper_client_protocol_wire::FORGE),
            bounds,
            env: Env { now, wall, limits: client },
            out: Queue::with_capacity(root::max_out(&client)),
            address: Address::Inbox { filter: Filter::ALL },
        }
    }
    pub fn start(&mut self, path: &[u8], secret: Option<Box<[u8]>>) {
        self.address = match root::address::parse(path) {
            Place::Address(address) => address,
            Place::Missing => Address::Inbox { filter: Filter::ALL },
        };
        let address = self.address.clone();
        self.event(Event::Start { address, kept: List::with_capacity(self.env.limits.client.domain.records) });
        let with = match secret {
            Some(bytes) => jig_client::With::Link(Secret::new(bytes)),
            None => jig_client::With::Transport,
        };
        self.event(Event::Client { event: jig_client::Event::SignIn { with } });
    }
    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    fn event(&mut self, event: Event) {
        root::step(&mut self.root, &self.env, event, &mut self.out);
    }
    pub fn act(&mut self, node: NodeId, act: Act) {
        if let Some(action) = view::act(&mut self.view, node, act) {
            self.event(Event::Act { action });
        }
    }
    /// The root's requests: 02.3 sends the client's to the server.
    pub fn requests(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Show { address, .. } => {
                    if address != self.address {
                        self.address = address.clone();
                        self.event(Event::Went { place: Place::Address(address) });
                    }
                }
                Request::SignOut => self.event(Event::Client { event: jig_client::Event::SignOut }),
                Request::Client { .. } | Request::Dropped { .. } | Request::Fact { .. } => {}
            }
        }
        root::take(&mut self.root, &self.env, &mut self.out);
    }
    pub fn build(&mut self, patches: &mut Queue<Patch>) {
        view::build(&mut self.view, &self.root, patches);
    }
}
