//! Arbitrary and mutated channel parts (protocol/channel.md, section 9).
use skein_lib::{Reader, Rng, Writer};
use temper_protocol_worker_wire::v1;

fn items(bytes: &[u8]) {
    if let Ok(record) = v1::Items::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Items");
    }
}

fn left(bytes: &[u8]) {
    if let Ok(record) = v1::Left::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Left");
    }
}

fn saved(bytes: &[u8]) {
    if let Ok(record) = v1::Saved::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Saved");
    }
}

fn preparation(bytes: &[u8]) {
    if let Ok(record) = v1::Preparation::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Preparation");
    }
}

fn identity(bytes: &[u8]) {
    if let Ok(record) = v1::Identity::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Identity");
    }
}

#[test]
fn worker_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 5] = [items, left, saved, preparation, identity];
    let mut rng = Rng::new(0x7e3d_0301);
    let goldens = temper_codecs_world::goldens("crates/temper-protocol-worker-wire/golden/v1");
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
