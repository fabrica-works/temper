//! The children share one durable jig client (client.md, 3).
use crate::{Event, Limits, Request, forge, route};
use skein_lib::{Env, Queue, Rng, Time};
#[derive(Debug)]
pub struct Domain {
    pub(crate) jig: jig_client_view::Ui,
    pub(crate) forge: forge::Side,
    pub(crate) client: jig_client::Client,
    pub(crate) salt: u64,
    pub(crate) unlinked: bool,
}
#[must_use]
pub fn new(limits: &Limits, seed: u64) -> Domain {
    build(limits, seed, false)
}
#[must_use]
pub fn unlinked(limits: &Limits, seed: u64) -> Domain {
    build(limits, seed, true)
}
fn build(limits: &Limits, seed: u64, unlinked: bool) -> Domain {
    assert!(crate::worst_case(limits).is_some(), "root capacities are checked");
    let mut rng = Rng::new(seed);
    let client = if unlinked {
        jig_client::Client::unlinked(&limits.client, rng.next_u64())
    } else {
        jig_client::Client::new(&limits.client, rng.next_u64())
    }
    .expect("checked client limits");
    Domain {
        jig: jig_client_view::Ui::new(
            &limits.jig,
            navigation(limits),
            jig_client_view::address::Address::Inbox { project: None },
            jig_client_view::page::Arrangement::none(),
        )
        .expect("temper UI configuration fits"),
        forge: forge::Side::new(&limits.forge, rng.next_u64()),
        client,
        salt: rng.next_u64(),
        unlinked,
    }
}
impl Domain {
    #[must_use]
    pub const fn client(&self) -> &jig_client::Client {
        &self.client
    }
    #[must_use]
    pub const fn jig(&self) -> &jig_client_view::Ui {
        &self.jig
    }
    #[must_use]
    pub const fn forge(&self) -> &forge::Side {
        &self.forge
    }
    #[must_use]
    pub const fn forge_number(&self) -> u16 {
        temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)
    }
}
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(root, env, event, out);
}
pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
    root.jig.fire(env.now, env.wall, &root.client, &mut child);
    route::route_jig(root, env, &mut child, out);
    let mut client = Queue::with_capacity(jig_client::max_out(&env.limits.client));
    root.client.fire(env.now, env.wall, &mut client);
    route::route_client(root, env, &mut client, out);
}
#[must_use]
pub fn next_deadline(root: &Domain) -> Option<Time> {
    [jig_client_view::next_deadline(&root.jig), forge::next_deadline(&root.forge), root.client.next_deadline()]
        .into_iter()
        .flatten()
        .min()
}
/// Reclaim only after every child's tells have been routed.
pub fn take(root: &mut Domain, _env: &Env<Limits>, _out: &mut Queue<Request>) {
    root.client.reclaim();
    jig_client_view::reclaim(&mut root.jig);
    forge::reclaim(&mut root.forge);
}
#[must_use]
pub fn shown(root: &Domain) -> u64 {
    root.jig.shown().checked_add(forge::shown(&root.forge)).expect("forge shown count fits")
}
/// Installs the application's navigation before Start; the jig child validates its bound.
pub fn configure_navigation(
    root: &mut Domain,
    navigation: jig_client_view::page::navigation::Navigation,
) -> Result<(), jig_client_view::page::navigation::Navigation> {
    root.jig.configure_navigation(navigation)
}

/// temper starts at the inbox and offers jig's three destinations.
#[must_use]
pub fn navigation(limits: &Limits) -> jig_client_view::page::navigation::Navigation {
    use jig_client_view::{
        address::Address,
        page::navigation::{Destination, Navigation},
    };
    let mut navigation = Navigation::new(limits.jig.navigation_entries);
    for address in [Address::Inbox { project: None }, Address::Conversations { project: None }, Address::Deployment] {
        navigation.entries.push(Destination::Jig(address)).expect("navigation holds temper's three entries");
    }
    navigation
}
