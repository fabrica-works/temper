pub mod support;
use std::fs;
use temper_forge_setup_shell::process;
#[test]
fn a_second_lock_is_refused_while_the_first_is_held() {
    let scratch = support::Scratch::new("lock_held");
    let path = scratch.path().join("lock");
    let first = process::lock(&path).expect("first lock");
    assert!(matches!(process::lock(&path), Err(process::Refused::Held)));
    drop(first);
    process::lock(&path).expect("released lock");
    scratch.remove();
}
#[test]
fn a_child_holds_the_lock_as_its_stdin_after_the_parent_lets_go() {
    let scratch = support::Scratch::new("lock_child");
    let path = scratch.path().join("lock");
    let spec = support::script(scratch.path(), "exec sleep 60");
    let lock = process::lock(&path).expect("first lock");
    let mut child = process::command(&spec, lock).spawn().expect("child");
    assert_eq!(fs::read_link(format!("/proc/{}/fd/0", child.id())).expect("stdin"), path);
    assert!(matches!(process::lock(&path), Err(process::Refused::Held)));
    child.kill().and_then(|()| child.wait()).expect("child reaped");
    process::lock(&path).expect("released lock");
    scratch.remove();
}
