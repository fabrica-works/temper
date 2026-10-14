//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{
    Ask, Event, Fact, Limits, Request, forge,
    owner::Owner,
    route,
    store::{Key, Record},
};
use skein_lib::{Env, Queue, RequestAnswered, RequestOut, RequestProgress, RequestTable, Rng, Time};
/// The two children and their shared durable request table.
#[derive(Debug)]
pub struct Domain {
    pub(crate) jig: jig_domain_client::Domain,
    pub(crate) forge: forge::Side,
    forge_number: u16,
    pub(crate) table: RequestTable<Owner, Ask>,
}
#[must_use]
pub fn new(limits: &Limits, seed: u64, forge: u16) -> Domain {
    assert!(crate::worst_case(limits).is_some(), "root capacities are checked");
    let mut rng = Rng::new(seed);
    Domain {
        jig: jig_domain_client::Domain::new(&limits.jig, rng.next_u64()),
        forge: forge::Side::new(&limits.forge),
        forge_number: forge,
        table: RequestTable::new(&limits.table, rng.next_u64()),
    }
}
impl Domain {
    #[must_use]
    pub const fn jig(&self) -> &jig_domain_client::Domain {
        &self.jig
    }
    #[must_use]
    pub const fn forge(&self) -> &forge::Side {
        &self.forge
    }
    #[must_use]
    pub const fn forge_number(&self) -> u16 {
        self.forge_number
    }
}
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(root, env, event, out);
}
pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_domain_client::max_out(&env.limits.jig));
    jig_domain_client::fire(&mut root.jig, &route::jig_env(env), &mut child);
    route::route_jig(root, env, &mut child, out);
    root.table.fire(env.now, env.wall);
}
#[must_use]
pub fn next_deadline(root: &Domain) -> Option<Time> {
    let mut next = jig_domain_client::next_deadline(&root.jig);
    for deadline in [forge::next_deadline(&root.forge), root.table.next_deadline()].into_iter().flatten() {
        next = match next {
            Some(old) => Some(old.min(deadline)),
            None => Some(deadline),
        };
    }
    next
}
/// Iterate's table phase: emit stores before sends, report progress, reclaim.
pub fn take(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    {
        let mut cursor = root.table.take(env.now, env.wall);
        for _ in 0..env.limits.table.out {
            match cursor.next_out() {
                Some(RequestOut::Save(record)) => out.push(Request::Save {
                    record: Record::Table { version: crate::store::VERSION, record: record.clone() },
                }),
                Some(RequestOut::Erase(key)) => out.push(Request::Erase { key: Key::Table(key) }),
                Some(RequestOut::Send { attempt, key, request }) => {
                    out.push(Request::Send { attempt, key, ask: request.clone() });
                }
                None => break,
            }
        }
    }
    for _ in 0..env.limits.table.requests {
        let Some((owner, progress)) = root.table.progress() else {
            break;
        };
        route::progress(root, env, owner, progress, out);
    }
    root.table.reclaim();
    jig_domain_client::reclaim(&mut root.jig);
    forge::reclaim(&mut root.forge);
}
#[must_use]
pub fn shown(root: &Domain) -> u64 {
    root.jig.shown().checked_add(forge::shown(&root.forge)).expect("forge shown count fits")
}
pub(crate) fn answered(
    root: &mut Domain,
    env: &Env<Limits>,
    attempt: skein_lib::Token,
    envelope: skein_lib::RequestEnvelope,
    answer: Option<crate::Answered>,
    out: &mut Queue<Request>,
) {
    match root.table.answered(env.now, attempt, envelope) {
        RequestAnswered::Final(owner) => match owner {
            Owner::Jig(request) => match answer {
                Some(crate::Answered::Jig(answer)) => {
                    route::jig(root, env, jig_domain_client::Event::Answered { request, answer }, out);
                }
                None => {
                    out.push(Request::Fact { fact: Fact::InvalidAnswer });
                }
            },
            Owner::Kept { scope } => {
                out.push(Request::Fact { fact: Fact::KeptAnswered });
                route::jig(root, env, jig_domain_client::Event::KeptGone { scope }, out);
            }
        },
        RequestAnswered::Pending => match envelope {
            skein_lib::RequestEnvelope::SignedOut => route::jig(root, env, jig_domain_client::Event::SignedOut, out),
            skein_lib::RequestEnvelope::Final
            | skein_lib::RequestEnvelope::Again
            | skein_lib::RequestEnvelope::Lost => {}
        },
        RequestAnswered::Stale => {}
    }
}
pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, owner: Owner, scope: u64, ask: Ask, out: &mut Queue<Request>) {
    let size = ask_size(&env.limits, &ask);
    if root.table.ask(owner, scope, size, ask).is_err() {
        out.push(Request::Fact { fact: Fact::TableFull });
        route::progress(root, env, owner, RequestProgress::Unknown, out);
    }
}
pub(crate) fn ask_size(limits: &Limits, ask: &Ask) -> u64 {
    match ask {
        Ask::Jig(_) => limits.jig.document_bytes,
    }
}

/// Installs application-configured provider words before Start (client.md, 12).
pub fn provider(
    root: &mut Domain,
    provider: jig_domain_client::signin::Provider,
) -> Result<(), jig_domain_client::signin::Provider> {
    root.jig.provider(provider)
}

/// Installs the application's navigation before Start; the jig child validates its bound.
pub fn configure_navigation(
    root: &mut Domain,
    navigation: jig_domain_client::pages::navigation::Navigation,
) -> Result<(), jig_domain_client::pages::navigation::Navigation> {
    root.jig.configure_navigation(navigation)
}

/// temper starts at the inbox and offers jig's three destinations.
#[must_use]
pub fn navigation(limits: &Limits) -> jig_domain_client::pages::navigation::Navigation {
    use jig_domain_client::{
        address::Address,
        pages::navigation::{Destination, Navigation},
    };
    let mut navigation = Navigation::new(limits.jig.navigation_entries);
    for address in [Address::Inbox { project: None }, Address::Conversations { project: None }, Address::Deployment] {
        navigation.entries.push(Destination::Jig(address)).expect("navigation holds temper's three entries");
    }
    navigation
}
