#![allow(clippy::disallowed_types, clippy::disallowed_methods, reason = "test assertions inspect the semantic tree")]
use super::fixtures as f;
use skein_lib::List;
use skein_ui::{Act, Role, State};
use temper_client_domain::{self as d, Address, Choice, Step, pages};
use temper_client_protocol_wire::v1 as w;
const HERE: u16 = 3;
const GITHUB: u16 = 2;
pub(super) fn row(
    number: u16,
    kind: w::Kind,
    mode: w::Mode,
    connection: w::Connection,
    offer: w::Offer,
) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            app: None,
            installations: List::with_capacity(0),
            ended: None,
            first: None,
            number,
            kind,
            mode,
            connection,
            offer,
        },
    )
    .unwrap()
}
pub(super) fn rows(connection: w::Connection, offer: w::Offer) -> List<w::ForgeRow> {
    let mut rows = List::with_capacity(2);
    rows.push(row(GITHUB, w::Kind::Github, w::Mode::Managed, w::Connection::NotConnected, w::Offer::NotOffered))
        .unwrap();
    rows.push(row(HERE, w::Kind::Forgejo, w::Mode::Owned, connection, offer)).unwrap();
    rows
}
fn root(choice: Option<Choice>, connection: w::Connection, offer: w::Offer) -> d::Domain {
    f::forge_root(Address::ConnectForge { forge: choice }, rows(connection, offer))
}
fn here(connection: w::Connection) -> d::Domain {
    root(Some(Choice::Forge(HERE)), connection, w::Offer::Offered)
}
pub(super) fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 42, login: b"temper".as_slice().into() }).unwrap(),
    )
}
fn node(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    f::find(tree, role, name).unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
}
fn press(root: &d::Domain, role: Role, name: &[u8]) -> d::Action {
    let mut view = f::page_view(root);
    let id = node(view.tree(), role, name);
    crate::act(&mut view, id, if role == Role::Choice { Act::Toggle } else { Act::Press }).expect("bound control")
}
pub(super) fn words_in_order(tree: &skein_ui::Tree, phrases: &[&str]) {
    let mut bytes = Vec::new();
    for (index, node) in tree.nodes().iter().enumerate() {
        if let Some(name) = &node.name
            && f::node_text(tree, index).is_empty()
        {
            bytes.extend_from_slice(name);
            bytes.push(b'\n');
        }
        for run in &node.text {
            bytes.extend_from_slice(&run.text);
        }
    }
    let words = String::from_utf8_lossy(&bytes);
    let mut remaining = words.as_ref();
    for phrase in phrases {
        let index = remaining.find(phrase).unwrap_or_else(|| panic!("missing or out of order: {phrase}\nin {words}"));
        remaining = &remaining[index.checked_add(phrase.len()).expect("phrase end fits")..];
    }
}
#[test]
fn choose_draws_a_card_per_choice_in_order() {
    let root = root(None, w::Connection::NotSetUp, w::Offer::Offered);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, b"Connect a forge");
    words_in_order(
        view.tree(),
        &[
            "Choose",
            "Details",
            "Setting up",
            "Ready",
            "GitHub",
            "Your repositories on GitHub. temper makes its own App there; you choose the account and the repositories it may use.",
            "In this tab, on GitHub · about two minutes",
            "Connect GitHub",
            "Use an App of your own ›",
            "Forgejo on this machine",
            "A forge of your own, run by temper box. No account needed: temper makes your first repository, and you start in it.",
            "Here · about a minute",
            "Choose",
            "A Forgejo elsewhere",
            "Your own Forgejo server. temper connects as its admin, or with its bot’s tokens.",
            "Needs a change to temper’s configuration and a restart",
            "Show how",
        ],
    );
    let regions: Vec<_> =
        view.tree().nodes().iter().filter(|n| n.role == Role::Region).filter_map(|n| n.name.as_deref()).collect();
    assert_eq!(regions, [b"LLM account".as_slice(), b"GitHub", b"Forgejo on this machine", b"A Forgejo elsewhere"]);
    assert_eq!(
        press(&root, Role::Link, b"Show how"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: Some(Choice::Elsewhere) } })
    );
}
#[test]
fn github_details_offer_only_the_app_of_ones_own_with_a_masked_key() {
    let root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    let view = f::browser_page_view(&root);
    let words = String::from_utf8_lossy(&f::text(view.tree())).into_owned();
    assert!(!words.contains("organization"));
    assert!(!words.contains("Connect GitHub"));
    assert!(!words.contains("not available yet"));
    node(view.tree(), Role::Heading { level: 3 }, b"An App of your own");
    let button = node(view.tree(), Role::Button, b"Connect the App");
    assert!(!view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
    node(view.tree(), Role::TextField, b"App id");
    let key = node(view.tree(), Role::TextField, b"Private key");
    assert_eq!(view.tree().nodes().iter().find(|n| n.id == key).unwrap().entry, Some(skein_ui::EntryKind::Secret));
}
#[test]
fn the_terminal_shows_ctl_or_the_web_page_instead_of_the_key_field() {
    let view = f::page_view(&root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered));
    assert!(f::find(view.tree(), Role::TextField, b"App id").is_none());
    assert!(f::find(view.tree(), Role::TextField, b"Private key").is_none());
    assert!(f::find(view.tree(), Role::Button, b"Connect the App").is_none());
    words_in_order(
        view.tree(),
        &[
            "Paste the key on this page in a browser, or run temper ctl forge-connect 2 github-app ID FILE, FILE the key's file, readable by you alone.",
        ],
    );
}

#[test]
fn forgejo_here_details_say_the_first_repository_and_set_it_up() {
    let mut root = here(w::Connection::NotSetUp);
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "Forgejo on this machine",
            "temper box downloads Forgejo and its runner, starts them, and makes your first repository, with a README and a CI workflow. No restart.",
            "Its CI jobs run on this machine, as you, with no isolation.",
            "Forgejo stops when temper box stops. Its repositories are kept in temper’s data directory.",
        ],
    );
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::TextField));
    let action = press(&root, Role::Button, b"Set up a Forgejo here");
    assert_eq!(action, d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
    f::root_event(&mut root, d::Event::Act { action });
    let view = f::page_view(&root);
    let id = node(view.tree(), Role::Button, b"Set up a Forgejo here");
    assert!(view.tree().nodes().iter().find(|n| n.id == id).unwrap().states.as_slice().contains(&State::Disabled));
}
#[test]
fn forgejo_here_without_the_box_lists_its_four_commands() {
    let view = f::page_view(&root(Some(Choice::Forge(HERE)), w::Connection::NotSetUp, w::Offer::NotUnderTheBox));
    node(view.tree(), Role::Heading { level: 2 }, b"Set it up from a terminal");
    words_in_order(
        view.tree(),
        &[
            "temper was not started by temper box, so it does not start a Forgejo itself. Run these, keep the last two running (a service manager is best), then restart temper:",
            "temper forge fetch 3",
            "temper forge prepare 3",
            "temper forge run 3 forgejo",
            "temper forge run 3 runner",
        ],
    );
    assert!(f::find(view.tree(), Role::Button, b"Set up a Forgejo here").is_none());
    assert_eq!(view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_CODE)).count(), 4);
}
#[test]
fn elsewhere_details_give_the_next_unused_number() {
    let view = f::page_view(&root(Some(Choice::Elsewhere), w::Connection::NotSetUp, w::Offer::Offered));
    words_in_order(
        view.tree(),
        &[
            "A Forgejo elsewhere",
            "Add its address to the configuration and restart temper box. Then connect as its admin, and temper makes and keeps its bot, or paste its bot’s tokens.",
            "application.forges",
            "\"number\": 4",
            "\"kind\": \"forgejo\"",
            "\"host\": \"forge.example\"",
            "\"log_origins\": null",
            "application.identities",
            "\"number\": 4, \"forge\": 4",
            "application.setup",
            "\"forge\": 4",
            "\"mode\": \"managed\"",
            "Without it, temper takes its bot’s tokens only.",
            "Restart temper box, then connect it from this page.",
            "Connecting a Forgejo elsewhere from temper is not available yet.",
        ],
    );
}
#[test]
fn setting_up_marks_the_rows_step() {
    for (step, done) in
        [(None, 0), (Some(w::SetUpStep::Fetch), 0), (Some(w::SetUpStep::Prepare), 1), (Some(w::SetUpStep::Run), 2)]
    {
        let view = f::page_view(&here(w::Connection::SettingUp(
            w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step }).unwrap(),
        )));
        node(view.tree(), Role::Heading { level: 2 }, b"Setting up Forgejo on this machine");
        assert_eq!(
            view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
            done
        );
        node(view.tree(), Role::Status, b"under way");
        words_in_order(
            view.tree(),
            &[
                "Fetching Forgejo and its runner",
                "Preparing Forgejo: its database, temper’s admin and your account",
                "Starting Forgejo and its runner",
                "Connecting",
                "You may leave this page: the steps go on, and the banner says where they are.",
            ],
        );
    }
    let view = f::page_view(&here(w::Connection::Connecting));
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        3
    );
}
#[test]
fn set_up_failed_offers_set_up_again() {
    let root = here(w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into() }).unwrap(),
    ));
    let view = f::page_view(&root);
    node(view.tree(), Role::Alert, b"Set up failed: fetch failed");
    assert!(f::find(view.tree(), Role::List, b"Each step").is_none());
    assert_eq!(press(&root, Role::Button, b"Set up again"), d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
}
#[test]
fn ready_names_the_forge_and_offers_adopt_a_repository() {
    let root = here(connected());
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 2 }, b"Forgejo on this machine is connected");
    node(view.tree(), Role::Paragraph, b"Connected as temper.");
    words_in_order(view.tree(), &["Forgejo on this machine is on the Forges page, with its details."]);
    assert_eq!(
        press(&root, Role::Link, b"Adopt a repository"),
        d::Action::Pages(pages::Act::Go { address: Address::Adopt { project: f::PROJECT } })
    );
    assert_eq!(press(&root, Role::Link, b"Forges"), d::Action::Pages(pages::Act::Go { address: Address::Forges }));
}
#[test]
fn the_welcomes_forge_step_is_choose_and_details() {
    let mut root = f::forge_root(
        Address::Welcome { step: Step::Forge { forge: None } },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    assert_eq!(
        press(&root, Role::Link, b"Show how"),
        d::Action::Pages(pages::Act::Go {
            address: Address::Welcome { step: Step::Forge { forge: Some(Choice::Elsewhere) } }
        })
    );
    f::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }),
        },
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Button, b"Set up a Forgejo here");
    assert_eq!(
        press(&root, Role::Link, "‹ All forges".as_bytes()),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
    );
}
#[test]
fn a_forge_under_way_is_no_choice_in_the_welcome() {
    let root = f::forge_root(
        Address::Welcome { step: Step::Forge { forge: None } },
        rows(w::Connection::Connecting, w::Offer::Offered),
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Forgejo on this machine · Connecting".as_bytes());
    assert!(f::find(view.tree(), Role::Link, b"Choose").is_none());
    assert_eq!(
        press(&root, Role::Link, b"View"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: Some(Choice::Forge(HERE)) } })
    );
}
#[test]
fn the_model_step_follows_the_persons_forge() {
    let mut forges = rows(w::Connection::Connecting, w::Offer::Offered);
    // Model follows the person's forge, even with another forge still connecting.
    *forges.get_mut(0).unwrap() = github_rows(true).get(0).unwrap().clone();
    let root = f::forge_root(Address::Welcome { step: Step::Model { way: None } }, forges);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "GitHub · no repository chosen yet".as_bytes());
    node(view.tree(), Role::Disclosure, b"Each step");
    node(view.tree(), Role::Heading { level: 1 }, b"Connect a model");
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        4
    );
}
#[test]
fn the_welcomes_ready_names_the_forge_and_the_model() {
    let (mut root, accounts) =
        f::forge_root_accounts(Address::Welcome { step: Step::Ready }, rows(connected(), w::Offer::Offered));
    f::welcome_account(&mut root, accounts);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, b"Almost ready");
    node(view.tree(), Role::Paragraph, b"Forge: Forgejo on this machine, connected as temper");
    let steps = view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_STEP));
    for step in steps {
        assert!(step.styles.as_slice().contains(&crate::look::FLOW_DONE));
        assert!(!step.states.as_slice().contains(&State::Current));
    }
    let account_view = f::page_view(&{
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
        root
    });
    let heading = account_view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Heading { level: 3 })
        .unwrap()
        .name
        .as_deref()
        .unwrap();
    let model = [b"Model: ".as_slice(), heading].concat();
    node(view.tree(), Role::Paragraph, &model);
}
#[test]
fn ready_without_a_forge_or_model_offers_the_two_steps() {
    let root = f::welcome_root(Step::Ready, false);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, b"Forge: none yet");
    node(view.tree(), Role::Paragraph, b"Model: none yet");
    node(view.tree(), Role::Link, b"Connect a forge");
    node(view.tree(), Role::Link, b"Connect a model");
}

#[test]
fn the_welcomes_ready_follows_a_forge_under_way() {
    let root =
        f::forge_root(Address::Welcome { step: Step::Ready }, rows(w::Connection::Connecting, w::Offer::Offered));
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Forge: Forgejo on this machine · Connecting".as_bytes());
    node(view.tree(), Role::Disclosure, b"Each step");
    assert!(f::find(view.tree(), Role::Link, b"Adopt a repository").is_none());
}

#[test]
fn the_forge_pages_have_navigation_and_no_playground_banner() {
    for address in [Address::ConnectForge { forge: None }, Address::Forges] {
        let root = f::demo_frame_root(address, f::PROJECT, true, None, None);
        assert!(root.demo().is_some());
        let view = f::page_view(&root);
        node(view.tree(), Role::Navigation, b"Main navigation");
        assert!(f::find(view.tree(), Role::Region, b"Playground").is_none());
    }
}

#[test]
fn a_set_up_refusal_is_said_below_its_button() {
    use jig_client::{Answered, Down, Family};
    use skein_lib::{Env, Queue, RequestEnvelope, Time};
    use temper_client_protocol_wire as wire;
    let mut root = here(w::Connection::NotSetUp);
    let limits = f::domain_limits();
    let mut out = Queue::with_capacity(d::max_out(&limits));
    d::step(
        &mut root,
        &Env { now: Time::ZERO, wall: f::NOW, limits },
        d::Event::Act { action: d::Action::Forge(d::forge::Act::SetUp { forge: HERE }) },
        &mut out,
    );
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let d::Request::Client { request: jig_client::Request::Down { down: Down::Send { attempt, .. } } } = request
        {
            sent = Some(attempt);
        }
    }
    let wire_limits = d::forge::wire_limits(&limits.forge).unwrap();
    let outcome = w::Outcome::SetUpRefused(
        w::SetUpRefused::new(&wire_limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
    );
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: sent.expect("set up sent"),
                    envelope: RequestEnvelope::Final,
                    answer: Some(Answered::Family(Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&wire_limits, w::AnswerParts { outcome }).unwrap()),
                    })),
                    why: None,
                },
            },
        },
    );
    let view = f::page_view(&root);
    let refusal = d::forge::words::set_up_refusal(d::forge::forges::SetUpRefusal::Busy);
    node(view.tree(), Role::Alert, refusal);
    let button = node(view.tree(), Role::Button, b"Set up a Forgejo here");
    assert!(!view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
}

fn drive(root: &mut d::Domain, event: d::Event) -> skein_lib::Queue<d::Request> {
    let limits = f::domain_limits();
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits }, event, &mut out);
    out
}
fn act(root: &mut d::Domain, act: d::forge::Act) -> skein_lib::Queue<d::Request> {
    drive(root, d::Event::Act { action: d::Action::Forge(act) })
}
fn attempt(mut out: skein_lib::Queue<d::Request>) -> skein_lib::Token {
    while let Some(request) = out.pop() {
        if let d::Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return attempt;
        }
    }
    panic!("request sent")
}
fn answer(root: &mut d::Domain, attempt: skein_lib::Token, outcome: w::Outcome) {
    f::root_event(root, answer_event(attempt, outcome));
}
fn answer_event(attempt: skein_lib::Token, outcome: w::Outcome) -> d::Event {
    d::Event::Client {
        event: jig_client::Event::Up {
            up: jig_client::Up::Answer {
                attempt,
                envelope: skein_lib::RequestEnvelope::Final,
                why: None,
                answer: Some(jig_client::Answered::Family(jig_client::Family {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                    bytes: temper_client_protocol_wire::answer(
                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap(),
                    ),
                })),
            },
        },
    }
}
pub(super) fn first_rows(connection: w::Connection, state: w::FirstState) -> List<w::ForgeRow> {
    let first = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: w::SetUpProject::Existing(
                w::ExistingProject::new(&w::CEILINGS, w::ExistingProjectParts { project: f::PROJECT }).unwrap(),
            ),
            state,
        },
    )
    .unwrap();
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                app: None,
                installations: List::with_capacity(0),
                ended: None,
                number: HERE,
                kind: w::Kind::Forgejo,
                mode: w::Mode::Owned,
                connection,
                offer: w::Offer::Offered,
                first: Some(first),
            },
        )
        .unwrap(),
    )
    .unwrap();
    rows
}
pub(super) fn deliver(
    root: &mut d::Domain,
    watch: skein_lib::Token,
    rows: List<w::ForgeRow>,
) -> skein_lib::Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap(),
                        ),
                    },
                },
            },
        },
    )
}
pub(super) fn made() -> w::FirstState {
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 42 }).unwrap())
}
fn has_state(tree: &skein_ui::Tree, role: Role, name: &[u8], state: State) -> bool {
    let id = node(tree, role, name);
    tree.nodes().iter().find(|n| n.id == id).unwrap().states.as_slice().contains(&state)
}
#[test]
fn details_show_the_set_ups_fields_with_their_defaults() {
    let mut root = here(w::Connection::NotSetUp);
    let view = f::page_view(&root);
    for (name, value) in [(b"Organization".as_slice(), b"work".as_slice()), (b"Repository", b"hello")] {
        let id = node(view.tree(), Role::TextField, name);
        assert_eq!(
            view.tree().nodes().iter().find(|n| n.id == id).unwrap().value.as_ref().unwrap().text.as_ref(),
            value
        );
    }
    node(view.tree(), Role::ChoiceGroup, b"Project");
    assert!(has_state(view.tree(), Role::Choice, b"Shop, the project you are in", State::Selected));
    let action = press(&root, Role::Choice, b"A new project");
    f::root_event(&mut root, d::Event::Act { action });
    let mut view = f::page_view(&root);
    let id = node(view.tree(), Role::TextField, b"A new project");
    let action = crate::act(&mut view, id, Act::Input { value: Box::default() }).unwrap();
    f::root_event(&mut root, d::Event::Act { action });
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    let view = f::page_view(&root);
    assert!(has_state(view.tree(), Role::TextField, b"A new project", State::Invalid));
    node(view.tree(), Role::Alert, b"Required.");
}
#[test]
fn an_invalid_refusal_marks_the_organization_and_repository() {
    let mut root = here(w::Connection::NotSetUp);
    let token = attempt(act(&mut root, d::forge::Act::SetUp { forge: HERE }));
    answer(
        &mut root,
        token,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Invalid }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    for name in [b"Organization".as_slice(), b"Repository"] {
        assert!(has_state(view.tree(), Role::TextField, name, State::Invalid));
    }
    node(view.tree(), Role::Alert, b"invalid: the organization, repository or project name is empty or too long");
    assert_eq!(root.forge().setup_form().text(d::forge::forges::SetupField::Repository), b"hello");
}
#[test]
fn setting_up_names_its_first_repository_and_project() {
    let root = f::forge_root(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        first_rows(w::Connection::Connecting, w::FirstState::Making),
    );
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &["Setting up Forgejo on this machine", "work/hello, into Shop", "Connecting, and making work/hello"],
    );
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        3_usize
    );
    node(view.tree(), Role::Status, b"under way");
}
#[test]
fn setting_up_uses_the_sent_fields_until_the_row_carries_them() {
    let mut root = here(w::Connection::NotSetUp);
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    // The flow keeps the submitted fields even when it is showing Details.
    assert_eq!(root.forge().set_up().unwrap().org.as_ref(), b"work");
    let (mut root, watch) = f::forge_root_watch(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    deliver(&mut root, watch, rows(w::Connection::Connecting, w::Offer::Offered));
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, b"work/hello, into Shop");
    words_in_order(view.tree(), &["Starting Forgejo and its runner", "Connecting"]);
    assert!(!f::text(view.tree()).windows(b"Connecting, and making".len()).any(|w| w == b"Connecting, and making"));
}
#[test]
fn ready_says_the_repository_is_ready_and_opens_its_project() {
    for address in [Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }, Address::Welcome { step: Step::Ready }]
    {
        let mut root = f::forge_root(address, first_rows(connected(), made()));
        f::root_event(
            &mut root,
            d::Event::Act { action: d::Action::Pages(pages::Act::Project { project: f::SECOND_PROJECT }) },
        );
        assert_eq!(root.pages().project(), Some(f::SECOND_PROJECT));
        let view = f::page_view(&root);
        node(view.tree(), Role::Heading { level: 2 }, b"work/hello is ready");
        node(view.tree(), Role::Paragraph, b"In Shop, on Forgejo on this machine.");
        assert_eq!(
            view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
            4_usize
        );
        let action = press(&root, Role::Link, b"Open Shop");
        assert_eq!(action, d::Action::Pages(pages::Act::OpenProject { project: f::PROJECT }));
        let mut out = drive(&mut root, d::Event::Act { action });
        assert_eq!(root.pages().project(), Some(f::PROJECT));
        assert_eq!(out.pop(), Some(d::Request::Show { address: Address::Chats, replace: false }));
    }
}
fn pending_adoption() -> (d::Domain, skein_lib::Token) {
    let (mut root, watch) = f::forge_root_watch(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    let token = attempt(act(&mut root, d::forge::Act::SetUp { forge: HERE }));
    answer(&mut root, token, w::Outcome::SettingUp);
    let token = attempt(deliver(&mut root, watch, first_rows(connected(), made())));
    (root, token)
}
#[test]
fn open_project_waits_on_the_adoption() {
    let (root, _) = pending_adoption();
    let view = f::page_view(&root);
    assert!(has_state(view.tree(), Role::Button, b"Open Shop", State::Disabled));
    assert!(f::find(view.tree(), Role::Link, b"Open Shop").is_none());
}
#[test]
fn a_refused_adoption_says_its_outcome() {
    let (mut root, token) = pending_adoption();
    answer(
        &mut root,
        token,
        w::Outcome::ForgeRefused(
            w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::Unreachable }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Forge 3 did not answer; nothing was adopted. Try again."]);
    node(view.tree(), Role::Link, b"Open Shop");
}
#[test]
fn a_failed_first_repository_says_its_step_with_set_up_again() {
    for (step, expected) in [
        (w::CreateStep::Admin, b"Set up failed: checking the admin's token".as_slice()),
        (w::CreateStep::Org, b"Set up failed: making the organization"),
        (w::CreateStep::Repository, b"Set up failed: making the repository"),
        (w::CreateStep::File, b"Set up failed: adding the starter workflow"),
    ] {
        for address in
            [Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }, Address::Welcome { step: Step::Ready }]
        {
            let root = f::forge_root(
                address,
                first_rows(
                    connected(),
                    w::FirstState::Failed(
                        w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: step.clone() }).unwrap(),
                    ),
                ),
            );
            let view = f::page_view(&root);
            node(view.tree(), Role::Alert, expected);
            node(view.tree(), Role::Button, b"Set up again");
            assert!(f::find(view.tree(), Role::List, b"Each step").is_none());
        }
    }
}
#[test]
fn the_forges_row_names_the_first_repository_and_project() {
    for (state, expected, first) in [
        (w::FirstState::Making, b"work/hello, into Shop".as_slice(), b"Making work/hello".as_slice()),
        (made(), "work/hello · project Shop".as_bytes(), b"work/hello made, in 1"),
    ] {
        let root = f::forge_root(Address::Forges, first_rows(connected(), state));
        let view = f::page_view(&root);
        let id = node(view.tree(), Role::Disclosure, b"Forgejo on this machine");
        let row = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
        let description = row.description.as_deref().unwrap();
        assert!(description.ends_with(expected));
        assert!(!row.states.as_slice().contains(&State::Expanded));
        node(view.tree(), Role::Paragraph, first);
    }
}

pub(super) fn github_rows(accepted: bool) -> List<w::ForgeRow> {
    let mut row = row(
        GITHUB,
        w::Kind::Github,
        w::Mode::Connected,
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 9, login: b"my-app[bot]".as_slice().into() })
                .unwrap(),
        ),
        w::Offer::NotOffered,
    )
    .into_parts();
    row.ended = Some(
        w::ConnectStopped::new(&w::CEILINGS, w::ConnectStoppedParts { why: b"fixture stopped".as_slice().into() })
            .unwrap(),
    );
    row.installations = List::with_capacity(3);
    if accepted {
        row.installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id: 1,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id: 10, login: b"acme".as_slice().into() })
                                .unwrap(),
                        ),
                        suspended: false,
                        stray: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    for (id, suspended, stray) in [(2, true, false), (3, false, true)] {
        row.installations
            .push(
                w::Installation::new(&w::CEILINGS, w::InstallationParts { id, account: None, suspended, stray })
                    .unwrap(),
            )
            .unwrap();
    }
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    rows
}
#[test]
fn connecting_on_github_offers_install_with_its_way_back() {
    for address in [
        Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) },
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(GITHUB)) } },
    ] {
        let root = f::forge_root(address.clone(), github_rows(false));
        let view = f::browser_page_view(&root);
        words_in_order(
            view.tree(),
            &["Install temper’s App", "my-app is connected to temper, and installed nowhere yet.", "Install on GitHub"],
        );
        let mut view = f::browser_page_view(&root);
        let button = node(view.tree(), Role::Button, b"Install on GitHub");
        assert_eq!(
            crate::act(&mut view, button, Act::Press),
            Some(d::Action::Forge(d::forge::Act::Install { forge: GITHUB, back: d::address::print(&address) }))
        );
    }
}
#[test]
fn an_accepted_installation_makes_github_ready_until_the_picker() {
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) }, github_rows(true));
    let view = f::browser_page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "GitHub is connected",
            "Connected as my-app[bot].",
            "temper’s App",
            "my-app · ",
            "acme",
            "Adopt a repository",
        ],
    );
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the test captures Debug to verify secret redaction")]
fn the_private_key_moves_as_a_secret_and_never_enters_the_view() {
    let mut root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    let mut view = f::browser_page_view(&root);
    let id = node(view.tree(), Role::TextField, b"App id");
    let event = crate::event(&mut view, id, Act::Input { value: b"7".as_slice().into() }).unwrap();
    f::root_event(&mut root, event);
    let key = node(view.tree(), Role::TextField, b"Private key");
    let event =
        crate::event(&mut view, key, Act::Input { value: b"private-app-fixture\nsecond line\n".as_slice().into() })
            .unwrap();
    assert!(!format!("{event:?}").contains("private-app-fixture"));
    let d::Event::Forge(d::forge::Event::AppKey { .. }) = &event else {
        panic!("the input moves straight into a secret key event");
    };
    f::root_event(&mut root, event);
    let view = f::browser_page_view(&root);
    assert!(!format!("{view:?}").contains("private-app-fixture"));
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Private key"))
        .unwrap();
    assert!(field.value.as_ref().unwrap().text.is_empty());
    let written = field.value.as_ref().unwrap().written;
    f::root_event(&mut root, d::Event::Act { action: d::Action::Forge(d::forge::Act::ConnectApp { forge: GITHUB }) });
    let view = f::browser_page_view(&root);
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Private key"))
        .unwrap();
    assert!(field.value.as_ref().unwrap().written > written);
    assert!(!root.forge().app_form().has_key());
}
#[test]
fn a_connect_refusal_is_said_below_its_button() {
    let mut root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    f::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Forge(d::forge::Act::AppWrite { text: b"7".as_slice().into() }) },
    );
    f::root_event(
        &mut root,
        d::Event::Forge(d::forge::Event::AppKey { value: skein_lib::Secret::new(b"fake-key".as_slice().into()) }),
    );
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&f::limits()));
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::limits() },
        d::Event::Act { action: d::Action::Forge(d::forge::Act::ConnectApp { forge: GITHUB }) },
        &mut out,
    );
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let d::Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Send { attempt: value, .. } },
        } = request
        {
            attempt = Some(value);
        }
    }
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: attempt.unwrap(),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::answer(
                            &w::Answer::new(
                                &w::CEILINGS,
                                w::AnswerParts {
                                    outcome: w::Outcome::ConnectRefused(
                                        w::ConnectRefused::new(
                                            &w::CEILINGS,
                                            w::ConnectRefusedParts { why: w::ForgeRefusal::NoKey },
                                        )
                                        .unwrap(),
                                    ),
                                },
                            )
                            .unwrap(),
                        ),
                    })),
                    why: None,
                },
            },
        },
    );
    let view = f::browser_page_view(&root);
    words_in_order(view.tree(), &["Connect the App", "The key did not reach temper: paste it again."]);
}

fn manifest_started(address: Address) -> (d::Domain, skein_lib::Queue<d::Request>) {
    let mut root = f::forge_root(address, rows(w::Connection::NotSetUp, w::Offer::Offered));
    let mut view = f::browser_page_view(&root);
    let button = node(view.tree(), Role::Button, b"Connect GitHub");
    let action = crate::act(&mut view, button, Act::Press).expect("live manifest button");
    let out = drive(&mut root, d::Event::Act { action });
    (root, out)
}
#[test]
fn connect_github_sends_the_pages_own_address() {
    for (address, back) in [
        (Address::ConnectForge { forge: None }, b"/forges/connect".as_slice()),
        (Address::Welcome { step: Step::Forge { forge: None } }, b"/welcome?at=forge".as_slice()),
    ] {
        let (_, mut out) = manifest_started(address);
        let mut sent = None;
        while let Some(request) = out.pop() {
            if let d::Request::Client {
                request: jig_client::Request::Down { down: jig_client::Down::Send { request, value, .. } },
            } = request
            {
                assert!(value.is_none());
                let jig_client::Ask::Family { record, .. } = request else { panic!("family") };
                sent = Some(
                    temper_client_protocol_wire::decode_requesting(&record.bytes, &w::CEILINGS)
                        .unwrap()
                        .into_parts()
                        .request,
                );
            }
        }
        let w::Request::Connect(connect) = sent.expect("manifest Connect") else { panic!("Connect") };
        assert_eq!(connect.forge(), GITHUB);
        let w::How::GithubManifest(manifest) = connect.how() else { panic!("manifest") };
        assert_eq!(manifest.back().as_deref(), Some(back));
    }
}
#[test]
fn connect_github_is_disabled_until_the_answer() {
    let (root, _) = manifest_started(Address::ConnectForge { forge: None });
    let view = f::browser_page_view(&root);
    let button = node(view.tree(), Role::Button, b"Connect GitHub");
    assert!(view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
}
#[test]
fn connect_github_refusal_shows_under_its_button() {
    let (mut root, out) = manifest_started(Address::ConnectForge { forge: None });
    answer(
        &mut root,
        attempt(out),
        w::Outcome::ConnectRefused(
            w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::Busy }).unwrap(),
        ),
    );
    let view = f::browser_page_view(&root);
    words_in_order(
        view.tree(),
        &["Connect GitHub", "temper is busy with forge 2: try again when it ends.", "Use an App of your own"],
    );
}
#[test]
fn connect_github_link_leaves_the_tab_once() {
    let (mut root, out) = manifest_started(Address::ConnectForge { forge: None });
    let attempt = attempt(out);
    let outcome = w::Outcome::Link(
        w::Link::new(
            &w::CEILINGS,
            w::LinkParts { address: b"http://fake.invalid/forges/2/github/manifest?state=fake".as_slice().into() },
        )
        .unwrap(),
    );
    let mut out = drive(&mut root, answer_event(attempt, outcome.clone()));
    let mut leaves = Vec::new();
    while let Some(request) = out.pop() {
        if let d::Request::Leave { to } = request {
            leaves.push(to);
        }
    }
    assert_eq!(leaves, [Box::<[u8]>::from(b"http://fake.invalid/forges/2/github/manifest?state=fake".as_slice())]);
    let mut out = drive(&mut root, answer_event(attempt, outcome));
    while let Some(request) = out.pop() {
        if let d::Request::Leave { .. } = request {
            panic!("only the first answer may leave the page");
        }
    }
    let view = f::browser_page_view(&root);
    let link = node(view.tree(), Role::Link, b"Connect GitHub");
    assert_eq!(
        view.tree().nodes().iter().find(|n| n.id == link).unwrap().external.as_deref(),
        Some(b"http://fake.invalid/forges/2/github/manifest?state=fake".as_slice())
    );
}
#[test]
fn connecting_uses_the_apps_way_for_its_words() {
    for (app, words) in [
        (
            Some(w::AppWay::Manifest),
            "my-app is made on GitHub, under your account, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
        (
            Some(w::AppWay::Pasted),
            "my-app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
        (
            None,
            "my-app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
    ] {
        let mut parts = github_rows(false).get(0).unwrap().clone().into_parts();
        parts.app = app;
        let mut rows = List::with_capacity(1);
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) }, rows);
        words_in_order(f::browser_page_view(&root).tree(), &["Install temper’s App", words, "Install on GitHub"]);
    }
}
