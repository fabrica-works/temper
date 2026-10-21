use super::*;
use skein_ui::{Role, Run};
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
pub fn page() -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    text(&mut b, Role::Heading { level: 1 }, b"Chat 9");
    text(&mut b, Role::Paragraph, b"Hello");
    text(&mut b, Role::Status, b"Writing");
    text(&mut b, Role::Paragraph, b"Spent 7; delegates spent 2");
    b.open(Role::Log);
    b.name(b"Conversation");
    text(&mut b, Role::Paragraph, b"Pat");
    text(&mut b, Role::Paragraph, b"Hello");
    text(&mut b, Role::Status, b"Read");
    text(&mut b, Role::Paragraph, b"Agent");
    golden_cards::report(&mut b, b"I can help.");
    text(&mut b, Role::Status, b"Writing...");
    text(&mut b, Role::Paragraph, b"Writing an answer");
    b.close();
    for (role, name) in [
        (Role::TextArea, b"Message".as_slice()),
        (Role::Button, b"Send"),
        (Role::Button, b"Stop"),
        (Role::Button, b"Close"),
        (Role::Button, b"Mark seen"),
        (Role::Button, b"Older"),
    ] {
        b.open(role);
        b.name(name);
        b.bind(binding());
        b.close();
    }
    b.close();
    b.close();
    b.finish()
}

/// The first run, through the production chat view and delegation card.
pub fn delegation_page(landed: bool) -> skein_ui::Tree {
    use jig_client::contract as c;
    use skein_lib::{Env, List, Queue, Time, Wall};
    use temper_client_domain::{Address, Event, Request};
    let mut limits = crate::limits::native().client;
    limits.client.domain.rows = 8;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut root = client::unlinked(&limits, 16);
    let mut out = Queue::with_capacity(client::max_out(&limits));
    client::step(
        &mut root,
        &env,
        Event::Start { address: Address::Chat { task: 1 }, kept: List::with_capacity(0) },
        &mut out,
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        client::step(&mut root, &env, Event::Client { event }, &mut out);
    }
    for _ in 0..128 {
        let Some(request) = out.pop() else { break };
        let up = match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                    },
            } => {
                let delivery = match subject {
                    c::Subject::Me => {
                        let mut me = jig_client_world::stories::frame(1);
                        me.projects.get_mut(0).expect("demo project").name =
                            Some(c::DocumentText::complete(delegation_text(b"demo")));
                        Some(c::Delivery::Me(Box::new(me)))
                    }
                    c::Subject::Task { task } => Some(c::Delivery::Task(Box::new(delegation_task(task, landed)))),
                    c::Subject::Deployment
                    | c::Subject::Tree { .. }
                    | c::Subject::Chips { .. }
                    | c::Subject::Inbox { .. }
                    | c::Subject::Tasks { .. }
                    | c::Subject::Run { .. }
                    | c::Subject::Accounts => None,
                };
                let Some(delivery) = delivery else { continue };
                Some(jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) })
            }
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                    },
            } => Some(jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                    delegation_activity(landed),
                )))),
            }),
            Request::Client { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            client::step(&mut root, &env, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
        }
    }
    assert!(out.is_empty(), "first run fixture settles");
    let mut view =
        temper_client_view::View::new(&temper_client_view::Limits { tree: tree_limits() }, &limits, page_words(), 1);
    temper_client_view::build(&mut view, &root, &mut Queue::with_capacity(tree_limits().patches));
    copy_tree(view.tree())
}
fn delegation_task(number: u64, landed: bool) -> contract::Task {
    use jig_client::contract as c;
    let mut task = jig_client_world::stories::conversations::chat(number, b"First run", skein_lib::Wall::EPOCH);
    task.chip.task = number;
    task.chip.title = c::DocumentText::complete(delegation_text(if number == 1 {
        b"Add CSV export to widget"
    } else {
        b"Export widget's table as CSV"
    }));
    task.hold = None;
    task.result = None;
    task.chip.phase = c::Phase::Running;
    task.proposals.clear();
    task.activity = if number == 1 { if landed { 3 } else { 2 } } else { 0 };
    if number == 2 {
        task.result_contract = c::TaskContractDocument::Change { connector: 1, kind: 1, words: 256 };
        if landed {
            task.result = Some(c::TaskResult::Ending(Box::new(c::OwnerChipEnding::Done(c::ResultValue::Change {
                kind: 1,
                part: c::OwnerPart { connector: 1, kind: c::OwnerPartKind::Result, key: c::OwnerPartKey::Number(2) },
                words: c::DocumentText::complete(delegation_text(b"1a2b3c4abcdef0123456789abcdef0123456789abcd")),
            }))));
        }
    }
    task
}
fn delegation_activity(landed: bool) -> contract::ActivityPage {
    use jig_client::contract as c;
    use skein_lib::{List, Wall};
    let mut blocks = List::with_capacity(3);
    blocks.push(c::Block::Text(delegation_text("I’ll delegate this change in demo/widget. It will push a branch, open a pull request, check CI and land into main on Demo forge.".as_bytes()))).expect("agent words");
    blocks
        .push(c::Block::Call(c::Call {
            number: 1,
            tool: delegation_text(b"delegate"),
            input: delegation_text(b"{}"),
            outcome: Some(delegation_text(b"Delegated tasks: 2")),
            failed: false,
            objects: List::with_capacity(0),
            parts: List::with_capacity(0),
        }))
        .expect("call");
    blocks
        .push(c::Block::Result { call: 1, summary: delegation_text(b"Delegated tasks: 2"), failed: false })
        .expect("acknowledgement");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("message");
    let mut rows = List::with_capacity(3);
    if landed {
        rows.push(c::ActivityLine { position: 3, at: Wall::EPOCH, event: c::Activity::News { summary: delegation_text("CSV export is available from widget’s table. The change landed on Demo forge. This was a scripted demo change.".as_bytes()), topic: 0, part: None } }).expect("reply");
    }
    rows.push(c::ActivityLine {
        position: 2,
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: 1, number: 1 },
            turn: c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: Some(1),
                cut: false,
            }),
        },
    })
    .expect("turn");
    rows.push(c::ActivityLine {
        position: 1,
        at: Wall::EPOCH,
        event: c::Activity::Words {
            by: c::Party { number: 1, kind: c::PartyKind::Person, name: delegation_text(b"Pat") },
            message: 1,
            words: delegation_text(b"Add CSV export to widget"),
        },
    })
    .expect("request");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

fn delegation_text(bytes: &[u8]) -> contract::Text {
    contract::Text::new(Box::from(bytes), 256).expect("first run text fits")
}
