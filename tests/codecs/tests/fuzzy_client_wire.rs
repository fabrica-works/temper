//! Arbitrary and mutated forge-family records (skein's codec.md, section 6).
use skein_lib::{Reader, Rng, Writer};
use temper_client_protocol_wire::v1;

fn adopt(bytes: &[u8]) {
    if let Ok(record) = v1::Adopt::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Adopt");
    }
}

fn answer(bytes: &[u8]) {
    if let Ok(record) = v1::AdoptAnswer::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "AdoptAnswer");
    }
}

#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 2] = [adopt, answer];
    let mut rng = Rng::new(0x7e3d_0302);
    let goldens = temper_codecs_world::goldens("crates/temper-client-protocol-wire/golden/v1");
    for _ in 0..2_000 {
        let bytes = temper_codecs_world::drawn(&mut rng, 512);
        for decode in decoders {
            decode(&bytes);
        }
    }
    for golden in &goldens {
        for _ in 0..8 {
            let bytes = temper_codecs_world::mutated(&mut rng, golden);
            for decode in decoders {
                decode(&bytes);
            }
        }
    }
}
