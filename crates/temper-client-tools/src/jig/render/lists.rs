//! Rows keep whole identities and opaque cursors in calls that can be replayed.
use super::Text;
use crate::jig::runner::reads::Reading;
use crate::jig::{
    call::{Call, Value, Values},
    call::{number, value},
};
use alloc::boxed::Box;
use jig_client::{Readout, contract as c, words::Word};
use skein_lib::List;
use skein_ui::phrases::Phrasebook;
pub(super) fn tasks(page: &c::TaskPage, call: &Call, rows_limit: u32, book: &Phrasebook, out: &mut Text) {
    let (rows, after) = if value(call, 4).is_some() {
        match &page.ended {
            c::EndedGroup::Rows { rows, after, .. } => (rows, after.as_ref()),
            c::EndedGroup::None => {
                out.put(b"no more\n");
                return;
            }
        }
    } else {
        (&page.live, page.live_after.as_ref())
    };
    for row in rows.as_slice().iter().take(usize::try_from(rows_limit).expect("rows fit")) {
        chip(&row.chip, book, out);
    }
    next(call, 6, after, out);
}
pub(super) fn task_read_on(page: &c::TaskPage, call: &Call) -> Option<Box<[u8]>> {
    Some(read_on(call, 6, task_after(page, call)?))
}
pub(super) fn task_after<'a>(page: &'a c::TaskPage, call: &Call) -> Option<&'a c::Cursor> {
    if value(call, 4).is_some() {
        match &page.ended {
            c::EndedGroup::Rows { after, .. } => after.as_ref(),
            c::EndedGroup::None => None,
        }
    } else {
        page.live_after.as_ref()
    }
}
pub(super) fn chip(chip: &c::Chip, book: &Phrasebook, out: &mut Text) {
    out.number(chip.task);
    out.put(b" \xc2\xb7 ");
    out.words(chip.title.bytes());
    out.put(b" \xc2\xb7 ");
    out.put(&book.say(0, Word::Phase(chip.phase).id(), &[]).expect("phase phrase").text);
    out.put(b"\n");
}
pub(super) fn tree(delivery: &c::Delivery, book: &Phrasebook, out: &mut Text) {
    out.put(b"tree:\n");
    match delivery {
        c::Delivery::Tree(tree) => {
            for row in &tree.nodes {
                chip(&row.chip, book, out);
            }
        }
        c::Delivery::OwnerTree(tree) => {
            chip(&tree.root, book, out);
            for row in &tree.rows {
                chip(&row.chip, book, out);
            }
        }
        c::Delivery::OwnerProject(_)
        | c::Delivery::Me(_)
        | c::Delivery::Task(_)
        | c::Delivery::OwnerProposal(_)
        | c::Delivery::Chips(_)
        | c::Delivery::Inbox(_)
        | c::Delivery::OwnerInbox(_)
        | c::Delivery::Tasks(_)
        | c::Delivery::Run(_)
        | c::Delivery::RunState(_)
        | c::Delivery::Deployment(_)
        | c::Delivery::OwnerDeployment(_) => unreachable!("tree watch delivers a tree"),
    }
}
pub(super) fn next(call: &Call, index: usize, cursor: Option<&c::Cursor>, out: &mut Text) {
    let Some(cursor) = cursor else {
        out.put(b"no more\n");
        return;
    };
    out.put(b"read on with: ");
    continuation(call, index, cursor, out);
    out.put(b"\n");
}
pub(super) fn read_on(call: &Call, index: usize, cursor: &c::Cursor) -> Box<[u8]> {
    let mut measure = Text::measure();
    continuation(call, index, cursor, &mut measure);
    let mut out = Text::writing_exact(measure);
    continuation(call, index, cursor, &mut out);
    out.finish()
}
fn continuation(call: &Call, index: usize, cursor: &c::Cursor, out: &mut Text) {
    let mut call = call.clone();
    *call.values.get_mut(index).expect("continuation argument") =
        Some(Values::One(Value::Cursor(cursor.value.bytes().into())));
    write_call(&call, out);
}
pub(super) fn write_call(call: &Call, out: &mut Text) {
    let mut call = call.clone();
    let mut argv = List::with_capacity(64);
    let tool = match call.tool {
        1 => &crate::jig::rows::INBOX,
        2 => &crate::jig::rows::TASKS,
        4 => &crate::jig::rows::CHAT,
        5 => &crate::jig::rows::ACTIVITY,
        _ => unreachable!("paged read"),
    };
    call.tool = 0;
    crate::jig::argv::back(&[tool], &call, &mut argv).expect("a parsed continuation call");
    for (index, word) in argv.as_slice().iter().enumerate() {
        if index > 0 {
            out.put(b" ");
        }
        argument(word, out);
    }
}
pub(super) fn argument(word: &[u8], out: &mut Text) {
    let mut quoted = word.is_empty();
    for byte in word {
        if !byte.is_ascii_alphanumeric() && !b"-_./:".contains(byte) {
            quoted = true;
        }
    }
    if !quoted {
        out.put(word);
        return;
    }
    out.put(b"'");
    for byte in word {
        if *byte == b'\'' {
            out.put(b"'\\''");
        } else {
            out.put(&[*byte]);
        }
    }
    out.put(b"'");
}
pub(super) fn activity_limit(call: &Call, rows: u32) -> u32 {
    match number(call, 2) {
        Some(limit) => u32::try_from(limit).expect("parsed row bound").min(rows),
        None => rows,
    }
}

pub(super) fn inboxes(reading: &Reading, rows: u32, out: &mut Text) {
    let updates_fit = updates_fit(reading, rows);
    for (local, page) in &reading.pages {
        if *local == 5 && !updates_fit {
            continue;
        }
        match page {
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => {
                for row in page.rows.as_slice().iter().take(usize::try_from(rows).expect("rows fit")) {
                    object(row.object, out);
                }
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::OwnerInbox(page))) => {
                for row in page.rows.as_slice().iter().take(usize::try_from(rows).expect("rows fit")) {
                    object(row.card_key(), out);
                }
            }
            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                unreachable!("validated inbox page")
            }
        }
    }
    next(&reading.called.call, 3, inbox_cursor(reading, rows).as_ref(), out);
}
pub(super) fn updates_fit(reading: &Reading, rows: u32) -> bool {
    let needs = match reading.pages.get(&1) {
        Some(page) => inbox_page(page).0,
        None => 0,
    };
    let updates = match reading.pages.get(&5) {
        Some(page) => inbox_page(page).0,
        None => 0,
    };
    needs.checked_add(updates).expect("two bounded pages") <= rows
}
pub(super) fn inbox_cursor(reading: &Reading, rows: u32) -> Option<c::Cursor> {
    if let Some(page) = reading.pages.get(&1)
        && let Some(after) = inbox_page(page).1
    {
        return Some(group_cursor(0, Some(after)));
    }
    if !updates_fit(reading, rows) {
        return Some(group_cursor(1, None));
    }
    let page = reading.pages.get(&5)?;
    let after = inbox_page(page).1?;
    Some(group_cursor(1, Some(after)))
}
fn inbox_page(page: &Readout) -> (u32, Option<&c::Cursor>) {
    match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => (page.rows.len(), page.after.as_ref()),
        Readout::Jig(c::ReadResult::Value(c::ReadValue::OwnerInbox(page))) => (page.rows.len(), page.after.as_ref()),
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
            unreachable!("validated inbox page")
        }
    }
}
fn group_cursor(group: u8, after: Option<&c::Cursor>) -> c::Cursor {
    let after: &[u8] = match after {
        Some(after) => after.value.bytes(),
        None => &[],
    };
    let mut bytes = skein_lib::Writer::new(after.len().checked_add(1).expect("bounded cursor"));
    bytes.put(&[group]).expect("group byte");
    bytes.put(after).expect("whole cursor");
    c::Cursor::new(bytes.finish(), u32::MAX).expect("bounded cursor")
}
fn object(key: c::ObjectKey, out: &mut Text) {
    match key {
        c::ObjectKey::Task(task) => {
            out.number(task);
            out.put(b" \xc2\xb7 task");
        }
        c::ObjectKey::Proposal(proposal) => {
            out.number(proposal.number);
            out.put(b" \xc2\xb7 proposal");
        }
        c::ObjectKey::Escalation(task) => {
            out.number(task);
            out.put(b" \xc2\xb7 escalation");
        }
        c::ObjectKey::Question { task, message } => {
            out.number(task);
            out.put(b" \xc2\xb7 question ");
            out.number(message);
        }
        c::ObjectKey::PersonTask(task) => {
            out.number(task);
            out.put(b" \xc2\xb7 person task");
        }
        c::ObjectKey::Result(task) => {
            out.number(task);
            out.put(b" \xc2\xb7 result");
        }
        c::ObjectKey::Note { project, note } => {
            out.number(note);
            out.put(b" \xc2\xb7 note of project ");
            out.number(u64::from(project));
        }
    }
    out.put(b"\n");
}
