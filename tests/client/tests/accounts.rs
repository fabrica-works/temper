use skein_lib::Secret;
use temper_client_domain::{
    self as d,
    pages::accounts::{Act, Operation, Selection, SignInState},
};
use temper_client_world::{fixtures, front::account_acts as accounts};
const NAME: [u8; 16] = [113; 16];
fn act(act: Act) -> temper_client_world::app::Event {
    d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) }.into()
}
fn select() -> temper_client_world::app::Event {
    act(Act::Select {
        selection: Selection {
            holder: jig_client::contract::AccountHolder::Own,
            provider: Some(accounts::PROVIDER),
            account: None,
            source: None,
        },
    })
}
#[test]
fn a_lost_key_answer_replays_under_the_same_name_without_a_second_account() {
    let (mut front, mut root) = accounts::fixture(fixtures::SEED);
    accounts::event(&mut front, &mut root, select());
    accounts::lose_next(&mut front);
    accounts::event(
        &mut front,
        &mut root,
        d::Event::AccountKey { value: Secret::new(Box::from(b"test-private-key".as_slice())) }.into(),
    );
    accounts::event(&mut front, &mut root, act(Act::Submit { name: NAME, operation: Operation::AddKey }));
    let source = accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    accounts::lost(&mut front, &mut root);
    let page = root.person().pages().accounts().expect("account fixture state");
    assert_eq!(page.pending.as_ref().expect("account fixture state").outcome, Some(jig_client::Outcome::Lost));
    accounts::event(
        &mut front,
        &mut root,
        d::Event::AccountKey { value: Secret::new(Box::from(b"test-private-key".as_slice())) }.into(),
    );
    accounts::event(&mut front, &mut root, act(Act::Submit { name: NAME, operation: Operation::AddKey }));
    let root = root.person();
    assert_eq!(
        root.pages()
            .accounts()
            .expect("account fixture state")
            .pending
            .as_ref()
            .expect("account fixture state")
            .outcome,
        Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(
            jig_client::contract::Outcome::Owner(jig_client::contract::OwnerOutcome::AccountKeyKept { source })
        ))))
    );
    let doc = d::pages::accounts::document(root.client()).expect("account fixture state");
    assert_eq!(doc.accounts.len(), 1);
    assert_eq!(doc.accounts.get(0).expect("account fixture state").sources.len(), 1);
    accounts::assert_no_carried(&front);
}
#[test]
fn a_sign_in_started_answer_is_followed_by_its_actual_end_on_the_accounts_watch() {
    use jig_domain_accounts as a;
    let (mut front, mut root) = accounts::fixture(fixtures::SEED);
    accounts::event(&mut front, &mut root, select());
    accounts::event(&mut front, &mut root, act(Act::Submit { name: NAME, operation: Operation::SignIn }));
    let source = accounts::signing(&front);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::Visited {
        source,
        address: Box::from(b"https://issuer/visit".as_slice()),
    }));
    accounts::settle(&mut front, &mut root);
    assert_eq!(
        root.person().pages().accounts().expect("account fixture state").sign_in_state(root.person().client()),
        SignInState::Signing { source }
    );
    front.engine.send(temper_engine_domain::Event::Account(a::Event::SignedIn {
        source,
        subject: Box::from(b"subject".as_slice()),
        display: Box::from(b"Test".as_slice()),
        declined: false,
    }));
    accounts::settle(&mut front, &mut root);
    let root = root.person();
    assert_eq!(
        root.pages().accounts().expect("account fixture state").sign_in_state(root.client()),
        SignInState::Ended { source }
    );
    assert!(d::pages::accounts::document(root.client()).expect("account fixture state").last_end.is_some());
    assert_eq!(root.pages().accounts().expect("account fixture state").sign_in_address(root.client()), None);
}
