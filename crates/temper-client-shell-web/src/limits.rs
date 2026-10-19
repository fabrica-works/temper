//! The page's bounds, the terminal's client bounds (client/README.md, 2).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use skein_lib::{Duration, RequestLimits};
use temper_client_domain as client;
/// The root's bounds: `temper-client-shell-terminal`'s `native().client`.
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
            document_bytes: 1_048_576,
            records: 16,
            kept: 16,
            projects: 2,
            text: 128,
            rows: 2,
            fields: 16,
            segments: 9,
        },
        exchanges: 8,
    };
    client::Limits { forge: client::forge::Limits { field_bytes: 256, checks: 8 }, client }
}
/// Checked words and the view bounds used at construction and accounting.
#[must_use]
pub fn view(client: &client::Limits) -> Option<(temper_client_view::Limits, temper_client_view::words::Words)> {
    let words =
        temper_client_view::words::Words::english(temper_client_protocol_wire::FORGE, client.client.domain.text)
            .ok()?;
    let view = temper_client_view::Limits::of(client, &words)?;
    Some((view, words))
}
