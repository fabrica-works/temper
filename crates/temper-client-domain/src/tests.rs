mod acts;
mod fixtures;
mod form;
mod pages;
use crate::{Address, Event, Fact, Request, forge};
use jig_client::contract::{self, Part, PartScope};
use skein_lib::{List, Queue};

#[test]
fn the_forge_address_goes_to_the_forge_side() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    fixtures::start(&mut root, List::with_capacity(0));
    let address = forge::Address::Adopt { project: fixtures::PROJECT };
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Went {
            place: crate::Place::Address(Address::Adopt {
                project: match address {
                    forge::Address::Adopt { project } => project,
                },
            }),
        },
        &mut out,
    );
    assert_eq!(root.forge().address(), Some(address));
    crate::step(&mut root, &fixtures::env(&limits), Event::Went { place: crate::Place::Missing }, &mut out);
    assert_eq!(root.forge().address(), None, "leaving drops the forge page");
}
#[test]
fn a_family_hello_outside_the_version_makes_the_side_out_of_date() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    for (name, versions, expected) in [
        ("served", contract::VersionRange { first: 1, last: 1 }, forge::Served::Yes),
        ("newer", contract::VersionRange { first: 2, last: 2 }, forge::Served::OutOfDate),
        ("inverted", contract::VersionRange { first: 2, last: 1 }, forge::Served::OutOfDate),
    ] {
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyServed { family: fixtures::FORGE, versions: Some(versions) },
                },
            },
            &mut out,
        );
        assert_eq!(root.forge().served(), expected, "{name}");
    }
}

#[test]
fn a_part_of_the_forges_number_is_held_by_its_side() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let forge = fixtures::FORGE;
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    let part = Part::Connector { connector: forge };
    crate::parts::show(&mut root, &env, part.clone(), PartScope::Project(fixtures::PROJECT), &mut out);
    assert!(root.forge().contains(&part), "forge {forge}");
    assert!(out.is_empty(), "forge {forge} reads no document");
}
#[test]
fn a_part_of_another_connector_is_reported_unknown() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    let part = Part::Connector { connector: fixtures::OTHER };
    crate::parts::show(&mut root, &env, part.clone(), PartScope::Project(fixtures::PROJECT), &mut out);
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::UnknownConnector }));
    assert!(out.is_empty(), "an unknown connector's part asks for nothing more");
    assert!(!root.forge().contains(&part), "the forge's side holds no other connector's part");
}
#[test]
fn parts_left_forget_the_parts_of_that_generation() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::parts::show(&mut root, &env, fixtures::resource(), PartScope::Deployment, &mut out);
    let mut child = Queue::with_capacity(1);
    child.push(jig_client_view::Request::PartsLeft);
    crate::route::route_jig(&mut root, &env, &mut child, &mut out);
    assert!(!root.forge().contains(&fixtures::resource()));
    assert!(out.is_empty());
}
#[test]
fn a_part_past_the_sides_bound_is_reported_parts_full() {
    let mut limits = fixtures::limits();
    limits.forge.parts = 1;
    let env = fixtures::env(&limits);
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::parts::show(&mut root, &env, fixtures::connector(), PartScope::Deployment, &mut out);
    crate::parts::show(&mut root, &env, fixtures::connector(), PartScope::Deployment, &mut out);
    crate::parts::show(&mut root, &env, fixtures::resource(), PartScope::Deployment, &mut out);
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::PartsFull }));
    assert!(out.is_empty());
    assert!(!root.forge().contains(&fixtures::resource()));
    assert_eq!(forge::shown(root.forge()), 1);
}

#[test]
fn the_reference_uis_records_never_reach_the_small_store() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let record = jig_client_view::store::Record::Place {
        version: jig_client_view::store::STORE_VERSION,
        party: 9,
        address: jig_client_view::address::Address::Inbox { project: None },
    };
    let mut keys = List::with_capacity(1);
    keys.push(record.key()).expect("one key");
    let mut records = List::with_capacity(1);
    records.push(record).expect("one record");
    for request in [jig_client_view::Request::Erase { records: keys }, jig_client_view::Request::Save { records }] {
        let mut child = Queue::with_capacity(1);
        child.push(request);
        let mut out = Queue::with_capacity(crate::max_out(&limits));
        crate::route::route_jig(&mut root, &fixtures::env(&limits), &mut child, &mut out);
        assert!(out.is_empty(), "reference UI records produce neither store requests nor facts");
    }
}

#[test]
fn an_asker_naming_no_child_is_a_fact_and_reaches_none() {
    const UNKNOWN: jig_client::Asker = jig_client::Asker(65_535);
    const REQUEST: u64 = 17;
    let limits = fixtures::limits();
    let mut root = fixtures::filled();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        jig_client::Told::Answered {
            key: jig_client::Key { asker: UNKNOWN, number: REQUEST },
            outcome: jig_client::Outcome::Unknown,
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::UnknownAsker }));
    assert_eq!(root.forge().form().expect("form").outcome, None);
    assert!(out.is_empty());
}

#[test]
fn the_forges_side_learns_its_party_from_its_own_me_handle() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, List::with_capacity(0));
    crate::route::forge(
        &mut root,
        &fixtures::env(&limits),
        forge::Event::Went { address: forge::Address::Adopt { project: fixtures::PROJECT } },
        &mut Queue::with_capacity(crate::max_out(&limits)),
    );
    assert!(!root.forge().form().expect("adoption page").confirmed);
    fixtures::confirm(&mut root, watch, fixtures::PARTY, &mut Queue::with_capacity(crate::max_out(&limits)));
    assert!(root.client().document(forge::me::handle()).is_some());
    assert!(root.forge().form().expect("adoption page").confirmed);
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        jig_client::Told::Ended { handle: forge::me::handle(), why: jig_client::Why::SignedOut },
        &mut Queue::with_capacity(crate::max_out(&limits)),
    );
    assert!(!root.forge().form().expect("adoption page").confirmed);
    assert_eq!(root.jig().frame_summary(root.client()).party, Some(fixtures::PARTY));
}

#[test]
fn dropped_work_goes_to_the_shell_with_its_count() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::route::told(&mut root, &fixtures::env(&limits), jig_client::Told::Dropped { count: 2 }, &mut out);
    assert_eq!(out.pop(), Some(Request::Dropped { count: 2 }));
    assert!(out.is_empty());
}
