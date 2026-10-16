use super::{BodyDecoder, Route, Unrouted, writes::WriteKind};
use crate::{
    decode::walk::{Kind, Walk},
    limits::Limits,
    path,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_lib::{List, Secret};
pub(super) fn route(method: Method, bytes: &[u8], limits: &Limits) -> Result<Route, Unrouted> {
    let target = path::read(bytes, limits).ok().ok_or(Unrouted::Malformed)?;
    match target.segments.as_ref() {
        [api, v1, users, login] if &**api == b"api" && &**v1 == b"v1" && &**users == b"users" => {
            if method != Method::Get {
                return Err(Unrouted::Method);
            }
            Ok(Route::Ready(Request::UserOf { login: login.clone() }))
        }
        [api, v1, admin, users]
            if &**api == b"api" && &**v1 == b"v1" && &**admin == b"admin" && &**users == b"users" =>
        {
            if method != Method::Post {
                return Err(Unrouted::Method);
            }
            Ok(Route::Body(BodyDecoder {
                walk: Walk::new(Kind::Write(WriteKind::NewUser), limits),
                request: Request::CreateUser {
                    login: Box::from([]),
                    email: Box::from([]),
                    full_name: Box::from([]),
                    password: Secret::new(Box::from([])),
                    must_change_password: false,
                },
                logins: List::with_capacity(limits.inner),
                terminal: false,
            }))
        }
        _ => Err(Unrouted::NotFound),
    }
}
