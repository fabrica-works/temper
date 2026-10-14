//! Documented native-client constants and checked memory sums (client.md, 2).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use jig_protocol_client_link as link;
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
    pub connections: jig_shell_native::Limits,
    pub io: skein_io::Limits,
}
#[must_use]
pub fn native() -> Limits {
    Limits {
        client: client::Limits {
            jig: jig_domain_client::Limits {
                watches: 16,                            // Ops's 8 raised to the target 16 live watches.
                notices: 8,                             // Notices retained for display.
                notice_life: Duration::from_secs(5),    // Notice display lifetime.
                linger: Duration::from_secs(30),        // Departed watch retention.
                save_delay: Duration::from_millis(250), // Draft write debounce.
                intents: 8,                             // In-flight user actions.
                fields: 16,                             // Editable fields held on pages.
                providers: 1,                           // Space for jig's provider words.
                objects: 16,                            // Open object cards.
                sources: 16,                            // Sources sharing watched objects.
                reads: 16,                              // Concurrent page readouts.
                parts: 16,                              // Connector parts held in slots.
                delegates: 8,                           // Delegates in a close confirmation.
                document_bytes: 1_048_576,              // Ops's 65,536 raised to the target 1,048,576 document bytes.
                records: 16,                            // Jig records restored at start.
                kept: 16,                               // Restored table scopes.
                drafts: 8,                              // Conversation drafts retained across navigation.
                projects: 16,                           // Project labels in the frame.
                roles: 16,                              // Visible role names.
                charters: 16,                           // Charter choices.
                text: 128,                              // Ops's 32 raised to 128; 256 is refused by Limits::compose.
                rows: 2,                                // Ops's 2; 4 is refused by Limits::compose.
                conversation_words: 8,                  // Pending conversation snapshots.
                navigation_entries: 8,                  // Navigation destinations.
                updates_count: 99,                      // Unread counter before its lower-bound mark.
                backoff: jig_domain_client::limits::Backoff {
                    first: Duration::from_secs(1), // First watch reconnect span.
                    most: Duration::from_secs(30), // Longest watch reconnect span.
                },
                heartbeat: Duration::from_secs(30), // Quiet stream before Offline.
            },
            forge: client::forge::Limits { parts: 16, field_bytes: 256, checks: 8 }, // Forge parts in slots.
            table: RequestLimits {
                requests: 16,                   // Durable keyed requests.
                bytes: 16_777_216,              // Whole request-table payload bytes.
                out: 32,                        // Table outputs per iteration.
                first: Duration::from_secs(1),  // First request retry span.
                most: Duration::from_secs(30),  // Longest request retry span.
                margin: Duration::from_secs(1), // Retention's safety margin.
            },
        },
        connections: jig_shell_native::Limits {
            exchanges: EXCHANGES,
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
/// # Errors
/// Returns the link's refusal if these client bounds cannot be encoded.
pub fn link(limits: &Limits) -> Result<link::Limits, link::Unusable> {
    link::Limits::new(limits.client.jig, EXCHANGES, DEADLINE)
}
/// Family bounds mirror the small forge's names, gates and protection contexts.
#[must_use]
pub fn family(limits: &Limits) -> Option<crate::route::family::Family> {
    Some(crate::route::family::Family {
        attempts: crate::route::family::Attempts::new(limits.client.table.requests),
        wire: temper_protocol_client_wire::limits(&temper_protocol_client_wire::Bounds {
            field_bytes: 256,
            checks: 8,
            contexts: 16,
            refusal_bytes: 65_536,
        })?,
        link: link(limits).ok()?,
    })
}
/// Checked words and the view bounds used at construction and accounting.
#[must_use]
pub fn view(
    limits: &Limits,
) -> Option<(temper_client_view::Limits, jig_view::words::Words, temper_client_view::words::Words)> {
    let jig = jig_view::words::Words {
        book: jig_view::phrases::english::english(limits.client.jig.text).ok()?,
        language: 0,
        offset_minutes: 0,
    };
    let forge =
        temper_client_view::words::Words::english(temper_protocol_client_wire::FORGE, limits.client.jig.text).ok()?;
    let view = temper_client_view::Limits::of(&limits.client, &jig, &forge)?;
    Some((view, jig, forge))
}
/// Sum the root, view, link and shell, with overflow refusing the preset.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let (view, jig, forge) = view(limits)?;
    let root = client::worst_case(&limits.client)?;
    let view = temper_client_view::worst_case(&view, &limits.client, &jig, &forge)?;
    let link = link::worst_case(&link(limits).ok()?)?;
    root.checked_add(view)?.checked_add(link)?.checked_add(shell(limits)?)?.checked_add(skein_lib::Set::<
        skein_lib::Token,
    >::worst_case(
        limits.client.table.requests
    )?)
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
    skein_io::worst_case(&limits.io)?.checked_add(connections.checked_mul(256)?.checked_mul(bytes)?)
}
