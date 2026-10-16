use super::fixtures;
use crate::command::{self, Command, Line, Lines, Refusal};
use skein_ui::{Act, Tree};

#[test]
fn each_command_line_reads_as_its_command() {
    let (tree, numbers) = fixtures::controls();
    let id = |n| numbers.node(n).expect("shown number");
    let rows: &[(&[u8], Command)] = &[
        (b"p", Command::Print),
        (b"  q ", Command::Close),
        (b"", Command::Nothing),
        (b"   ", Command::Nothing),
        (b"1", Command::Act { node: id(1), act: Act::Press }),
        (b"2", Command::Act { node: id(2), act: Act::Press }),
        (b"3", Command::Act { node: id(3), act: Act::Press }),
        (b"4", Command::Act { node: id(4), act: Act::Toggle }),
        (b"5", Command::Act { node: id(5), act: Act::Toggle }),
        (b"6 acme", Command::Write { node: id(6), value: Box::from(*b"acme"), submit: true }),
        (b"6", Command::Write { node: id(6), value: Box::from([]), submit: true }),
        (b"6  two words ", Command::Write { node: id(6), value: Box::from(*b"two words"), submit: true }),
        (b"7 a  b\t\xff", Command::Write { node: id(7), value: Box::from(*b"a  b\t\xff"), submit: false }),
        (b"7", Command::Write { node: id(7), value: Box::from([]), submit: false }),
    ];
    for (line, expected) in rows {
        assert_eq!(&command::read(line, Some(&numbers), &tree), expected, "line {line:?}");
    }
}
#[test]
fn each_refused_line_reads_as_its_refusal() {
    let (tree, numbers) = fixtures::controls();
    let missing = Tree::new(fixtures::tree_limits().nodes);
    let rows: &[(&[u8], bool, &Tree, Refusal)] = &[
        (b"0", true, &tree, Refusal::NoNumber { number: 0 }),
        (b"8", true, &tree, Refusal::NoNumber { number: 8 }),
        (b"4294967295", true, &tree, Refusal::NoNumber { number: u32::MAX }),
        (b"1", true, &missing, Refusal::Gone { number: 1 }),
        (b"1 words", true, &tree, Refusal::NoWords { number: 1 }),
        (b"1", false, &tree, Refusal::NoPage),
        (b"x", true, &tree, Refusal::Unknown),
        (b"3x", true, &tree, Refusal::Unknown),
        (b"12345678901", true, &tree, Refusal::Unknown),
        (b"4294967296", true, &tree, Refusal::Unknown),
        (b"p words", true, &tree, Refusal::Unknown),
        (b"1\twords", true, &tree, Refusal::Unknown),
    ];
    for (line, printed, tree, expected) in rows {
        assert_eq!(
            command::read(line, printed.then_some(&numbers), tree),
            Command::Refused(*expected),
            "line {line:?}"
        );
    }
}
#[test]
fn each_refusal_has_its_line() {
    let rows: &[(Refusal, &[u8], bool)] = &[
        (Refusal::NoNumber { number: 8 }, b"temper: no [8] on the page\n", true),
        (Refusal::Gone { number: 4 }, b"temper: [4] is gone\n", true),
        (Refusal::NoWords { number: 1 }, b"temper: [1] takes no words\n", false),
        (Refusal::NoPage, b"temper: no page yet\n", false),
        (Refusal::TooLong, b"temper: line too long\n", false),
        (Refusal::Unknown, b"temper: commands: n, n words, p, q\n", false),
    ];
    for (refusal, expected, reprints) in rows {
        assert_eq!(command::refusal_line(*refusal).as_ref(), *expected, "refusal {refusal:?}");
        assert_eq!(refusal.reprints(), *reprints, "refusal {refusal:?}");
    }
}
#[test]
fn a_number_kept_across_a_rebuild_names_the_same_node() {
    let (old, numbers) = fixtures::controls();
    let node = numbers.node(4).expect("check shown");
    let limits = fixtures::tree_limits();
    for (name, state, bound, expected) in [
        ("bound", skein_ui::State::Pressed, true, Command::Act { node, act: Act::Toggle }),
        ("disabled", skein_ui::State::Disabled, true, Command::Refused(Refusal::Gone { number: 4 })),
        ("unbound", skein_ui::State::Pressed, false, Command::Refused(Refusal::Gone { number: 4 })),
    ] {
        let mut next = fixtures::control_tree_with(Some(state), bound);
        skein_ui::diff(&old, &mut next, &limits, &mut old.len(), &mut skein_lib::Queue::with_capacity(limits.patches));
        assert_eq!(next.find(node).expect("same check identity").role, skein_ui::Role::Check, "{name}: identity");
        assert_eq!(command::read(b"4", Some(&numbers), &next), expected, "{name}: command");
    }
}
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
