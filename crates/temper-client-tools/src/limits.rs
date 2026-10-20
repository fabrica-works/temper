//! The tools' preset and host bounds (client/tools.md, 12).
pub use crate::jig::call::KEY_BYTES;
use skein_lib::{Duration, RequestLimits};
use temper_client_domain as client;
pub const ROWS: u32 = 32;
pub const TEXT: u32 = 32_768;
pub const PART: u32 = 16_384;
pub const DOCUMENT: u64 = 1_048_576;
pub const DEADLINE: Duration = Duration::from_secs(30);
pub const LOCK: Duration = Duration::from_secs(60);
pub const LOCK_TRY: Duration = Duration::from_millis(100);
pub const WAIT: Duration = Duration::from_secs(300);
pub const WAIT_MOST: Duration = Duration::from_secs(3600);
/// The terminal's client preset, with the tools' rows and result text.
#[must_use]
pub fn client() -> client::Limits {
    let requests = RequestLimits {
        requests: 16,                   // Durable keyed requests.
        bytes: 16_777_216,              // Whole request-table payload bytes.
        out: 32,                        // Table outputs per iteration.
        first: Duration::from_secs(1),  // First request retry span.
        most: Duration::from_secs(30),  // Longest request retry span.
        margin: Duration::from_secs(1), // Retention's safety margin.
    };
    let client = jig_client::Limits {
        domain: jig_client::DomainLimits {
            owners: &[jig_client::Spoken {
                family: temper_client_protocol_wire::FORGE,
                version: temper_client_protocol_wire::CLIENT_VERSION,
            }],
            requests,
            ask_bytes: requests.bytes,
            handles: 17,
            watches: 16, // Includes the shared me watch.
            turn_bytes: 1_048_576,
            backoff: jig_client::Backoff { first: Duration::from_secs(1), most: Duration::from_secs(30) },
            authority: 12,
            heartbeat: Duration::from_secs(30),
            reads: 16,
            parts: 16,
            delegates: 8,
            document_bytes: DOCUMENT,
            records: 16,
            kept: 16,
            projects: 2,
            text: TEXT,
            rows: ROWS,
            fields: 16,
            segments: 9,
        },
        exchanges: 8,
    };

    client::Limits {
        forge: client::forge::Limits { field_bytes: 256, checks: 8 },
        client,
        // A page shows sixteen accounts per holder, eight sources/uses each, four live sign-ins.
        // These modest display counts are independent of task windows and document byte capacity.
        accounts: client::limits::AccountLimits { accounts: 16, sources: 8, uses: 8, sign_ins: 4 },
    }
}
/// Bounds consumed by the generic tools.
#[must_use]
pub fn tools() -> crate::jig::Limits {
    let client = client().client.domain;
    crate::jig::Limits {
        rows: client.rows,
        ask_bytes: client.ask_bytes,
        text: TEXT,
        part: PART,
        wait: WAIT,
        wait_most: WAIT_MOST,
    }
}
