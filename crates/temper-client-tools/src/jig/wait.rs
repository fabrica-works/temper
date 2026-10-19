//! Level-triggered waits over the client's current documents (tools contract, 5.4).
use crate::jig::{
    Limits,
    call::{Value, value},
    conditions::{self, Facts},
    digest::{self, Digest},
    render::{self, Part},
    runner::{self, Called, Out, Runner},
    subject::{Condition, Subject},
};
use jig_client::{Client, Event, Handle, Latest, State, Told, Watchable, contract as c};
use skein_lib::{Env, Queue, Time};
#[derive(Debug)]
pub(crate) struct Wait {
    pub(crate) called: Called,
    pub(crate) handle: Handle,
    pub(crate) until: Option<Condition>,
    since: Option<Digest>,
    first: Option<Digest>,
    me: bool,
    deadline: Time,
    name: &'static [u8],
}
impl Wait {
    fn answers(&self, digest: Digest, holds: Option<bool>) -> bool {
        let differs = match self.since.or(self.first) {
            Some(old) => old != digest,
            None => false,
        };
        match (&self.until, self.since) {
            (Some(_), Some(_)) => differs && holds == Some(true),
            (Some(_), None) => holds == Some(true),
            (None, _) => differs,
        }
    }
}
pub(crate) fn start(runner: &mut Runner, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    let Some(Value::Subject(subject)) = value(&called.call, 0) else { unreachable!("checked subject") };
    let me = live_me(client, runner.me()).expect("signed-in call");
    let Some((subject, name)) = watch(subject, me, env.limits.rows) else {
        let printed = render::failure(b"unknown project", called.json, 1);
        runner::finish(called, printed, out);
        return;
    };
    let handle = if subject == c::Subject::Me {
        runner.me()
    } else {
        Handle { asker: runner.asker, number: u64::from(called.call_number) << 32u32 }
    };
    let until = match value(&called.call, 2) {
        Some(Value::Condition(until)) => Some(*until),
        _ => None,
    };
    let since = match value(&called.call, 1) {
        Some(Value::Digest(bytes)) => Some(Digest::from_bytes(*bytes)),
        _ => None,
    };
    let Some(Value::Duration(timeout)) = value(&called.call, 3) else { unreachable!("checked timeout") };
    let deadline = env.now.checked_add(*timeout).unwrap_or(Time::from_nanos(u64::MAX));
    let wait = Wait {
        called,
        handle,
        until,
        since,
        first: None,
        me: until == Some(Condition::WaitingOnMe) || until == Some(Condition::Replied),
        deadline,
        name,
    };
    let number = wait.called.call_number;
    runner.waits.insert(number, wait);
    if handle == runner.me() {
        changed(runner, client, env, number, out);
    } else {
        out.push(Out::Client { event: Event::Watch { handle, subject: Watchable::Jig(subject) } });
    }
}
fn watch(subject: &Subject, me: &c::Me, window: u32) -> Option<(c::Subject, &'static [u8])> {
    Some(match subject {
        Subject::Me => (c::Subject::Me, b"me"),
        Subject::Inbox => (c::Subject::Inbox { window }, b"inbox"),
        Subject::Task(task) => (c::Subject::Task { task: *task }, b"task"),
        Subject::Tree(task) => (c::Subject::Tree { task: *task, window }, b"tree"),
        Subject::Run(task) => (c::Subject::Run { task: *task, turn: None, offset: 0 }, b"run"),
        Subject::Tasks(call) => (c::Subject::Tasks { query: runner::reads::task_query(call, me)?, window }, b"tasks"),
    })
}
fn live_me(client: &Client, handle: Handle) -> Option<&c::Me> {
    let document = client.document(handle)?;
    if document.state != State::Live {
        return None;
    }
    match document.latest {
        Some(Latest::Jig(c::Delivery::Me(me))) => Some(me),
        _ => None,
    }
}
fn current_run(client: &Client, handle: Handle) -> Option<Latest> {
    let document = client.document(handle)?;
    let Some(Latest::Jig(c::Delivery::RunState(run))) = document.latest else { return None };
    let mut run = run.clone();
    run.writing = client.text(handle).cloned();
    Some(Latest::Jig(c::Delivery::RunState(run)))
}
pub(crate) fn told(runner: &mut Runner, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    let handle = match told {
        Told::Changed { handle } | Told::Text { handle } => *handle,
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return,
    };
    let mut next = None;
    for _ in 0..runner.waits.len() {
        let entry = match next {
            None => runner.waits.first_key_value(),
            Some(number) => {
                runner.waits.range((core::ops::Bound::Excluded(number), core::ops::Bound::Unbounded)).next()
            }
        };
        let Some((&number, wait)) = entry else { break };
        next = Some(number);
        if wait.handle == handle || (handle == runner.me() && wait.me) {
            changed(runner, client, env, number, out);
        }
    }
}
fn changed(runner: &mut Runner, client: &Client, env: &Env<Limits>, number: u32, out: &mut Queue<Out>) {
    let Some(wait) = runner.waits.get_mut(&number) else { return };
    let Some(document) = client.document(wait.handle) else { return };
    if document.state != State::Live {
        return;
    }
    let run = current_run(client, wait.handle);
    let Some(latest) = run.as_ref().or(document.latest) else { return };
    let Latest::Jig(delivery) = latest else { return };
    let Some(part) = Part::delivery(delivery) else { return };
    let digest = digest::part(&part, &env.limits);
    let holds = match wait.until {
        Some(until) => conditions::holds(
            &until,
            &Facts { subject: latest, me: live_me(client, Handle { asker: runner.asker, number: 0 }) },
        ),
        None => None,
    };
    if wait.answers(digest, holds) {
        complete(runner, client, env, number, Ok(digest), out);
    } else if wait.first.is_none() {
        wait.first = Some(digest);
    }
}
fn complete(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    number: u32,
    result: Result<Digest, &[u8]>,
    out: &mut Queue<Out>,
) {
    let Some(wait) = runner.waits.remove(&number) else { return };
    let run = current_run(client, wait.handle);
    let latest = match client.document(wait.handle) {
        Some(document) => run.as_ref().or(document.latest),
        None => None,
    };
    let delivery = match latest {
        Some(Latest::Jig(delivery)) => Some(delivery),
        _ => None,
    };
    let (met, why, digest) = match result {
        Ok(digest) => (true, b"".as_slice(), Some(digest)),
        Err(why) => (false, why, None),
    };
    let printed = render::waiting(
        render::document::waiting::Waiting {
            delivery,
            me: live_me(client, runner.me()),
            name: wait.name,
            met,
            why,
            digest,
        },
        wait.called.json,
        &env.limits,
        runner.owner_part.expect("host vocabulary"),
    );
    if wait.handle != runner.me() {
        out.push(Out::Client { event: Event::Unwatch { handle: wait.handle } });
    }
    runner::finish(wait.called, printed, out);
}
pub(crate) fn fire(runner: &mut Runner, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut next = None;
    for _ in 0..runner.waits.len() {
        let entry = match next {
            None => runner.waits.first_key_value(),
            Some(number) => {
                runner.waits.range((core::ops::Bound::Excluded(number), core::ops::Bound::Unbounded)).next()
            }
        };
        let Some((&number, wait)) = entry else { break };
        next = Some(number);
        if wait.deadline <= env.now {
            complete(runner, client, env, number, Err(b"timeout"), out);
        }
    }
}
#[must_use]
pub(crate) fn next_deadline(runner: &Runner) -> Option<Time> {
    let mut deadline = None;
    for wait in runner.waits.values() {
        deadline = Some(match deadline {
            Some(old) => wait.deadline.min(old),
            None => wait.deadline,
        });
    }
    deadline
}
