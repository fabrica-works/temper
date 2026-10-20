use super::fixtures::{DIRECTORIES, facts, file};
use crate::{Directory, files};

#[test]
fn the_digest_is_the_secrets() {
    for (row, secret, digits, digest) in [
        (
            "zero",
            [0; 32],
            b"0000000000000000000000000000000000000000000000000000000000000000\n",
            b"60e05bd1b195af2f94112fa7197a5c88289058840ce7c6df9693756bc6250f55\n",
        ),
        (
            "ascending",
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
                28, 29, 30, 31,
            ],
            b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\n",
            b"6c86c6aac5fb24bcf5d9939cb7d7d5645ce39418f449e03b262dd4fa14b4b92b\n",
        ),
    ] {
        let answered = files(&facts(), &DIRECTORIES, &secret).expect("valid facts");
        assert_eq!(file(&answered, b"worker.secret").bytes.as_ref(), digits, "{row}");
        assert_eq!(file(&answered, b"worker.digest").bytes.as_ref(), digest, "{row}");
    }
}

#[test]
fn the_files_are_named_in_order() {
    let answered = files(&facts(), &DIRECTORIES, &[0; 32]).expect("valid facts");
    let expected = [
        ("secret", b"worker.secret".as_slice(), Directory::Data, true),
        ("digest", b"worker.digest".as_slice(), Directory::Data, true),
        ("chat", b"charters/chat.txt".as_slice(), Directory::Config, false),
        ("coding", b"charters/coding.txt".as_slice(), Directory::Config, false),
        ("agent", b"agent.json".as_slice(), Directory::Config, false),
        ("document last", b"temper.json".as_slice(), Directory::Config, false),
    ];
    assert_eq!(answered.len(), expected.len());
    for (actual, (row, name, directory, private)) in answered.iter().zip(expected) {
        assert_eq!((actual.name, actual.directory, actual.private), (name, directory, private), "{row}");
    }
}

#[test]
fn the_charters_are_the_owned_examples() {
    let answered = files(&facts(), &DIRECTORIES, &[0; 32]).expect("valid facts");
    for (row, name, expected) in [
        (
            "chat",
            b"charters/chat.txt".as_slice(),
            include_bytes!("../../../../docs/examples/owned/charters/chat.txt").as_slice(),
        ),
        (
            "coding",
            b"charters/coding.txt".as_slice(),
            include_bytes!("../../../../docs/examples/owned/charters/coding.txt").as_slice(),
        ),
    ] {
        assert_eq!(file(&answered, name).bytes.as_ref(), expected, "{row}");
    }
}
