use super::*;
use jig_client::{Down, Request, Watchable};

pub(super) fn started_kept() -> (NativeClient, skein_lib::Token) {
    let mut host = common::native();
    host.root = domain::unlinked(&host.env.limits, 7);
    let record = common::table_record();
    host.kept_keys.insert(record.request.key).expect("one key");
    let mut kept = List::with_capacity(1);
    kept.push(record).expect("one record");
    host.event(domain::Event::Start { address: host.address.clone(), kept: List::with_capacity(0) });
    host.event(domain::Event::Client { event: jig_client::Event::StartUnlinked { kept } });
    host.event(domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    host.event(domain::Event::Client {
        event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    });
    let mut frame = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request:
                Request::Down { down: Down::Open { watch, subject: Watchable::Jig(jig_client::contract::Subject::Me) } },
        } = request
        {
            frame = Some(watch);
        }
    }
    (host, frame.expect("frame watch"))
}
