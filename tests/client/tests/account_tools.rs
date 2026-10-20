use skein_lib::Secret;
use temper_client_world::{
    app::{Event, Root},
    fixtures,
    front::{Joint, account_acts as accounts},
};
const KEY: &[u8] = b"test-private-key";
const ADD: &[&[u8]] = &[b"add-key", b"17", b"file", b"--key", b"tools-key"];
fn call(front: &mut Joint, root: &mut Root, number: u32, words: &[&[u8]], value: Option<Secret>) {
    let Event::Call { call, json, .. } = fixtures::call(words) else { unreachable!() };
    accounts::event(front, root, Event::Call { number, call, json, value });
}
#[test]
fn a_lost_tools_key_answer_recovers_one_account() {
    let (mut front, mut root) = accounts::tools_fixture(fixtures::SEED);
    accounts::lose_next(&mut front);
    call(&mut front, &mut root, 1, ADD, Some(Secret::new(KEY.into())));
    accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    accounts::lost(&mut front, &mut root);
    assert_eq!(root.agent().printed(1).expect("lost answer").status, 1);
    call(&mut front, &mut root, 2, ADD, Some(Secret::new(KEY.into())));
    assert_eq!(root.agent().printed(2).expect("original answer").status, 0);
    call(&mut front, &mut root, 4, &[b"accounts", b"--json"], None);
    let printed = root.agent().printed(4).expect("one account projection");
    let text = std::str::from_utf8(&printed.stdout).expect("JSON");
    assert_eq!(text.matches("\"display\"").count(), 1, "{text}");
    accounts::assert_no_carried(&front);
}
#[test]
fn a_tools_connect_wait_settles_at_the_sign_in_end() {
    use jig_domain_accounts as a;
    let (mut front, mut root) = accounts::tools_fixture(fixtures::SEED);
    call(&mut front, &mut root, 1, &[b"connect", b"17", b"--key", b"sign-in"], None);
    let source = accounts::signing(&front);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::Visited {
        source,
        address: b"http://engine/visit".as_slice().into(),
    }));
    accounts::settle(&mut front, &mut root);
    assert_eq!(root.agent().printed(1).expect("durable visit").status, 0);
    call(&mut front, &mut root, 2, &[b"wait", b"accounts", b"--until", b"settled"], None);
    assert!(root.agent().printed(2).is_none());
    front.engine.send(temper_engine_domain::Event::Account(a::Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"Test".as_slice().into(),
        declined: false,
    }));
    accounts::settle(&mut front, &mut root);
    assert_eq!(root.agent().printed(2).expect("settled watch").status, 0);
}
#[test]
fn tools_rank_reaches_the_account_holder() {
    let (mut front, mut root, account) = key_fixture();
    call(&mut front, &mut root, 3, &[b"rank", account.as_bytes(), b"1", b"--source", b"key"], None);
    assert_eq!(root.agent().printed(3).expect("ranked").status, 0);
}
#[test]
fn tools_retry_preserves_the_engines_not_limited_refusal() {
    let (mut front, mut root, account) = key_fixture();
    call(&mut front, &mut root, 3, &[b"retry", account.as_bytes()], None);
    assert_eq!(root.agent().printed(3).expect("not limited").status, 2);
}
#[test]
fn tools_forget_removes_the_account() {
    let (mut front, mut root, account) = key_fixture();
    call(&mut front, &mut root, 3, &[b"forget", account.as_bytes()], None);
    assert_eq!(root.agent().printed(3).expect("forgotten").status, 0);
    call(&mut front, &mut root, 4, &[b"accounts", b"--json"], None);
    let printed = root.agent().printed(4).expect("account removed");
    assert!(skein_lib::bytes::find(&printed.stdout, b"\"party\":[]").is_some());
}
fn key_fixture() -> (Joint, Root, String) {
    let (mut front, mut root) = accounts::tools_fixture(fixtures::SEED);
    call(&mut front, &mut root, 1, ADD, Some(Secret::new(KEY.into())));
    let source = accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    assert_eq!(root.agent().printed(1).expect("key kept").status, 0);
    assert_ne!(source, 0);
    call(&mut front, &mut root, 2, &[b"accounts", b"--json"], None);
    let text = std::str::from_utf8(&root.agent().printed(2).expect("account number").stdout).expect("JSON");
    let account =
        text.split("\"number\":").nth(1).expect("party account").split(',').next().expect("number end").to_owned();
    (front, root, account)
}
