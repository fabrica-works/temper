//! The forge application's complete typed request and terminal bridge.
use jig_client_world::{
    harness::ClientId,
    joint::{
        transport::{KeyedOwner, Reply, Transport},
        wire,
    },
};
use skein_lib::{RequestEnvelope, Token};
use temper_client_domain::{self as client, forge};
use temper_engine_domain::{self as engine, Delivery, PartyInput, PartyRequest};
use temper_engine_domain_forge as owner;
#[must_use]
pub fn adoption(value: &forge::Adoption, host: &[u8]) -> owner::Adoption {
    owner::Adoption {
        project: value.project,
        home: value.home,
        forge: value.forge,
        host: host.into(),
        owner: value.owner.clone(),
        name: value.name.clone(),
        prefix: value.prefix.clone(),
        role: match value.role {
            forge::Role::Owned => owner::Role::Owned,
            forge::Role::Adopted => owner::Role::Adopted,
            forge::Role::Fork => owner::Role::Fork,
            forge::Role::Context => owner::Role::Context,
        },
        landing: value.landing.clone(),
        ci: value.ci,
        checks: value.checks.clone(),
    }
}
#[must_use]
pub fn outcome(value: &owner::ApplicationOutcome) -> forge::Answer {
    match value {
        owner::ApplicationOutcome::Adopted(value) => forge::Answer::Adopted(forge::Adopted {
            forge: value.repository.forge,
            repository: value.repository.repository,
            owner: value.owner.clone(),
            name: value.name.clone(),
            role: match value.role {
                owner::Role::Owned => forge::Role::Owned,
                owner::Role::Adopted => forge::Role::Adopted,
                owner::Role::Fork => forge::Role::Fork,
                owner::Role::Context => forge::Role::Context,
            },
            home: value.home,
            kinds: forge::Kinds {
                read: value.kinds.read,
                push: value.kinds.push,
                open: value.kinds.open,
                land: value.kinds.land,
                review: value.kinds.review,
                status: value.kinds.status,
                comment: value.kinds.comment,
                issue: value.kinds.issue,
                branch: value.kinds.branch,
            },
            protection: match &value.protection {
                owner::Protection::Unknown => forge::Protection::Unknown,
                owner::Protection::Absent => forge::Protection::Absent,
                owner::Protection::Rule(value) => forge::Protection::Rule(forge::Rule {
                    branch: value.branch.clone(),
                    contexts: value.contexts.clone(),
                    approvals: value.approvals,
                    dismiss_stale: value.dismiss_stale,
                }),
            },
        }),
        owner::ApplicationOutcome::CoreRefused(why) => forge::Answer::CoreRefused(wire::refusals::refusal(*why)),
        owner::ApplicationOutcome::ForgeRefused(why) => forge::Answer::ForgeRefused(match why {
            owner::AdoptionRefusal::NoSuchForge => forge::Reason::NoSuchForge,
            owner::AdoptionRefusal::NotFound => forge::Reason::NotFound,
            owner::AdoptionRefusal::NoAccess => forge::Reason::NoAccess,
            owner::AdoptionRefusal::Refused => forge::Reason::Refused,
            owner::AdoptionRefusal::InFlight => forge::Reason::InFlight,
            owner::AdoptionRefusal::TooLarge => forge::Reason::TooLarge,
            owner::AdoptionRefusal::Unreachable => forge::Reason::Unreachable,
            owner::AdoptionRefusal::InvalidAnswer => forge::Reason::InvalidAnswer,
        }),
    }
}
#[must_use]
pub fn digest(value: &forge::Adoption) -> [u8; 32] {
    let mut bytes = b"temper-adoption/request/1".to_vec();
    bytes.extend_from_slice(&crate::fixtures::FORGE.to_be_bytes());
    bytes.extend_from_slice(&value.project.to_be_bytes());
    bytes.extend_from_slice(&value.forge.to_be_bytes());
    bytes.extend_from_slice(&[
        u8::from(value.home),
        u8::from(value.ci),
        match value.role {
            forge::Role::Owned => 0,
            forge::Role::Adopted => 1,
            forge::Role::Fork => 2,
            forge::Role::Context => 3,
        },
    ]);
    for text in [&value.owner, &value.name, &value.prefix, &value.landing] {
        bytes.extend_from_slice(&u64::try_from(text.len()).expect("bounded field").to_be_bytes());
        bytes.extend_from_slice(text);
    }
    bytes.extend_from_slice(&u64::try_from(value.checks.len()).expect("bounded checks").to_be_bytes());
    for check in &value.checks {
        bytes.extend_from_slice(&check.to_be_bytes());
    }
    let mut writer = skein_lib::Writer::new(32);
    for word in skein_io::digest::digest(&bytes).0 {
        writer.put(&word.to_be_bytes()).expect("32-byte SHA-256");
    }
    writer.finish().as_ref().try_into().expect("32-byte SHA-256")
}
#[must_use]
pub fn request(
    transport: &mut Transport,
    who: ClientId,
    attempt: Token,
    key: [u8; 16],
    scope: jig_client::contract::Scope,
    ask: &jig_client::Family,
) -> engine::Event {
    use temper_client_protocol_wire::{self as w, v1};
    assert_eq!(w::owner(ask.family), Some(w::Owner::Forge));
    assert_eq!(ask.version, w::CLIENT_VERSION);
    let limits = forge::wire_limits(&crate::app::limits().forge).expect("wire bounds");
    let mut reader = skein_lib::Reader::new(&ask.bytes);
    let a = v1::Adopt::decode(&limits, &mut reader).expect("whole adoption bytes").into_parts();
    assert!(reader.is_empty());
    let value = forge::Adoption {
        project: a.project,
        home: a.home,
        forge: a.forge,
        owner: a.owner,
        name: a.name,
        prefix: a.prefix,
        landing: a.landing,
        ci: a.ci,
        checks: a.checks.into_boxed(),
        role: match a.role {
            v1::Role::Owned => forge::Role::Owned,
            v1::Role::Adopted => forge::Role::Adopted,
            v1::Role::Fork => forge::Role::Fork,
            v1::Role::Context => forge::Role::Context,
        },
    };
    let sign_in = transport.sign_in(who).expect("authenticated adoption");
    let reply_to = transport.reply_to(Reply::Keyed {
        client: who,
        attempt,
        key,
        owner: KeyedOwner::Application { connector: crate::fixtures::FORGE },
    });
    engine::Event::Party(PartyInput::Application {
        scope: match scope {
            jig_client::contract::Scope::Project(project) => jig_domain_people::ApplicationScope::Project(project),
            jig_client::contract::Scope::Deployment => jig_domain_people::ApplicationScope::Deployment,
            jig_client::contract::Scope::Own => jig_domain_people::ApplicationScope::Own,
        },
        reply_to,
        sign_in,
        key,
        digest: digest(&value),
        carried: None,
        request: adoption(&value, crate::fixtures::FORGE_HOST),
    })
}
/// # Errors
/// Returns every request belonging to another owner unchanged.
pub fn incoming(
    transport: &mut Transport,
    request: engine::Request,
) -> Result<Option<(ClientId, client::Event)>, Box<engine::Request>> {
    let delivery = match request {
        engine::Request::Party(PartyRequest::Deliver(delivery)) => delivery,
        request @ (engine::Request::Store(_)
        | engine::Request::Worker(_)
        | engine::Request::Forge { .. }
        | engine::Request::Account(_)
        | engine::Request::ToChild(_)
        | engine::Request::Busy(_)
        | engine::Request::Drained
        | engine::Request::Stop
        | engine::Request::Party(
            PartyRequest::SignInRefused { .. } | PartyRequest::WatchRefused { .. } | PartyRequest::RunText { .. },
        )) => return Err(Box::new(request)),
    };
    let (to, body) = match delivery {
        Delivery::Application { to, outcome: value } => (to, value),
        Delivery::WebReply { to, reply } => {
            // Preserve the affine answer right until its route is known.
            let matching = transport.reply_tokens().into_iter().find(|token| to == skein_lib::ReplyTo::new(*token));
            let Some(token) = matching else {
                return Err(Box::new(engine::Request::Party(PartyRequest::Deliver(Delivery::WebReply { to, reply }))));
            };
            if !matches!(transport.reply(token), Some(Reply::Keyed { owner: KeyedOwner::Application { .. }, .. })) {
                return Err(Box::new(engine::Request::Party(PartyRequest::Deliver(Delivery::WebReply { to, reply }))));
            }
            let (envelope, answer) =
                wire::answers::keyed(wire::answers::reply(reply)).expect("application admission disposition");
            let Reply::Keyed { client, attempt, .. } = transport.take_reply(to.into_token()).expect("original route")
            else {
                panic!("keyed route")
            };
            if envelope == RequestEnvelope::SignedOut {
                transport.signed_out(client);
            }
            return Ok(Some((
                client,
                client::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            envelope,
                            answer: answer.map(jig_client::Answered::Jig),
                            why: None,
                        },
                    },
                },
            )));
        }
        delivery @ (Delivery::Result { .. }
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
        | Delivery::Read { .. }
        | Delivery::Document { .. }
        | Delivery::WatchEnded { .. }
        | Delivery::CarriedUnused { .. }) => {
            return Err(Box::new(engine::Request::Party(PartyRequest::Deliver(delivery))));
        }
    };
    let Some(route) = transport.take_reply(to.into_token()) else { return Ok(None) };
    Ok(Some(answered(&route, body)))
}
/// The application's outcome, answered on its keyed route as the forge family's bytes.
fn answered(route: &Reply, body: Option<owner::ApplicationOutcome>) -> (ClientId, client::Event) {
    let Reply::Keyed { client, attempt, owner: KeyedOwner::Application { connector }, .. } = *route else {
        panic!("application reply route")
    };
    assert_eq!(connector, crate::fixtures::FORGE);
    (
        client,
        client::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: connector,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_service::family::answer(
                            body,
                            &forge::wire_limits(&crate::app::limits().forge).expect("wire bounds"),
                            &jig_client_protocol_link::Limits::new(
                                crate::app::limits().client.domain,
                                crate::app::limits().client.exchanges,
                            )
                            .expect("link bounds")
                            .wire,
                        )
                        .expect("whole forge answer"),
                    })),
                },
            },
        },
    )
}
