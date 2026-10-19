//! Request conventions (protocol/forge.md, section 3.1).
use alloc::boxed::Box;
use core::net::{IpAddr, SocketAddr};
use skein_lib::{Decimal, List};
/// Names the application and its workspace version.
pub const USER_AGENT: &str = "temper/0.0.0";
/// The configured server name, or bracketed IPv6/address, plus a nondefault port.
#[must_use]
pub fn host(address: SocketAddr, server_name: Option<&[u8]>) -> Option<Box<[u8]>> {
    let first = match server_name {
        Some(name) => u32::try_from(name.len()).ok()?,
        None => 41,
    };
    let mut out = List::with_capacity(first.checked_add(6)?);
    match server_name {
        Some(name) => {
            if name.is_empty() {
                return None;
            }
            for byte in name {
                if *byte <= b' ' || *byte >= 127 || b"/:[]".contains(byte) {
                    return None;
                }
            }
            put(&mut out, name)?;
        }
        None => match address.ip() {
            IpAddr::V4(ip) => {
                for (at, octet) in ip.octets().iter().enumerate() {
                    if at != 0 {
                        put(&mut out, b".")?;
                    }
                    put(&mut out, Decimal::of(u64::from(*octet)).as_bytes())?;
                }
            }
            IpAddr::V6(ip) => {
                put(&mut out, b"[")?;
                for (at, segment) in ip.segments().iter().enumerate() {
                    if at != 0 {
                        put(&mut out, b":")?;
                    }
                    let mut begun = false;
                    for shift in [12_u32, 8, 4, 0] {
                        let digit = u8::try_from((segment >> shift) & 15).ok()?;
                        if begun || digit != 0 || shift == 0 {
                            begun = true;
                            put(&mut out, &[*b"0123456789abcdef".get(usize::from(digit)).expect("hexadecimal digit")])?;
                        }
                    }
                }
                put(&mut out, b"]")?;
            }
        },
    }
    let default = if server_name.is_some() { 443 } else { 80 };
    if address.port() != default {
        put(&mut out, b":")?;
        put(&mut out, Decimal::of(u64::from(address.port())).as_bytes())?;
    }
    Some(out.into_boxed())
}
fn put(out: &mut List<u8>, bytes: &[u8]) -> Option<()> {
    for byte in bytes {
        out.push(*byte).ok()?;
    }
    Some(())
}
