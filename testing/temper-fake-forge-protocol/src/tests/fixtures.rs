use crate::{Config, Faults, Limits, Repository};
use alloc::boxed::Box;
use skein_lib::{List, Secret, bytes};
use temper_forge_forgejo::request::Repo;
pub const TOKEN: &[u8] = b"1111111111111111111111111111111111111111";
pub fn config() -> Config {
    let mut tokens = List::with_capacity(1);
    tokens.push((Secret::new(bytes::copy_of(TOKEN)), 1)).expect("one token");
    let mut users = List::with_capacity(2);
    users.push((1, bytes::copy_of(b"engine"))).expect("two users");
    users.push((2, bytes::copy_of(b"person"))).expect("two users");
    let mut repositories = List::with_capacity(1);
    repositories
        .push(Repository { id: 2, full: bytes::copy_of(b"org/repo"), protected: None })
        .expect("one repository");
    Config {
        base: bytes::copy_of(b"http://127.0.0.1:3000"),
        tokens,
        users,
        repositories,
        max_response_items: 2,
        default_paging_num: 30,
        faults: Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
pub fn at() -> Repo {
    Repo { owner: bytes::copy_of(b"org"), name: bytes::copy_of(b"repo") }
}
pub fn limits() -> Limits {
    Limits {
        calls: 4,
        connections: 4,
        raw_bytes: 4096,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 16,
            body: 16384,
            read: 1024,
            response: 4096,
            send: 1024,
        },
        documents: temper_forge_forgejo::limits::Limits {
            json: skein_json::tokenizer::Limits { chunk: 1024, string: 16384, number: 32, depth: 16, length: 65536 },
            writer: skein_json::writer::Limits { depth: 16, length: 65536 },
            string_bytes: 512,
            body_bytes: 16384,
            inner: 32,
            target_bytes: 4096,
        },
    }
}
pub fn boxed(text: &[u8]) -> Box<[u8]> {
    bytes::copy_of(text)
}
