//! Forge records, protocol/credentials.md, 2.1. No store schema is changed.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
#[expect(clippy::disallowed_types, reason = "temper-01 explicitly requires the codec writing half to return Vec")]
use alloc::vec::Vec;
use skein_forge_connection::{Access, settings::Kind};
use skein_forge_setup::item;
use skein_lib::{Map, Secret, Writer};

pub const FAMILY_FORGE: u16 = 1;
pub const VERSION: u8 = 1;
pub const CARRIED: u64 = 1 << 63;
const FORGES: u32 = 65536;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum Item {
    AdminToken = 1,
    ApiToken,
    GitToken,
    User,
    WebhookSecret,
    AppId,
    AppSlug,
    AppKey,
    Accepted,
    Origin,
}
impl Item {
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::AdminToken => 1,
            Self::ApiToken => 2,
            Self::GitToken => 3,
            Self::User => 4,
            Self::WebhookSecret => 5,
            Self::AppId => 6,
            Self::AppSlug => 7,
            Self::AppKey => 8,
            Self::Accepted => 9,
            Self::Origin => 10,
        }
    }
}
#[must_use]
pub fn number(forge: u16, item: Item) -> u64 {
    (u64::from(FAMILY_FORGE) << 48) | (u64::from(forge) << 16) | u64::from(item.code())
}
#[must_use]
pub fn parse(number: u64) -> Option<(u16, u16, Item)> {
    if number >= CARRIED {
        return None;
    }
    let item = match number & 0xffff {
        1 => Item::AdminToken,
        2 => Item::ApiToken,
        3 => Item::GitToken,
        4 => Item::User,
        5 => Item::WebhookSecret,
        6 => Item::AppId,
        7 => Item::AppSlug,
        8 => Item::AppKey,
        9 => Item::Accepted,
        10 => Item::Origin,
        _ => return None,
    };
    Some((u16::try_from(number >> 48_u32).ok()?, u16::try_from((number >> 16_u32) & 0xffff_ffff).ok()?, item))
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unreadable;
pub fn value(bytes: &[u8]) -> Result<&[u8], Unreadable> {
    match bytes.split_first() {
        Some((&VERSION, rest)) => Ok(rest),
        Some(_) | None => Err(Unreadable),
    }
}
/// The codec's writing half; runtime writes arrive in temper-05.
#[must_use]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "temper-01 explicitly exposes encode as Vec; allocated once at the checked length"
)]
pub fn encode(bytes: &[u8]) -> Vec<u8> {
    let mut out = Writer::new(bytes.len().checked_add(1).expect("record length"));
    out.put(&[VERSION]).expect("version");
    out.put(bytes).expect("item");
    out.finish().into_vec()
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Origin {
    pub kind: Kind,
    pub api: Box<str>,
}
impl Origin {
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let mut out = Writer::new(self.api.len().checked_add(1).expect("origin length"));
        out.put(&[match self.kind {
            Kind::Forgejo => 1,
            Kind::GitHub => 2,
        }])
        .expect("kind");
        out.put(self.api.as_bytes()).expect("api");
        out.finish()
    }
    #[expect(clippy::disallowed_methods, reason = "the origin codec validates its UTF-8 text")]
    pub fn decode(bytes: &[u8]) -> Result<Self, Unreadable> {
        let (kind, api) = bytes.split_first().ok_or(Unreadable)?;
        let kind = match kind {
            1 => Kind::Forgejo,
            2 => Kind::GitHub,
            _ => return Err(Unreadable),
        };
        let api = core::str::from_utf8(api).or(Err(Unreadable))?;
        if api.is_empty() {
            return Err(Unreadable);
        }
        Ok(Self { kind, api: Box::from(api) })
    }
}
/// Display is part of the stored format, including IPv6 brackets.
#[must_use]
#[expect(clippy::disallowed_macros, reason = "credentials.md pins Rust's socket Display as Origin's bytes")]
pub fn origin(kind: Kind, address: &jig_protocol_config::Address) -> Origin {
    let api = match address {
        jig_protocol_config::Address::Addr(addr) => {
            alloc::format!("{}", addr.socket().expect("checked forge socket")).into_boxed_str()
        }
        jig_protocol_config::Address::Host { name, port } => {
            let port = skein_lib::Decimal::of(u64::from(*port));
            let mut exact = Writer::new(
                name.len().checked_add(1).expect("separator").checked_add(port.as_bytes().len()).expect("port"),
            );
            exact.put(name).expect("host");
            exact.put(b":").expect("separator");
            exact.put(port.as_bytes()).expect("port");
            let text = exact.finish();
            text_to_box(&text)
        }
    };
    Origin { kind, api }
}
#[expect(clippy::disallowed_methods, reason = "configuration has already validated host text")]
fn text_to_box(bytes: &[u8]) -> Box<str> {
    Box::from(core::str::from_utf8(bytes).expect("checked host text"))
}
#[derive(Debug)]
pub struct Rows {
    forges: Map<u16, ForgeRows>,
}
#[derive(Debug)]
pub struct ForgeRows {
    values: [Option<Secret>; 10],
    unreadable: bool,
}
#[expect(clippy::new_without_default, reason = "the programming model uses explicit constructors")]
impl Rows {
    #[must_use]
    pub fn new() -> Self {
        Self { forges: Map::with_capacity(FORGES) }
    }
    /// Only numbers below CARRIED are records; no value at or above it is read.
    pub fn push(&mut self, number: u64, bytes: Secret, value_bytes: u32) {
        if number >= CARRIED || number >> 48_u32 != u64::from(FAMILY_FORGE) {
            return;
        }
        let Ok(forge) = u16::try_from((number >> 16_u32) & 0xffff_ffff) else { return };
        if self.forges.get(&forge).is_none() {
            let row =
                ForgeRows { values: [None, None, None, None, None, None, None, None, None, None], unreadable: false };
            self.forges.insert(forge, row).expect("all u16 forge numbers fit");
        }
        let row = self.forges.get_mut(&forge).expect("inserted forge");
        let Some((_, _, item)) = parse(number) else {
            row.unreadable = true;
            return;
        };
        if bytes.expose().len() > usize::try_from(value_bytes).expect("u32 bound") {
            row.unreadable = true;
            return;
        }
        let index = usize::from(item.code()).checked_sub(1).expect("positive item");
        *row.values.get_mut(index).expect("ten item codes") = Some(bytes);
    }
    #[must_use]
    pub fn forge(&self, forge: u16) -> Option<&ForgeRows> {
        self.forges.get(&forge)
    }
}
impl ForgeRows {
    /// Read an item past temper's version byte; malformed rows are refused.
    pub fn get(&self, item: Item) -> Result<&[u8], NotConnected> {
        let index = usize::from(item.code()).checked_sub(1).expect("positive item");
        value(self.values.get(index).expect("ten codes").as_ref().ok_or(NotConnected::Incomplete)?.expose())
            .or(Err(NotConnected::Unreadable))
    }
    #[must_use]
    pub fn has(&self, item: Item) -> bool {
        self.get(item) != Err(NotConnected::Incomplete)
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum NotConnected {
    NoRecords,
    Incomplete,
    Unreadable,
    OtherForge { kind: Kind, api: Box<str> },
}
// The requested diagnostic vocabulary is a format boundary, not a step effect.
#[expect(clippy::disallowed_macros, reason = "temper-01 requires Display for operator diagnostics")]
impl core::fmt::Display for NotConnected {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoRecords => out.write_str("it has no records"),
            Self::Incomplete => out.write_str("its records are incomplete"),
            Self::Unreadable => out.write_str("its records are unreadable"),
            Self::OtherForge { kind, api } => write!(
                out,
                "its records were made for {} at {api}",
                match kind {
                    Kind::Forgejo => "forgejo",
                    Kind::GitHub => "github",
                }
            ),
        }
    }
}
#[derive(Debug)]
pub struct Start {
    pub access: Access,
    pub accepted: Box<[u64]>,
    pub slug: Option<Box<[u8]>>,
}
pub fn access(
    kind: Kind,
    expected: &Origin,
    rows: &ForgeRows,
    bounds: &item::Bounds,
    webhooks: bool,
) -> Result<Start, NotConnected> {
    if rows.unreadable {
        return Err(NotConnected::Unreadable);
    }
    // Validate even optional/unused items: one unreadable row refuses the set.
    for (index, bytes) in rows.values.iter().enumerate() {
        let Some(bytes) = bytes else { continue };
        let bytes = value(bytes.expose()).or(Err(NotConnected::Unreadable))?;
        if index == 9 {
            Origin::decode(bytes).or(Err(NotConnected::Unreadable))?;
            continue;
        }
        let kit = [
            item::Item::AdminToken,
            item::Item::ApiToken,
            item::Item::GitToken,
            item::Item::User,
            item::Item::WebhookSecret,
            item::Item::AppId,
            item::Item::AppSlug,
            item::Item::AppKey,
            item::Item::Accepted,
        ];
        if !item::fits(*kit.get(index).expect("nine setup items"), bytes, bounds) {
            return Err(NotConnected::Unreadable);
        }
    }
    let found = Origin::decode(rows.get(Item::Origin)?).or(Err(NotConnected::Unreadable))?;
    if found != *expected {
        return Err(NotConnected::OtherForge { kind: found.kind, api: found.api });
    }
    let other = match kind {
        Kind::Forgejo => rows.has(Item::AppKey),
        Kind::GitHub => rows.has(Item::ApiToken),
    };
    if other {
        return Err(NotConnected::OtherForge {
            kind: match kind {
                Kind::Forgejo => Kind::GitHub,
                Kind::GitHub => Kind::Forgejo,
            },
            api: found.api,
        });
    }
    let user = item::user(rows.get(Item::User)?).ok_or(NotConnected::Unreadable)?.get();
    let (credential, slug, accepted, webhook_secret) = match kind {
        Kind::Forgejo => (
            skein_forge::credential::Credential::Token(Secret::new(Box::from(rows.get(Item::ApiToken)?))),
            None,
            Box::from([]),
            if webhooks && rows.has(Item::WebhookSecret) {
                Some(Secret::new(Box::from(rows.get(Item::WebhookSecret)?)))
            } else {
                None
            },
        ),
        Kind::GitHub => (
            skein_forge::credential::Credential::App {
                id: item::app_id(rows.get(Item::AppId)?).ok_or(NotConnected::Unreadable)?.get(),
                key: Secret::new(Box::from(rows.get(Item::AppKey)?)),
            },
            Some(Box::from(rows.get(Item::AppSlug)?)),
            if rows.has(Item::Accepted) {
                item::accepted(rows.get(Item::Accepted)?, bounds.installations).ok_or(NotConnected::Unreadable)?
            } else {
                Box::new([])
            },
            None,
        ),
    };
    Ok(Start { access: Access { credential, user, webhook_secret }, accepted, slug })
}
