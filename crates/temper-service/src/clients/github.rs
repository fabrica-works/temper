//! Deferred replies for the setup route (client/forge.md, 9.1).
use super::Clients;
use alloc::boxed::Box;
use skein_http::{Header, server as http};
use skein_lib::{
    List, Token, bytes,
    stream::{Down, Read, Up},
};

#[derive(Debug)]
pub(super) struct Reply {
    body: Option<Box<[u8]>>,
    offset: usize,
}
pub(super) fn claims(target: &[u8]) -> bool {
    let path = match bytes::find(target, b"?") {
        Some(at) => target.get(..at).expect("query"),
        None => target,
    };
    path.starts_with(b"/forges/") && path.ends_with(b"/github/setup")
}
impl Clients {
    pub(super) fn github_call(&mut self, call: Token, head: http::Call) {
        self.github.insert(call, Reply { body: None, offset: 0 }).expect("bounded HTTP calls");
        if self.github_calls.room() == 0 {
            self.github_answer(call, 503, None, b"temper is busy; try again.");
            return;
        }
        self.github_calls.push((call, head));
    }
    pub(crate) fn github_answer(&mut self, call: Token, status: u16, location: Option<&[u8]>, body: &[u8]) {
        let Some(reply) = self.github.get_mut(&call) else { return };
        reply.body = Some(bytes::copy_of(body));
        let mut headers = List::with_capacity(4);
        for (name, value) in [
            (b"Cache-Control".as_slice(), b"no-store".as_slice()),
            (b"Referrer-Policy", b"no-referrer"),
            (b"Content-Type", b"text/html; charset=utf-8"),
        ] {
            headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("four headers");
        }
        if let Some(location) = location {
            headers
                .push(Header { name: bytes::copy_of(b"Location"), value: bytes::copy_of(location) })
                .expect("location header");
        }
        self.server.answer(call, http::Request::Discard, &mut self.server_out);
        self.server.answer(
            call,
            http::Request::Respond(http::Response {
                status,
                headers: headers.into_boxed(),
                body: http::Body::Length(u64::try_from(body.len()).expect("bounded page")),
                close: false,
            }),
            &mut self.server_out,
        );
        if body.is_empty() {
            return;
        }
        self.server.answer(
            call,
            http::Request::Reply(Down::Demand {
                read: Read::Nothing,
                room: self.send.min(u32::try_from(body.len()).expect("bounded page")),
            }),
            &mut self.server_out,
        );
    }
    pub(super) fn github_event(&mut self, call: Token, event: http::Event) {
        match event {
            http::Event::Reply(Up::Room) => {
                let reply = self.github.get_mut(&call).expect("route reply");
                let Some(body) = &reply.body else { return };
                let end = reply.offset.saturating_add(usize::try_from(self.send).expect("chunk bound")).min(body.len());
                let chunk = bytes::copy_of(body.get(reply.offset..end).expect("page cursor"));
                reply.offset = end;
                let left = body.len().checked_sub(end).expect("page cursor");
                self.server.answer(call, http::Request::Reply(Down::Send(chunk)), &mut self.server_out);
                if left == 0 {
                    self.server.answer(call, http::Request::Reply(Down::Finish), &mut self.server_out);
                } else {
                    self.server.answer(
                        call,
                        http::Request::Reply(Down::Demand {
                            read: Read::Nothing,
                            room: self.send.min(u32::try_from(left).expect("page bound")),
                        }),
                        &mut self.server_out,
                    );
                }
            }
            http::Event::Done(_)
            | http::Event::Ended
            | http::Event::Refused(_)
            | http::Event::Failed(_)
            | http::Event::Closed
            | http::Event::Reply(Up::Failed(_))
            | http::Event::Body(Up::Failed(_)) => {
                self.github.remove(&call);
                self.server.done(call, &mut self.server_out);
            }
            http::Event::Body(_) | http::Event::Reply(_) => {}
            http::Event::Call(_) => unreachable!("head dispatched separately"),
        }
    }
}
