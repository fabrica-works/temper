use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::Time;
const WAIT_ACCOUNTS: &[&str] = &["wait", "accounts"];

#[test]
fn account_read_and_wait_share_the_digest_and_recompute_on_either_joined_delivery() {
    for owner in [false, true] {
        let mut root = f::account_root(owner);
        let mut party = f::accounts();
        let mut deployment = f::deployment();
        let before = f::account_read(&mut root, party.clone(), owner.then_some(deployment.clone()));
        let digest = f::last_digest(&before.stdout);
        let call = root.wait_since(WAIT_ACCOUNTS, &["--until", "settled"], &digest);
        f::deliver_accounts(&mut root, party.clone());
        if owner {
            f::deliver_deployment(&mut root, deployment.clone());
        }
        assert!(root.answer(call).is_none(), "identical joined digest stays waiting");
        if owner {
            deployment.accounts.get_mut(0).expect("deployment account").generation = Some(2);
            f::deliver_deployment(&mut root, deployment.clone());
        } else {
            party.accounts.get_mut(0).expect("party account").place = 2;
            f::deliver_accounts(&mut root, party.clone());
        }
        let changed = root.answer(call).expect("change and settled on the same current projection");
        let read = f::account_read(&mut root, party, owner.then_some(deployment));
        assert_eq!(changed.status, 0, "{changed:?}");
        let prefix = changed.stdout.strip_suffix(b",\"met\":true}").expect("wait envelope");
        assert_eq!(f::appended(prefix, b"}"), read.stdout, "read and wait expose identical facts/digest");
        assert_eq!(root.watches.len(), 1, "both joined handles closed");
    }
}
#[test]
fn usable_and_settled_follow_the_party_booleans_verbatim() {
    for usable in [false, true] {
        for settled in [false, true] {
            for (condition, expected) in [("usable", usable), ("settled", settled)] {
                let mut root = f::account_root(true);
                let call = root.submit(&["wait", "accounts", "--until", condition, "--json"]);
                let mut party = f::accounts();
                party.usable = usable;
                party.settled = settled;
                f::deliver_accounts(&mut root, party.clone());
                assert!(root.answer(call).is_none(), "owner's deployment is still unknown");
                f::deliver_deployment(&mut root, f::deployment());
                if expected {
                    assert_eq!(root.answer(call).expect("supplied true").status, 0);
                } else {
                    assert!(root.answer(call).is_none(), "deployment usability cannot satisfy party condition");
                    party.usable = true;
                    party.settled = true;
                    f::deliver_accounts(&mut root, party);
                    assert_eq!(root.answer(call).expect("now true").status, 0);
                }
            }
        }
    }
}
#[test]
fn a_behind_join_never_answers_from_its_cached_facts() {
    for behind in [&f::ACCOUNT_WATCH, &f::DEPLOYMENT_WATCH] {
        let mut root = f::account_root(true);
        let call = root.submit(&["wait", "accounts", "--json"]);
        let mut party = f::accounts();
        let mut deployment = f::deployment();
        f::deliver_accounts(&mut root, party.clone());
        f::deliver_deployment(&mut root, deployment.clone());
        // jig-client marks a retained document Behind when its stream drops.
        root.end(call, behind, c::WatchEnd::Dropped);
        if *behind == f::ACCOUNT_WATCH {
            deployment.accounts.get_mut(0).expect("deployment account").generation = Some(2);
            f::deliver_deployment(&mut root, deployment.clone());
        } else {
            party.usable = true;
            f::deliver_accounts(&mut root, party.clone());
        }
        assert!(root.answer(call).is_none(), "other handle changed while join was Behind");
        root.now = crate::root::next_deadline(&root.root).expect("reopen deadline");
        root.fire();
        f::deliver_accounts(&mut root, party);
        f::deliver_deployment(&mut root, deployment);
        assert_eq!(root.answer(call).expect("both current after refresh").status, 0);
    }
}
#[test]
fn account_timeout_handles_unknown_facts_and_releases_both_watches() {
    for delivered in [false, true] {
        let mut root = f::account_root(true);
        let call = root.submit(&["wait", "accounts", "--until", "usable", "--timeout", "1", "--json"]);
        root.watch_event(&f::ACCOUNT_WATCH, c::WatchEvent::Opened);
        root.watch_event(&f::DEPLOYMENT_WATCH, c::WatchEvent::Opened);
        if delivered {
            f::deliver_accounts(&mut root, f::accounts());
            f::deliver_deployment(&mut root, f::deployment());
        }
        root.now = Time::ZERO.checked_add(skein_lib::Duration::from_secs(1)).expect("deadline");
        root.fire();
        let printed = root.answer(call).expect("timeout");
        assert_eq!(printed.status, 3);
        assert!(f::json_has(&printed, b"\"why\":\"timeout\",\"met\":false"));
        assert_eq!(root.watches.len(), 1);
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
    let mut root = f::account_root(true);
    let call = root.submit(&["wait", "accounts", "--until", "usable"]);
    root.watch_event(&f::ACCOUNT_WATCH, c::WatchEvent::Opened);
    root.watch_event(&f::DEPLOYMENT_WATCH, c::WatchEvent::Opened);
    root.end(call, &f::DEPLOYMENT_WATCH, c::WatchEnd::NotAllowed);
    assert_eq!(root.answer(call).expect("deployment permission lost").status, 2);
    assert_eq!(root.watches.len(), 1);
}
#[test]
fn account_since_and_until_need_both_and_text_uses_the_same_read_form() {
    let mut root = f::account_root(false);
    let mut party = f::accounts();
    let before = f::account_read(&mut root, party.clone(), None);
    let call = root.wait_since(WAIT_ACCOUNTS, &["--until", "usable"], &f::last_digest(&before.stdout));
    f::deliver_accounts(&mut root, party.clone());
    party.accounts.get_mut(0).expect("row").place = 2;
    f::deliver_accounts(&mut root, party.clone());
    assert!(root.answer(call).is_none(), "digest changed but usable is false");
    party.usable = true;
    f::deliver_accounts(&mut root, party.clone());
    assert_eq!(root.answer(call).expect("both hold").status, 0);
    let read = root.submit(&["accounts"]);
    f::deliver_accounts(&mut root, party.clone());
    let expected = root.answer(read).expect("text read");
    let wait = root.submit(&["wait", "accounts", "--until", "usable"]);
    f::deliver_accounts(&mut root, party);
    assert_eq!(root.answer(wait).expect("text wait").stdout, f::appended(&expected.stdout, b"met\n"));
}
