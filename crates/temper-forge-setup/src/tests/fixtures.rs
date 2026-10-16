use crate::{
    Setup,
    check::{Forge, Members, User},
    limits,
    problem::Problems,
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
use skein_json::Document;
use skein_lib::{Writer, bytes};
pub const TWO: &[u8] = br#"[
{"forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["acme"],"mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}},
{"forge":2,"directory":"forge-2","bot":{"login":"temper","email":"temper@example.invalid"},"orgs":["work"],"mode":{"owned":{"forgejo":"/opt/forgejo/forgejo","runner":{"binary":"/opt/forgejo/forgejo-runner","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"},"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"ada","email":"ada@example.invalid"}}}}
]"#;
pub fn document(bytes: &[u8]) -> Document {
    jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("test JSON")
}
pub fn setup() -> Setup {
    Setup::read(&document(TWO), 0, &limits::small(), &mut Problems::empty()).expect("design's setup")
}
pub fn members() -> Members {
    Members {
        forges: Box::new([
            Forge {
                number: 1,
                api: None,
                plaintext: false,
                git: bytes::copy_of(b"https://forge.example"),
                user: User::File(bytes::copy_of(b"forge-1/user")),
                api_token: bytes::copy_of(b"forge-1/api_token"),
                identity_token: bytes::copy_of(b"forge-1/git_token"),
                identity_login: bytes::copy_of(b"temper"),
            },
            Forge {
                number: 2,
                api: Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
                plaintext: true,
                git: bytes::copy_of(b"http://127.0.0.1:3000"),
                user: User::File(bytes::copy_of(b"forge-2/user")),
                api_token: bytes::copy_of(b"forge-2/api_token"),
                identity_token: bytes::copy_of(b"forge-2/git_token"),
                identity_login: bytes::copy_of(b"temper"),
            },
        ]),
        listeners: Box::new([]),
    }
}
pub fn changed(from: &[u8], to: &[u8]) -> Box<[u8]> {
    let at = bytes::find(TWO, from).expect("fixture text exists");
    let after = at.checked_add(from.len()).unwrap();
    let len = TWO.len().checked_sub(from.len()).unwrap().checked_add(to.len()).unwrap();
    let mut w = Writer::new(len);
    for part in [TWO.get(..at).unwrap(), to, TWO.get(after..).unwrap()] {
        w.put(part).unwrap();
    }
    w.finish()
}
pub fn owned() -> crate::setup::Owned {
    match setup().entries[1].mode.clone() {
        crate::setup::Mode::Owned(owned) => owned,
        crate::setup::Mode::Managed { .. } => panic!("owned fixture"),
    }
}
pub fn place() -> crate::place::Place {
    crate::place::Place { directory: b"/srv/forge-2".as_slice().into(), user: b"ada".as_slice().into() }
}
pub fn secrets() -> crate::state::Secrets {
    use skein_lib::Secret;
    crate::state::Secrets {
        secret_key: Secret::new(b"key-secret".as_slice().into()),
        internal_token: Secret::new(b"internal-secret".as_slice().into()),
        jwt_secret: Secret::new(b"jwt-secret".as_slice().into()),
    }
}
