mod chats;
mod inbox;
use super::fixtures;
use crate::{
    Action, Address,
    pages::{self, Act, Event, Pages, Request},
};
use jig_client::{Link, Told};
use skein_lib::Queue;
const PAGES: jig_client::Asker = jig_client::Asker(2);

#[test]
fn the_pages_watch_me_from_start_under_their_own_asker() {
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(2);
    pages::step(&mut pages, &fixtures::env(&fixtures::limits()), root.client(), Event::Started, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: jig_client::Handle { asker: PAGES, number: 0 },
            subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn an_asker_naming_pages_reaches_the_pages_child() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let before = pages::shown(root.pages());
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        Told::Changed { handle: pages::me_handle() },
        &mut Queue::with_capacity(crate::max_out(&limits)),
    );
    assert_eq!(pages::shown(root.pages()), before + 1);
}
#[test]
fn sign_out_gives_the_client_one_sign_out() {
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(2);
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        root.client(),
        Event::Act { act: Act::SignOut },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::SignOut)));
    assert!(out.is_empty());
}
#[test]
fn a_client_started_again_opens_at_the_inbox() {
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(2);
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        root.client(),
        Event::Went { address: Address::Chats },
        &mut out,
    );
    pages::step(&mut pages, &fixtures::env(&fixtures::limits()), root.client(), Event::Started, &mut out);
    assert_eq!(pages.address(), &Address::Inbox { filter: crate::address::Filter::ALL });
}
#[test]
fn a_link_state_change_prints_the_page_again() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let before = crate::shown(&root);
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        Told::Link { link: Link::Offline },
        &mut Queue::with_capacity(crate::max_out(&limits)),
    );
    assert!(crate::shown(&root) > before);
}
#[test]
fn the_pages_sign_out_act_reaches_the_shared_client() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        crate::Event::Act { action: Action::Pages(Act::SignOut) },
        &mut out,
    );
    assert_eq!(root.client().link().state, Link::SignedOut);
}

#[test]
fn leaving_a_page_unwatches_its_handles_and_drops_their_tells() {
    const LEFT: jig_client::Handle = jig_client::Handle { asker: PAGES, number: 1 };
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, skein_lib::List::with_capacity(0));
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    fixtures::confirm(&mut root, watch, fixtures::PARTY, &mut out);
    while out.pop().is_some() {}
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        crate::Event::Went { place: crate::Place::Address(Address::Adopt { project: fixtures::PROJECT }) },
        &mut out,
    );
    let mut closes = 0_u32;
    while let Some(request) = out.pop() {
        if let crate::Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Close { .. } } } =
            request
        {
            closes += 1;
        }
    }
    assert!(closes > 0, "the reference inbox's watch is closed on leaving");
    let before = pages::shown(root.pages());
    crate::route::told(&mut root, &fixtures::env(&limits), Told::Changed { handle: LEFT }, &mut out);
    assert_eq!(pages::shown(root.pages()), before);
}
