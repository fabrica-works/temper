//! Documented native-client constants and checked memory sums (client.md, 2).
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
/// Concurrent HTTP exchanges, beside the event stream.
pub const EXCHANGES: u32 = 8;
/// Each exchange's deadline, including a quiet engine.
pub const DEADLINE: Duration = Duration::from_secs(30);
/// Root, native transport and io bounds.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub client: client::Limits,
    pub connections: jig_client_shell_native::Limits,
    pub io: skein_io::Limits,
}
#[must_use]
pub fn native() -> Limits {
    Limits {
        client: {
            let jig = jig_client_view::Limits {
                segments: 9,
                watches: 16,                            // Ops's 8 raised to the target 16 live watches.
                notices: 8,                             // Notices retained for display.
                notice_life: Duration::from_secs(5),    // Notice display lifetime.
                linger: Duration::from_secs(30),        // Departed watch retention.
                save_delay: Duration::from_millis(250), // Draft write debounce.
                intents: 8,                             // In-flight user actions.
                fields: 16,                             // Editable fields held on pages.
                objects: 16,                            // Open object cards.
                sources: 16,                            // Sources sharing watched objects.
                reads: 16,                              // Concurrent page readouts.
                parts: 16,                              // Connector parts held in slots.
                delegates: 8,                           // Delegates in a close confirmation.
                document_bytes: 1_048_576,              // Ops's 65,536 raised to the target 1,048,576 document bytes.
                records: 16,                            // Jig records restored at start.
                kept: 16,                               // Restored table scopes.
                drafts: 8,                              // Conversation drafts retained across navigation.
                // Ops's 2 stays below the receiving wire's frame_projects ceiling of 7.
                projects: 2,
                roles: 16,             // Visible role names.
                text: 128,             // Ops's 32 raised to 128; 256 is refused by Limits::compose.
                rows: 2,               // Ops's 2; 4 is refused by Limits::compose.
                conversation_words: 8, // Pending conversation snapshots.
                navigation_entries: 8, // Navigation destinations.
                updates_count: 99,     // Unread counter before its lower-bound mark.
            };
            let requests = RequestLimits {
                requests: 16,                   // Durable keyed requests.
                bytes: 16_777_216,              // Whole request-table payload bytes.
                out: 32,                        // Table outputs per iteration.
                first: Duration::from_secs(1),  // First request retry span.
                most: Duration::from_secs(30),  // Longest request retry span.
                margin: Duration::from_secs(1), // Retention's safety margin.
            };
            client::Limits {
                jig,
                forge: client::forge::Limits { parts: 16, field_bytes: 256, checks: 8 },
                client: client::limits::client(
                    &jig,
                    requests,
                    EXCHANGES,
                    jig_client::Backoff { first: Duration::from_secs(1), most: Duration::from_secs(30) },
                    Duration::from_secs(30),
                )
                .expect("client profile fits"),
            }
        },
        connections: jig_client_shell_native::Limits {
            exchanges: EXCHANGES,
            deadline: DEADLINE,
            body_bytes: 33_554_432, // Whole small store and HTTP response bodies.
            http: skein_http::client::Limits {
                head: 4096,    // Incoming HTTP head bytes.
                headers: 8,    // Incoming HTTP fields.
                request: 4096, // Outgoing HTTP head bytes.
                read: 1024,    // Stream body chunk bytes.
                send: 1024,    // Outgoing body chunk bytes.
            },
        },
        io: skein_io::Limits {
            sockets: 16,                           // Stream and exchange sockets, with close overlap.
            names: 1,                              // No in-process listener, one spare name.
            refusals: 16,                          // Deferred io admission refusals.
            intake: 8192,                          // Per-socket unparsed bytes.
            receive: 4096,                         // One receive's bytes.
            output: 8192,                          // Per-socket queued bytes.
            sends: 8,                              // Queued sends per socket.
            accepts: 1,                            // io's minimum, this client never listens.
            backlog: 1,                            // Unused listener backlog.
            close_timeout: Duration::from_secs(5), // Graceful close bound.
            retry: Duration::from_millis(10),      // Kernel resource retry delay.
        },
    }
}
/// Checked words and the view bounds used at construction and accounting.
#[must_use]
pub fn view(
    limits: &Limits,
) -> Option<(temper_client_view::Limits, jig_client_view::words::Words, temper_client_view::words::Words)> {
    let jig = jig_client_view::words::Words {
        book: jig_client_view::phrases::english::english(limits.client.jig.text).ok()?,
        language: 0,
        offset_minutes: 0,
    };
    let forge =
        temper_client_view::words::Words::english(temper_client_protocol_wire::FORGE, limits.client.jig.text).ok()?;
    let view = temper_client_view::Limits::of(&limits.client, &jig, &forge)?;
    Some((view, jig, forge))
}
/// Sum the root, view, link and shell, with overflow refusing the preset.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let (view, jig, forge) = view(limits)?;
    let root = client::worst_case(&limits.client)?;
    let view = temper_client_view::worst_case(&view, &limits.client, &jig, &forge)?;
    root.checked_add(view)?.checked_add(shell(limits)?)
}
/// The published `Shell::worst_case` formula, with every operation checked.
#[must_use]
pub fn shell(limits: &Limits) -> Option<u64> {
    if limits.connections.exchanges != EXCHANGES {
        return None;
    }
    if limits.io.intake < limits.connections.http.head.max(limits.connections.http.read) {
        return None;
    }
    if limits.io.output < limits.connections.http.request.max(limits.connections.http.send) {
        return None;
    }
    let bytes = u64::from(limits.io.intake)
        .checked_add(u64::from(limits.io.output))?
        .checked_add(u64::from(limits.connections.body_bytes))?
        .checked_add(1024)?;
    let connections = u64::from(limits.connections.exchanges.checked_add(1)?);
    let stream = jig_client_protocol_http::worst_case(&jig_client_protocol_http::Limits {
        record: limits.connections.body_bytes.checked_add(512)?,
        chunk: limits.connections.http.read,
    })?;
    let heard = limits.connections.http.read.checked_mul(3)?.checked_add(8)?;
    skein_io::worst_case(&limits.io)?
        .checked_add(stream)?
        .checked_add(skein_lib::Queue::<jig_client_protocol_http::Heard>::worst_case(heard)?)?
        .checked_add(connections.checked_mul(256)?.checked_mul(bytes)?)?
        .checked_add(connections.checked_mul(u64::from(limits.io.intake))?)
}
/// The terminal's own fixed bounds, independent of configuration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Terminal {
    pub line: u32,
    pub read: u32,
    pub interval: skein_lib::Duration,
    pub lines: u32,
    pub first: skein_lib::Duration,
    pub most: skein_lib::Duration,
}
#[must_use]
pub const fn terminal() -> Terminal {
    Terminal {
        line: 4096,
        read: 4096,
        interval: skein_lib::Duration::from_secs(1),
        lines: 32,
        first: skein_lib::Duration::from_millis(250),
        most: skein_lib::Duration::from_secs(8),
    }
}
