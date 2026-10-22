//! Make the forge's whole document in the decision that asked (domain/root.md, 5.1).
use super::{Decision, Delivery, Domain, Env, Limits, Queue, Token, core, emit, environment_core};
use alloc::boxed::Box;
use jig_domain_people::ApplicationScope;
use jig_domain_views as views;

pub(crate) fn document(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    connector: u16,
    watcher: Token,
    scope: ApplicationScope,
    subject: Token,
) {
    let mut panel = if connector == domain.config.forge_connector {
        domain.forge.panel(subject.raw(), scope, env.wall, &env.limits.forge)
    } else {
        None
    };
    let outcome = if panel.is_some() { views::Made::Ready } else { views::Made::Gone { why: views::End::Gone } };
    let core::Requests::Out(mut requests) = core::step(
        &mut domain.core,
        &environment_core(env),
        core::Event::View(views::Event::Made { watcher, outcome }),
    );
    for _ in 0..requests.len() {
        let request = requests.pop().expect("document completion output");
        if let core::Request::Held(held) = &request
            && let core::Held::ApplicationDelivery { watcher: due, connector: owner, subject: named } = &**held
        {
            assert!(*due == watcher && *owner == connector && *named == subject, "document keeps its subject");
            emit(
                decision,
                &env.limits,
                Delivery::Panel { watcher, panel: Box::new(panel.take().expect("one whole ready panel")) },
            );
        } else {
            let mut forward = Queue::with_capacity(1);
            forward.push(request);
            super::route_core_requests(domain, env, decision, core::Requests::Out(forward));
        }
    }
    assert!(panel.is_none(), "every ready panel was taken where it was made");
}
