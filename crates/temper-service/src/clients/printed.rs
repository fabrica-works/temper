//! Print the sign-in URL after the listener address is known.
use alloc::boxed::Box;
pub(super) fn link(address: ::core::net::SocketAddr, secret: &skein_lib::Secret) -> Box<[u8]> {
    let host = address_text(address).expect("bound listener address");
    let encoded = url(secret.expose());
    let parts: [&[u8]; 4] = [b"http://", &host, b"/jig/v1/sign-in/link?secret=", &encoded];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("short printed URL");
    }
    let mut out = skein_lib::Writer::new(length);
    for part in parts {
        out.put(part).expect("measured URL");
    }
    out.finish()
}

fn address_text(address: ::core::net::SocketAddr) -> Option<Box<[u8]>> {
    use ::core;
    let mut text = skein_lib::Intake::with_capacity(64);
    match address.ip() {
        core::net::IpAddr::V4(ip) => {
            for (at, octet) in ip.octets().iter().enumerate() {
                if at != 0 {
                    text.append(b".").ok()?;
                }
                text.append(skein_lib::Decimal::of(u64::from(*octet)).as_bytes()).ok()?;
            }
        }
        core::net::IpAddr::V6(ip) => {
            if !ip.is_loopback() {
                return None;
            }
            text.append(b"[::1]").ok()?;
        }
    }
    text.append(b":").ok()?;
    text.append(skein_lib::Decimal::of(u64::from(address.port())).as_bytes()).ok()?;
    text.meet(skein_lib::stream::Read::Fill(text.len()))
}

fn url(bytes: &[u8]) -> Box<[u8]> {
    let length = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Url,
        u32::try_from(bytes.len()).expect("short secret"),
    )
    .expect("encoded secret");
    let mut out = skein_lib::Writer::new(usize::try_from(length).expect("u32 fits"));
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, bytes, &mut out).expect("short secret");
    out.finish()
}
