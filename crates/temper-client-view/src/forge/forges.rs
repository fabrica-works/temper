//! Folded forge rows and the playground apart (client/forge.md, 5.4).
use super::{bindings::Bindings as ForgeBindings, connect};
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{
    Address, Choice, Domain,
    forge::{
        self,
        forges::{Connection, Kind, Mode, Offer, Row, State},
        listed,
    },
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::FORGES);
    connect::text(b, w, Role::Heading { level: 1 }, Word::ForgesTitle);
    let rows = listed::rows(root.forges(), root.forge().demo());
    let count = u64::from(rows.len());
    let heading = match root.forges() {
        State::Rows(_) => w.say(Word::YourForges, &[Hole::Number(count)]),
        State::Unknown | State::Ended => w.say(Word::YourForgesTitle, &[]),
    };
    crate::cards::text(b, Role::Heading { level: 2 }, &heading);
    match root.forges() {
        State::Unknown => {}
        State::Ended => crate::cards::refusal(b, w, &jig_client::contract::Refusal::NotOffered),
        State::Rows(_) => {
            connect::text(b, w, Role::Paragraph, Word::ForgesDescription);
            connect::link(b, w, bindings, Word::AddForge, Address::ConnectForge { forge: None });
            if count == 0 {
                connect::text(b, w, Role::Paragraph, Word::NoForges);
                connect::link(b, w, bindings, Word::ConnectForge, Address::ConnectForge { forge: None });
            }
            for row in &rows {
                personal(b, root, w, bindings, forge, row);
            }
            if let Some((demo, row)) = listed::playground(root) {
                connect::text(b, w, Role::Heading { level: 2 }, Word::ForgePlaygroundHeading);
                b.open(Role::Disclosure);
                b.style(look::FORGE_ROW);
                b.name(&w.say(
                    Word::ForgePlaygroundSummary,
                    &[Hole::Name(forge::words::lossy(&demo.owner)), Hole::Name(forge::words::lossy(&demo.name))],
                ));
                account(b, w, row);
                b.close();
            }
        }
    }
    b.close();
}
fn connection(w: &Words, row: &Row) -> Box<[u8]> {
    match &row.connection {
        Connection::Connected { login, .. } => {
            w.say(Word::ForgeRowConnected, &[Hole::Name(forge::words::lossy(login))])
        }
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => forge::words::forge_connection(row),
    }
}
fn personal(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings, row: &Row) {
    b.open(Role::Disclosure);
    b.style(look::FORGE_ROW);
    b.name(&connect::name(w, row, root));
    let kind = match row.kind {
        Kind::Forgejo => Word::ForgeOther,
        Kind::GitHub => Word::ForgeGitHub,
    };
    b.description(&w.say(
        Word::ForgeSummary,
        &[Hole::Number(u64::from(row.number)), Hole::Name(w.say(kind, &[])), Hole::Name(connection(w, row))],
    ));
    account(b, w, row);
    match row.connection {
        Connection::SetUpFailed { .. } if row.offer == Offer::Offered => {
            connect::setup_control(b, root, w, forge, row.number, Word::ForgeAgain);
        }
        Connection::SettingUp { .. } | Connection::Connecting => connect::link(
            b,
            w,
            bindings,
            Word::ForgeOpenSetup,
            Address::ConnectForge { forge: Some(Choice::Forge(row.number)) },
        ),
        Connection::Stopped { .. } if row.mode == Mode::Owned => {
            connect::text(b, w, Role::Paragraph, Word::ForgeRestartBox);
        }
        Connection::Connected { .. } => connect::adopt(b, root, w, bindings),
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => {}
    }
    b.close();
}
fn account(b: &mut Builder, w: &Words, row: &Row) {
    if let Connection::Connected { id, login } = &row.connection {
        b.open(Role::Slot);
        connect::text(b, w, Role::Term, Word::ForgeAccountLabel);
        crate::cards::text(
            b,
            Role::Description,
            &w.say(Word::ForgeAccount, &[Hole::Name(forge::words::lossy(login)), Hole::Number(*id)]),
        );
        b.close();
    }
}
