//! Pure facts for connecting a forge (client/forge.md, 5.4).
use super::{
    Demo,
    forges::{Connection, Kind, Mode, Offer, Row, State},
};
use crate::Choice;
use skein_lib::List;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Choose,
    Details(Choice),
    SettingUp(u16),
    Ready(u16),
}
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn row<'a>(number: u16, rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    if let Some(demo) = demo
        && demo.forge == number
    {
        return None;
    }
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.number == number && row.mode != Mode::Fake {
            return Some(row);
        }
    }
    None
}
#[must_use]
pub fn step(choice: Option<Choice>, rows: &State, demo: Option<&Demo>) -> Shown {
    match choice {
        None => Shown::Choose,
        Some(Choice::Elsewhere) => Shown::Details(Choice::Elsewhere),
        Some(Choice::Forge(number)) => {
            let Some(row) = row(number, rows, demo) else { return Shown::Choose };
            match &row.connection {
                Connection::NotSetUp | Connection::NotConnected => {
                    if offered(row) {
                        Shown::Details(Choice::Forge(number))
                    } else {
                        Shown::Choose
                    }
                }
                Connection::SettingUp { .. } | Connection::SetUpFailed { .. } | Connection::Connecting => {
                    Shown::SettingUp(number)
                }
                Connection::Connected { .. } => Shown::Ready(number),
                Connection::Stopped { .. } => Shown::Choose,
            }
        }
    }
}
fn offered(row: &Row) -> bool {
    match row.kind {
        Kind::GitHub => row.connection == Connection::NotConnected,
        Kind::Forgejo => {
            row.mode == Mode::Owned && row.connection == Connection::NotSetUp && row.offer != Offer::NotOffered
        }
    }
}
#[must_use]
pub fn choices(rows: &State, demo: Option<&Demo>) -> List<Choice> {
    let mut choices = List::with_capacity(temper_client_protocol_wire::FORGES.checked_add(1).expect("choices fit"));
    if let State::Rows(rows) = rows {
        for kind in [Kind::GitHub, Kind::Forgejo] {
            // The forges decoder admits rows in increasing number order.
            for row in rows {
                let demo = match demo {
                    Some(demo) => row.number == demo.forge,
                    None => false,
                };
                if row.kind == kind && row.mode != Mode::Fake && !demo && offered(row) {
                    choices.push(Choice::Forge(row.number)).expect("bounded rows");
                }
            }
        }
    }
    choices.push(Choice::Elsewhere).expect("one extra choice");
    choices
}

/// The welcome follows the person's forge once its setup has started.
#[must_use]
pub fn personal<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake {
            continue;
        }
        if let Some(demo) = demo
            && row.number == demo.forge
        {
            continue;
        }
        match row.connection {
            Connection::SettingUp { .. }
            | Connection::Connecting
            | Connection::Connected { .. }
            | Connection::SetUpFailed { .. } => return Some(row),
            Connection::NotConnected | Connection::NotSetUp | Connection::Stopped { .. } => {}
        }
    }
    None
}
#[must_use]
pub fn underway(row: &Row) -> bool {
    match row.connection {
        Connection::SettingUp { .. } | Connection::Connecting => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Connected { .. }
        | Connection::Stopped { .. } => false,
    }
}
/// A setup line remains visible even when another forge is already connected.
#[must_use]
pub fn underway_forge<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake || !underway(row) {
            continue;
        }
        if let Some(demo) = demo
            && row.number == demo.forge
        {
            continue;
        }
        return Some(row);
    }
    None
}
#[must_use]
pub fn next_number(rows: &State) -> u32 {
    let mut largest = 0;
    if let State::Rows(rows) = rows {
        for row in rows {
            largest = largest.max(u32::from(row.number));
        }
    }
    largest.checked_add(1).expect("u16 forge numbers fit u32")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Name {
    Here,
    GitHub,
    Forgejo,
    Playground,
}
#[must_use]
pub fn name(row: &Row, demo: Option<&Demo>) -> Name {
    if let Some(demo) = demo
        && demo.forge == row.number
    {
        return Name::Playground;
    }
    match row.kind {
        Kind::GitHub => Name::GitHub,
        Kind::Forgejo => {
            if row.mode == Mode::Owned {
                Name::Here
            } else {
                Name::Forgejo
            }
        }
    }
}
