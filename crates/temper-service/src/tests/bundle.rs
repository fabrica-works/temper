use super::fixtures;
use crate::clients::bundle::{Answer, route};
use skein_http::Method;
use skein_lib::Writer;

const ETAG: Fields = &[(b"If-None-Match", b"\"0011223344556677\"")];
type Fields = &'static [(&'static [u8], &'static [u8])];
type Row = (bool, Method, &'static [u8], Fields, Answer);
const ROUTES: [Row; 10] = [
    (false, Method::Get, b"/", &[], Answer::NotFound),
    (false, Method::Post, b"/", &[], Answer::NotFound),
    (true, Method::Post, b"/bundle/0123456789abcdef/boot.js", &[], Answer::NotAllowed),
    (true, Method::Get, b"/bundle/0123456789abcdef/boot.js", &[], Answer::File),
    (true, Method::Get, b"/bundle/fedcba9876543210/boot.js", &[], Answer::NotFound),
    (true, Method::Get, b"/bundle/0123456789abcdef/missing.js", &[], Answer::NotFound),
    (true, Method::Get, b"/robots.txt", &[], Answer::Root),
    (true, Method::Get, b"/jig/v1/sign-in/link?secret=AAAA", &[], Answer::Page),
    (true, Method::Get, b"/chats/7", &[], Answer::Page),
    (true, Method::Get, b"/chats/7", ETAG, Answer::NotModified),
];
#[test]
fn each_head_is_routed_in_the_designs_order() {
    let bundle = fixtures::bundle::bundle();
    for (bundled, method, target, headers, answer) in ROUTES {
        let head = fixtures::bundle::head(method, target, headers);
        assert_eq!(route(bundled.then_some(&bundle), &head), answer, "{method:?} {target:?}");
    }
}
fn get(target: &[u8], field: &[u8]) -> (crate::Service, fixtures::http::Peer, u16, Box<[skein_http::Header]>) {
    let mut service = fixtures::started();
    service.adopt_bundle(fixtures::bundle::bundle());
    let parts =
        [b"GET ".as_slice(), target, b" HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nConnection: close\r\n", field, b"\r\n"];
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("fixture request size");
    }
    let mut request = Writer::new(size);
    for part in parts {
        request.put(part).expect("fixture request");
    }
    let request = request.finish();
    let mut peer = fixtures::request(&mut service, &request);
    let (status, headers) = fixtures::answered(&mut service, &mut peer);
    (service, peer, status, headers)
}
#[test]
fn a_bundle_file_is_served_immutable_with_its_kind() {
    let (_, peer, status, headers) = get(b"/bundle/0123456789abcdef/boot.js", b"");
    assert_eq!(status, 200);
    assert_eq!(fixtures::header(&headers, b"content-type"), Some(b"text/javascript".as_slice()));
    assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"public, max-age=31536000, immutable".as_slice()));
    assert_eq!(fixtures::bundle::body(&peer), b"boot();");
}
#[test]
fn the_page_is_served_with_its_etag_for_revalidation() {
    let (_, peer, status, headers) = get(b"/chats/7", b"");
    assert_eq!(status, 200);
    assert_eq!(fixtures::header(&headers, b"content-type"), Some(b"text/html; charset=utf-8".as_slice()));
    assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"no-cache".as_slice()));
    assert_eq!(fixtures::header(&headers, b"etag"), Some(b"\"0011223344556677\"".as_slice()));
    assert_eq!(fixtures::bundle::body(&peer), b"<!doctype html><title>t</title>");
}
#[test]
fn a_matching_etag_is_answered_not_modified() {
    let (_, peer, status, headers) = get(b"/chats/7", b"If-None-Match: \"0011223344556677\"\r\n");
    assert_eq!(status, 304);
    assert_eq!(fixtures::header(&headers, b"etag"), Some(b"\"0011223344556677\"".as_slice()));
    assert_eq!(fixtures::bundle::body(&peer), b"");
}
#[test]
fn a_file_longer_than_one_send_arrives_whole() {
    let (_, peer, status, headers) = get(b"/bundle/0123456789abcdef/module.wasm", b"");
    assert_eq!(status, 200);
    assert_eq!(fixtures::header(&headers, b"content-type"), Some(b"application/wasm".as_slice()));
    assert!(
        fixtures::bundle::WASM_BYTES
            > crate::limits::server(&jig_protocol_config::profile(jig_protocol_config::Profile::Small)).http.send
    );
    assert_eq!(fixtures::bundle::body(&peer), &*fixtures::bundle::wasm());
}
#[test]
fn the_printed_links_route_serves_the_page() {
    let (service, peer, status, _) = get(b"/jig/v1/sign-in/link?secret=AAAA", b"");
    assert_eq!(status, 200);
    assert_eq!(fixtures::bundle::body(&peer), b"<!doctype html><title>t</title>");
    assert_eq!(service.engine.clients.early.len(), 0, "nothing reached the root");
}
