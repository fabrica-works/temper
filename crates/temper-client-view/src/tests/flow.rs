use super::fixtures as f;
use crate::flow::{self, Mark};
use skein_ui::{Builder, Role, State};
#[test]
fn flow_steps_mark_the_current_one_and_those_before_it_done() {
    for current in 0..4 {
        let mut b = Builder::new(&f::tree_limits());
        flow::steps(&mut b, &f::words(), &[b"Choose", b"Details", b"Setting up", b"Ready"], current);
        let tree = b.finish();
        assert!(f::find(&tree, Role::List, b"Steps").is_some());
        for (index, item) in tree.nodes().iter().filter(|node| node.role == Role::Item).enumerate() {
            assert_eq!(item.states.as_slice().contains(&State::Current), index == current);
            assert_eq!(item.styles.as_slice().contains(&crate::look::FLOW_DONE), index < current);
        }
    }
}
#[test]
fn a_progress_says_each_lines_state() {
    let mut b = Builder::new(&f::tree_limits());
    flow::progress(
        &mut b,
        &f::words(),
        b"Set up",
        &[(b"Fetch", Mark::Done), (b"Prepare", Mark::UnderWay), (b"Run", Mark::Failed), (b"Connect", Mark::ToCome)],
    );
    let tree = b.finish();
    assert_eq!(f::text(&tree).as_ref(), b"FetchdonePrepareunder wayRunfailedConnect");
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Status).count(), 3);
}
