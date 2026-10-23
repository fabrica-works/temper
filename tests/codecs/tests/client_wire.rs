//! jig's core refusal remains opaque in the forge family (client/README.md, 4.6).
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
    let parts = v1::AnswerParts { outcome: v1::Outcome::CoreRefused(carried) };
    let answer = v1::Answer::new(&v1::CEILINGS, parts).expect("an answer");
    let bytes = temper_client_protocol_wire::answer(&answer);
    let decoded = v1::Answer::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).expect("decodes");
    let v1::Outcome::CoreRefused(kept) = decoded.into_parts().outcome else { panic!("a core refusal") };
    let back = jig::Refusal::decode(&jig::CEILINGS, &mut Reader::new(kept.refusal())).expect("jig's refusal");
    assert_eq!(back, refusal);
}

#[test]
fn version_four_request_answer_and_forges_goldens_round_trip() {
    use temper_client_protocol_wire as wire;
    for bytes in [
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_requesting_smallest.bin")
            .as_slice(),
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_requesting_full.bin").as_slice(),
    ] {
        let record = wire::decode_requesting(bytes, &v1::CEILINGS).expect("request golden");
        assert_eq!(wire::requesting(&record).as_ref(), bytes);
    }
    for bytes in [
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_answer_smallest.bin").as_slice(),
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_answer_full.bin").as_slice(),
    ] {
        let record = v1::Answer::decode(&v1::CEILINGS, &mut Reader::new(bytes)).expect("answer golden");
        assert_eq!(wire::answer(&record).as_ref(), bytes);
    }
    for bytes in [
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_forges_smallest.bin").as_slice(),
        include_bytes!("../../../crates/temper-client-protocol-wire/golden/v1/record_forges_full.bin").as_slice(),
    ] {
        let record = wire::decode_forges(bytes, &v1::CEILINGS).expect("forges golden");
        assert_eq!(wire::forges(&record).as_ref(), bytes);
    }
}
