//! Direct root fixture shared by focused routes and the ending sweep.

use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use std::collections::VecDeque;
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Write};

use crate::protocol::Protocol;
use crate::store::Store;
use crate::walking::{config, limits};

pub struct Driver {
    pub root: engine::Domain,
    pub env: Env<engine::Limits>,
    pub out: Queue<engine::Request>,
    pub events: VecDeque<engine::Event>,
    pub store: Store,
    pub delivered: Vec<Delivery>,
    /// The protocol layer this world plays: decodes, renders, outcomes, accounts.
    pub protocol: Protocol,
    pub serial: u64,
    pub stopped: bool,
    pub fail_archive_once: bool,
    pub transactions: Vec<Vec<Write>>,
    pub host_requests: Vec<Box<engine::HostRequest>>,
    /// Inputs `try_step` handed back while the journal was full.
    pub returned: Vec<engine::Event>,
    /// Other outputs without a durability dependency.
    pub immediate: Vec<engine::Request>,
}

impl Driver {
    #[must_use]
    pub fn new(store: Store) -> Driver {
        Driver::configured(store, config(91), &limits())
    }

    #[must_use]
    pub fn configured(store: Store, config: engine::Config, limits: &engine::Limits) -> Driver {
        Driver {
            root: engine::Domain::new(config, limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            out: Queue::with_capacity(engine::max_out(limits)),
            events: VecDeque::from([engine::Event::Restart]),
            store,
            delivered: Vec::new(),
            protocol: Protocol::default(),
            serial: 100,
            stopped: false,
            fail_archive_once: false,
            transactions: Vec::new(),
            host_requests: Vec::new(),
            returned: Vec::new(),
            immediate: Vec::new(),
        }
    }

    /// Offer one input through admission; an input the journal cannot take
    /// is handed back to the world, which keeps it (jig's `domain/root.md`, 3).
    pub fn send(&mut self, event: engine::Event) {
        if let Err(event) = engine::try_step(&mut self.root, &self.env, event) {
            self.returned.push(event);
        }
        engine::release(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.root.reclaim();
    }

    pub fn collect(&mut self) {
        for _ in 0..self.out.len() {
            let request = match self.protocol.play(self.out.pop().expect("output count")) {
                Ok(events) => {
                    self.events.extend(events);
                    continue;
                }
                Err(request) => request,
            };
            match request {
                engine::Request::ToChild(released) => self.events.push_back(engine::Event::Released(released)),
                engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                    self.transactions.push(writes.to_vec());
                    self.store.pending.push_back((number, writes));
                }
                engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, bytes }) => {
                    let (rows, next) = self.store.page(&range, after.as_ref(), most);
                    assert!(rows.len() <= usize::try_from(most).expect("small count"));
                    assert!(bytes >= self.env.limits.loads.bytes);
                    if self.fail_archive_once
                        && matches!(
                            range,
                            temper_engine_domain::Range::Core(
                                temper_engine_domain::CoreRange::EscalationDecision { .. }
                            )
                        )
                    {
                        self.fail_archive_once = false;
                        self.events.push_back(engine::Event::Store(engine::StoreInput::Unloaded { owner }));
                    } else {
                        self.events.push_back(engine::Event::Store(engine::StoreInput::Loaded { owner, rows, next }));
                    }
                }
                engine::Request::Worker(engine::WorkerRequest::Deliver(delivery))
                | engine::Request::Party(engine::PartyRequest::Deliver(delivery)) => self.delivered.push(delivery),
                engine::Request::Forge { .. } => {
                    panic!("root route fixture did not adopt forge")
                }
                engine::Request::Stop => self.stopped = true,
                engine::Request::Worker(engine::WorkerRequest::Host(request)) => self.host_requests.push(request),
                other @ (engine::Request::Party(_)
                | engine::Request::Worker(_)
                | engine::Request::Store(_)
                | engine::Request::Account(_)
                | engine::Request::Busy(_)
                | engine::Request::Drained) => self.immediate.push(other),
            }
        }
    }

    pub fn advance(&mut self, apply: bool) {
        if let Some(event) = self.events.pop_front() {
            self.send(event);
        }
        if apply && !self.store.pending.is_empty() {
            let number = self.store.apply();
            self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
        crate::poll(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.root.reclaim();
    }

    pub fn settle(&mut self) {
        for _ in 0..200 {
            self.advance(true);
            if self.root.quiescent() && self.events.is_empty() && self.store.pending.is_empty() {
                return;
            }
        }
        panic!("direct root routes did not settle: {:?}", self.root);
    }

    /// The deployment's first period for project one, at its first start.
    pub fn period(&mut self) {
        self.send(engine::Event::Party(engine::PartyInput::Period { project: 1, period: 1, budget: 1000, next: None }));
    }

    pub fn sign_in(&mut self) {
        self.serial += 1;
        self.send(engine::Event::Party(engine::PartyInput::SignedIn {
            reply_to: ReplyTo::new(Token::new(self.serial)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        }));
    }

    #[must_use]
    pub fn session(&self) -> u64 {
        self.delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::WebReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } = delivery {
                    Some(*sign_in)
                } else {
                    None
                }
            })
            .expect("durable sign-in reply")
    }

    // Leave the newly durable callback behind three later unanswered commits.
    pub fn pressure(&mut self) {
        assert_eq!(self.store.pending.len(), 1);
        self.sign_in();
        self.sign_in();
        assert_eq!(self.store.pending.len(), 3);
        let number = self.store.apply();
        self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        self.sign_in();
        assert_eq!(self.store.pending.len(), 3);
        for _ in 0..20 {
            self.advance(false);
        }
        assert!(!self.root.quiescent(), "owed callback and store terminals prevent completion");
        assert_eq!(self.store.pending.len(), 3, "no callback write entered the full journal");
    }
}
