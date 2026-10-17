//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Domain, Fact, Limits, Request, forge, route};
use jig_client::contract::Part;
use skein_lib::{Env, Queue};
pub(crate) fn show(
    root: &mut Domain,
    env: &Env<Limits>,
    part: Part,
    _scope: jig_client::contract::PartScope,
    out: &mut Queue<Request>,
) {
    if connector(&part) == root.forge_number() {
        route::forge(root, env, forge::Event::Show { part }, out);
    } else {
        out.push(Request::Fact { fact: Fact::UnknownConnector });
    }
}

#[must_use]
pub fn connector(part: &Part) -> u16 {
    match part {
        Part::Connector { connector }
        | Part::Resource { connector, .. }
        | Part::Kind { connector, .. }
        | Part::Panel { connector, .. }
        | Part::Result { connector, .. }
        | Part::OwnerEffect { connector, .. }
        | Part::Effect { connector, .. }
        | Part::Outbox { connector, .. }
        | Part::Hold { connector, .. }
        | Part::Project { connector, .. }
        | Part::Deployment { connector } => *connector,
    }
}
