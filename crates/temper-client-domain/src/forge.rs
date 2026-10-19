//! The forge's adoption form, requests and receiving state.
use skein_lib::{Env, Queue, Time};
pub mod family;
mod form;
pub(crate) mod me;
mod vocabulary;
pub mod words;
pub use form::{Act, Broken, Check, Field, Form, Marks, Text, Typed, adoption, broken, form, write};
pub use vocabulary::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub field_bytes: u32,
    pub checks: u32,
}
#[derive(Debug)]
pub enum Event {
    Act { act: Act },
    Progress { request: skein_lib::Token, progress: skein_lib::RequestProgress },
    Answered { request: skein_lib::Token, answer: Answer },
    Went { address: Address },
    Left,
    Served { served: Served },
}
#[derive(Debug)]
pub enum Request {
    Show { address: Address },
    Client { asked: jig_client::Event },
    Send { request: skein_lib::Token, ask: jig_client::Ask },
}
#[derive(Debug)]
pub struct Side {
    address: Option<Address>,
    served: Served,
    form: Option<Form>,
    serial: u64,
    party: Option<u64>,
    me: bool,
    link: jig_client::Link,
    shown: u64,
}
impl Side {
    #[must_use]
    pub fn new(_limits: &Limits, seed: u64) -> Side {
        Side {
            address: None,
            served: Served::Unknown,
            form: None,
            serial: skein_lib::Rng::new(seed).next_u64(),
            party: None,
            me: false,
            link: jig_client::Link::Starting,
            shown: 0,
        }
    }
    #[must_use]
    pub const fn address(&self) -> Option<Address> {
        self.address
    }
    #[must_use]
    pub const fn served(&self) -> Served {
        self.served
    }
    #[must_use]
    pub const fn form(&self) -> Option<&Form> {
        self.form.as_ref()
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("forge shown count fits");
    }
}
pub fn step(side: &mut Side, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Went { address } => {
            side.address = Some(address);
            let Address::Adopt { project } = address;
            let mut draft = form(project);
            draft.served = side.served;
            draft.confirmed = side.party.is_some();
            side.form = Some(draft);
            side.changed();
        }
        Event::Left => {
            side.form = None;
            if side.address.take().is_some() {
                side.changed();
            }
        }
        Event::Served { served: value } => served(side, value),
        Event::Act { act } => act_on(side, env.limits, act, out),
        Event::Progress { request, progress } => {
            if let Some(draft) = &mut side.form
                && let Some((pending, _)) = draft.pending
                && pending == request
            {
                match progress {
                    skein_lib::RequestProgress::Unknown => {
                        draft.pending = None;
                        draft.outcome = Some(Answer::Unknown);
                    }
                    skein_lib::RequestProgress::InFlight
                    | skein_lib::RequestProgress::Retrying
                    | skein_lib::RequestProgress::Parked => {
                        let progress = match draft.served {
                            Served::OutOfDate => skein_lib::RequestProgress::Parked,
                            Served::Unknown | Served::Yes => progress,
                        };
                        draft.pending = Some((request, progress));
                    }
                }
                side.changed();
            }
        }
        Event::Answered { request, answer } => {
            if let Some(draft) = &mut side.form
                && let Some((pending, _)) = draft.pending
                && pending == request
            {
                draft.pending = None;
                match answer {
                    Answer::Adopted(_) => form::reset(draft, answer),
                    Answer::CoreRefused(_) | Answer::ForgeRefused(_) | Answer::EntranceRefused(_) | Answer::Unknown => {
                        draft.outcome = Some(answer);
                    }
                }
                side.changed();
            }
        }
    }
}
fn act_on(side: &mut Side, limits: Limits, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Go { address } => out.push(Request::Show { address }),
        Act::Adopt => {
            let Some(_party) = side.party else {
                return;
            };
            let Some(draft) = &mut side.form else {
                return;
            };
            if draft.pending.is_some() {
                return;
            }
            match draft.served {
                Served::OutOfDate => return,
                Served::Yes | Served::Unknown => {}
            }
            form::mark(draft, limits.checks);
            if let Some(adoption) = adoption(draft, limits.checks) {
                side.serial = side.serial.checked_add(1).expect("forge request serial fits u64");
                let request = skein_lib::Token::new(side.serial);
                draft.pending = Some((request, skein_lib::RequestProgress::Parked));
                out.push(Request::Send {
                    request,
                    ask: ask(&adoption, &limits).expect("validated adoption fits wire"),
                });
            }
            side.changed();
        }
        Act::Write { field, text, typed } => {
            if let Some(draft) = &mut side.form {
                write(draft, field, text, typed, limits.field_bytes, limits.checks);
                side.changed();
            }
        }
        Act::Choose { role } => {
            if let Some(draft) = &mut side.form {
                draft.role = role;
                draft.outcome = None;
                draft.home.broken = None;
                side.changed();
            }
        }
        Act::Home => {
            if let Some(draft) = &mut side.form {
                draft.home.on = !draft.home.on;
                draft.home.broken = None;
                draft.outcome = None;
                side.changed();
            }
        }
        Act::Ci => {
            if let Some(draft) = &mut side.form {
                draft.ci.on = !draft.ci.on;
                draft.outcome = None;
                side.changed();
            }
        }
    }
}
#[must_use]
pub const fn next_deadline(_side: &Side) -> Option<Time> {
    None
}
pub const fn reclaim(_side: &mut Side) {}
#[must_use]
pub const fn shown(side: &Side) -> u64 {
    side.shown
}
#[must_use]
pub const fn max_out(_limits: &Limits) -> u32 {
    1
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.field_bytes < 7 {
        return None;
    }
    u64::from(limits.field_bytes).checked_mul(6)?.checked_add(u64::from(limits.checks).checked_mul(4)?)
}

fn served(side: &mut Side, served: Served) {
    side.served = served;
    if let Some(draft) = &mut side.form {
        draft.served = side.served;
        if side.served == Served::OutOfDate
            && let Some((request, _)) = draft.pending
        {
            draft.pending = Some((request, skein_lib::RequestProgress::Parked));
        }
    }
    side.changed();
}

/// The forge's receiving limits, shared by its bytes and bounds accounting.
#[must_use]
pub fn wire_limits(limits: &Limits) -> Option<temper_client_protocol_wire::v1::Limits> {
    temper_client_protocol_wire::limits(&temper_client_protocol_wire::Bounds {
        field_bytes: limits.field_bytes,
        checks: limits.checks,
        contexts: limits.checks,
        refusal_bytes: 65_536,
    })
}
pub(crate) fn told(client: &jig_client::Client, limits: &crate::Limits, told: jig_client::Told) -> Option<Event> {
    use jig_client::{Outcome, Told};
    match told {
        Told::Pending { key } => {
            let pending = client.pending(key)?;
            let progress = match pending.progress {
                jig_client::Progress::InFlight => skein_lib::RequestProgress::InFlight,
                jig_client::Progress::Retrying { .. } => skein_lib::RequestProgress::Retrying,
                jig_client::Progress::NotServed | jig_client::Progress::Parked => skein_lib::RequestProgress::Parked,
                jig_client::Progress::Unknown => skein_lib::RequestProgress::Unknown,
            };
            Some(Event::Progress { request: skein_lib::Token::new(key.number), progress })
        }
        Told::Answered { key, outcome } => {
            let request = skein_lib::Token::new(key.number);
            let answer = match outcome {
                Outcome::Answered(answered) => answer(limits, &answered)?,
                Outcome::Unknown | Outcome::Limit | Outcome::Held | Outcome::NotKept | Outcome::Unconfirmed => {
                    Answer::Unknown
                }
            };
            Some(Event::Answered { request, answer })
        }
        Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Link { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => None,
    }
}

/// Encode the validated adoption in its project scope.
#[must_use]
pub fn ask(adoption: &Adoption, limits: &Limits) -> Option<jig_client::Ask> {
    Some(jig_client::Ask::Family {
        scope: jig_client::contract::Scope::Project(adoption.project),
        record: family::request(adoption, &wire_limits(limits)?)?,
    })
}
/// Decode the same terminal answers the adoption side receives.
#[must_use]
pub fn answer(limits: &crate::Limits, answered: &jig_client::Answered) -> Option<Answer> {
    match answered {
        jig_client::Answered::Family(record) => {
            let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).ok()?;
            family::answer(record, &wire_limits(&limits.forge)?, &link)
        }
        jig_client::Answered::Jig(jig_client::contract::Answer::Refused { refusal, .. }) => {
            Some(Answer::EntranceRefused(jig_client::contract::EntranceRefusal::new(refusal.clone()).ok()?))
        }
        jig_client::Answered::Jig(jig_client::contract::Answer::Done(_)) => Some(Answer::Unknown),
    }
}
