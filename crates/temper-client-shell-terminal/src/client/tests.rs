use super::*;
use crate::tests::fixtures as common;
mod fixtures;
#[path = "../../tests/support/mod.rs"]
mod machine;
const CURRENT_PARTY: u64 = 9;
const PRINTED: skein_lib::Duration = skein_lib::Duration::from_millis(30);
const STILL_PRINTED: skein_lib::Duration = skein_lib::Duration::from_secs(1);
#[test]
fn another_partys_kept_work_prints_its_line_once() {
    let (mut host, watch) = fixtures::started_kept();
    for _ in 0..2 {
        host.event(domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        jig_client_world::stories::frame(CURRENT_PARTY),
                    ))),
                },
            },
        });
    }
    let mut world = machine::World::signed_in();
    let Some(Shown::Terminal(notice @ crate::terminal::Notice::OtherPartyKept)) = host.notices() else {
        panic!(
            "other-party notice: link {:?}, pending {:?}, keys {:?}",
            host.root.client().link(),
            host.root.client().pending(common::table_record().request.key),
            host.kept_keys
        )
    };
    world.client.notice_at_once(notice);
    assert!(host.notices().is_none(), "the native host emits it once");
    world.run_until(PRINTED);
    world.run_until(STILL_PRINTED);
    assert_eq!(
        world.shell_lines(),
        vec![Box::<[u8]>::from(b"temper: work of another party is kept in client, and waits for them\n".as_slice())]
    );
}
#[test]
fn a_service_credential_is_a_notice_never_a_root_event() {
    let mut host = common::native();
    host.transport(jig_client::Event::Reply { exchange: common::ATTEMPT, message: common::service_answer() });
    let Some(Shown::Notice(route::Notice::Credential { party, credential })) = host.notices() else {
        panic!("credential notice")
    };
    assert_eq!(party, common::SERVICE);
    assert_eq!(credential.expose(), common::CREDENTIAL);
    host.outputs();
    assert!(host.notices().is_none());
}
