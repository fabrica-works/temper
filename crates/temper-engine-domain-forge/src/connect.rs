//! Volatile public connects; only handles cross this boundary (client/README.md, 8).
use crate::{Answered, Ask, ConnectAnswer, Connection, Domain, ForgeRefusal, InstallAnswer, Request};
use alloc::boxed::Box;
use jig_domain_core::Carried;
use jig_domain_people::Role;
use skein_lib::{Map, Queue, Token};

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) answers: Map<Token, Answered>,
    pending: Map<Token, Token>,
    running: Map<u16, (Token, Option<Carried>)>,
}
impl State {
    pub(crate) fn new(bound: u32) -> Self {
        Self {
            answers: Map::with_capacity(bound),
            pending: Map::with_capacity(bound),
            running: Map::with_capacity(bound),
        }
    }
    fn answer(&mut self, request: Token, token: Token, answer: Answered, out: &mut Queue<Request>) {
        if self.answers.len() == self.answers.capacity() && !self.answers.contains_key(&token) {
            let oldest = *self.answers.first().expect("full answers").0;
            self.answers.remove(&oldest);
        }
        self.answers.insert(token, answer).expect("bounded answers");
        out.push(Request::ApplicationDone { request, token });
    }
}
pub(crate) fn worst_case(bound: u32) -> Option<u64> {
    Map::<Token, Answered>::worst_case(bound)?
        .checked_add(Map::<Token, Token>::worst_case(bound)?)?
        .checked_add(Map::<u16, (Token, Option<Carried>)>::worst_case(bound)?)?
        .checked_add(u64::from(bound).checked_mul(2048)?)
}
pub(crate) fn start(
    d: &mut Domain,
    request: Token,
    token: Token,
    ask: Ask,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    let (forge, install) = match &ask {
        Ask::Connect { forge, .. } => (*forge, false),
        Ask::Install { forge, .. } => (*forge, true),
        Ask::Adopt(_) | Ask::SetUp { .. } | Ask::StartDemo => unreachable!("connect admission"),
    };
    let (connected, connecting) = match d.connection(forge) {
        Connection::Connected { .. } => (true, false),
        Connection::Connecting => (false, true),
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => (false, false),
    };
    let refusal = if role != Some(Role::Owner) {
        Some(ForgeRefusal::Role)
    } else if !d.configured.contains_key(&forge) {
        Some(ForgeRefusal::NoSuchForge)
    } else if let Some(config) = d.configured.get(&forge)
        && (config.kind != crate::Kind::GitHub || config.mode == crate::Mode::Fake)
    {
        Some(ForgeRefusal::NotTaken)
    } else if d.connect.running.contains_key(&forge)
        || d.connect.pending.len() == d.connect.pending.capacity()
        || d.connect.running.len() == d.connect.running.capacity()
    {
        Some(ForgeRefusal::Busy)
    } else if install && !connected {
        Some(ForgeRefusal::NotConnected)
    } else if !install && (connected || connecting) {
        Some(ForgeRefusal::Connected)
    } else if !install && carried.is_none() {
        Some(ForgeRefusal::NoKey)
    } else {
        None
    };
    if let Some(why) = refusal {
        if let Some(value) = carried {
            out.push(Request::CarriedUnused { number: value.number });
        }
        let answer = if install {
            Answered::Install(InstallAnswer::Refused(why))
        } else {
            Answered::Connect(ConnectAnswer::Refused(why))
        };
        d.connect.answer(request, token, answer, out);
        return;
    }
    if d.ended.remove(&forge).is_some() {
        out.push(Request::MarkedForges);
    }
    d.connect.pending.insert(request, token).expect("room checked");
    match ask {
        Ask::Connect { app, .. } => {
            let carried = carried.expect("admitted key");
            d.connect.running.insert(forge, (request, Some(carried))).expect("room checked");
            let kept = match d.connections.get(&forge) {
                Some((_, kept)) => *kept,
                None => None,
            };
            d.connections.insert(forge, (Connection::Connecting, kept)).expect("configured forge");
            out.push(Request::MarkedForges);
            out.push(Request::Connect { forge, request, app, carried });
        }
        Ask::Install { back, .. } => {
            if let Some(value) = carried {
                out.push(Request::CarriedUnused { number: value.number });
            }
            d.connect.running.insert(forge, (request, None)).expect("room checked");
            out.push(Request::Install { forge, request, back });
        }
        Ask::Adopt(_) | Ask::SetUp { .. } | Ask::StartDemo => unreachable!("connect admission"),
    }
}
pub(crate) fn connected(d: &mut Domain, forge: u16, request: Token, answer: ConnectAnswer, out: &mut Queue<Request>) {
    if let Some(token) = d.connect.pending.remove(&request) {
        if let ConnectAnswer::Refused(_) = answer
            && let Some((held, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            if let Some((_, Some(carried))) = d.connect.running.remove(&forge) {
                out.push(Request::CarriedUnused { number: carried.number });
            }
            let kept = match d.connections.get(&forge) {
                Some((_, kept)) => *kept,
                None => None,
            };
            d.connections.insert(forge, (Connection::NotConnected, kept)).expect("configured forge");
            out.push(Request::MarkedForges);
        }
        d.connect.answer(request, token, Answered::Connect(answer), out);
    }
}
pub(crate) fn installed(
    d: &mut Domain,
    forge: u16,
    request: Token,
    answer: InstallAnswer,
    stopped: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    if let Some(token) = d.connect.pending.remove(&request) {
        if let Some((held, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            d.connect.running.remove(&forge);
            if let Some(why) = stopped {
                stopped_end(d, forge, &why, out);
            }
        }
        d.connect.answer(request, token, Answered::Install(answer), out);
    }
}
pub(crate) fn ended(
    d: &mut Domain,
    forge: u16,
    request: Token,
    connected: bool,
    why: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    if let Some((held, _)) = d.connect.running.get(&forge)
        && *held == request
    {
        if let Some((_, Some(carried))) = d.connect.running.remove(&forge) {
            out.push(Request::EraseCarried { number: carried.number });
        }
        if !connected {
            if let Some(why) = why {
                stopped_end(d, forge, &why, out);
            }
            let kept = match d.connections.get(&forge) {
                Some((_, kept)) => *kept,
                None => None,
            };
            d.connections.insert(forge, (Connection::NotConnected, kept)).expect("configured forge");
            out.push(Request::MarkedForges);
        }
    }
}

/// Volatile per-forge diagnostics, bounded in bytes without splitting UTF-8 (client/forge.md, 5.4).
fn stopped_end(d: &mut Domain, forge: u16, why: &[u8], out: &mut Queue<Request>) {
    // The host supplies UTF-8. Back over continuation bytes at the cut;
    // text validation belongs to the protocol boundary.
    let mut end = why.len().min(crate::CONNECT_WHY_BYTES);
    while end > 0 {
        let Some(byte) = why.get(end) else { break };
        if byte & 0xc0 != 0x80 {
            break;
        }
        end = end.saturating_sub(1);
    }
    d.ended.insert(forge, Box::from(why.get(..end).expect("bounded reason prefix"))).expect("configured forge ends");
    out.push(Request::MarkedForges);
}
