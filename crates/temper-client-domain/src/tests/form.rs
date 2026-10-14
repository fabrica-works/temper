use super::fixtures;
use crate::{Answered, Event, Request, forge};
use skein_lib::{Queue, RequestEnvelope};

#[test]
fn a_new_form_holds_the_designs_defaults() {
    let root = fixtures::on_adoption_page();
    let form = root.forge().form().expect("form entered");
    let rows: [(forge::Text, &[u8]); 6] = [
        (forge::Text::Forge, b"1"),
        (forge::Text::Owner, b""),
        (forge::Text::Name, b""),
        (forge::Text::Prefix, b"temper/"),
        (forge::Text::Landing, b""),
        (forge::Text::Checks, b""),
    ];
    for (field, expected) in rows {
        assert_eq!(&*form.texts[field.index()].text, expected, "{field:?}");
    }
    assert_eq!(
        (form.project, form.role, form.home.on, form.ci.on),
        (fixtures::PROJECT, forge::Role::Owned, true, true)
    );
    assert_eq!((&form.pending, &form.outcome), (&None, &None));
}
#[test]
fn each_rule_the_draft_breaks_marks_its_field_and_sends_nothing() {
    let rows = [
        (fixtures::owner_empty(), Some(forge::Text::Owner), forge::Broken::Required),
        (fixtures::forge_zero(), Some(forge::Text::Forge), forge::Broken::ForgeNumber),
        (fixtures::checks_twice(), Some(forge::Text::Checks), forge::Broken::Checks),
        (fixtures::context_home(), None, forge::Broken::ContextHome),
        (fixtures::no_ci_no_check(), Some(forge::Text::Checks), forge::Broken::NoCheck),
    ];
    for (acts, place, broken) in rows {
        let mut root = fixtures::on_adoption_page();
        let sent = fixtures::act_then_adopt(&mut root, &acts);
        assert_eq!(sent, 0, "nothing sent: {broken:?}");
        assert_eq!(fixtures::marked(&root, place), Some(broken), "{broken:?} marks {place:?}");
    }
}
#[test]
fn a_value_past_its_bound_is_not_taken() {
    let mut root = fixtures::filled();
    let long = Box::from([b'x'; 257]);
    let out = fixtures::form_act(
        &mut root,
        forge::Act::Write { field: forge::Text::Owner, text: long, typed: forge::Typed::Bytes },
    );
    assert!(out.is_empty());
    let form = root.forge().form().expect("form remains");
    assert_eq!(&*form.texts[forge::Text::Owner.index()].text, b"acme");
    assert_eq!(form.texts[forge::Text::Owner.index()].written, 2, "the retained value is written back");
    assert_eq!(fixtures::marked(&root, Some(forge::Text::Owner)), Some(forge::Broken::Long { bound: 256 }));
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "long write cannot send old value");
}
#[test]
fn adopt_sends_one_keyed_request_and_a_second_press_none() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let Some(Request::Save { record: crate::store::Record::Table { record, .. } }) = out.pop() else {
        panic!("saved before sent");
    };
    assert_eq!(record.scope, fixtures::PARTY, "confirmed party scopes the table, distinct from project");
    let Some(Request::Send { key, ask: crate::Ask::Forge(forge::Ask::Adopt(adoption)), .. }) = out.pop() else {
        panic!("one keyed adoption");
    };
    assert_eq!(key, record.key);
    assert_eq!(
        (adoption.project, adoption.forge, &*adoption.owner, &*adoption.name),
        (1, 1, &b"acme"[..], &b"shop"[..])
    );
    assert!(out.is_empty(), "one send");
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "second press sends nothing");
}
#[test]
fn an_adopted_outcome_resets_the_fields_and_stays_shown() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    let attempt = sent(&mut out);
    let limits = fixtures::limits();
    crate::step(
        &mut root,
        &fixtures::env(&limits),
        Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::Forge(fixtures::adopted())) },
        &mut out,
    );
    let form = root.forge().form().expect("page stays");
    assert_eq!(form.outcome, Some(fixtures::adopted()));
    assert_eq!(&*form.texts[forge::Text::Owner.index()].text, b"");
    assert_eq!((form.home.on, form.ci.on, form.role, form.pending), (true, true, forge::Role::Owned, None));
    crate::fire(&mut root, &fixtures::env(&limits), &mut out);
    assert_eq!(root.forge().form().expect("page stays").outcome, Some(fixtures::adopted()), "outcome stays shown");
    fixtures::form_act(&mut root, fixtures::write(forge::Text::Owner, b"other", forge::Typed::Bytes));
    assert_eq!(root.forge().form().expect("page stays").outcome, None, "edit clears outcome");
}
#[test]
fn out_of_date_disables_adopt_and_keeps_the_pending_request() {
    let mut root = fixtures::filled();
    let mut out = fixtures::form_act(&mut root, forge::Act::Adopt);
    sent(&mut out);
    let (request, _) = root.forge().form().expect("page stays").pending.expect("pending adoption");
    let limits = fixtures::limits();
    let mut out = fixtures::forge_event(
        &mut root,
        forge::Event::Served { versions: jig_domain_client::contract::VersionRange { first: 2, last: 2 } },
    );
    assert_eq!(root.forge().form().expect("page stays").served, forge::Served::OutOfDate);
    assert_eq!(root.forge().form().expect("page stays").pending, Some((request, skein_lib::RequestProgress::Parked)));
    assert!(fixtures::form_act(&mut root, forge::Act::Adopt).is_empty(), "disabled sends nothing");
    root.table.link(false);
    root.table.link(true);
    crate::take(&mut root, &fixtures::env(&limits), &mut out);
    assert_eq!(fixtures::sends(&mut out), 0, "out-of-date requests stay held after reconnect");
    assert_eq!(root.table.len(), 1, "pending adoption remains in the durable table");
}
fn sent(out: &mut Queue<Request>) -> skein_lib::Token {
    for _ in 0..out.len() {
        if let Some(Request::Send { attempt, .. }) = out.pop() {
            return attempt;
        }
    }
    panic!("one send present")
}
