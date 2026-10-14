use jig_domain_client::contract;
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
