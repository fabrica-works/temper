mod fixtures;
mod form;
use crate::{Address, Ask, Event, Fact, Request, forge, store::Record};
use jig_domain_client::boundary::Offset;
use jig_domain_client::{
    contract::{self, Part, PartScope},
    pages::navigation::Destination,
};
use skein_lib::{List, Queue, RequestRecord};

#[test]
fn the_forge_address_goes_to_the_forge_side() {
    let limits = fixtures::limits();
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    fixtures::start(&mut root, List::with_capacity(0));
    let address = forge::Address::Adopt { project: fixtures::PROJECT };
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Went { place: crate::Place::Address(Address::Forge(address)) },
        &mut out,
    );
    assert_eq!(root.forge().address(), Some(address));
    crate::step(&mut root, &fixtures::env(&limits), Event::Went { place: crate::Place::Missing }, &mut out);
    assert_eq!(root.forge().address(), None, "leaving drops the forge page");
}
#[test]
fn a_family_hello_outside_the_version_makes_the_side_out_of_date() {
    let limits = fixtures::limits();
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    for (name, versions, expected) in [
        ("served", contract::VersionRange { first: 1, last: 1 }, forge::Served::Yes),
        ("newer", contract::VersionRange { first: 2, last: 2 }, forge::Served::OutOfDate),
        ("inverted", contract::VersionRange { first: 2, last: 1 }, forge::Served::OutOfDate),
    ] {
        crate::step(&mut root, &fixtures::env(&limits), Event::Forge(forge::Event::Served { versions }), &mut out);
        assert_eq!(root.forge().served(), expected, "{name}");
    }
}

fn kept(version: u16) -> List<Record> {
    let mut records = List::with_capacity(3);
    for (key, scope) in [(1, 7), (2, 7), (3, 8)] {
        records
            .push(Record::Table {
                version,
                record: RequestRecord {
                    key: [key; 16],
                    scope,
                    first_sent: None,
                    request: Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })),
                },
            })
            .expect("kept fixtures fit");
    }
    records
}
#[test]
fn a_part_of_the_forges_number_is_held_by_its_side() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    for forge in [fixtures::FORGE, 9] {
        let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: forge, version: 1 });
        let mut out = Queue::with_capacity(crate::max_out(&limits));
        let part = Part::Connector { connector: forge };
        crate::parts::show(&mut root, &env, part.clone(), 1, PartScope::Project(fixtures::PROJECT), &mut out);
        assert!(root.forge().contains(&part, 1), "forge {forge}");
        assert!(out.is_empty(), "forge {forge} reads no document");
    }
}
#[test]
fn a_part_of_another_connector_is_reported_unknown() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    let part = Part::Connector { connector: fixtures::OTHER };
    crate::parts::show(&mut root, &env, part.clone(), 1, PartScope::Project(fixtures::PROJECT), &mut out);
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::UnknownConnector }));
    assert!(out.is_empty(), "an unknown connector's part asks for nothing more");
    assert!(!root.forge().contains(&part, 1), "the forge's side holds no other connector's part");
}
#[test]
fn parts_left_forget_the_parts_of_that_generation() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    for generation in [1, 2, 3] {
        crate::parts::show(&mut root, &env, fixtures::resource(), generation, PartScope::Deployment, &mut out);
    }
    let mut child = Queue::with_capacity(1);
    child.push(jig_domain_client::Request::PartsLeft { generation: 2 });
    crate::route::route_jig(&mut root, &env, &mut child, &mut out);
    for generation in [1, 2, 3] {
        assert_eq!(
            root.forge().contains(&fixtures::resource(), generation),
            generation == 3,
            "generation {generation}"
        );
    }
    assert!(out.is_empty());
}
#[test]
fn a_part_past_the_sides_bound_is_reported_parts_full() {
    let mut limits = fixtures::limits();
    limits.forge.parts = 1;
    let env = fixtures::env(&limits);
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::parts::show(&mut root, &env, fixtures::connector(), 1, PartScope::Deployment, &mut out);
    crate::parts::show(&mut root, &env, fixtures::connector(), 1, PartScope::Deployment, &mut out);
    crate::parts::show(&mut root, &env, fixtures::resource(), 1, PartScope::Deployment, &mut out);
    assert_eq!(out.pop(), Some(Request::Fact { fact: Fact::PartsFull }));
    assert!(out.is_empty());
    assert!(!root.forge().contains(&fixtures::resource(), 1));
    assert_eq!(forge::shown(root.forge()), 1);
}
#[test]
fn kept_requests_are_restored_parked_and_counted_per_scope() {
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let watch = fixtures::start(&mut root, kept(crate::store::VERSION));
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::take(&mut root, &env, &mut out);
    assert!(out.is_empty(), "kept work starts parked");
    fixtures::confirm(&mut root, watch, 9, &mut out);
    assert_eq!(root.jig().page(), &jig_domain_client::pages::Page::SomeoneElse);
    out = Queue::with_capacity(crate::max_out(&limits));
    for (scope, expected) in [(7, 2), (8, 1)] {
        root.table.confirmed(Some(scope));
        root.table.link(true);
        crate::take(&mut root, &env, &mut out);
        let mut sent = 0_u32;
        for _ in 0..out.len() {
            match out.pop().expect("counted output") {
                Request::Send { .. } => sent += 1,
                Request::Save { .. }
                | Request::Erase { .. }
                | Request::Jig(_)
                | Request::Show { .. }
                | Request::SignIn { .. }
                | Request::SignOut
                | Request::Fact { .. } => {}
            }
        }
        assert_eq!(sent, expected, "scope {scope}");
    }
}
#[test]
fn a_table_record_of_another_version_is_dropped_as_invalid() {
    let limits = fixtures::limits();
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    let env = fixtures::env(&limits);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        &mut root,
        &env,
        Event::Start {
            address: Address::Jig(jig_domain_client::address::Address::Inbox { project: None }),
            kept: kept(2),
            offset: Offset { minutes: 0 },
        },
        &mut out,
    );
    let mut invalid = 0_u32;
    for _ in 0..out.len() {
        match out.pop().expect("counted output") {
            Request::Fact { fact: Fact::InvalidRecord } => invalid += 1,
            Request::Fact {
                fact:
                    Fact::Jig(_)
                    | Fact::TableFull
                    | Fact::PartsFull
                    | Fact::InvalidAnswer
                    | Fact::UnknownConnector
                    | Fact::KeptAnswered,
            }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Send { .. }
            | Request::Jig(_)
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut => {}
        }
    }
    assert_eq!(invalid, 3);
    root.table.confirmed(Some(7));
    root.table.link(true);
    crate::take(&mut root, &env, &mut out);
    assert!(out.is_empty(), "invalid version restores no requests");
}
#[test]
fn temper_navigation_is_the_inbox_conversations_and_deployment() {
    use jig_domain_client::address::Address as JigAddress;
    let limits = fixtures::limits();
    let navigation = crate::navigation(&limits);
    assert_eq!(navigation.home, Destination::Jig(JigAddress::Inbox { project: None }));
    assert_eq!(
        navigation.entries.as_slice(),
        &[
            Destination::Jig(JigAddress::Inbox { project: None }),
            Destination::Jig(JigAddress::Conversations { project: None }),
            Destination::Jig(JigAddress::Deployment),
        ]
    );
    let mut root = crate::new(&limits, fixtures::SEED, forge::Config { connector: fixtures::FORGE, version: 1 });
    assert_eq!(crate::configure_navigation(&mut root, navigation), Ok(()));
}
