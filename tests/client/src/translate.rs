//! One exhaustive bridge between jig's client contract and temper's party boundary.
use jig_client::{Ask, Down, Readable, Watchable};
use jig_client_world::{
    harness::ClientId,
    joint::{
        packets, read_packets,
        transport::{KeyedOwner, Reply, Transport, Watched},
        wire,
    },
};
use jig_domain_core as core;
use jig_domain_people as people;
use temper_client_domain::{Event, Request};
use temper_engine_domain::{self as engine, Delivery, PartyInput, PartyRequest};
pub mod forge;

/// Translate only party inputs the client protocol makes; other core work stays inside the engine.
#[must_use]
pub fn party(event: core::Event) -> engine::Event {
    engine::Event::Party(match event {
        core::Event::SignIn { reply_to, identity } => PartyInput::SignedIn { reply_to, identity },
        core::Event::People(people::Event::Ask { reply_to, sign_in, key, ask }) => {
            PartyInput::Ask { reply_to, sign_in, key, ask }
        }
        core::Event::People(people::Event::SignOut { reply_to, sign_in }) => PartyInput::SignOut { reply_to, sign_in },
        core::Event::People(event) => unsupported_people(&event),
        core::Event::Key { reply_to, sign_in, key, ask, carried } => {
            PartyInput::Key { reply_to, sign_in, key, ask, carried }
        }
        core::Event::Read { reply, sign_in, read } => PartyInput::Read { reply_to: reply, sign_in, read },
        core::Event::Watch { watcher, sign_in, subject, window } => {
            PartyInput::Watch { watcher, sign_in, subject, window }
        }
        core::Event::WatchRun { watcher, sign_in, task, turn, offset } => {
            PartyInput::WatchRun { watcher, sign_in, task, turn, offset }
        }
        core::Event::Unwatch { watcher } => PartyInput::Unwatch { watcher },
        core::Event::Delivered { watcher } => PartyInput::Delivered { watcher },
        core::Event::StreamDelivered { watcher } => PartyInput::StreamDelivered { watcher },
        core::Event::SubscribeTopic { .. } => panic!("the client kit does not make SubscribeTopic"),
        core::Event::Fill { .. } => panic!("the client kit does not make Fill"),
        core::Event::Stopping => panic!("the client kit does not make Stopping"),
        core::Event::Marked { .. } => panic!("the client kit does not make Marked"),
        core::Event::ReadLoaded { .. } => panic!("the client kit does not make ReadLoaded"),
        core::Event::Application { .. } => panic!("the client kit does not make Application"),
        core::Event::ApplicationPart { .. } => panic!("the client kit does not make ApplicationPart"),
        core::Event::ApplicationFinished { .. } => panic!("the client kit does not make ApplicationFinished"),
        core::Event::ProjectionSettled { .. } => panic!("the client kit does not make ProjectionSettled"),
        core::Event::DecidedProposal { .. } => panic!("the client kit does not answer archived proposals"),
        core::Event::SettledCall { .. } => panic!("the client kit does not make SettledCall"),
        core::Event::EffectStart { .. } => panic!("the client kit does not make EffectStart"),
        core::Event::EffectConnector(..) => panic!("the client kit does not make EffectConnector"),
        core::Event::EffectDeadline => panic!("the client kit does not make EffectDeadline"),
        core::Event::RunCall { .. } => panic!("the client kit does not make RunCall"),
        core::Event::RunText { .. } => panic!("the client kit does not make RunText"),
        core::Event::Tasks(..) => panic!("the client kit does not make Tasks"),
        core::Event::StartRecurring { .. } => panic!("the client kit does not make StartRecurring"),
        core::Event::StartBrief { .. } => panic!("the client kit does not make StartBrief"),
        core::Event::BriefNotes { .. } => panic!("the client kit does not make BriefNotes"),
        core::Event::BriefAssembled { .. } => panic!("the client kit does not make BriefAssembled"),
        core::Event::WorkspacePrepared { .. } => panic!("the client kit does not make WorkspacePrepared"),
        core::Event::ClaimPrepared { .. } => panic!("the client kit does not make ClaimPrepared"),
        core::Event::ClaimRefused { .. } => panic!("the client kit does not make ClaimRefused"),
        core::Event::ConnectorRepair { .. } => panic!("the client kit does not make ConnectorRepair"),
        core::Event::Period { .. } => panic!("the client kit does not make Period"),
        core::Event::PreparationFailed { .. } => panic!("the client kit does not make PreparationFailed"),
        core::Event::TranscriptUnread { .. } => panic!("the client kit does not make TranscriptUnread"),
        core::Event::PersonProposal(..) => panic!("the client kit does not make PersonProposal"),
        core::Event::PersonEscalation(..) => panic!("the client kit does not make PersonEscalation"),
        core::Event::TaskEscalation(..) => panic!("the client kit does not make TaskEscalation"),
        core::Event::Fleet(..) => panic!("the client kit does not make Fleet"),
        core::Event::TurnPayload { .. } => panic!("the client kit does not make TurnPayload"),
        core::Event::AnswerPayload { .. } => panic!("the client kit does not make AnswerPayload"),
        core::Event::AcceptedTurn { .. } => panic!("the client kit does not make AcceptedTurn"),
        core::Event::RefusedPayload { .. } => panic!("the client kit does not make RefusedPayload"),
        core::Event::Activate { .. } => panic!("the client kit does not make Activate"),
        core::Event::PreparedAgent { .. } => panic!("the client kit does not make PreparedAgent"),
        core::Event::Forget { .. } => panic!("the client kit does not make Forget"),
        core::Event::SeenConversationLoaded { .. } => panic!("the client kit does not make SeenConversationLoaded"),
        core::Event::EndedReadLoaded { .. } => panic!("the client kit does not make EndedReadLoaded"),
        core::Event::PartyInputsLoaded { .. } => panic!("the client kit does not make PartyInputsLoaded"),
        core::Event::HistoricalProposal { .. } => panic!("the client kit does not make HistoricalProposal"),
        core::Event::HistoricalPersonTask { .. } => panic!("the client kit does not make HistoricalPersonTask"),
        core::Event::HistoricalEscalation { .. } => panic!("the client kit does not make HistoricalEscalation"),
        core::Event::NamedAnswer { .. } => panic!("the client kit does not make NamedAnswer"),
        core::Event::NamedAction { .. } => panic!("the client kit does not make NamedAction"),
        core::Event::DelegateValidated { .. } => panic!("the client kit does not make DelegateValidated"),
        core::Event::DelegateInputFailed { .. } => panic!("the client kit does not make DelegateInputFailed"),
        core::Event::DelegateHoldings { .. } => panic!("the client kit does not make DelegateHoldings"),
        core::Event::ProcedureHoldings { .. } => panic!("the client kit does not make ProcedureHoldings"),
        core::Event::ProcedureStep { .. } => panic!("the client kit does not make ProcedureStep"),
        core::Event::Brief(..) => panic!("the client kit does not make Brief"),
        core::Event::Holdings { .. } => panic!("the client kit does not make Holdings"),
        core::Event::Account(..) => panic!("the client kit does not make Account"),
        core::Event::View(..) => panic!("the client kit does not make View"),
        core::Event::Notes(..) => panic!("the client kit does not make Notes"),
    })
}
fn unsupported_people(event: &people::Event) -> PartyInput {
    match event {
        people::Event::MoveChat { .. } => panic!("the client kit does not make People::MoveChat"),
        people::Event::ChatState { .. } => panic!("the client kit does not make People::ChatState"),
        people::Event::ConversationSeen { .. } => panic!("the client kit does not make People::ConversationSeen"),
        people::Event::Forget { .. } => panic!("the client kit does not make People::Forget"),
        people::Event::MakeService { .. } => panic!("the client kit does not make People::MakeService"),
        people::Event::Seed { .. } => panic!("the client kit does not make People::Seed"),
        people::Event::Waiting { .. } => panic!("the client kit does not make People::Waiting"),
        people::Event::ApplyRoles { .. } => panic!("the client kit does not make People::ApplyRoles"),
        people::Event::SignedIn { .. } => panic!("the client kit does not make People::SignedIn"),
        people::Event::SignOut { .. } => panic!("the client kit does not make People::SignOut"),
        people::Event::Roles { .. } => panic!("the client kit does not make People::Roles"),
        people::Event::Ask { .. } => panic!("the client kit does not make People::Ask"),
        people::Event::Decided { .. } => panic!("the client kit does not make People::Decided"),
        people::Event::Restore { .. } => panic!("the client kit does not make People::Restore"),
        people::Event::Restored => panic!("the client kit does not make People::Restored"),
    }
}
/// Retain native shell tokens, then make the real engine's party input.
#[must_use]
pub fn request(
    transport: &mut Transport,
    who: ClientId,
    request: &Request,
    limits: &jig_client::DomainLimits,
) -> Vec<engine::Event> {
    let mut events = Vec::new();
    let Request::Client { request: jig_client::Request::Down { down } } = request else { panic!("typed wire request") };
    match down {
        Down::Send { attempt, key, request: ask, value } => {
            // Temper's client carries no value with an ask yet (jig's client.md, 5).
            assert!(value.is_none(), "no carried value");
            let sign_in = transport.sign_in(who).expect("authenticated keyed request");
            let ask = match ask {
                Ask::Jig(ask) => ask,
                Ask::Family { record: ask, scope } => {
                    events.push(forge::request(transport, who, *attempt, *key, *scope, ask));
                    return events;
                }
            };
            let ask = wire::asks::ask(ask).expect("whole jig ask translates");
            let reply_to =
                transport.reply_to(Reply::Keyed { client: who, attempt: *attempt, key: *key, owner: KeyedOwner::Jig });
            events.push(party(core::Event::People(people::Event::Ask { reply_to, sign_in, key: *key, ask })));
        }

        Down::Read { read, what: Readable::Jig(what) } => {
            let sign_in = transport.sign_in(who).expect("authenticated read");
            let owner = wire::reads::read(what, limits.text).expect("whole read translates");
            let reply = transport.reply_to(Reply::Read { client: who, read: *read, what: what.clone() });
            events.push(party(core::Event::Read { reply, sign_in, read: owner }));
        }
        Down::Open { watch, subject: Watchable::Jig(subject) } => {
            let sign_in = transport.sign_in(who).expect("authenticated watch");
            let watcher = transport.open(who, *watch, Watched::Jig(subject.clone()));
            events.push(party(
                wire::watches::watch(watcher, sign_in, subject, limits.rows).expect("whole watch translates"),
            ));
        }
        Down::Continue { watch, subject } => {
            if let Some(event) = transport.continued(who, *watch, subject.clone()).expect("original run continuation") {
                events.push(party(event));
            }
        }
        Down::Close { watch } => {
            if let Some(watcher) = transport.watch_token(who, *watch) {
                let terminals = transport.close_watch(watcher);
                events.push(party(core::Event::Unwatch { watcher }));
                events.extend(terminals.into_iter().map(party));
            }
        }
        Down::Read { what: Readable::Family { .. }, .. } | Down::Open { subject: Watchable::Family { .. }, .. } => {
            panic!("no forge reads or watches")
        }
        Down::Save { .. } | Down::Erase { .. } => panic!("shell duty never goes on wire"),
    }
    events
}
/// A released root output reaches the kit through its original party route.
#[must_use]
pub fn incoming(
    transport: &mut Transport,
    request: engine::Request,
    limits: &jig_client::DomainLimits,
    codec: &jig_protocol_transcript::Limits,
) -> Option<packets::Incoming> {
    match request {
        engine::Request::Party(request) => match request {
            PartyRequest::Deliver(delivery) => delivery_packet(transport, delivery, limits, codec),
            PartyRequest::SignInRefused { to } => {
                let Some(Reply::SignIn { client }) = transport.take_reply(to.into_token()) else { return None };
                Some(packets::Incoming::SignInFailed { client, failure: packets::SignInFailure::Capacity })
            }
            PartyRequest::WatchRefused { watcher, why } => transport
                .watch(watcher)
                .is_some()
                .then(|| packets::refused(transport, watcher, why).expect("original refused watch")),
            PartyRequest::RunText { watcher, turn, from, to, text, cut, call } => {
                transport.watch(watcher)?;
                let writing = jig_domain_views::Writing { turn, from, to, text, cut, call };
                Some(packets::stream(transport, watcher, &writing, limits).expect("whole run stream translates"))
            }
        },
        engine::Request::Store(_)
        | engine::Request::Worker(_)
        | engine::Request::Forge { .. }
        | engine::Request::Account(_)
        | engine::Request::ToChild(_)
        | engine::Request::Busy(_)
        | engine::Request::Drained
        | engine::Request::Stop => None,
    }
}
fn delivery_packet(
    transport: &mut Transport,
    delivery: Delivery,
    limits: &jig_client::DomainLimits,
    codec: &jig_protocol_transcript::Limits,
) -> Option<packets::Incoming> {
    match delivery {
        Delivery::WebReply { to, reply } => {
            let to = to.into_token();
            transport
                .reply(to)
                .is_some()
                .then(|| packets::people_reply(transport, to, reply, limits).expect("whole people reply translates"))
        }
        Delivery::Read { reply, answer } => {
            let reply = reply.into_token();
            transport.reply(reply).is_some().then(|| {
                read_packets::read_with_codec(transport, reply, &answer, limits, codec).expect("whole read translates")
            })
        }
        Delivery::Document { watcher, document } => transport.watch(watcher).is_some().then(|| {
            packets::document(transport, watcher, &document, limits)
                .unwrap_or_else(|error| panic!("whole jig document translates: {error:?}; original {document:?}"))
        }),
        Delivery::WatchEnded { watcher, why } => transport
            .watch(watcher)
            .is_some()
            .then(|| packets::ended(transport, watcher, why).expect("original watch terminal")),
        // Jig has no separate live requester Result; the task document carries it.
        Delivery::Application { .. }
        | Delivery::Result { .. }
        | Delivery::Host(_)
        | Delivery::Link { .. }
        | Delivery::ForgeCommitted { .. }
        | Delivery::ForgeCall { .. }
        | Delivery::Procedure { .. }
        | Delivery::Relay { .. }
        | Delivery::ReadEscalationDecision { .. }
        | Delivery::Fleet(_)
        | Delivery::Assigned { .. }
        | Delivery::ReadLoad { .. }
        | Delivery::Drained
        | Delivery::Refuse { .. }
        | Delivery::TurnBusy { .. }
        | Delivery::Load { .. }
        | Delivery::CarriedUnused { .. } => None,
    }
}
/// The kit's shared packet carrier has a test application owner; only jig belongs here.
#[must_use]
pub fn callback(event: jig_client_test_domain::Event) -> Event {
    match event {
        jig_client_test_domain::Event::Client { event } => Event::Client { event },
        jig_client_test_domain::Event::Jig(_) => unreachable!("joint transport delivers client events"),
        jig_client_test_domain::Event::SignedOut => {
            Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }
        }
        jig_client_test_domain::Event::Start { .. }
        | jig_client_test_domain::Event::Went { .. }
        | jig_client_test_domain::Event::Act { .. }
        | jig_client_test_domain::Event::First(_) => panic!("no test application event belongs to temper"),
    }
}
