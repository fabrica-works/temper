use jig_domain_core::reads::{Answer, Read};
use jig_domain_people::Ask;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

#[test]
fn configured_document_rows_reach_the_cores_limits() {
    for (rows, expected) in [(1, 1), (2, 2)] {
        let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
        jig.core.documents.rows = rows;
        let own = temper_service::limits::profile(temper_service::limits::Profile::Small);
        let configured = temper_service::limits::engine(&jig, &own).expect("configured root bounds");
        let mut limits = walking::limits();
        limits.documents = configured.documents;
        let mut driver = Driver::configured(Store::new(), walking::config(91), &limits);
        driver.hello(Token::new(7));
        driver.settle();
        driver.sign_in();
        driver.settle();
        for key in [31, 32] {
            driver.send(Event::Party(PartyInput::Ask {
                reply_to: ReplyTo::new(Token::new(u64::from(key))),
                sign_in: driver.session(),
                key: [key; 16],
                ask: Ask::StartChat { project: 1, words: b"hello".as_slice().into() },
            }));
            driver.settle();
        }
        driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(7)),
            sign_in: driver.session(),
            read: Read::Conversations { from: None },
        }));
        driver.settle();
        let document = driver.delivered.iter().find_map(|delivery| {
            let Delivery::Read { reply, answer } = delivery else { return None };
            (*reply == ReplyTo::new(Token::new(7))).then_some(answer)
        });
        let Some(Answer::Page(page)) = document else { panic!("rows {rows}: conversation page") };
        assert_eq!(page.rows.len(), expected, "rows {rows}: core builds the configured number of document rows");
    }
}
