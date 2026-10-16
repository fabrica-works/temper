use crate::lines::{Line, Lines};

#[test]
fn a_line_cut_at_any_byte_makes_the_same_lines() {
    let input = b"3 acme\r\np\n";
    for split in 0..=input.len() {
        let mut reader = Lines::new(16);
        let mut found = Vec::new();
        for chunk in [&input[..split], &input[split..]] {
            for byte in chunk {
                if let Some(line) = reader.take(*byte) {
                    found.push(line);
                }
            }
        }
        assert_eq!(found, vec![Line::Whole(Box::from(*b"3 acme")), Line::Whole(Box::from(*b"p"))], "split {split}");
        assert!(!reader.holding(), "split {split}: no partial line");
    }
}
#[test]
fn a_line_past_the_limit_is_refused_through_its_newline() {
    let rows: &[(&[u8], Line)] = &[
        (b"abcd\n", Line::Whole(Box::from(*b"abcd"))),
        (b"abcd\r\n", Line::Whole(Box::from(*b"abcd"))),
        (b"abcde\n", Line::TooLong),
        (b"abcd\rXmore\n", Line::TooLong),
    ];
    for (input, expected) in rows {
        let mut reader = Lines::new(4);
        for byte in &input[..input.len() - 1] {
            assert_eq!(reader.take(*byte), None, "input {input:?}: waits for newline");
            assert!(reader.holding(), "input {input:?}: partial line held");
        }
        assert_eq!(reader.take(b'\n').as_ref(), Some(expected), "input {input:?}");
        assert!(!reader.holding(), "input {input:?}: ended line cleared");
        assert_eq!(reader.take(b'p'), None, "input {input:?}: next line begins");
        assert_eq!(reader.take(b'\n'), Some(Line::Whole(Box::from(*b"p"))), "input {input:?}: next line survives");
    }
}
