//! A call joins its selected documents before answering once.
use super::{Cut, Limits, Text};
use crate::jig::runner::reads::Reading;
use jig_client::{Readout, contract as c};
use skein_json::writer::Encoder;
use skein_ui::phrases::Phrasebook;
pub(super) fn text(
    reading: &Reading,
    me: Option<&c::Me>,
    book: &Phrasebook,
    limits: &Limits,
    owner_part: fn(&c::OwnerPart, &mut Text),
    out: &mut Text,
) {
    let call = &reading.called.call;
    if call.tool == 1 {
        super::super::lists::inboxes(reading, limits.rows, out);
        return;
    }
    if let Some(thread) = &reading.thread {
        super::super::activity::chat(thread, call, limits, out);
        return;
    }
    for page in reading.pages.values() {
        match page {
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) => {
                super::task_text(task, me, book, owner_part, out);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) => {
                super::super::lists::tasks(page, call, limits.rows, book, out);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) => {
                super::super::activity::page(page, call, limits, out);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Turn(turn))) => {
                super::super::turn::text(turn.task, turn.name, &turn.turn, call, limits, out);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(_) | c::ReadValue::OwnerInbox(_))) => {
                unreachable!("inbox handled above")
            }
            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                unreachable!("validated read value")
            }
        }
    }
    for (local, document) in &reading.documents {
        match local {
            3 => super::super::lists::tree(document, book, out),
            4 => super::super::activity::run(document, reading.text.as_ref(), out),
            _ => unreachable!("selected watch"),
        }
    }
}
pub(super) fn encode(reading: &Reading, limits: &Limits, out: &mut Encoder, budget: &mut Cut) {
    let compound = reading.called.call.tool == 1 || reading.called.call.tool == 3 || reading.called.call.tool == 4;
    if compound {
        out.object_start();
    }
    for (local, page) in &reading.pages {
        if reading.called.call.tool == 1 && *local == 5 && !super::super::lists::updates_fit(reading, limits.rows) {
            continue;
        }
        if reading.called.call.tool == 1 {
            out.key(if *local == 1 { b"needs" } else { b"updates" });
        }
        match page {
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) => {
                if compound {
                    out.key(b"task");
                }
                super::encode_task(task, out, budget);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) => {
                let call = &reading.called.call;
                let next = super::super::lists::task_read_on(page, call);
                super::encode_task_page(page, crate::jig::call::value(call, 4).is_some(), next.as_deref(), out, budget);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) => {
                if compound {
                    out.key(b"activity");
                }
                let call = &reading.called.call;
                let task = crate::jig::call::number(call, 0).expect("activity task");
                budget.turn = Some(super::turn_window::Window::new(
                    call,
                    task,
                    c::TurnName { attempt: 0, number: 0 },
                    limits.part,
                ));
                let next = super::super::activity::read_on(page, call, limits.rows);
                super::encode_activity_page(
                    page,
                    super::super::lists::activity_limit(&reading.called.call, limits.rows),
                    next.as_deref(),
                    out,
                    budget,
                );
                budget.turn = None;
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Turn(turn))) => {
                budget.turn =
                    Some(super::turn_window::Window::new(&reading.called.call, turn.task, turn.name, limits.part));
                super::encode_archived_turn(turn, out, budget);
                budget.turn = None;
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => {
                super::encode_inbox_page(page, out, budget);
            }
            Readout::Jig(c::ReadResult::Value(c::ReadValue::OwnerInbox(page))) => {
                super::encode_owner_inbox_page(page, out, budget);
            }
            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                unreachable!("validated read value")
            }
        }
    }
    for (local, document) in &reading.documents {
        if compound {
            out.key(if *local == 3 { b"tree" } else { b"run" });
        }
        match document {
            c::Delivery::Tree(tree) => super::encode_tree(tree, out, budget),
            c::Delivery::OwnerTree(tree) => super::encode_owner_tree(tree, out, budget),
            c::Delivery::RunState(_) | c::Delivery::Run(_) => {
                encode_run(document, reading.text.as_ref(), out, budget);
            }
            c::Delivery::OwnerProject(_)
            | c::Delivery::Me(_)
            | c::Delivery::Task(_)
            | c::Delivery::OwnerProposal(_)
            | c::Delivery::Chips(_)
            | c::Delivery::Inbox(_)
            | c::Delivery::OwnerInbox(_)
            | c::Delivery::Tasks(_)
            | c::Delivery::Deployment(_)
            | c::Delivery::OwnerDeployment(_) => unreachable!("selected watch value"),
        }
    }
    if reading.called.call.tool == 1 {
        out.key(b"read_on");
        match super::super::lists::inbox_cursor(reading, limits.rows) {
            Some(cursor) => out.string(&super::super::lists::read_on(&reading.called.call, 3, &cursor)),
            None => out.null(),
        }
    }
    if compound {
        out.object_end();
    }
}

fn encode_run(delivery: &c::Delivery, text: Option<&c::RunText>, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    out.key(b"state");
    match delivery {
        c::Delivery::RunState(run) => super::encode_run_state(run, out, budget),
        c::Delivery::Run(run) => super::encode_run_text(run, out, budget),
        c::Delivery::OwnerProject(_)
        | c::Delivery::Me(_)
        | c::Delivery::Task(_)
        | c::Delivery::OwnerProposal(_)
        | c::Delivery::Tree(_)
        | c::Delivery::OwnerTree(_)
        | c::Delivery::Chips(_)
        | c::Delivery::Inbox(_)
        | c::Delivery::OwnerInbox(_)
        | c::Delivery::Tasks(_)
        | c::Delivery::Deployment(_)
        | c::Delivery::OwnerDeployment(_) => unreachable!("run delivery"),
    }
    out.key(b"provisional");
    match text {
        Some(text) => super::encode_run_text(text, out, budget),
        None => out.null(),
    }
    out.object_end();
}
