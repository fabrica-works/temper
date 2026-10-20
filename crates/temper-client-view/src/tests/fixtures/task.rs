use super::*;
use jig_client::contract as c;
const EXCHANGES: u32 = 8;
const SPENT: u64 = 7;
const BELOW: u64 = 2;
pub fn document() -> c::Task {
    use jig_client_protocol_wire::v10 as w;
    let mut record = jig_client_test_records::task_record().into_parts();
    record.chip = jig_client_test_records::owner_waiting_chip_record();
    let record = w::Task::new(&w::CEILINGS, record).expect("checked fixture");
    let link = jig_client_protocol_link::Limits::new(limits().client.domain, EXCHANGES).expect("link");
    let mut task = jig_client_protocol_link::translate::documents::task(record, &link).expect("task");
    let c::ChipPhase::Owner { phase, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    *phase = c::OwnerChipPhase::Active(c::RunStage::Idle);
    task.budget.numbers.spent = SPENT;
    task.budget.numbers.spent_below = BELOW;
    task
}
const TASK: u64 = 9;
const EMPTY: u32 = 0;
const NO_TIME: Time = Time::ZERO;
const AT_EPOCH: Wall = Wall::EPOCH;
pub fn tree() -> c::Delivery {
    c::Delivery::OwnerTree(Box::new(c::OwnerTree {
        root: {
            let mut chip = document().chip;
            chip.task = TASK;
            chip
        },
        rows: List::with_capacity(EMPTY),
        folded: List::with_capacity(EMPTY),
        held_below: 0,
        next: None,
    }))
}
pub fn root(executor: c::Executor, priority: Option<u32>) -> temper_client_domain::Domain {
    root_with_hold(executor, priority, false)
}
pub fn held_root() -> temper_client_domain::Domain {
    root_with_hold(c::Executor::Agent { charter: 1 }, Some(1), true)
}
fn root_with_hold(executor: c::Executor, priority: Option<u32>, held: bool) -> temper_client_domain::Domain {
    use jig_client::{Event as E, Up};
    let limits = domain_limits();
    let (mut root, _, watch, tree_watch) =
        root_at_with_tree(temper_client_domain::Address::Task { task: TASK }, &limits);
    let mut document = document();
    document.chip.task = TASK;
    document.chip.executor = executor;
    document.priority = priority;
    if held {
        document.hold = Some(c::TaskHold::Owner(Box::new(c::OwnerChipHold {
            account: None,
            source: None,
            reason: c::HoldCategory::Failures { class: c::HoldFailure::Run },
            revision: HOLD_REVISION.value,
            before: c::OwnerPriorPhase::Active(c::RunStage::Idle),
            escalation: None,
            tries: c::OwnerFailureTries { transient: 0, permanent: 0, run: 2, agent: 0, lost: 0, invalid: 0 },
            hold_wait_since: None,
        })));
        document.acts = List::with_capacity(ONE_OFFER);
        document
            .acts
            .push(c::Offered {
                act: c::ActKind::Release,
                admission: c::Admission::Allowed,
                effect: Some(c::Effect::Release(c::ReleaseEffect::RunAgain)),
            })
            .expect("release");
    }
    let card = document.clone();
    let mut tree = tree();
    if held && let c::Delivery::OwnerTree(value) = &mut tree {
        value.root.phase = c::Phase::Held;
    }
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits));
    let env = Env { now: NO_TIME, wall: AT_EPOCH, limits };
    temper_client_domain::step(
        &mut root,
        &env,
        temper_client_domain::Event::Client {
            event: E::Up {
                up: Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(document))),
                },
            },
        },
        &mut out,
    );
    if let Some(watch) = tree_watch {
        temper_client_domain::step(
            &mut root,
            &env,
            temper_client_domain::Event::Client {
                event: E::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(tree) } },
            },
            &mut out,
        );
    }
    if held {
        while let Some(request) = out.pop() {
            if let temper_client_domain::Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { .. }) },
                    },
            } = request
            {
                root_event(
                    &mut root,
                    temper_client_domain::Event::Client {
                        event: E::Up {
                            up: Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(
                                    card,
                                )))),
                            },
                        },
                    },
                );
                break;
            }
        }
    }
    root
}

pub const HOLD_REVISION: c::Revision = c::Revision { value: 7 };
const ONE_OFFER: u32 = 1;
