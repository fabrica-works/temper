use jig_client_world::{harness::App, observe};
use skein_lib::Token;
use temper_client_world::app::Temper;
const TASK: u64 = 9;
const PROJECT: u32 = 1;
const REVISION: jig_client::contract::Revision = jig_client::contract::Revision { value: 7 };
const ATTEMPT: Token = Token::new(1);
const REQUEST: [u8; 16] = [3; 16];
#[test]
fn a_release_is_observed_against_the_inbox_escalation_the_person_saw() {
    let request = temper_client_domain::Request::Client {
        request: jig_client::Request::Down {
            down: jig_client::Down::Send {
                attempt: ATTEMPT,
                key: REQUEST,
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Release {
                    project: PROJECT,
                    task: TASK,
                    revision: REVISION,
                    again: None,
                })),
            },
        },
    };
    let sent = Temper::sent(&request).expect("keyed request");
    assert_eq!(sent.card, Some(observe::key(jig_client::contract::ObjectKey::Escalation(TASK))));
    assert_eq!(sent.revision, Some((2, REVISION.value)));
}

#[test]
fn confirming_preserves_the_inbox_cards_visible_revision() {
    let event = temper_client_world::app::Event::Root(temper_client_domain::Event::Act {
        action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Confirm),
    });
    assert!(!Temper::window_changed(&event));
}
