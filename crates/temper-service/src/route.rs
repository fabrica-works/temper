//! Exhaustive root routing, with forge calls unavailable until its component.
use crate::{engine::Engine, stop};
use alloc::boxed::Box;
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client as client;

pub(crate) fn request(engine: &mut Engine, request: root::Request) {
    match request {
        root::Request::Store(request) => {
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
        root::Request::Worker(_) => stop::unavailable(engine, b"workers"),
        root::Request::Party(_) => stop::unavailable(engine, b"clients"),
    }
}
