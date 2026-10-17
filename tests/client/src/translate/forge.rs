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
    ask: &forge::Ask,
) -> engine::Event {
    let forge::Ask::Adopt(value) = ask;
    let sign_in = transport.sign_in(who).expect("authenticated adoption");
    let reply_to = transport.reply_to(Reply::Keyed {
        client: who,
        attempt,
        key,
        owner: KeyedOwner::Application { connector: crate::fixtures::FORGE },
    });
    engine::Event::Party(PartyInput::Application {
        reply_to,
        sign_in,
        key,
        digest: digest(value),
        request: adoption(value, crate::fixtures::FORGE_HOST),
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
        Delivery::Application { to, outcome: value } => (to, value.as_ref().map(outcome)),
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
            let answer = answer.map(|value| match value {
                jig_client::contract::Answer::Refused(why) => client::Answered::Forge(forge::Answer::EntranceRefused(
                    jig_client::contract::EntranceRefusal::new(why).expect("fixed entrance facts"),
                )),
                jig_client::contract::Answer::Done(_) => panic!("adoption's body comes from its connector"),
            });
            let Reply::Keyed { client, attempt, .. } = transport.take_reply(to.into_token()).expect("original route")
            else {
                panic!("keyed route")
            };
            if envelope == RequestEnvelope::SignedOut {
                transport.signed_out(client);
            }
            return Ok(Some((client, client::Event::Answer { attempt, envelope, answer })));
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
        | Delivery::WatchEnded { .. }) => {
            return Err(Box::new(engine::Request::Party(PartyRequest::Deliver(delivery))));
        }
    };
    let Some(route) = transport.take_reply(to.into_token()) else { return Ok(None) };
    let Reply::Keyed { client, attempt, owner: KeyedOwner::Application { connector }, .. } = route else {
        panic!("application reply route")
    };
    assert_eq!(connector, crate::fixtures::FORGE);
    Ok(Some((
        client,
        client::Event::Answer {
            attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(client::Answered::Forge(body.unwrap_or(forge::Answer::Unknown))),
        },
    )))
}
