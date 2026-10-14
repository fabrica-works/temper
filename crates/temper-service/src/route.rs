//! Exhaustive root routing, with forge calls unavailable until its component.
use crate::{engine::Engine, stop};
use alloc::boxed::Box;
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client as client;

pub(crate) fn request(engine: &mut Engine, request: root::Request) {
    match request {
        root::Request::Store(request) => {
            if let root::StoreRequest::Commit { number, writes } = &request {
                engine.clients.commit(*number, writes);
            }
            if !engine.store.down(request) {
                stop::unavailable(engine, b"deployment");
            }
        }
        root::Request::Account(request) => {
            if let Some(accounts) = &mut engine.accounts {
                accounts.request(request);
            }
        }
        root::Request::Forge { call, .. } => {
            engine.events.push(root::Event::Forge(Box::new(forge::Event::Client(client::Event::Answered {
                call,
                cost: 0,
                result: Err(client::api::Error::Unavailable),
            }))));
        }
        root::Request::ToChild(released) => engine.events.push(root::Event::Released(released)),
        root::Request::Busy(event) => {
            if let Some(drain) = &mut engine.drain {
                drain.busy.push(event);
            } else {
                engine.events.push(root::Event::Forge(event));
            }
        }
        root::Request::Drained => stop::drained(engine),
        root::Request::Stop => stop::unavailable(engine, b"deployment"),
        root::Request::Worker(request) => worker(engine, request),
        root::Request::Party(request) => {
            if let root::PartyRequest::WatchRefused { watcher, .. } = &request
                && engine.admission == Some(*watcher)
            {
                engine.admission = None;
            }
            let event = match request {
                root::PartyRequest::Deliver(root::Delivery::Application { to, outcome }) => {
                    match crate::family::answer(
                        outcome,
                        &temper_protocol_client_wire::v1::CEILINGS,
                        &engine.clients.env.limits.wire,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Answered {
                                to,
                                family: temper_protocol_client_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                request @ (root::PartyRequest::SignInRefused { .. }
                | root::PartyRequest::WatchRefused { .. }
                | root::PartyRequest::RunText { .. }
                | root::PartyRequest::Deliver(_)) => crate::clients::translation::to_component(request),
            };
            if let Some(event) = event {
                engine.clients.down(event);
            }
        }
    }
}

fn worker(engine: &mut Engine, request: root::WorkerRequest) {
    match request {
        root::WorkerRequest::Host(request) => match *request {
            root::HostRequest::Decode { to, task, attempt, call } => {
                engine.events.push(crate::tools::decode(to, task, attempt, &call));
            }
            root::HostRequest::Dropped { .. } => {}
        },
        root::WorkerRequest::ReadResult { payload, outcome } => {
            engine.events.push(crate::tools::result(payload, &outcome));
        }
        root::WorkerRequest::Deliver(delivery) => match delivery {
            root::Delivery::Assigned { channel, assignment } => {
                engine.workers.assignment(engine.env.now, channel, assignment, &engine.values, &mut engine.events);
            }
            root::Delivery::Link { channel, down } => {
                engine.workers.send(engine.env.now, channel, down, &engine.values, &mut engine.events);
            }
            root::Delivery::TurnBusy { channel, task, attempt, turn } => engine.workers.down(
                engine.env.now,
                jig_protocol_engine_workers::Event::TurnBusy {
                    channel,
                    run: skein_lib::Token::new(task),
                    attempt: skein_lib::Token::new(attempt),
                    turn,
                },
                &mut engine.events,
            ),
            root::Delivery::Refuse { channel } => engine.workers.down(
                engine.env.now,
                jig_protocol_engine_workers::Event::NotAdmitted { channel, text: b"not admitted".as_slice().into() },
                &mut engine.events,
            ),
            root::Delivery::Host(delivery) => match *delivery {
                root::HostDelivery::Render { to, answer, .. } => engine.events.push(crate::tools::render(to, &answer)),
                root::HostDelivery::Answer { channel, task, attempt, name, call } => engine.workers.send(
                    engine.env.now,
                    channel,
                    jig_domain_host::LinkDown::Relayed {
                        run: skein_lib::Token::new(task),
                        attempt: skein_lib::Token::new(attempt),
                        call: name,
                        answer: crate::tools::settled(call.answer),
                    },
                    &engine.values,
                    &mut engine.events,
                ),
            },
            root::Delivery::Procedure { .. }
            | root::Delivery::Application { .. }
            | root::Delivery::WebReply { .. }
            | root::Delivery::Read { .. }
            | root::Delivery::Document { .. }
            | root::Delivery::WatchEnded { .. }
            | root::Delivery::Result { .. }
            | root::Delivery::ForgeCommitted { .. }
            | root::Delivery::ForgeCall { .. }
            | root::Delivery::Relay { .. }
            | root::Delivery::ReadEscalationDecision { .. }
            | root::Delivery::Fleet(_)
            | root::Delivery::ReadLoad { .. }
            | root::Delivery::Load { .. }
            | root::Delivery::Drained => stop::unavailable(engine, b"workers"),
        },
    }
}
