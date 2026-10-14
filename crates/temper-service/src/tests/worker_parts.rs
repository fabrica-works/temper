use crate::worker_route;
use jig_domain_host as host;
use skein_lib::Writer;
use smith_host_domain as agent;
use temper_worker_domain::Invalid;

#[test]
fn delivery_evidence_disagreeing_with_its_outcome_is_invalid() {
    let record = smith_channel::DeliveryReply::new(
        &smith_channel::CEILINGS,
        smith_channel::DeliveryReplyParts { value: smith_channel::Delivery::Nothing },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    for (row, outcome, evidence, expected) in [
        ("agreeing", host::DeliveryOutcome::Nothing, bytes.as_ref(), Ok(agent::Delivery::Nothing)),
        ("disagreeing", host::DeliveryOutcome::Delivered, bytes.as_ref(), Err(Invalid::Evidence)),
        ("malformed", host::DeliveryOutcome::Nothing, &[255, 0, 1], Err(Invalid::Evidence)),
    ] {
        assert_eq!(worker_route::evidence(outcome, evidence), expected, "{row}");
    }
    let mut writer = Writer::new(bytes.len() + 1);
    writer.put(&bytes).unwrap();
    writer.put(&[0]).unwrap();
    let trailing = writer.finish();
    assert_eq!(worker_route::evidence(host::DeliveryOutcome::Nothing, &trailing), Err(Invalid::Evidence), "trailing");
}

#[test]
fn a_call_name_is_encoded_as_smiths_channel_record() {
    let config = super::fixtures::agent();
    let name = agent::CallName { activation: 7, completion: 2, position: 3 };
    let encoded = worker_route::encode(temper_worker_domain::Encodable::CallName(name), &config);
    let temper_worker_domain::Encoded::CallName(bytes) = encoded else { panic!("call name encoded") };
    let record = smith_channel::CallName::decode(&config.limits.bodies, &mut skein_lib::Reader::new(&bytes)).unwrap();
    assert_eq!((record.activation(), record.completion(), record.position()), (7, 2, 3));
}

#[test]
fn a_process_exit_keeps_its_owner_and_status() {
    let owner = skein_lib::Token::new(9);
    assert_eq!(
        worker_route::process(smith_host_protocol::ProcessEvent::Exited {
            agent: owner,
            exit: Some(skein_io::kernel::Exit::Code(3)),
        }),
        Some(temper_worker_domain::Event::Process(Box::new(agent::process::Event::Exited {
            owner,
            exit: Some(agent::Exit::Code(3)),
        }))),
    );
}
