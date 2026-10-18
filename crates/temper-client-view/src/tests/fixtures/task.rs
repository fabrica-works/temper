use super::*;
use jig_client::contract as c;
const EXCHANGES: u32 = 8;
const SPENT: u64 = 7;
const BELOW: u64 = 2;
pub fn document() -> c::Task {
    use jig_client_protocol_wire::v8 as w;
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
    use jig_client::{Event as E, Up};
    let limits = domain_limits();
    let (mut root, _, watch, tree_watch) =
        root_at_with_tree(temper_client_domain::Address::Task { task: TASK }, &limits);
    let mut document = document();
    document.chip.task = TASK;
    document.chip.executor = executor;
    document.priority = priority;
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
                event: E::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(tree()) } },
            },
            &mut out,
        );
    }
    root
}
