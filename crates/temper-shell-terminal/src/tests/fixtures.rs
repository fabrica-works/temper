use jig_domain_client::contract;
use jig_protocol_client_link as link;
use skein_lib::{RequestRecord, Token};
use temper_client_domain as client;
pub const ATTEMPT: Token = Token::new(101);
pub const READ: Token = Token::new(102);
pub const WATCH: Token = Token::new(103);
pub const SERVICE: u64 = 7;
pub const CREDENTIAL: &[u8] = b"one-time-service-credential";
pub fn jig_record() -> jig_domain_client::store::Record {
    jig_domain_client::store::Record::Place {
        version: jig_domain_client::store::STORE_VERSION,
        party: SERVICE,
        address: jig_domain_client::address::Address::Inbox { project: None },
    }
}
pub fn table_record() -> client::store::Record {
    client::store::Record::Table {
        version: client::store::VERSION,
        record: RequestRecord {
            key: [9; 16],
            scope: SERVICE,
            first_sent: None,
            request: client::Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })),
        },
    }
}
pub fn inbox_read() -> contract::Read {
    contract::Read::Inbox { project: None, group: contract::InboxReadGroup::Needs, kind: None, goal: None, after: None }
}
pub fn family() -> crate::route::family::Family {
    crate::limits::family(&crate::limits::native()).expect("bounded fixture family")
}
pub fn wire_limits() -> temper_protocol_client_wire::v1::Limits {
    family().wire
}
pub fn adoption() -> client::forge::Adoption {
    client::forge::Adoption {
        project: 1,
        forge: 1,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        prefix: Box::from(*b"temper/"),
        landing: Box::from(*b"main"),
        role: client::forge::Role::Owned,
        home: true,
        ci: true,
        checks: Box::from([1, 2]),
    }
}
pub fn adopted(protection: client::forge::Protection) -> client::forge::Adopted {
    client::forge::Adopted {
        forge: 1,
        repository: 7,
        owner: Box::from(*b"acme"),
        name: Box::from(*b"shop"),
        role: client::forge::Role::Owned,
        home: true,
        kinds: client::forge::Kinds {
            read: true,
            push: false,
            open: true,
            land: false,
            review: true,
            status: false,
            comment: true,
            issue: false,
            branch: true,
        },
        protection,
    }
}
pub fn send_adoption(attempt: Token) -> client::Request {
    client::Request::Send { attempt, key: [9; 16], ask: client::Ask::Forge(client::forge::Ask::Adopt(adoption())) }
}
pub fn final_answer(attempt: Token, answer: link::Answered) -> link::Request {
    link::Request::Answer { attempt, envelope: skein_lib::RequestEnvelope::Final, answer: Some(answer) }
}
pub fn role_refusal() -> contract::Refusal {
    contract::Refusal::Role { held: None }
}
pub fn entrance(refusal: contract::Refusal) -> contract::EntranceRefusal {
    contract::EntranceRefusal::new(refusal).expect("fixed entrance facts")
}
pub fn core_refusal() -> Box<[u8]> {
    use jig_protocol_client_wire as w;
    let limits = family().link.wire;
    let value = w::Refusal::Role(w::RefusalRole::new(&limits, w::RefusalRoleParts { held: None }).expect("role fits"));
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measure fits"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}
