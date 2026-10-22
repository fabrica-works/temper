//! The init template's two-row bound is below ctl's unchanged 32-row window.
use skein_world::domain::heap::Counting;
use std::{cell::RefCell, rc::Rc};
use temper_real_loop_world::template_bounds::{self, Seen};
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn reads_answer_at_the_templates_bounds() {
    let seen = Rc::new(RefCell::new(Seen::default()));
    template_bounds::run(&seen);
    let seen = seen.borrow();
    assert!(seen.completions <= 4, "delegation and wait, plus two reports: {} completions", seen.completions);
    assert!(seen.activity.len() >= 2, "ended activity spans multiple pages");
    assert!(seen.activity.iter().map(|page| page["rows"].as_array().expect("rows").len()).sum::<usize>() >= 3);
    let tree = seen.tree.split_once("tree:\n").expect("tree").1.split_once("digest:").expect("tree digest").0;
    assert_eq!(tree.lines().filter(|line| line.contains(" · ")).count(), 3, "root and two rows: {tree}");
    assert!(tree.starts_with("1 · "), "{tree}");
    assert!(tree.contains("cut: 1 more not shown (1 Waiting); read on with: tasks --project 1 --parent 1\n"), "{tree}");
    assert_eq!(seen.delegates, [4, 5, 6], "all live delegates reached");
}
