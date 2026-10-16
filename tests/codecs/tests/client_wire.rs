//! jig's core refusal remains opaque in the forge family (client.md, 4.6).
use jig_client_protocol_wire as jig;
use skein_lib::{Reader, Writer};
use temper_client_protocol_wire::v1;

#[test]
fn a_core_refusal_carries_jigs_refusal_bytes_unchanged() {
    let refusal = jig::Refusal::NotWaiting;
    let mut writer = Writer::new(usize::try_from(refusal.measure()).expect("measured length"));
    refusal.encode(&mut writer).expect("measured room");
    let carried = v1::CoreRefused::new(&v1::CEILINGS, v1::CoreRefusedParts { refusal: writer.finish() })
        .expect("within the ceiling");
    let parts = v1::AdoptAnswerParts { outcome: v1::Outcome::CoreRefused(carried) };
    let answer = v1::AdoptAnswer::new(&v1::CEILINGS, parts).expect("an answer");
    let bytes = temper_client_protocol_wire::answer(&answer);
    let decoded = v1::AdoptAnswer::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).expect("decodes");
    let v1::Outcome::CoreRefused(kept) = decoded.into_parts().outcome else { panic!("a core refusal") };
    let back = jig::Refusal::decode(&jig::CEILINGS, &mut Reader::new(kept.refusal())).expect("jig's refusal");
    assert_eq!(back, refusal);
}
