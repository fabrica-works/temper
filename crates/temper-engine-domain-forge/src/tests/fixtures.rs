//! Shared adoption form fixture.
use super::*;
pub fn adoption() -> Adoption {
    Adoption {
        project: 5,
        home: true,
        forge: REPO.forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    }
}

/// Play the core after the connector's locate, returning the adoption outputs.
pub fn begin_adoption(d: &mut Domain, request: Token, adoption: Adoption) -> Box<[Request]> {
    outputs(d, Event::Application { request, token: request, retention: Duration::from_secs(60), adoption });
    let sent = ready(d);
    let [Request::Call { call, .. }] = sent.as_ref() else { panic!("one locate call") };
    outputs(
        d,
        Event::Client(skein_forge_client::Event::Answered {
            call: *call,
            cost: 1,
            result: Ok(skein_forge::api::Answer::Located { repository: REPO.repository }),
        }),
    );
    outputs(d, Event::Adopt { request })
}
