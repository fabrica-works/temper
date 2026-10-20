//! One client host shared by the browser's platform parts.
use crate::{limits, records};
use jig_client_protocol_http::Heard as Above;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{Env, List, Queue, Secret, Time, Token, Wall};
use skein_ui::{Act, NodeId, Patch};
use std::collections::BTreeMap;
use temper_client_domain::{
    self as root, Action, Address, Event, Place, Request,
    address::{Filter, parse, print},
    forge, pages,
};
use temper_client_view as view;

pub struct Host {
    root: root::Domain,
    view: view::View,
    pub bounds: view::Limits,
    env: Env<root::Limits>,
    out: Queue<Request>,
    address: Address,
    mapping: jig_client_protocol_http::Mapping,
    stream: jig_client_protocol_http::Stream,
    tickets: BTreeMap<u32, Token>,
    next: u32,
    pub show: Option<(Box<[u8]>, bool)>,
}
impl Host {
    pub fn new(seed: u64, now: Time, wall: Wall) -> Self {
        let client = limits::client();
        assert!(limits::worst_case(&client).is_some(), "web capacities are checked");
        let (bounds, words) = limits::view(&client).expect("web view capacities are checked");
        Self {
            root: root::new(&client, seed),
            view: view::View::new(&bounds, &client, words, temper_client_protocol_wire::FORGE),
            bounds,
            env: Env { now, wall, limits: client },
            out: Queue::with_capacity(root::max_out(&client)),
            address: Address::Inbox { filter: Filter::ALL },
            mapping: jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix"),
            stream: jig_client_protocol_http::Stream::new(&limits::stream().expect("stream bounds")),
            tickets: BTreeMap::new(),
            next: 0,
            show: None,
        }
    }
    pub fn start(&mut self, path: &[u8], records: Vec<skein_ui_web::store::Record>, secret: Option<Box<[u8]>>) {
        let mut kept = List::with_capacity(u32::try_from(records.len()).expect("bounded store"));
        for record in records {
            if let Some(record) = records::decode(record) {
                kept.push(record).expect("counted store");
            }
        }
        self.address = self.opened(path);
        self.event(Event::Start { address: self.address.clone(), kept });
        let with = match secret {
            Some(bytes) => jig_client::With::Link(Secret::new(bytes)),
            None => jig_client::With::Transport,
        };
        self.client(jig_client::Event::SignIn { with });
    }
    // The page a path names; a path that prints otherwise is replaced by the printed one.
    fn opened(&mut self, path: &[u8]) -> Address {
        let address = match parse(path) {
            Place::Address(address) => address,
            Place::Missing => Address::Inbox { filter: Filter::ALL },
        };
        let printed = print(&address);
        if *printed != *path {
            self.show = Some((printed, true));
        }
        address
    }
    pub fn went(&mut self, path: &[u8]) {
        self.address = self.opened(path);
        self.event(Event::Went { place: Place::Address(self.address.clone()) });
    }
    pub fn href(&mut self, node: NodeId) -> Option<Box<[u8]>> {
        match view::act(&mut self.view, node, Act::Press)? {
            Action::Pages(pages::Act::Go { address }) => Some(print(&address)),
            Action::Forge(forge::Act::Go { address: forge::Address::Adopt { project } }) => {
                Some(print(&Address::Adopt { project }))
            }
            Action::Pages(_) | Action::Forge(_) => None,
        }
    }
    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    fn event(&mut self, event: Event) {
        root::step(&mut self.root, &self.env, event, &mut self.out);
    }
    fn client(&mut self, event: jig_client::Event) {
        self.event(Event::Client { event });
    }
    pub fn due(&mut self) {
        root::fire(&mut self.root, &self.env, &mut self.out);
    }
    pub fn act(&mut self, node: NodeId, act: Act) {
        if let Some(action) = view::act(&mut self.view, node, act) {
            self.event(Event::Act { action });
        }
    }
    pub fn heard(&mut self, heard: Heard) {
        let mut above = Queue::with_capacity(4096);
        match heard {
            Heard::Answered { exchange, status, body } => {
                if let Some(token) = self.tickets.remove(&exchange) {
                    above.push(jig_client_protocol_http::answered(token, status, body));
                }
            }
            Heard::Failed { exchange } => {
                if let Some(exchange) = self.tickets.remove(&exchange) {
                    above.push(Above::Lost { exchange });
                }
            }
            Heard::StreamOpened => self.stream.opened(200, &mut above),
            Heard::StreamRefused { status } => self.stream.opened(status, &mut above),
            Heard::Piece { bytes } => self.stream.body(&bytes, &mut above),
            Heard::StreamEnded => self.stream.ended(&mut above),
            Heard::Stored { .. } => self.client(jig_client::Event::Stored),
            Heard::NotStored { .. } => self.client(jig_client::Event::NotStored),
        }
        while let Some(heard) = above.pop() {
            self.client(match heard {
                Above::Reply { exchange, message } => jig_client::Event::Reply { exchange, message },
                Above::Lost { exchange } => jig_client::Event::Lost { exchange },
                Above::Streamed { message } => jig_client::Event::Streamed { message },
                Above::StreamGone => jig_client::Event::StreamGone,
            });
        }
    }
    // A step's writes are one durability answer: a first successful write
    // cannot release held requests before a later refused write is heard.
    pub fn carried(&mut self, reports: Vec<skein_ui_web::transport::Carried>, stored: bool) {
        let mut refused = false;
        for report in reports {
            match jig_client_shell_web::adapter::carried(report) {
                Heard::NotStored { .. } => refused = true,
                Heard::Stored { .. } => {}
                report @ (Heard::Answered { .. }
                | Heard::Failed { .. }
                | Heard::StreamOpened
                | Heard::StreamRefused { .. }
                | Heard::Piece { .. }
                | Heard::StreamEnded) => self.heard(report),
            }
        }
        if stored {
            self.client(if refused { jig_client::Event::NotStored } else { jig_client::Event::Stored });
        }
    }
    pub fn requests(&mut self) -> Vec<Asked> {
        let mut asked = Vec::new();
        while let Some(request) = self.out.pop() {
            match request {
                Request::Client { request } => self.carry(request, &mut asked),
                Request::Show { address, replace } => {
                    if address != self.address {
                        let path = print(&address);
                        if path != print(&self.address) {
                            self.show = Some((path, replace));
                        }
                        self.address = address.clone();
                        self.event(Event::Went { place: Place::Address(address) });
                    }
                }
                Request::SignOut => self.client(jig_client::Event::SignOut),
                Request::Dropped { .. } | Request::Fact { .. } => {}
            }
        }
        root::take(&mut self.root, &self.env, &mut self.out);
        asked
    }
    fn carry(&mut self, request: jig_client::Request, asked: &mut Vec<Asked>) {
        match request {
            jig_client::Request::Save { records } => {
                for record in records.into_boxed().into_vec() {
                    let (key, value) = records::encode(record);
                    asked.push(Asked::Save { key, value });
                }
            }
            jig_client::Request::Erase { records } => {
                for key in records.into_boxed().into_vec() {
                    asked.push(Asked::Erase { key });
                }
            }
            jig_client::Request::Exchange { exchange, credential, message } => {
                self.next = self.next.checked_add(1).expect("fetch tickets fit");
                self.tickets.insert(self.next, exchange);
                asked.push(Asked::Exchange {
                    id: self.next,
                    call: jig_client_protocol_http::exchange(&self.mapping, credential.as_ref(), message),
                    bound: limits::BODY_BYTES,
                });
            }
            jig_client::Request::OpenStream { credential } => {
                self.stream = jig_client_protocol_http::Stream::new(&limits::stream().expect("stream bounds"));
                asked.push(Asked::OpenStream {
                    call: jig_client_protocol_http::stream(&self.mapping, credential.as_ref()),
                    piece: limits::PIECE,
                });
            }
            jig_client::Request::CloseStream => asked.push(Asked::CloseStream),
            jig_client::Request::Told { .. }
            | jig_client::Request::Down { .. }
            | jig_client::Request::SignOut
            | jig_client::Request::ServiceCredential { .. } => {}
        }
    }
    pub fn build(&mut self, patches: &mut Queue<Patch>) {
        view::build(&mut self.view, &self.root, patches);
    }
    pub fn deadline(&self) -> Option<Time> {
        root::next_deadline(&self.root)
    }
}
