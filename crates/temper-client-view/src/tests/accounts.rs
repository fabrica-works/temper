#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test assertions inspect semantic trees and diagnostic redaction"
)]
use super::fixtures;
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{
    self as d,
    pages::accounts::{Act as A, Operation, Selection},
};
const PROVIDER: u32 = 17;
const SOURCE: u32 = 43;
const ADDRESS: &[u8] = b"https://issuer.example/visit";
fn offer(provider: u32, name: &[u8], sign_in: bool, program: bool) -> c::ProviderOffer {
    let mut programs = List::with_capacity(1);
    if program {
        programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
    }
    c::ProviderOffer {
        provider,
        name: text(name),
        key_bytes: 4096,
        sign_in,
        signed_in_plan: true,
        key: true,
        programs,
        programs_cut: false,
        none: false,
        manage_usage: None,
    }
}
fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(Box::from(value), 64).unwrap())
}
fn ways_root(offers: List<c::ProviderOffer>, demo: bool) -> d::Domain {
    let limits = fixtures::limits();
    let (mut root, watch, _) = fixtures::root_at(d::Address::Accounts, &limits);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = offers;
    me.providers_cut = true;
    me.deployment_role = Some(c::HeldRole::Owner);
    if demo {
        return fixtures::demo_frame_root(d::Address::Accounts, fixtures::PROJECT, true, None, Some(me));
    }
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    root
}
pub(super) fn offered_root() -> d::Domain {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    ways_root(providers, false)
}
fn button_names(root: &d::Domain) -> Vec<Vec<u8>> {
    let mut view = fixtures::view();
    fixtures::build(&mut view, root)
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Button)
        .filter_map(|node| node.name.as_ref().map(|name| name.to_vec()))
        .collect()
}
#[test]
fn the_accounts_page_names_the_plan_sign_in_and_key_ways() {
    let names = button_names(&offered_root());
    assert!(names.contains(&b"Continue with ChatGPT".to_vec()));
    assert!(names.contains(&b"Use an API key".to_vec()));
}
#[test]
fn two_key_providers_name_each_key_way_by_its_brand() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", false, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"Scripted", false, false)).unwrap();
    let names = button_names(&ways_root(providers, false));
    assert!(names.contains(&b"Use an OpenAI API key".to_vec()));
    assert!(names.contains(&b"Use a Scripted API key".to_vec()));
    assert!(!names.contains(&b"Use an API key".to_vec()));
}
#[test]
fn a_provider_without_a_plan_names_its_sign_in_by_its_brand() {
    let mut providers = List::with_capacity(1);
    let mut provider = offer(PROVIDER, b"anthropic", true, false);
    provider.signed_in_plan = false;
    providers.push(provider).unwrap();
    assert!(button_names(&ways_root(providers, false)).contains(&b"Sign in to Anthropic".to_vec()));
}
#[test]
fn the_demos_provider_offers_no_way() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"demo", true, true)).unwrap();
    providers.push(offer(PROVIDER + 1, b"openai", true, false)).unwrap();
    let names = button_names(&ways_root(providers, true));
    assert!(names.contains(&b"Continue with ChatGPT".to_vec()));
    assert!(names.contains(&b"Use an API key".to_vec()));
    assert!(!names.iter().any(|name| name.windows(b"demo".len()).any(|part| part == b"demo")));
}
#[test]
fn the_deployment_holder_offers_keys_only() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"anthropic", false, true)).unwrap();
    let mut root = ways_root(providers, false);
    action(
        &mut root,
        A::Select { selection: Selection { holder: c::AccountHolder::Deployment, ..Default::default() } },
    );
    let names = button_names(&root);
    assert!(names.contains(&b"Use an OpenAI API key".to_vec()));
    assert!(names.contains(&b"Use an Anthropic API key".to_vec()));
    assert!(!names.contains(&b"Continue with ChatGPT".to_vec()));
    assert!(!names.contains(&b"Use your Claude subscription".to_vec()));
}
#[test]
fn a_program_way_shows_the_default_name_and_the_command_on_each_host() {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"anthropic", false, true)).unwrap();
    let mut root = ways_root(providers, false);
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::Button, b"Use your Claude subscription", Act::Press);
    let tree = fixtures::build(&mut view, &root);
    let field = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"Program name")).unwrap();
    assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"claude-pat");
    assert!(fixtures::find(tree, Role::Button, b"Declare").is_some());
    assert!(
        String::from_utf8(fixtures::text(tree).into_vec())
            .unwrap()
            .contains("On each host, run: temper program-login claude-pat")
    );
}
fn card_document(provider: &[u8], display: &[u8], kind: c::OwnerAccountKind) -> c::OwnerAccounts {
    let mut doc = document(1);
    let row = doc.accounts.get_mut(0).unwrap();
    row.provider_name = Some(text(provider));
    row.display = text(display);
    row.plan = kind != c::OwnerAccountKind::Key;
    let source = row.sources.get_mut(0).unwrap();
    source.kind = kind;
    if kind == c::OwnerAccountKind::Program {
        source.name = Some(Box::from(display));
    }
    doc
}
fn card_heading(doc: c::OwnerAccounts) -> Vec<u8> {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Heading { level: 3 })
        .unwrap()
        .name
        .as_ref()
        .unwrap()
        .to_vec()
}
#[test]
fn a_cards_region_is_named_by_its_heading() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"alice@example.com", c::OwnerAccountKind::SignIn));
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let heading = tree.nodes().iter().find(|node| node.role == Role::Heading { level: 3 }).unwrap();
    assert!(tree.nodes().iter().any(|node| node.role == Role::Region && node.name == heading.name));
}
#[test]
fn a_plan_card_names_the_plan_display_brand_and_holder() {
    assert_eq!(
        card_heading(card_document(b"openai", b"alice@example.com", c::OwnerAccountKind::SignIn)),
        "ChatGPT · alice@example.com · OpenAI · held by Pat".as_bytes()
    );
}
#[test]
fn a_key_only_card_names_the_brand_and_holder_once() {
    assert_eq!(
        card_heading(card_document(b"openai", b"", c::OwnerAccountKind::Key)),
        "OpenAI API key · held by Pat".as_bytes()
    );
}
#[test]
fn a_program_card_names_the_subscription_and_login() {
    let mut doc = card_document(b"anthropic", b"claude-alice", c::OwnerAccountKind::Program);
    // A program declaration has no provider display; its portable name belongs to the source.
    doc.accounts.get_mut(0).unwrap().display = text(b"");
    assert_eq!(card_heading(doc), "Claude subscription · claude-alice · Anthropic · held by Pat".as_bytes());
}
#[test]
fn a_deployment_card_names_its_holder_in_words() {
    let mut doc = card_document(b"openai", b"", c::OwnerAccountKind::Key);
    doc.accounts.get_mut(0).unwrap().holder = c::OwnerAccountHolder::Deployment;
    assert_eq!(card_heading(doc), "OpenAI API key · held by the deployment".as_bytes());
}
#[test]
fn an_empty_display_leaves_no_empty_heading_part() {
    assert_eq!(
        card_heading(card_document(b"openai", b"", c::OwnerAccountKind::SignIn)),
        "ChatGPT · OpenAI · held by Pat".as_bytes()
    );
}
#[test]
fn sources_name_their_kinds_in_the_providers_words() {
    for (provider, display, kind, expected) in [
        (b"openai".as_slice(), b"alice@example.com".as_slice(), c::OwnerAccountKind::SignIn, "ChatGPT sign-in · Ready"),
        (b"openai", b"", c::OwnerAccountKind::Key, "API key · Ready"),
        (b"anthropic", b"claude-alice", c::OwnerAccountKind::Program, "Claude Code login claude-alice · Ready"),
    ] {
        assert!(shown(card_document(provider, display, kind)).contains(expected));
    }
}
#[test]
fn add_an_account_follows_both_account_lists() {
    let (root, _) = owner_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let headings: Vec<_> = tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Heading { level: 2 })
        .map(|node| node.name.as_ref().unwrap().to_vec())
        .collect();
    assert_eq!(headings, [b"Your LLM accounts".to_vec(), b"Deployment accounts".to_vec(), b"Add an account".to_vec()]);
}
#[test]
fn a_cut_provider_name_keeps_the_numbered_card_fallback() {
    let mut doc = card_document(b"openai", b"Pat", c::OwnerAccountKind::SignIn);
    doc.accounts.get_mut(0).unwrap().provider_name.as_mut().unwrap().cut = true;
    assert_eq!(card_heading(doc), "Account 1 · LLM provider 17 · Pat".as_bytes());
}
#[test]
fn an_unknown_provider_keeps_its_configured_name() {
    assert_eq!(
        card_heading(card_document(b"Custom", b"Pat", c::OwnerAccountKind::SignIn)),
        "Custom · Pat · Custom · held by Pat".as_bytes()
    );
}
fn source(number: u32) -> c::OwnerAccountSource {
    c::OwnerAccountSource {
        name: None,
        number,
        place: number,
        kind: c::OwnerAccountKind::Key,
        state: c::OwnerAccountState::Granted,
        usable: true,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}
fn document(count: u32) -> c::OwnerAccounts {
    let mut accounts = List::with_capacity(count);
    let mut signing = List::with_capacity(count);
    for number in 1..=count {
        let mut sources = List::with_capacity(count);
        for n in 1..=count {
            sources.push(source(n)).unwrap();
        }
        accounts
            .push(c::OwnerAccountLine {
                number,
                place: number,
                uses: None,
                plan: true,
                provider: Some(PROVIDER),
                provider_name: Some(c::DocumentText::complete(
                    c::Text::new(Box::from(b"Scripted".as_slice()), 32).unwrap(),
                )),
                manage_usage: None,
                display: c::DocumentText::complete(c::Text::new(Box::from(b"Pat".as_slice()), 32).unwrap()),
                holder: c::OwnerAccountHolder::Party(73),
                sources,
                cut: true,
            })
            .unwrap();
        signing
            .push(c::OwnerAccountSignIn {
                source: number,
                provider: PROVIDER,
                holder: c::OwnerAccountHolder::Party(73),
            })
            .unwrap();
    }
    c::OwnerAccounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing,
        last_end: None,
        usable: true,
        settled: false,
        cut: true,
    }
}
fn deliver(root: &mut d::Domain, watch: Token, doc: c::OwnerAccounts) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(doc))),
                },
            },
        },
    );
}
fn action(root: &mut d::Domain, act: A) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) });
}
fn press(view: &mut crate::View, root: &mut d::Domain, role: Role, name: &[u8], act: Act) {
    let node = fixtures::find(fixtures::build(view, root), role, name).expect("offered account control");
    let event = crate::event(view, node, act).expect("account control decodes");
    fixtures::root_event(root, event);
}
fn shown(doc: c::OwnerAccounts) -> String {
    shown_at(doc, fixtures::limits().accounts)
}
fn shown_at(doc: c::OwnerAccounts, counts: d::limits::AccountLimits) -> String {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = bounded_view(counts);
    String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap()
}
fn bounded_view(counts: d::limits::AccountLimits) -> crate::View {
    let mut domain = fixtures::limits();
    domain.accounts = counts;
    let words = fixtures::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    crate::View::new(&bounds, &domain, words, fixtures::FORGE)
}
#[test]
fn the_accounts_page_keeps_accounts_beyond_rows() {
    assert!(shown(document(3)).contains("Account 3"));
}
#[test]
fn the_accounts_page_keeps_sources_beyond_rows() {
    assert_eq!(shown(document(3)).matches("Source 3").count(), 3);
}
#[test]
fn the_accounts_page_keeps_sign_ins_beyond_rows() {
    assert!(shown(document(3)).contains("Sign-in in progress: source 3"));
}
#[test]
fn the_accounts_page_preserves_supplied_order() {
    let text = shown(document(3));
    assert!(text.find("Account 1").unwrap() < text.find("Account 3").unwrap());
}
#[test]
fn the_accounts_page_names_cut_facts() {
    assert!(shown(document(1)).contains("Some account facts were omitted."));
}
#[test]
fn the_accounts_page_caps_uses() {
    let mut doc = document(1);
    let mut uses = List::with_capacity(300);
    for party in 1..=300 {
        uses.push(c::OwnerAccountUse::Party(party)).unwrap();
    }
    doc.accounts.get_mut(0).unwrap().uses = Some(uses);
    let text = shown(doc);
    assert!(text.contains("Uses left out: 292"));
    assert!(!text.contains("Person 9;"));
}
fn key_draft() -> (d::Domain, crate::View) {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::TextField, b"Provider number", Act::Input { value: Box::from(b"17".as_slice()) });
    press(&mut view, &mut root, Role::TextField, b"Account number", Act::Input { value: Box::from(b"1".as_slice()) });
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    (root, view)
}
fn key_field(view: &crate::View) -> &skein_ui::Node {
    view.tree().nodes().iter().find(|n| n.name.as_deref() == Some(b"Key (pasted once)")).unwrap()
}
#[test]
fn the_key_field_declares_a_secret_entry() {
    let (_, view) = key_draft();
    assert_eq!(key_field(&view).entry, Some(skein_ui::EntryKind::Secret));
}
#[test]
fn a_key_draft_never_enters_the_view() {
    let (_, view) = key_draft();
    assert!(!format!("{view:?}").contains("private-once"));
}
#[test]
fn a_held_key_has_an_empty_view_value() {
    let (root, view) = key_draft();
    assert!(root.pages().accounts().unwrap().has_key());
    assert!(key_field(&view).value.as_ref().unwrap().text.is_empty());
}
#[test]
fn sending_a_key_writes_an_empty_field() {
    let (mut root, mut view) = key_draft();
    let before = key_field(&view).value.as_ref().unwrap().written;
    press(&mut view, &mut root, Role::Button, b"Add key to account", Act::Press);
    fixtures::build(&mut view, &root);
    let value = key_field(&view).value.as_ref().unwrap();
    assert!(value.text.is_empty());
    assert_ne!(value.written, before);
}
fn sign_in_address(address: &[u8]) -> d::Domain {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    action(&mut root, A::Select { selection: Selection { provider: Some(PROVIDER), ..Default::default() } });
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    d::step(
        &mut root,
        &env,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Accounts(A::Submit {
                name: [97; 16],
                operation: Operation::SignIn,
            })),
        },
        &mut out,
    );
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Client {
                request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Show { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Fact { .. } => None,
        })
        .expect("sign-in attempt");
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                        c::OwnerOutcome::AccountSignInStarted { source: SOURCE, address: Box::from(address) },
                    )))),
                    why: None,
                },
            },
        },
    );
    root
}
#[test]
fn a_sign_in_address_has_a_visible_link_word() {
    let root = sign_in_address(ADDRESS);
    let mut view = fixtures::view();
    let text = fixtures::text(fixtures::build(&mut view, &root));
    assert!(String::from_utf8(text.into_vec()).unwrap().contains("Open the sign-in page"));
}
#[test]
fn a_sign_in_address_names_the_engine_machine() {
    let root = sign_in_address(ADDRESS);
    let mut view = fixtures::view();
    let text = fixtures::text(fixtures::build(&mut view, &root));
    assert!(String::from_utf8(text.into_vec()).unwrap().contains("on the engine's machine"));
}
#[test]
fn an_address_containing_a_space_is_refused() {
    unsafe_address(b"https://issuer.example/bad path");
}
#[test]
fn an_uppercase_scheme_is_refused() {
    unsafe_address(b"HTTPS://issuer.example/visit");
}
fn unsafe_address(address: &[u8]) {
    let root = sign_in_address(address);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
    assert!(String::from_utf8(fixtures::text(tree).into_vec()).unwrap().contains("not an http(s) address"));
}
#[test]
fn a_long_sign_in_address_is_refused() {
    let address = format!("https://issuer.example/{}", "x".repeat(1000));
    let root = sign_in_address(address.as_bytes());
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
    let text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert!(text.contains(&format!("longer than this client shows; bytes: {}", address.len())));
}

#[test]
fn supplied_source_states_keep_their_words() {
    use c::OwnerAccountState as S;
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    for (state, expected) in [
        (S::Asking, b"Checking".as_slice()),
        (S::Granted, b"Ready"),
        (S::Later { until: None }, b"Limited. Tasks on it wait."),
        (S::SignInAgain, b"Needs sign-in"),
        (S::Misconfigured, b"Check the configuration's registration."),
        (S::Cancelled, b"Cancelled"),
        (S::Closing, b"Closing"),
        (
            S::Unloaded { at: c::OwnerAccountPlace::File, why: c::OwnerAccountUnread::Stalled },
            b"Cannot load File: Stalled",
        ),
    ] {
        let mut doc = document(1);
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = state;
        deliver(&mut root, watch, doc);
        let shown = fixtures::text(fixtures::build(&mut view, &root));
        assert!(skein_lib::bytes::find(&shown, expected).is_some(), "{state:?}");
        assert!(skein_lib::bytes::find(&shown, b"Limited until").is_none(), "no invented reset");
    }
}
#[test]
fn a_supplied_reset_is_visible() {
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state =
        c::OwnerAccountState::Later { until: Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000)) };
    deliver(&mut root, watch, doc);
    assert!(
        skein_lib::bytes::find(&fixtures::text(fixtures::build(&mut view, &root)), b"Limited until Oct 12, 18:04 UTC")
            .is_some()
    );
}
#[test]
fn sign_in_ends_preserve_their_typed_details() {
    use c::{OwnerAccountEnd as E, OwnerAccountFailure as F, OwnerAccountRefusal as R};
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    for (end, expected) in [
        (E::Made { account: 29, declined: true }, b"You signed in without plan usage.".as_slice()),
        (E::Renewed { account: 29, declined: false }, b"same account again; account 29"),
        (E::Refused { refusal: R::NotKept { why: c::OwnerAccountKeyFailure::TooLong } }, b"The key is too long."),
        (E::Refused { refusal: R::Limit { bound: c::OwnerAccountBound::Sources } }, b"Source limit reached."),
        (E::Refused { refusal: R::Kind { kind: c::OwnerAccountKind::Program } }, b"Source kind refused: Program login"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, b"Missing required scope 19"),
        (E::Failed { failure: F::Later { after: Some(skein_lib::Duration::from_secs(5)) } }, b"Try again after 5"),
    ] {
        let mut doc = document(0);
        doc.last_end = Some(c::OwnerAccountLastEnd { source: SOURCE, end });
        deliver(&mut root, watch, doc);
        assert!(skein_lib::bytes::find(&fixtures::text(fixtures::build(&mut view, &root)), expected).is_some());
    }
}
fn sign_in_end_words(mut doc: c::OwnerAccounts, end: c::OwnerAccountEnd) -> Vec<u8> {
    doc.last_end = Some(c::OwnerAccountLastEnd { source: SOURCE, end });
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    for (index, status) in tree.nodes().iter().enumerate() {
        if status.role != Role::Status {
            continue;
        }
        let node = tree.nodes().get(index.checked_add(1).unwrap()).unwrap();
        let mut words = Vec::new();
        for run in &node.text {
            words.extend_from_slice(&run.text);
        }
        if words.starts_with(b"Signed in") || words.starts_with(b"That's") {
            return words;
        }
    }
    panic!("sign-in end status");
}
#[test]
fn a_completed_sign_in_names_the_plan_and_display() {
    assert_eq!(
        sign_in_end_words(
            card_document(b"openai", b"alice@example.com", c::OwnerAccountKind::SignIn),
            c::OwnerAccountEnd::Made { account: 1, declined: false }
        ),
        b"Signed in to ChatGPT as alice@example.com"
    );
}
#[test]
fn a_renewed_sign_in_explains_how_to_add_another_account() {
    assert_eq!(sign_in_end_words(card_document(b"openai", b"alice@example.com", c::OwnerAccountKind::SignIn), c::OwnerAccountEnd::Renewed { account: 1, declined: false }), b"That's alice@example.com again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.");
}
#[test]
fn a_sign_in_end_keeps_the_number_when_its_account_has_not_arrived() {
    for (end, expected) in [
        (c::OwnerAccountEnd::Made { account: 29, declined: false }, b"Signed in; account 29".as_slice()),
        (
            c::OwnerAccountEnd::Renewed { account: 29, declined: false },
            b"Signed in to the same account again; account 29",
        ),
    ] {
        assert_eq!(sign_in_end_words(document(0), end), expected);
    }
}
#[test]
fn a_sign_in_end_without_a_display_names_the_plan_unless_renewed() {
    for (end, expected) in [
        (c::OwnerAccountEnd::Made { account: 1, declined: false }, b"Signed in to ChatGPT".as_slice()),
        (
            c::OwnerAccountEnd::Renewed { account: 1, declined: false },
            b"Signed in to the same account again; account 1",
        ),
    ] {
        assert_eq!(sign_in_end_words(card_document(b"openai", b"", c::OwnerAccountKind::SignIn), end), expected);
    }
}
#[test]
fn a_named_sign_in_end_still_reports_declined_plan_usage() {
    assert!(
        sign_in_end_words(
            card_document(b"openai", b"alice@example.com", c::OwnerAccountKind::SignIn),
            c::OwnerAccountEnd::Made { account: 1, declined: true }
        )
        .ends_with(b"You signed in without plan usage.")
    );
}
fn answer_account_operation(root: &mut d::Domain, operation: Operation, outcome: c::OwnerOutcome) {
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    for act in [
        A::Select { selection: Selection { account: Some(1), source: Some(1), ..Default::default() } },
        A::Submit { name: [98; 16], operation },
        A::Confirm,
    ] {
        d::step(root, &env, d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) }, &mut out);
    }
    d::step(root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Client {
                request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Show { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Fact { .. } => None,
        })
        .expect("account operation attempt");
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(outcome)))),
                    why: None,
                },
            },
        },
    );
}
#[test]
fn only_settled_account_statuses_have_the_done_style() {
    use crate::accounts::words::{self, Word as W};
    for (outcome, word, done) in [
        (None, W::Settled, true),
        (
            Some((
                Operation::SignOut,
                c::OwnerOutcome::AccountSignedOut { confirmed: true, failed_hosts: Box::new([]) },
            )),
            W::Confirmed,
            true,
        ),
        (
            Some((
                Operation::SignOut,
                c::OwnerOutcome::AccountSignedOut { confirmed: false, failed_hosts: Box::new([]) },
            )),
            W::Unconfirmed,
            false,
        ),
        (
            Some((
                Operation::RemoveKey,
                c::OwnerOutcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
            )),
            W::Removed,
            true,
        ),
        (
            Some((
                Operation::RemoveKey,
                c::OwnerOutcome::AccountRemoved { confirmed: false, failed_hosts: Box::new([]) },
            )),
            W::RemovedUnconfirmed,
            false,
        ),
    ] {
        let (mut root, watch) = fixtures::account_root();
        let mut doc = document(1);
        doc.settled = outcome.is_none();
        deliver(&mut root, watch, doc);
        if let Some((operation, outcome)) = outcome {
            answer_account_operation(&mut root, operation, outcome);
        }
        let mut view = fixtures::view();
        let tree = fixtures::build(&mut view, &root);
        let name = words::say(&fixtures::words(), word, &[]);
        let status = tree
            .nodes()
            .iter()
            .find(|n| n.role == Role::Status && n.name.as_deref() == Some(name.as_ref()))
            .expect("account status line");
        assert_eq!(
            status.styles.as_slice().contains(&crate::look::STATUS_DONE),
            done,
            "{}",
            String::from_utf8_lossy(&name)
        );
    }
}
#[path = "../../../../testing/memory_literal.rs"]
mod memory_literal;
// Base 87c82f753: 4,420,462,083 + 304 added nodes x 574,364 B = 4,595,068,739 B.
// A chat's 5 delegation cards (client/README.md, 5.7): 5 x (66 markdown + 14)
// nodes x 574,364 B + 5 x 8 B of carded tasks = 229,745,640 B, so 4,824,814,379 B.
// Cards remembered across the window: 5 x 8 B = 40 B, so 4,824,814,419 B.
// The live delegates' trailing region adds 574,364 B, so 4,825,388,783 B.
// Goal cards add 236 nodes x 574,364 B and 4,736 B for the larger phrasebook,
// giving 4,960,943,423 B.
// Plan rows add 32 x 574,364 B, phrases 2,368 B and a tool name 32 B,
// less 8 B of root stack: +18,382,040 B, giving 4,979,325,463 B.
// Lower-case phase words add 16 phrases' room, 2,368 B: 4,979,327,831 B.
// Smaller receiving bounds and the full chat window price each turn's shared budget (client/README.md, 5.4).
// Agent lineage, taskbar, brief, run and dependencies add 109 nodes x 574,364 B
// = 62,605,676 B, giving 5,041,933,507 B; its six phrases fit the reserved book.
// The context line adds one repository/forge text node; chat words use existing phrase room.
// Your chats adds filter/empty nodes and five phrases to the checked envelope.
// Procedure goal slot: five added nodes including its paragraph and link text.
// The chat projection adds seven phrases while retaining the agent's work-line words.
// Amendment editor: sixteen nodes and eleven phrases; the web root also retains
// its bounded draft, typed specification and separate amendment outcome.
// Added dependency marks (client/README.md, 5.8): two nodes per dependency,
// across two plan rows and the brief: (2 * 2 + 2) * 2 = 12 nodes x 574,364 B.
// Their five phrases add five entries' room, 740 B.
// Naming navigation and cards adds two phrases (296 B).
// A committed turn's own text budget prices both conversation drawings.
// A task's name over its words in a chat adds one phrase (148 B).
// The four time forms add 47 phrases x 148 B = 6,956 B.
// The change panel adds 86 forge phrases (12,728 B); its full address and
// combined pull/bounds texts fit the existing maximum text envelope.
// The procedure's ended page and its three phrases move the checked envelope.
// skein f043ea16: b6ea7b9c's placeholder and completed state; boxed patch attributes, no slotless splice room.
// The page reserves its patch queue once, at four patches a node.
// A chat's composer placeholder adds one phrase (148 B).
// Exceeded adds nine jig phrases: 9 * 148 B = 1,332 B.
#[expect(clippy::unreadable_literal, reason = "the drift test regenerates this decimal memory count")]
const ACCOUNT_BYTES: u64 = 6978199165;
#[test]
#[cfg(target_pointer_width = "64")]
fn account_bounds_have_a_literal_price() {
    let domain = fixtures::limits();
    let words = fixtures::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    let actual = crate::worst_case(&bounds, &domain, &words).expect("priced fixture");
    memory_literal::check(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/accounts.rs"),
        "const ACCOUNT_BYTES: u64 = ",
        ACCOUNT_BYTES,
        actual,
    );
}
fn owner_root() -> (d::Domain, Token) {
    let limits = fixtures::limits();
    let (mut root, me_watch, _) = fixtures::root_at(d::Address::Accounts, &limits);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Owner);
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let watch =
        out.iter()
            .find_map(|r| {
                if let d::Request::Client {
                    request:
                        jig_client::Request::Down {
                            down:
                                jig_client::Down::Open {
                                    watch,
                                    subject: jig_client::Watchable::Jig(c::Subject::Deployment),
                                },
                        },
                } = r
                {
                    Some(*watch)
                } else {
                    None
                }
            })
            .unwrap();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let deployment = jig_client_protocol_link::translate::documents::owner_deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(deployment))),
                },
            },
        },
    );
    (root, watch)
}
#[test]
fn an_owner_sees_deployment_accounts_after_their_own() {
    let (root, _) = owner_root();
    let mut view = fixtures::view();
    let text = fixtures::text(fixtures::build(&mut view, &root));
    assert!(
        skein_lib::bytes::find(&text, b"Your LLM accounts").unwrap()
            < skein_lib::bytes::find(&text, b"Deployment accounts").unwrap()
    );
    assert!(skein_lib::bytes::find(&text, b"Account 41").is_some());
}
fn invalid_number(view: &mut crate::View, root: &d::Domain) -> Option<d::Event> {
    let node = fixtures::find(fixtures::build(view, root), Role::TextField, b"Account number").unwrap();
    crate::event(view, node, Act::Input { value: Box::from(b"not-a-number".as_slice()) })
}
#[test]
fn invalid_number_input_does_not_dispatch() {
    let (root, mut view) = key_draft();
    assert!(invalid_number(&mut view, &root).is_none());
}
#[test]
fn invalid_number_input_preserves_a_key() {
    let (mut root, mut view) = key_draft();
    if let Some(event) = invalid_number(&mut view, &root) {
        fixtures::root_event(&mut root, event);
    }
    assert!(root.pages().accounts().unwrap().has_key());
}
#[test]
fn invalid_numbers_disable_submission() {
    let (root, mut view) = key_draft();
    invalid_number(&mut view, &root);
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::Button, b"Add key to account").unwrap();
    let button = view.tree().nodes().iter().find(|n| n.id == node).unwrap();
    assert!(button.states.as_slice().contains(&skein_ui::State::Disabled));
    assert!(crate::event(&mut view, node, Act::Press).is_none());
}
#[test]
fn cancel_refreshes_invalid_number_fields() {
    let (mut root, mut view) = key_draft();
    invalid_number(&mut view, &root);
    let before = root.pages().accounts().unwrap().key_written();
    press(&mut view, &mut root, Role::Button, b"Cancel", Act::Press);
    let tree = fixtures::build(&mut view, &root);
    for field in tree.nodes().iter().filter(|n| n.role == Role::TextField) {
        assert!(field.value.as_ref().unwrap().written > before);
    }
}

#[test]
fn expiring_has_a_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().expiring = true;
    assert!(shown(doc).contains("Ready · Expiring"));
}
#[test]
fn attention_has_a_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().attention = true;
    assert!(shown(doc).contains("Ready · Needs attention"));
}
#[test]
fn spent_has_its_own_time_word() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().spent_until =
        Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000));
    assert!(shown(doc).contains("Ready · Usage limit reached · Usage limit until Oct 12, 18:04 UTC"));
}
fn confirmation_words(operation: Operation) -> String {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(1));
    action(&mut root, A::Select { selection: Selection { account: Some(1), source: Some(1), ..Default::default() } });
    action(&mut root, A::Submit { name: [72; 16], operation });
    let mut view = fixtures::view();
    String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap()
}
#[test]
fn sign_out_confirmation_names_its_source() {
    assert!(
        confirmation_words(Operation::SignOut)
            .contains("Sign this source out? · Your own accounts · Account 1 · Source 1")
    );
}
#[test]
fn key_removal_confirmation_names_its_source() {
    assert!(
        confirmation_words(Operation::RemoveKey)
            .contains("Remove this key? · Your own accounts · Account 1 · Source 1")
    );
}
#[test]
fn account_removal_confirmation_names_its_holder() {
    let (mut root, _) = owner_root();
    action(
        &mut root,
        A::Select {
            selection: Selection { holder: c::AccountHolder::Deployment, account: Some(41), ..Default::default() },
        },
    );
    action(&mut root, A::Submit { name: [72; 16], operation: Operation::RemoveAccount });
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("Remove this account and all its sources? · The deployment's accounts · Account 41"));
}
#[test]
fn the_current_holder_is_pressed() {
    let (mut root, _) = owner_root();
    action(
        &mut root,
        A::Select { selection: Selection { holder: c::AccountHolder::Deployment, ..Default::default() } },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let control = tree.nodes().iter().find(|n| n.name.as_deref() == Some(b"Use deployment accounts")).unwrap();
    assert!(control.states.as_slice().contains(&skein_ui::State::Pressed));
    assert!(!control.states.as_slice().contains(&skein_ui::State::Selected));
}
#[test]
fn an_account_without_a_provider_has_one_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().provider = None;
    doc.accounts.get_mut(0).unwrap().provider_name = None;
    assert!(shown(doc).contains("Account 1 · Pat"));
}
#[test]
fn empty_uses_have_no_label() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().uses = Some(List::with_capacity(0));
    assert!(!shown(doc).contains("Uses:"));
}
#[test]
fn an_empty_deployment_names_its_empty_list() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    doc.accounts = List::with_capacity(0);
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(doc))),
                },
            },
        },
    );
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("Deployment accountsNo accounts shown."));
}
#[test]
fn accounts_past_the_count_are_named() {
    let mut counts = fixtures::limits().accounts;
    counts.accounts = 2;
    assert!(shown_at(document(3), counts).contains("Accounts left out: 1"));
}
#[test]
fn sources_past_the_count_are_named() {
    let mut counts = fixtures::limits().accounts;
    counts.sources = 2;
    assert!(shown_at(document(3), counts).contains("Sources left out: 1"));
}
#[test]
fn sign_ins_past_the_count_are_named() {
    let mut counts = fixtures::limits().accounts;
    counts.sign_ins = 2;
    assert!(shown_at(document(3), counts).contains("Sign-ins left out: 1"));
}
#[test]
fn duplicate_numbers_offer_one_rank_control() {
    let mut doc = document(16);
    for at in 0..doc.accounts.len() {
        let row = doc.accounts.get_mut(at).unwrap();
        row.number = 1;
        row.place = 2;
        for at in 0..row.sources.len() {
            let source = row.sources.get_mut(at).unwrap();
            source.number = 1;
            source.place = 2;
        }
    }
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    action(&mut root, A::Select { selection: Selection { account: Some(1), source: Some(1), ..Default::default() } });
    let mut view = bounded_view(fixtures::limits().accounts);
    let tree = fixtures::build(&mut view, &root);
    assert_eq!(tree.nodes().iter().filter(|n| n.name.as_deref() == Some(b"Move source up")).count(), 1);
}
#[test]
fn bindings_at_capacity_refuse_another_control() {
    let counts = fixtures::limits().accounts;
    let mut bindings = crate::accounts::Bindings::new(counts, fixtures::limits().client.domain.rows);
    for _ in 0..crate::accounts::binding_capacity(&counts, fixtures::limits().client.domain.rows) {
        assert!(bindings.insert(crate::accounts::Meaning::Key).is_some());
    }
    assert!(bindings.insert(crate::accounts::Meaning::Key).is_none());
}

#[test]
fn deployment_source_confirmation_names_its_account() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let mut sources = List::with_capacity(1);
    sources.push(source(43)).unwrap();
    doc.accounts.get_mut(0).unwrap().sources = sources;
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(doc))),
                },
            },
        },
    );
    action(
        &mut root,
        A::Select {
            selection: Selection { holder: c::AccountHolder::Deployment, source: Some(43), ..Default::default() },
        },
    );
    action(&mut root, A::Submit { name: [72; 16], operation: Operation::RemoveKey });
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("Remove this key? · The deployment's accounts · Account 41 · Source 43"));
}
#[test]
fn deployment_accounts_past_the_count_are_named() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let row = doc.accounts.get(0).unwrap().clone();
    let mut rows = List::with_capacity(3);
    for _ in 0_u32..3 {
        rows.push(row.clone()).unwrap();
    }
    doc.accounts = rows;
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(doc))),
                },
            },
        },
    );
    let mut counts = fixtures::limits().accounts;
    counts.accounts = 2;
    let mut view = bounded_view(counts);
    let tree = fixtures::build(&mut view, &root);
    let text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert_eq!(
        tree.nodes()
            .iter()
            .filter(|node| node.role == Role::Heading { level: 3 }
                && node.name.as_ref().is_some_and(|name| name.starts_with(b"Account 41")))
            .count(),
        2
    );
    assert!(text.contains("Accounts left out: 1"));
}

#[test]
fn the_link_word_counts_towards_the_address_bound() {
    let address = format!("https://i/{}", "x".repeat(490));
    assert!(address.len() < usize::try_from(fixtures::tree_limits().text_per_node).unwrap());
    let root = sign_in_address(address.as_bytes());
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
}

#[test]
fn typing_a_key_keeps_the_write_revision() {
    let (mut root, mut view) = key_draft();
    let before = key_field(&view).value.as_ref().unwrap().written;
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"another-key".as_slice()) },
    );
    fixtures::build(&mut view, &root);
    assert_eq!(key_field(&view).value.as_ref().unwrap().written, before);
}

#[test]
fn accounts_show_the_configured_workers_program_observation() {
    let mut doc = document(1);
    let mut programs = List::with_capacity(1);
    programs
        .push(c::OwnerProgramLogin {
            name: b"work-login".as_slice().into(),
            state: Some(c::OwnerProgramState::SignedInOtherwise { method: b"other-method".as_slice().into() }),
            sign_out: None,
            cleanup: false,
            cut: false,
        })
        .unwrap();
    doc.program_hosts = List::with_capacity(1);
    doc.program_hosts
        .push(c::OwnerProgramHost {
            name: b"configured-worker".as_slice().into(),
            channel: 17,
            connected: true,
            awaiting_logins: false,
            programs,
            cut: false,
        })
        .unwrap();
    let text = shown(doc);
    assert!(
        text.contains("configured-worker") && text.contains("work-login") && text.contains("other-method"),
        "{text}"
    );
}
