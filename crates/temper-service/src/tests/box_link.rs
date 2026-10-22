use crate::box_link::{Link, Out, Refused, Step};
use alloc::boxed::Box;
use skein_lib::Queue;

const LINE_BYTES: u32 = 32;
const FORGE: u16 = 7;
const OTHER: u16 = 8;
// Enough for the longest script below, including its ask's three outputs.
const OUTPUTS: u32 = 32;

fn asked() -> (Link, Queue<Out>) {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, &mut out);
    discard(&mut out);
    (link, out)
}
fn discard(out: &mut Queue<Out>) {
    for _ in 0..out.len() {
        drop(out.pop());
    }
}
fn named(text: &[u8]) -> Out {
    Out::Named(Box::from(text))
}
fn accepted() -> [Out; 3] {
    [
        Out::Write(Box::from(b"start 7\n".as_slice())),
        Out::SettingUp { forge: FORGE, step: None },
        Out::Answered { forge: FORGE, answer: Ok(()) },
    ]
}
fn tells(out: &mut Queue<Out>, expected: &[Out]) {
    for event in expected {
        assert_eq!(out.pop(), Some(event.clone()));
    }
    assert_eq!(out.pop(), None);
}
#[test]
fn an_ask_writes_start_and_sets_up_with_no_step() {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.ask(FORGE, &mut out);
    tells(&mut out, &accepted());
}
#[test]
fn another_ask_is_refused_busy_until_started_or_failed() {
    for ending in [b"started 7\n".as_slice(), b"failed 7 prepare exited 1\n"] {
        let (mut link, mut out) = asked();
        link.ask(FORGE, &mut out);
        link.ask(OTHER, &mut out);
        tells(
            &mut out,
            &[
                Out::Answered { forge: FORGE, answer: Err(Refused::Busy) },
                named(b"box link: refused forge 7: busy"),
                Out::Answered { forge: OTHER, answer: Err(Refused::Busy) },
                named(b"box link: refused forge 8: busy"),
            ],
        );
        link.read(ending, &mut out);
        discard(&mut out);
        link.ask(FORGE, &mut out);
        tells(&mut out, &accepted());
    }
}
#[test]
fn every_ask_is_refused_not_under_the_box_without_a_pipe() {
    let mut link = Link::new(LINE_BYTES, false);
    let mut out = Queue::with_capacity(OUTPUTS);
    for (forge, message) in [
        (FORGE, b"box link: refused forge 7: not under the box".as_slice()),
        (OTHER, b"box link: refused forge 8: not under the box".as_slice()),
        (FORGE, b"box link: refused forge 7: not under the box".as_slice()),
    ] {
        link.ask(forge, &mut out);
        assert_eq!(out.pop(), Some(Out::Answered { forge, answer: Err(Refused::NotUnderTheBox) }));
        assert_eq!(out.pop(), Some(named(message)));
        assert_eq!(out.pop(), None);
    }
    link.read(b"started 7\n", &mut out);
    link.closed(&mut out);
    tells(&mut out, &[]);
}
#[test]
fn each_line_for_the_asked_forge_is_told() {
    let rows = [
        (b"step 7 fetch\n".as_slice(), Out::SettingUp { forge: FORGE, step: Some(Step::Fetch) }),
        (b"step 7 prepare\n", Out::SettingUp { forge: FORGE, step: Some(Step::Prepare) }),
        (b"step 7 run\n", Out::SettingUp { forge: FORGE, step: Some(Step::Run) }),
        (b"started 7\n", Out::Started { forge: FORGE }),
        (
            b"failed 7 prepare 7 exited 1\n",
            Out::Failed { forge: FORGE, why: Box::from(b"prepare 7 exited 1".as_slice()) },
        ),
        (b"failed 7  raw\xff  why \r\n", Out::Failed { forge: FORGE, why: Box::from(b" raw\xff  why \r".as_slice()) }),
    ];
    for (line, event) in rows {
        let (mut link, mut out) = asked();
        link.read(line, &mut out);
        tells(&mut out, &[event]);
    }
}
#[test]
fn a_line_for_another_forge_or_with_none_asked_is_named_and_dropped() {
    let rows = [
        (b"step 8 fetch\n".as_slice(), b"box link: dropped step for forge 8, not asked".as_slice()),
        (b"started 8\n", b"box link: dropped started for forge 8, not asked"),
        (b"failed 8 busy\n", b"box link: dropped failed for forge 8, not asked"),
    ];
    for (line, message) in rows {
        for ask in [false, true] {
            let mut link = Link::new(LINE_BYTES, true);
            let mut out = Queue::with_capacity(OUTPUTS);
            if ask {
                link.ask(FORGE, &mut out);
                discard(&mut out);
            }
            link.read(line, &mut out);
            tells(&mut out, &[named(message)]);
            if ask {
                link.read(b"started 7\n", &mut out);
                tells(&mut out, &[Out::Started { forge: FORGE }]);
            }
        }
    }
}
#[test]
fn an_unknown_word_step_or_number_is_named_and_dropped() {
    let lines: &[&[u8]] = &[
        b"unknown 7\n",
        b"step 7 stop\n",
        b"started x\n",
        b"started 65536\n",
        b"started +7\n",
        b"started -7\n",
        b"started \n",
        b"started 7 \n",
        b"started  7\n",
        b"step 7  fetch\n",
        b"step 7 fetch \n",
        b"step 7\n",
        b"failed 7\n",
        b"failed 7 \n",
        b"started 7\r\n",
        b"\n",
        b"\xff 7\n",
    ];
    for line in lines {
        let (mut link, mut out) = asked();
        link.read(line, &mut out);
        tells(&mut out, &[named(b"box link: dropped an unknown line")]);
        link.read(b"started 7\n", &mut out);
        tells(&mut out, &[Out::Started { forge: FORGE }]);
    }
}
#[test]
fn a_line_at_the_bound_is_read_and_one_byte_past_it_is_skipped_to_its_line_feed() {
    // "failed 7 " + 22 reason bytes + LF = LINE_BYTES.
    let rows = [
        (
            b"failed 7 1234567890123456789012\n".as_slice(),
            Out::Failed { forge: FORGE, why: Box::from(b"1234567890123456789012".as_slice()) },
        ),
        (b"failed 7 12345678901234567890123\n", named(b"box link: dropped a line past box.line_bytes (32)")),
    ];
    for (line, event) in rows {
        let (mut link, mut out) = asked();
        // The overflow ends in a later read, after the bound was filled.
        let (before_lf, lf) = line.split_at_checked(line.len() - 1).expect("fixture ends in LF");
        link.read(before_lf, &mut out);
        tells(&mut out, &[]);
        link.read(lf, &mut out);
        tells(&mut out, &[event]);
        link.ask(FORGE, &mut out);
        discard(&mut out);
        link.read(b"step 7 run\n", &mut out);
        tells(&mut out, &[Out::SettingUp { forge: FORGE, step: Some(Step::Run) }]);
    }
}
#[test]
fn lines_split_and_joined_across_reads_are_read_alike() {
    const SCRIPT: &[u8] = b"step 7 fetch\nstep 7 prepare\nstep 7 run\nfailed 7 12345678901234567890123\nstarted 7\n";
    const ODD_SPLIT: usize = 17; // Inside prepare's step line.
    let (mut whole, mut whole_out) = asked();
    let (mut bytes, mut bytes_out) = asked();
    let (mut chunks, mut chunks_out) = asked();
    whole.read(SCRIPT, &mut whole_out);
    for byte in SCRIPT {
        bytes.read(core::slice::from_ref(byte), &mut bytes_out);
    }
    let (first, second) = SCRIPT.split_at_checked(ODD_SPLIT).expect("split inside script");
    chunks.read(first, &mut chunks_out);
    chunks.read(second, &mut chunks_out);
    let expected = [
        Out::SettingUp { forge: FORGE, step: Some(Step::Fetch) },
        Out::SettingUp { forge: FORGE, step: Some(Step::Prepare) },
        Out::SettingUp { forge: FORGE, step: Some(Step::Run) },
        named(b"box link: dropped a line past box.line_bytes (32)"),
        Out::Started { forge: FORGE },
    ];
    tells(&mut whole_out, &expected);
    tells(&mut bytes_out, &expected);
    tells(&mut chunks_out, &expected);
}
#[test]
fn the_pipe_closed_fails_the_ask_and_refuses_every_later_one() {
    let (mut link, mut out) = asked();
    link.read(b"step 7 pre", &mut out);
    link.closed(&mut out);
    tells(
        &mut out,
        &[
            named(b"box link: the box closed the pipe"),
            Out::Failed { forge: FORGE, why: Box::from(b"the box closed the pipe".as_slice()) },
        ],
    );
    link.read(b"pare\nstarted 7\n", &mut out);
    link.closed(&mut out);
    tells(&mut out, &[]);
    for _ in [FORGE, FORGE] {
        link.ask(FORGE, &mut out);
        tells(
            &mut out,
            &[
                Out::Answered { forge: FORGE, answer: Err(Refused::NotUnderTheBox) },
                named(b"box link: refused forge 7: not under the box"),
            ],
        );
    }
}
#[test]
fn an_idle_pipe_closes_once_without_failing_a_forge() {
    let mut link = Link::new(LINE_BYTES, true);
    let mut out = Queue::with_capacity(OUTPUTS);
    link.closed(&mut out);
    link.closed(&mut out);
    tells(&mut out, &[named(b"box link: the box closed the pipe")]);
}
#[test]
fn forge_numbers_at_both_u16_bounds_are_read() {
    for (forge, line) in [(0, b"started 0\n".as_slice()), (u16::MAX, b"started 65535\n")] {
        let mut link = Link::new(LINE_BYTES, true);
        let mut out = Queue::with_capacity(OUTPUTS);
        link.ask(forge, &mut out);
        discard(&mut out);
        link.read(line, &mut out);
        tells(&mut out, &[Out::Started { forge }]);
    }
}
