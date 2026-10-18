use super::fixtures;
use skein_lib::{Queue, Time, Token};
const LISTENER: Token = Token::new(70);
const SOCKET: Token = Token::new(71);
const OWNER: Token = Token::new(0);
const CONNECTION: Token = Token::new(1);
const STALL_AT: Time = Time::from_nanos(30_000_000_000);

fn accepted() -> (crate::server::Server, Queue<crate::server::Routed>) {
    let mut server = fixtures::server();
    let mut out = Queue::with_capacity(16);
    assert!(server.advance(&mut out));
    let Some(crate::server::Routed::Io(skein_io::Request::Listen { owner, .. })) = out.pop() else {
        panic!("listener requested")
    };
    assert_eq!(owner, OWNER);
    server
        .up(skein_io::Event::Listening { owner: OWNER, listener: LISTENER, addr: fixtures::http::address() }, &mut out);
    server.up(skein_io::Event::Accepted { owner: OWNER, socket: SOCKET, peer: fixtures::http::address() }, &mut out);
    let Some(crate::server::Routed::Io(skein_io::Request::Bind { socket, .. })) = out.pop() else {
        panic!("socket bound")
    };
    assert_eq!(socket, SOCKET);
    assert_eq!(server.listening(), fixtures::http::address().socket());
    (server, out)
}
#[test]
fn an_accepted_connection_hands_its_call_to_the_mount() {
    let (mut server, mut out) = accepted();
    let head = fixtures::server_head(&mut server, &mut out);
    assert_eq!(head.method, skein_http::Method::Post);
    assert_eq!(&*head.target, b"/jig/v1/sign-in/link");
}
fn refused(bytes: &[u8], expected: u16) {
    let mut service = fixtures::started();
    let mut call = fixtures::request(&mut service, bytes);
    let (status, _) = fixtures::answered(&mut service, &mut call);
    assert_eq!(status, expected);
    assert_eq!(service.engine.clients.early.len(), 0, "nothing reached the root");
}
#[test]
fn a_path_outside_jig_is_answered_not_found() {
    refused(b"GET /elsewhere HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n", 404);
}
#[test]
fn a_get_of_the_printed_links_route_is_not_found_with_no_referrer() {
    let mut service = fixtures::started();
    let mut call = fixtures::request(
        &mut service,
        b"GET /jig/v1/sign-in/link?secret=AAAA HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n",
    );
    let (status, headers) = fixtures::answered(&mut service, &mut call);
    assert_eq!(status, 404);
    assert_eq!(fixtures::header(&headers, b"referrer-policy"), Some(b"no-referrer".as_slice()));
    assert_eq!(service.engine.clients.early.len(), 0, "nothing reached the root");
}
#[test]
fn a_host_off_the_list_is_answered_421_with_nothing_done() {
    refused(b"POST /jig/v1/read HTTP/1.1\r\nHost: evil.test:7100\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", 421);
}
#[test]
fn a_post_not_octet_stream_is_answered_415_with_nothing_done() {
    refused(b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", 415);
}
#[test]
fn a_cookie_without_the_browser_header_is_answered_403_with_nothing_done() {
    refused(b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nCookie: jig-sign-in=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", 403);
}
#[test]
fn a_post_under_jig_reaches_the_component_as_its_message() {
    let mut service = fixtures::started();
    fixtures::listening(&mut service);
    let link = service.engine.clients.links.pop().expect("printed link");
    let body = fixtures::http::link_body(&service.engine, &link);
    let mut peer = fixtures::http::Peer::new(&mut service.engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(&mut service.engine, false);
    assert_eq!(service.engine.clients.early.len(), 1, "sign in reached the root");
}
#[test]
fn the_printed_link_names_the_bound_address_and_the_secret() {
    let secret = skein_lib::Secret::new(Box::new([0; 32]));
    assert_eq!(
        &*crate::clients::printed::link(fixtures::http::address().socket().unwrap(), &secret),
        b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    );
}

#[test]
fn closing_ends_a_held_call_after_its_component_finishes() {
    let (mut server, mut out) = accepted();
    fixtures::server_head(&mut server, &mut out);
    while server.advance(&mut out) {
        while out.pop().is_some() {}
    }
    server.close(&mut out);
    while out.pop().is_some() {}
    // The component has closed and no response is queued for the held call.
    server.finish();
    while server.advance(&mut out) {}
    assert_eq!(server.next_deadline(), None);
    let Some(crate::server::Routed::Io(skein_io::Request::Close { entity })) = out.pop() else {
        panic!("held connection closed")
    };
    assert_eq!(entity, SOCKET);
}

#[test]
fn closing_sends_a_queued_response_before_ending_its_call() {
    let (mut server, mut out) = accepted();
    let (call, _) = fixtures::server_call(&mut server, &mut out);
    while server.advance(&mut out) {
        while out.pop().is_some() {}
    }
    fixtures::server_response(&mut server, call, &mut out);
    server.close(&mut out);
    server.finish();
    while out.pop().is_some() {}
    assert!(server.advance(&mut out));
    assert!(server.advance(&mut out));
    let Some(crate::server::Routed::Io(skein_io::Request::Stream { stream, .. })) = out.pop() else {
        panic!("queued response sent before closing")
    };
    assert_eq!(stream, SOCKET);
    while server.advance(&mut out) {}
    assert_eq!(server.next_deadline(), None);
}

#[test]
fn a_completed_closing_response_drains_while_the_server_serves() {
    let (mut server, mut out) = accepted();
    let (call, _) = fixtures::server_call(&mut server, &mut out);
    while server.advance(&mut out) {
        while out.pop().is_some() {}
    }
    server.answer(call, skein_http::server::Request::Discard, &mut out);
    fixtures::server_response(&mut server, call, &mut out);
    let mut completed = false;
    while server.advance(&mut out) {
        while let Some(event) = out.pop() {
            match event {
                crate::server::Routed::Io(skein_io::Request::Stream {
                    down: skein_lib::stream::Down::Demand { room, .. },
                    ..
                }) if room > 0 => {
                    server.up(skein_io::Event::Stream { owner: CONNECTION, up: skein_lib::stream::Up::Room }, &mut out);
                }
                crate::server::Routed::Http { event: skein_http::server::Event::Done(reuse), .. } => {
                    assert_eq!(reuse, skein_http::server::Reuse::Close);
                    completed = true;
                }
                crate::server::Routed::Io(_)
                | crate::server::Routed::Call { .. }
                | crate::server::Routed::Http { .. } => {}
            }
        }
    }
    assert!(completed, "response completed");
    server.done(call, &mut out);
    let Some(crate::server::Routed::Io(skein_io::Request::Close { entity })) = out.pop() else {
        panic!("completed response drains")
    };
    assert_eq!(entity, SOCKET);
}

#[test]
fn closing_waits_for_the_component_before_ending_its_call() {
    let (mut server, mut out) = accepted();
    fixtures::server_head(&mut server, &mut out);
    while server.advance(&mut out) {
        while out.pop().is_some() {}
    }
    server.close(&mut out);
    while out.pop().is_some() {}
    assert!(!server.advance(&mut out));
    assert_eq!(server.next_deadline(), Some(STALL_AT));
}
