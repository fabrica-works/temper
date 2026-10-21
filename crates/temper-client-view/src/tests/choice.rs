#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and allocate maximal model-name fixtures"
)]
use super::fixtures;
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Builder, Role};
use temper_client_domain::{self as d, Address};
fn delivered(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}
fn banner(usable: Option<bool>) -> (d::Domain, Token) {
    let (mut root, watch) = fixtures::account_root();
    if let Some(usable) = usable {
        delivered(
            &mut root,
            watch,
            c::Delivery::OwnerAccounts(Box::new(c::OwnerAccounts {
                program_hosts: List::with_capacity(0),
                accounts: List::with_capacity(0),
                signing: List::with_capacity(0),
                last_end: None,
                usable,
                settled: true,
                cut: false,
                readiness: List::with_capacity(0),
                readiness_cut: false,
            })),
        );
    }
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Chats) });
    (root, watch)
}
fn text(root: &d::Domain) -> String {
    String::from_utf8(fixtures::text(fixtures::page_view(root).tree()).into_vec()).unwrap()
}
#[test]
fn the_accounts_banner_precedes_main() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let nodes = view.tree().nodes();
    let banner = nodes.iter().position(|n| n.name.as_deref() == Some(b"Agents need an LLM account to run.")).unwrap();
    assert!(nodes.iter().position(|n| n.role == Role::Navigation).unwrap() < banner);
    assert!(banner < nodes.iter().position(|n| n.role == Role::Main).unwrap());
}
#[test]
fn usable_accounts_remove_the_banner() {
    assert!(!text(&banner(Some(true)).0).contains("Agents need"));
}
#[test]
fn unknown_accounts_have_no_banner() {
    assert!(!text(&banner(None).0).contains("Agents need"));
}
#[test]
fn a_behind_accounts_document_removes_the_banner() {
    let (mut root, watch) = banner(Some(false));
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
        },
    );
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn missing_accounts_leave_start_available() {
    assert!(fixtures::find(fixtures::page_view(&banner(Some(false)).0).tree(), Role::Button, b"Start").is_some());
}
#[test]
fn the_accounts_page_has_no_accounts_banner() {
    let (mut root, _) = banner(Some(false));
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn an_absent_choice_is_labelled_engine_default() {
    assert!(text(&fixtures::chats_root()).contains("Engine default for next run"));
}
fn task_root() -> d::Domain {
    account_readiness(None)
}
fn account_readiness(ready: Option<c::PayerReadiness>) -> d::Domain {
    account_facts(ready, List::with_capacity(0))
}
fn account_facts(ready: Option<c::PayerReadiness>, accounts: List<c::OwnerAccountLine>) -> d::Domain {
    let (mut root, me_watch, watch, _, accounts_watch) =
        fixtures::root_at_full(Address::Chat { task: 9 }, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let mut models = List::with_capacity(2);
    models
        .push(c::MeModel {
            prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
            description: None,
            name: c::Text::new(Box::from(*b"chosen"), 32).unwrap(),
            provider: Some(17),
            window: 1024,
            output: 128,
            efforts: List::with_capacity(0),
        })
        .unwrap();
    models
        .push(c::MeModel {
            prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
            description: None,
            name: c::Text::new(Box::from(*b"second"), 32).unwrap(),
            provider: None,
            window: 1024,
            output: 128,
            efforts: List::with_capacity(0),
        })
        .unwrap();
    me.projects.get_mut(0).unwrap().models = models;
    me.providers = List::with_capacity(1);
    me.providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Supplied Provider"), 32).unwrap()),
            key_bytes: 4096,
            sign_in: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            signed_in_plan: false,
            manage_usage: None,
        })
        .unwrap();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    let mut readiness = List::with_capacity(1);
    if let Some(ready) = ready {
        readiness.push(ready).unwrap();
    }
    delivered(
        &mut root,
        accounts_watch.unwrap(),
        c::Delivery::OwnerAccounts(Box::new(c::OwnerAccounts {
            program_hosts: List::with_capacity(0),
            accounts,
            signing: List::with_capacity(0),
            last_end: None,
            usable: false,
            settled: true,
            cut: false,
            readiness,
            readiness_cut: false,
        })),
    );
    let mut task = fixtures::task::document();
    task.chip.task = 9;
    task.project = fixtures::PROJECT;
    task.chip.executor = c::Executor::Agent { charter: 1 };
    task.budget.funder = c::Funder::Pool { project: fixtures::PROJECT, person: 73, period: 1 };
    task.acts = List::with_capacity(2);
    for act in [c::ActKind::Message, c::ActKind::Amend] {
        task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    task.llm = Some(c::LlmChoice {
        account: Some(3),
        main: c::ModelEffort { model: c::Text::new(Box::from(*b"chosen"), 32).unwrap(), effort: None },
        subagents: List::with_capacity(0),
    });
    task.kept_account = Some(7);
    delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    root
}
#[test]
fn a_requested_model_is_labelled_for_the_next_run() {
    assert!(text(&task_root()).contains("Requested for next run: chosen"));
}
#[test]
fn a_claimed_account_keeps_its_own_label() {
    assert!(text(&task_root()).contains("Last claimed account: 7"));
}
fn wait(why: c::TaskWaitWhy) -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(32);
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &task_root(),
        Some(c::TaskRunWait::Account { account: Some(7), source: Some(8), why }),
        &fixtures::words(),
        &mut bindings,
    );
    b.close();
    b.finish()
}
#[test]
fn a_limited_wait_links_to_resume() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None })),
            Role::Link,
            b"Resume source"
        )
        .is_some()
    );
}
#[test]
fn a_sign_in_wait_links_to_sign_in() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::NeedsSignIn)),
            Role::Link,
            b"Sign in with an LLM provider"
        )
        .is_some()
    );
}
#[test]
fn an_unknown_reset_has_no_time() {
    assert!(
        !String::from_utf8(
            fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None }))).into_vec()
        )
        .unwrap()
        .contains("until")
    );
}
#[test]
fn a_known_reset_shows_its_utc_moment() {
    let until = Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000));
    let shown = String::from_utf8(
        fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until }))).into_vec(),
    )
    .unwrap();
    assert!(shown.contains("Limited until Oct 12, 18:04 UTC"), "{shown}");
}

fn model_words() -> crate::words::Words {
    crate::words::Words::english(fixtures::FORGE, 128, crate::words::Shell::Terminal).unwrap()
}
const NAME_BYTES: u32 = jig_client_protocol_wire::v11::CEILINGS.me_model_name;
const EFFORT_BYTES: u32 = jig_client_protocol_wire::v11::CEILINGS.me_model_efforts_item;
const CUT_MARK: &[u8] = "…".as_bytes();
fn shown_bytes() -> usize {
    usize::try_from(crate::choice::MODEL_SHOWN).unwrap()
}
fn prefix_bytes() -> usize {
    shown_bytes().checked_sub(CUT_MARK.len()).expect("display bound includes cut mark")
}
fn catalogue(name: &[u8], efforts: &[&[u8]]) -> d::Domain {
    let mut declared = List::with_capacity(u32::try_from(efforts.len()).unwrap());
    for effort in efforts {
        declared.push(c::Text::new(Box::from(*effort), EFFORT_BYTES).unwrap()).unwrap();
    }
    let mut models = List::with_capacity(1);
    models
        .push(c::MeModel {
            prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
            description: None,
            name: c::Text::new(Box::from(name), NAME_BYTES).unwrap(),
            provider: None,
            window: 1024,
            output: 128,
            efforts: declared,
        })
        .unwrap();
    catalogue_rows(models)
}
fn catalogue_rows(models: List<c::MeModel>) -> d::Domain {
    let (mut root, me_watch, _) = fixtures::root_at(Address::Chats, &fixtures::limits());
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.templates = List::with_capacity(1);
    project
        .templates
        .push(c::Template {
            number: 1,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), 32).unwrap()),
            executor: c::TemplateExecutor::Charter(1),
            contract: c::TaskContractDocument::Report { words: 32 },
            shallower: 1,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags { conversation: true, listed: false, retention: c::Retention::Kept },
            label: 7,
            grace: None,
            llm: None,
        })
        .unwrap();
    project.models = models;
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    root
}
fn catalogue_view(root: &d::Domain) -> crate::View {
    let mut tree = fixtures::tree_limits();
    tree.text_per_node = 1024;
    let mut view = crate::View::new(&crate::Limits { tree }, &fixtures::limits(), model_words(), fixtures::FORGE);
    fixtures::build(&mut view, root);
    view
}
fn shows_requested(view: &crate::View, label: &[u8]) {
    assert!(fixtures::find(view.tree(), Role::Button, label).is_some());
    let requested = [b"Requested for next run: ".as_slice(), label].concat();
    assert!(fixtures::find(view.tree(), Role::Paragraph, &requested).is_some());
}
fn marked_prefix() -> Vec<u8> {
    [vec![b'm'; prefix_bytes()], CUT_MARK.to_vec()].concat()
}
#[test]
fn a_long_model_name_shows_a_marked_prefix() {
    let mut root = catalogue(&vec![b'm'; usize::try_from(NAME_BYTES).unwrap()], &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0 });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}
#[test]
fn a_name_at_the_display_bound_shows_whole() {
    let name = vec![b'm'; shown_bytes()];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0 });
    shows_requested(&catalogue_view(&root), &name);
}
#[test]
fn a_name_one_past_the_display_bound_shows_a_marked_prefix() {
    let name = vec![b'm'; shown_bytes() + 1];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0 });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}
#[test]
fn a_cut_model_name_ends_at_a_utf8_boundary() {
    let letter = "é";
    let name = letter.repeat(usize::try_from(NAME_BYTES).unwrap() / letter.len());
    let shown = crate::choice::shown(name.as_bytes(), &model_words());
    assert_eq!(
        String::from_utf8(shown.into_vec()).unwrap(),
        format!("{}…", letter.repeat(prefix_bytes() / letter.len()))
    );
}
#[test]
fn only_the_selected_duplicate_model_shows_its_efforts() {
    let mut models = List::with_capacity(2);
    for (provider, effort) in [(17, b"first-effort".as_slice()), (18, b"second-effort".as_slice())] {
        let mut efforts = List::with_capacity(1);
        efforts.push(c::Text::new(Box::from(effort), EFFORT_BYTES).unwrap()).unwrap();
        models
            .push(c::MeModel {
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                name: c::Text::new(Box::from(*b"shared"), NAME_BYTES).unwrap(),
                provider: Some(provider),
                window: 1024,
                output: 128,
                efforts,
            })
            .unwrap();
    }
    let mut root = catalogue_rows(models);
    edit(&mut root, d::pages::choice::Act::Model { index: 1 });
    let view = catalogue_view(&root);
    let efforts: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|node| {
            node.role == Role::Button
                && [b"first-effort".as_slice(), b"second-effort"].contains(&node.name.as_deref().unwrap_or_default())
        })
        .map(|node| node.name.as_deref())
        .collect();
    assert_eq!(efforts, [Some(b"second-effort".as_slice())]);
}

#[test]
fn the_chosen_model_and_effort_are_pressed() {
    let mut root = catalogue(b"needs-effort", &[b"low", b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0 });
    edit(&mut root, d::pages::choice::Act::Effort { index: 1 });
    let view = catalogue_view(&root);
    let states = |name: &[u8]| {
        let node = view.tree().nodes().iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(name));
        node.unwrap().states.as_slice().to_vec()
    };
    for name in [b"needs-effort".as_slice(), b"high"] {
        assert!(states(name).contains(&skein_ui::State::Pressed));
        assert!(!states(name).contains(&skein_ui::State::Selected));
    }
    assert!(!states(b"low").contains(&skein_ui::State::Pressed));
}
#[test]
fn an_invalid_draft_withholds_start() {
    let mut root = catalogue(b"needs-effort", &[b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0 });
    assert!(fixtures::find(catalogue_view(&root).tree(), Role::Button, b"Start").is_none());
}

fn readiness(state: c::Readiness) -> c::PayerReadiness {
    c::PayerReadiness { provider: 17, payer: c::OwnerAccountUse::Party(73), readiness: state, settled: Some(true) }
}
#[test]
fn an_unavailable_provider_needs_an_account() {
    assert!(text(&account_readiness(Some(readiness(c::Readiness::Unavailable)))).contains("needs an account"));
}
#[test]
fn a_usable_provider_does_not_need_an_account() {
    assert!(!text(&account_readiness(Some(readiness(c::Readiness::Usable)))).contains("needs an account"));
}
#[test]
fn unknown_provider_readiness_is_named() {
    assert!(text(&account_readiness(Some(readiness(c::Readiness::Unknown)))).contains("Account readiness unknown"));
}
#[test]
fn missing_provider_readiness_stays_unknown() {
    assert!(text(&account_readiness(None)).contains("Account readiness unknown"));
}
#[test]
fn another_payers_readiness_does_not_decide_this_payers() {
    let mut row = readiness(c::Readiness::Unavailable);
    row.payer = c::OwnerAccountUse::Project(fixtures::PROJECT);
    assert!(!text(&account_readiness(Some(row))).contains("needs an account"));
}
#[test]
fn the_provider_keeps_its_supplied_name() {
    assert!(text(&task_root()).contains("Supplied Provider"));
}
#[test]
fn the_accounts_banner_uses_its_slot_token() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let node = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::ACCOUNTS)).unwrap();
    assert_eq!(node.role, Role::Slot);
}
#[test]
fn the_account_line_precedes_entry_inside_the_composer_area() {
    let view = fixtures::page_view(&task_root());
    let nodes = view.tree().nodes();
    let area = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::COMPOSER_AREA)).unwrap();
    let line = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::ACCOUNT_LINE)).unwrap();
    let entry = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::ENTRY)).unwrap();
    assert_eq!(nodes[line].role, Role::Paragraph);
    assert!(area < line && line < entry && line < area + usize::try_from(nodes[area].size).unwrap());
}

#[test]
fn the_start_choice_is_a_sibling_before_entry() {
    let view = fixtures::page_view(&catalogue(b"first", &[]));
    let nodes = view.tree().nodes();
    let choice = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::ACCOUNT_LINE)).unwrap() - 1;
    let entry = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::ENTRY)).unwrap();
    assert_eq!(nodes[choice].role, Role::Slot);
    assert_eq!(choice + usize::try_from(nodes[choice].size).unwrap(), entry);
    let children = &nodes[entry + 1..entry + usize::try_from(nodes[entry].size).unwrap()];
    let buttons: Vec<_> = children.iter().filter(|n| n.role == Role::Button).map(|n| n.name.as_deref()).collect();
    assert_eq!(buttons, [Some(b"Start".as_slice())]);
}
fn edit(root: &mut d::Domain, act: d::pages::choice::Act) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Choice(act)) });
}

#[test]
fn an_unapplied_model_has_a_separate_draft_label() {
    let mut root = task_root();
    edit(&mut root, d::pages::choice::Act::Model { index: 1 });
    let shown = text(&root);
    assert!(shown.contains("Requested for next run: chosen"));
    assert!(shown.contains("Next run if applied: second"));
    assert!(!shown.contains("Requested for next run: second"));
}
fn named_account() -> d::Domain {
    one_account(3, false)
}
fn one_account(number: u32, plan: bool) -> d::Domain {
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::OwnerAccountLine {
            number,
            place: 1,
            uses: None,
            plan,
            provider: None,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(Box::from(*b"Personal"), 32).unwrap()),
            holder: c::OwnerAccountHolder::Party(73),
            sources: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    account_facts(None, accounts)
}

#[test]
fn a_kept_account_separates_its_display_from_its_plan() {
    assert!(text(&one_account(7, true)).contains("Personal · Signed-in plan"));
}
#[test]
fn a_requested_account_shows_its_display() {
    let root = named_account();
    assert!(text(&root).contains("Requested account: Personal"));
}
#[test]
fn a_waiting_account_shows_its_display() {
    let root = named_account();
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        Some(c::TaskRunWait::Account { account: Some(3), source: None, why: c::TaskWaitWhy::Refreshing }),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(32),
    );
    b.close();
    assert!(String::from_utf8(fixtures::text(&b.finish()).into_vec()).unwrap().contains("Waiting account: Personal"));
}

#[test]
fn a_refused_next_run_says_so() {
    let root = task_root();
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::choice::waiting(
        &mut builder,
        &root,
        Some(c::TaskRunWait::Refused(c::TaskWaitRefused::Program(c::OpenProgram::Model))),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(32),
    );
    assert_eq!(fixtures::text(&builder.finish()).as_ref(), b"Next run refused");
}
