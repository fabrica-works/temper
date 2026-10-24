//! `ConnectForge` and the welcome's shared forge body (client/forge.md, 5.4).
use super::bindings::{Bindings as ForgeBindings, Meaning};
use crate::{
    flow::{self, Control, Mark},
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Choice, Domain, Step,
    forge::{
        self,
        connect::{self, Name, Shown},
        forges::{Connection, FirstState, Kind, Mode, Offer, Row, SetUpAnswer, SetUpOutcome, SetUpProject, SetupField},
    },
    pages::Act,
};

#[derive(Clone, Copy)]
pub(crate) enum Place {
    Connect,
    Welcome,
}
impl Place {
    fn address(self, forge: Option<Choice>) -> Address {
        match self {
            Self::Connect => Address::ConnectForge { forge },
            Self::Welcome => Address::Welcome { step: Step::Forge { forge } },
        }
    }
}
pub(crate) fn text(b: &mut Builder, w: &Words, role: Role, word: Word) {
    crate::cards::text(b, role, &w.say(word, &[]));
}
pub(crate) fn link(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, address: Address) {
    flow::control(b, Control::Link { name: &w.say(word, &[]), to: bindings.insert(Act::Go { address }) });
}
pub(crate) fn name(w: &Words, row: &Row, root: &Domain) -> Box<[u8]> {
    w.say(
        match connect::name(row, root.forge().demo()) {
            Name::Here => Word::ForgeHere,
            Name::GitHub => Word::ForgeGitHub,
            Name::Forgejo => Word::ForgeOther,
            Name::Playground => Word::ForgePlayground,
        },
        &[],
    )
}
pub(crate) fn personal(root: &Domain) -> Option<&Row> {
    connect::personal(root.forges(), root.forge().demo())
}
pub(crate) fn underway(row: &Row) -> bool {
    connect::underway(row)
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    choice: Option<Choice>,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::CONNECT_FORGE);
    text(b, w, Role::Heading { level: 1 }, Word::WelcomeForgeTitle);
    let shown = connect::step(choice, root.forges(), root.forge().demo());
    let connecting = match choice {
        Some(Choice::Elsewhere) => true,
        Some(Choice::Forge(number)) => match connect::row(number, root.forges(), root.forge().demo()) {
            Some(row) => row.kind == Kind::GitHub || row.mode != Mode::Owned,
            None => false,
        },
        None => false,
    };
    let choose = w.say(Word::ForgeChoose, &[]);
    let details = w.say(Word::ForgeDetails, &[]);
    let setting = w.say(if connecting { Word::ForgeConnecting } else { Word::ForgeSettingUp }, &[]);
    let ready = w.say(Word::ForgeReady, &[]);
    let current = match shown {
        Shown::Choose => 0,
        Shown::Details(_) => 1,
        Shown::SettingUp(_) => 2,
        Shown::Ready(_) => 3,
    };
    flow::steps(b, w, &[&choose, &details, &setting, &ready], current);
    match shown {
        Shown::Choose | Shown::Details(_) => body(b, root, w, bindings, forge, choice, Place::Connect),
        Shown::SettingUp(number) => {
            if let Some(row) = connect::row(number, root.forges(), root.forge().demo()) {
                setting_up(b, root, w, forge, row);
            }
        }
        Shown::Ready(number) => {
            if let Some(row) = connect::row(number, root.forges(), root.forge().demo()) {
                ready_card(b, root, w, bindings, forge, row);
            }
        }
    }
    b.close();
}
pub(crate) fn body(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    choice: Option<Choice>,
    place: Place,
) {
    match connect::step(choice, root.forges(), root.forge().demo()) {
        Shown::Choose => choose(b, root, w, bindings, forge, place),
        Shown::Details(choice) => {
            if let Place::Connect = place {
                link(b, w, bindings, Word::ForgeAll, place.address(None));
            }
            match choice {
                Choice::Elsewhere => elsewhere(b, root, w),
                Choice::Forge(number) => {
                    if let Some(row) = connect::row(number, root.forges(), root.forge().demo()) {
                        match row.kind {
                            Kind::GitHub => github(b, root, w, forge, row),
                            Kind::Forgejo => here(b, root, w, forge, row),
                        }
                    }
                }
            }
        }
        Shown::SettingUp(_) | Shown::Ready(_) => {}
    }
}
/// The welcome keeps GitHub's installation and Ready steps in its own flow.
pub(crate) fn welcome(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    choice: Option<Choice>,
) {
    if let Some(row) = personal(root) {
        if row.kind == Kind::GitHub
            && let Connection::Connected { .. } = row.connection
        {
            if forge::forges::accepted(row) {
                ready_card(b, root, w, bindings, forge, row);
            } else {
                setting_up(b, root, w, forge, row);
            }
        } else {
            status(b, root, w, row, false);
            first_line(b, root, w, row);
            link(b, w, bindings, Word::ForgeView, Address::ConnectForge { forge: Some(Choice::Forge(row.number)) });
        }
    } else {
        body(b, root, w, bindings, forge, choice, Place::Welcome);
    }
}
fn choose(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, forge: &mut ForgeBindings, place: Place) {
    b.open(Role::Slot);
    b.style(look::WELCOME_CHOICES);
    for choice in &connect::choices(root.forges(), root.forge().demo()) {
        let (heading, words, note, action) = match *choice {
            Choice::Elsewhere => {
                (Word::ForgeElsewhere, Word::ForgeElsewhereDescription, Word::ForgeElsewhereTime, Word::ForgeShowHow)
            }
            Choice::Forge(number) => {
                let Some(row) = connect::row(number, root.forges(), root.forge().demo()) else { continue };
                match row.kind {
                    Kind::GitHub => {
                        (Word::ForgeGitHub, Word::ForgeGitHubDescription, Word::ForgeGitHubTime, Word::ForgeChoose)
                    }
                    Kind::Forgejo => (
                        Word::ForgeHere,
                        Word::ForgeHereDescription,
                        if row.offer == Offer::NotUnderTheBox { Word::ForgeCommandsTime } else { Word::ForgeHereTime },
                        Word::ForgeChoose,
                    ),
                }
            }
        };
        if heading == Word::ForgeGitHub {
            flow::card(b, &w.say(heading, &[]));
            text(b, w, Role::Paragraph, words);
            text(b, w, Role::Paragraph, note);
            if let Choice::Forge(number) = *choice {
                app_control(b, root, w, forge, number, Word::ForgeConnectGitHub, Some(root.pages().address().clone()));
            }
            link(b, w, bindings, Word::ForgeUseApp, place.address(Some(*choice)));
            b.close();
            continue;
        }
        flow::choice(
            b,
            &w.say(heading, &[]),
            &w.say(words, &[]),
            &w.say(note, &[]),
            Control::Link {
                name: &w.say(action, &[]),
                to: bindings.insert(Act::Go { address: place.address(Some(*choice)) }),
            },
        );
    }
    b.close();
}
fn github(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    flow::card(b, &w.say(Word::ForgeAppTitle, &[]));
    text(b, w, Role::Paragraph, Word::ForgeAppDescription);
    crate::cards::text(b, Role::Paragraph, &w.say(Word::ForgeSetupUrl, &[Hole::Number(u64::from(row.number))]));
    if w.shell == crate::words::Shell::Terminal {
        crate::cards::text(b, Role::Paragraph, &forge::words::app_terminal(row.number));
    } else {
        let draft = root.forge().app_form();
        for (secret, word, meaning, required) in [
            (false, Word::ForgeAppId, Meaning::AppId, draft.id.is_empty()),
            (true, Word::ForgePrivateKey, Meaning::AppKey, !draft.has_key() && !draft.too_long),
        ] {
            b.open(Role::Slot);
            b.open(Role::TextField);
            b.name(&w.say(word, &[]));
            b.bind(bindings.insert(meaning));
            if secret {
                b.entry(skein_ui::EntryKind::Secret);
                b.value(b"", draft.key_written);
            } else {
                b.value(&draft.id, draft.written);
            }
            if draft.checked && (required || (!secret && draft.number().is_none())) {
                b.state(State::Invalid);
            }
            b.close();
            if draft.checked && required {
                text(b, w, Role::Alert, Word::SetupRequired);
            } else if draft.checked && !secret && draft.number().is_none() {
                crate::cards::text(b, Role::Alert, forge::words::APP_ID);
            }
            b.close();
        }
        app_control(b, root, w, bindings, row.number, Word::ForgeConnectAppButton, None);
        if draft.checked && draft.too_long {
            crate::cards::text(b, Role::Alert, forge::words::APP_TOO_LONG);
        }
        text(b, w, Role::Paragraph, Word::ForgePasted);
    }
    if let Some(line) = forge::words::last_connect(row) {
        crate::cards::text(b, Role::Alert, &line);
    }
    b.close();
}
pub(crate) fn app_control(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    number: u16,
    word: Word,
    back: Option<Address>,
) {
    let pending = root.forge().app();
    let busy = match pending {
        Some(pending) => pending.answer.is_none(),
        None => false,
    };
    let name = w.say(word, &[]);
    if busy || root.forge().served() == forge::Served::OutOfDate {
        flow::control(b, Control::Disabled { name: &name });
    } else {
        let meaning = match back {
            Some(back) if word == Word::ForgeConnectGitHub => Meaning::ConnectManifest { forge: number, back },
            Some(back) => Meaning::Install { forge: number, back },
            None => Meaning::ConnectApp(number),
        };
        flow::control(b, Control::Button { name: &name, to: bindings.insert(meaning) });
    }
    if let Some(pending) = pending
        && pending.forge == number
        && let Some(answer) = &pending.answer
    {
        if let Some(words) = forge::words::app_refusal(answer.clone(), number) {
            crate::cards::text(b, Role::Alert, &words);
        }
        if let forge::github::Answer::Entrance(refusal) = answer {
            b.open(Role::Alert);
            w.refusal(b, refusal.refusal(), w.name_bytes);
            b.close();
        }
        if let forge::github::Answer::Link(address) = answer {
            b.open(Role::Link);
            b.name(&name);
            b.text(Run::Plain, &name);
            b.external(address);
            b.close();
        }
    }
}
fn here(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    let terminal = row.offer == Offer::NotUnderTheBox;
    flow::card(b, &w.say(if terminal { Word::ForgeTerminal } else { Word::ForgeHere }, &[]));
    if terminal {
        text(b, w, Role::Paragraph, Word::ForgeTerminalDetails);
        let commands = forge::words::set_up_commands(row.number);
        let mut rest = commands.as_ref();
        b.open(Role::Paragraph);
        b.style(look::FLOW_CODE);
        // The shared forge words delimit exactly the four terminal commands with backticks.
        for index in 0_u32..4 {
            let Some(start) = skein_lib::bytes::find(rest, b"`") else { break };
            let tail = rest.get(start..).expect("command start").strip_prefix(b"`").expect("backtick");
            let Some(end) = skein_lib::bytes::find(tail, b"`") else { break };
            if index != 0 {
                b.text(Run::Code, b"\n");
            }
            b.text(Run::Code, tail.get(..end).expect("command end"));
            rest = tail.get(end..).expect("command end").strip_prefix(b"`").expect("backtick");
        }
        b.close();
        text(b, w, Role::Paragraph, Word::ForgeTerminalNext);
    } else {
        text(b, w, Role::Paragraph, Word::ForgeHereDetails);
        setup_fields(b, root, w, bindings, row.number);
        text(b, w, Role::Paragraph, Word::ForgeIsolation);
        text(b, w, Role::Paragraph, Word::ForgeStorage);
        setup_control(b, root, w, bindings, row.number, Word::ForgeSetUp);
    }
    b.close();
}
fn code(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::FLOW_CODE);
    b.text(Run::Code, words);
    b.close();
}
fn elsewhere(b: &mut Builder, root: &Domain, w: &Words) {
    flow::card(b, &w.say(Word::ForgeElsewhere, &[]));
    text(b, w, Role::Paragraph, Word::ForgeElsewhereDetails);
    let number = u64::from(connect::next_number(root.forges()));
    for (word, snippet) in [
        (Word::ForgeConfig, Word::ForgeConfigCode),
        (Word::ForgeIdentity, Word::ForgeIdentityCode),
        (Word::ForgeSetupEntry, Word::ForgeSetupCode),
    ] {
        text(b, w, Role::Paragraph, word);
        let holes: &[Hole] = if snippet == Word::ForgeIdentityCode {
            &[Hole::Number(number), Hole::Number(number)]
        } else {
            &[Hole::Number(number)]
        };
        code(b, &w.say(snippet, holes));
    }
    for word in [Word::ForgeBotOnly, Word::ForgeRestart, Word::ForgeElsewhereUnavailable] {
        text(b, w, Role::Paragraph, word);
    }
    b.close();
}
pub(crate) fn setup_control(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    number: u16,
    word: Word,
) {
    let pending = match root.forge().set_up() {
        Some(last) => last.progress.is_some(),
        None => false,
    };
    let name = w.say(word, &[]);
    if pending || root.forge().served() == forge::Served::OutOfDate {
        flow::control(b, Control::Disabled { name: &name });
    } else {
        flow::control(b, Control::Button { name: &name, to: bindings.insert(Meaning::SetUp(number)) });
    }
    if let Some(last) = root.forge().set_up()
        && last.forge == number
        && let Some(answer) = &last.answer
    {
        match answer {
            SetUpAnswer::Outcome(SetUpOutcome::Refused(why)) => {
                crate::cards::text(b, Role::Alert, forge::words::set_up_refusal(*why));
            }
            SetUpAnswer::EntranceRefused(refusal) => {
                b.open(Role::Alert);
                w.refusal(b, refusal.refusal(), w.name_bytes);
                b.close();
            }
            SetUpAnswer::Outcome(SetUpOutcome::SettingUp | SetUpOutcome::Unknown) => {}
        }
    }
}
pub(crate) fn progress_in(b: &mut Builder, root: Option<&Domain>, w: &Words, row: &Row) {
    if row.kind == Kind::GitHub {
        flow::progress(
            b,
            w,
            &w.say(Word::ForgeProgress, &[]),
            &[(
                &forge::words::forge_connection(row),
                if forge::forges::accepted(row) { Mark::Done } else { Mark::UnderWay },
            )],
        );
        return;
    }
    let current = match row.connection {
        Connection::SettingUp { step: Some(forge::forges::Step::Fetch) } => Some(0),
        Connection::SettingUp { step: Some(forge::forges::Step::Prepare) } => Some(1),
        Connection::SettingUp { step: Some(forge::forges::Step::Run) } => Some(2),
        Connection::Connecting => Some(3),
        Connection::Connected { .. } => Some(match root {
            Some(root) if adopted_project(root, row).is_some() => 5,
            Some(_) | None => 4,
        }),
        Connection::SettingUp { step: None }
        | Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => None,
    };
    let fetch = w.say(if current > Some(0) { Word::ForgeFetched } else { Word::ForgeFetch }, &[]);
    let prepare = w.say(Word::ForgePrepare, &[]);
    let run = w.say(Word::ForgeRun, &[]);
    let connecting = w.say(Word::ForgeConnectingAsTemper, &[]);
    let target: Option<(&[u8], &[u8], &SetUpProject)> = match &row.first {
        Some(first) => Some((&first.org, &first.repository, &first.project)),
        None => match root {
            Some(root) => setup_target(root, row),
            None => None,
        },
    };
    let making = match target {
        Some((org, repository, project)) => {
            let project = match root {
                Some(root) => project_name(root, w, project),
                None => match project {
                    SetUpProject::New(name) => copy_of(name),
                    SetUpProject::Existing(project) => w.say(Word::Project, &[Hole::Number(u64::from(*project))]),
                },
            };
            w.say(
                Word::ForgeMakingAndAdopting,
                &[Hole::Name(forge::words::full_name(org, repository)), Hole::Name(project)],
            )
        }
        None => w.say(Word::ForgeMakingFirst, &[]),
    };
    let (making, last_mark) = match &row.first {
        Some(first) => match first.state {
            FirstState::Failed { .. } => (forge::words::first_repository(first), Mark::Failed),
            FirstState::Making | FirstState::Made { .. } => (making, stage_mark(current, 4)),
        },
        None => (making, stage_mark(current, 4)),
    };
    if current.is_none()
        && let Connection::SettingUp { step: None } = row.connection
    {
        text(b, w, Role::Status, Word::FlowUnderWay);
    }
    flow::progress(
        b,
        w,
        &w.say(Word::ForgeProgress, &[]),
        &[
            (&fetch, stage_mark(current, 0)),
            (&prepare, stage_mark(current, 1)),
            (&run, stage_mark(current, 2)),
            (&connecting, stage_mark(current, 3)),
            (&making, last_mark),
        ],
    );
}
fn stage_mark(current: Option<usize>, index: usize) -> Mark {
    match current {
        Some(current) => mark(current, index),
        None => Mark::ToCome,
    }
}
fn setup_target<'a>(root: &'a Domain, row: &Row) -> Option<(&'a [u8], &'a [u8], &'a SetUpProject)> {
    match root.forge().set_up() {
        Some(last) if last.forge == row.number => Some((&last.org, &last.repository, &last.project)),
        Some(_) | None => None,
    }
}

fn mark(current: usize, index: usize) -> Mark {
    match index.cmp(&current) {
        core::cmp::Ordering::Less => Mark::Done,
        core::cmp::Ordering::Equal => Mark::UnderWay,
        core::cmp::Ordering::Greater => Mark::ToCome,
    }
}
pub(crate) fn setting_up(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    if row.kind == Kind::GitHub {
        if forge::forges::slug(row).is_some() {
            flow::card(b, &w.say(Word::ForgeInstallTitle, &[]));
            if row.app == Some(forge::forges::AppWay::Manifest) {
                let slug = forge::forges::slug(row).expect("connected App slug");
                crate::cards::text(
                    b,
                    Role::Paragraph,
                    &w.say(Word::ForgeManifestInstall, &[Hole::Name(copy_of(slug))]),
                );
            } else {
                crate::cards::text(b, Role::Paragraph, &forge::words::app_install(row));
            }
            app_control(b, root, w, bindings, row.number, Word::ForgeInstall, Some(root.pages().address().clone()));
        } else {
            flow::card(b, &w.say(Word::ForgeConnecting, &[]));
            text(b, w, Role::Paragraph, Word::ForgeLeave);
        }
        if let Some(line) = forge::words::last_connect(row) {
            crate::cards::text(b, Role::Alert, &line);
        }
        b.close();
        return;
    }
    flow::card(b, &w.say(Word::ForgeSettingUpTitle, &[]));
    first_line(b, root, w, row);
    if connect::failed(row) {
        crate::cards::text(b, Role::Alert, &forge::words::forge_connection(row));
        setup_control(b, root, w, bindings, row.number, Word::ForgeAgain);
    } else {
        progress_in(b, Some(root), w, row);
        text(b, w, Role::Paragraph, Word::ForgeLeave);
    }
    b.close();
}
pub(crate) fn ready_card(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    row: &Row,
) {
    if let Some(first) = &row.first {
        let repository = forge::words::full_name(&first.org, &first.repository);
        let project = project_name(root, w, &first.project);
        flow::card(b, &w.say(Word::SetupReady, &[Hole::Name(repository)]));
        crate::cards::text(
            b,
            Role::Paragraph,
            &w.say(Word::SetupIn, &[Hole::Name(project.clone()), Hole::Name(name(w, row, root))]),
        );
        progress_in(b, Some(root), w, row);
        let pending = match root.forge().set_up() {
            Some(last) if last.forge == row.number => match &last.adoption {
                Some(adoption) => adoption.outcome.is_none(),
                None => true,
            },
            Some(_) | None => false,
        };
        let open = w.say(Word::SetupOpen, &[Hole::Name(project)]);
        if pending {
            flow::control(b, Control::Disabled { name: &open });
        } else {
            let project = match root.forge().set_up() {
                Some(last) if last.forge == row.number => match &last.adoption {
                    Some(adoption) => match &adoption.outcome {
                        Some(forge::Answer::Adopted(_)) => Some(adoption.value.project),
                        Some(
                            forge::Answer::CoreRefused(_)
                            | forge::Answer::ForgeRefused(_)
                            | forge::Answer::EntranceRefused(_)
                            | forge::Answer::Unknown,
                        )
                        | None => None,
                    },
                    None => None,
                },
                Some(_) | None => match first.project {
                    SetUpProject::Existing(project) => Some(project),
                    SetUpProject::New(_) => None,
                },
            };
            if let Some(project) = project {
                flow::control(b, Control::Link { name: &open, to: bindings.insert(Act::OpenProject { project }) });
            }
        }
        setup_ending(b, root, w, forge, row);
    } else {
        flow::card(b, &w.say(Word::ForgeConnectedTitle, &[Hole::Name(name(w, row, root))]));
        if let Connection::Connected { login, .. } = &row.connection {
            crate::cards::text(b, Role::Paragraph, &w.say(Word::ForgeConnectedAs, &[Hole::Name(copy_of(login))]));
        }
        app_line(b, w, row);
        adopt(b, root, w, bindings);
    }
    b.open(Role::Paragraph);
    w.text(b, Word::ForgeOnForges, &[Hole::Name(name(w, row, root))]);
    link(b, w, bindings, Word::ForgesTitle, Address::Forges);
    w.text(b, Word::ForgeOnForgesTail, &[]);
    b.close();
    b.close();
}
/// Completion requires the originating tab's successful Adopt, rather than a Made row alone.
pub(crate) fn adopted_project(root: &Domain, row: &Row) -> Option<u32> {
    if let Connection::Connected { .. } = row.connection
        && let Some(first) = &row.first
        && let FirstState::Made { .. } = first.state
        && let Some(last) = root.forge().set_up()
        && last.forge == row.number
        && let Some(adoption) = &last.adoption
        && let Some(forge::Answer::Adopted(_)) = &adoption.outcome
    {
        return Some(adoption.value.project);
    }
    None
}
/// Refusals remain actionable from the welcome's checklist as well as `ConnectForge`'s card.
pub(crate) fn setup_ending(b: &mut Builder, root: &Domain, w: &Words, forge: &mut ForgeBindings, row: &Row) {
    if connect::failed(row) {
        crate::cards::text(b, Role::Alert, &forge::words::forge_connection(row));
        setup_control(b, root, w, forge, row.number, Word::ForgeAgain);
    }
    if let Some(first) = &row.first {
        if let Some(last) = root.forge().set_up()
            && last.forge == row.number
            && let Some(starting) = &last.starting
            && let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(
                jig_client::contract::Answer::Refused { refusal, .. },
            ))) = &starting.value.outcome
        {
            b.open(Role::Alert);
            w.refusal(b, refusal, w.name_bytes);
            b.close();
            setup_field(b, root, w, forge, SetupField::Project, Word::SetupNew, row.number);
            let label = w.say(Word::SetupAdopt, &[Hole::Name(forge::words::full_name(&first.org, &first.repository))]);
            flow::control(b, Control::Button { name: &label, to: forge.insert(Meaning::StartIn(row.number)) });
        }
        if let Some(last) = root.forge().set_up()
            && last.forge == row.number
            && let Some(adoption) = &last.adoption
            && let Some(answer) = &adoption.outcome
        {
            match answer {
                forge::Answer::Adopted(_) => {}
                forge::Answer::CoreRefused(_)
                | forge::Answer::ForgeRefused(_)
                | forge::Answer::EntranceRefused(_)
                | forge::Answer::Unknown => {
                    adoption_outcome(b, w, &adoption.value, answer);
                }
            }
        }
    }
}
pub(crate) fn adopt(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if let Some(project) = root.pages().project() {
        link(b, w, bindings, Word::ForgeAdopt, Address::Adopt { project });
    }
}
pub(crate) fn status(b: &mut Builder, root: &Domain, w: &Words, row: &Row, ready: bool) {
    let word = if ready { Word::WelcomeForgeStatus } else { Word::ForgeStatus };
    crate::cards::text(
        b,
        Role::Paragraph,
        &w.say(word, &[Hole::Name(name(w, row, root)), Hole::Name(forge::words::forge_connection(row))]),
    );
}
/// The welcome's Model step says what Forge did (client/README.md, 3.5).
pub(crate) fn welcome_line(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    let connected = match row.connection {
        Connection::Connected { .. } => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => false,
    };
    let done = match &row.first {
        Some(first) => match first.state {
            FirstState::Made { .. } => connected,
            FirstState::Making | FirstState::Failed { .. } => false,
        },
        None => false,
    };
    let target = match &row.first {
        Some(first) => Some((&first.org, &first.repository, &first.project)),
        None => match root.forge().set_up() {
            Some(last) if last.forge == row.number => Some((&last.org, &last.repository, &last.project)),
            Some(_) | None => None,
        },
    };
    let forge = name(w, row, root);
    let line = if connected && !underway(row) && row.first.is_none() {
        w.say(Word::WelcomeForgeNoRepository, &[Hole::Name(forge)])
    } else if let Some((org, repository, project)) = target {
        let project = match project {
            SetUpProject::New(name) => w.say(Word::WelcomeNewProject, &[Hole::Name(copy_of(name))]),
            SetUpProject::Existing(_) => project_name(root, w, project),
        };
        let repository = forge::words::full_name(org, repository);
        if done {
            w.say(Word::WelcomeForgeDone, &[Hole::Name(forge), Hole::Name(repository), Hole::Name(project)])
        } else {
            w.say(
                Word::WelcomeForgeLine,
                &[
                    Hole::Name(forge),
                    Hole::Name(forge::words::forge_connection(row)),
                    Hole::Name(repository),
                    Hole::Name(project),
                ],
            )
        }
    } else {
        w.say(Word::ForgeStatus, &[Hole::Name(forge), Hole::Name(forge::words::forge_connection(row))])
    };
    b.open(Role::Paragraph);
    if done {
        b.style(look::FLOW_DONE);
    }
    b.name(&line);
    if done {
        b.text(Run::Plain, "✓ ".as_bytes());
    }
    b.text(Run::Plain, &line);
    b.close();
    if !done {
        each_step(b, root, w, row);
    }
}
pub(crate) fn each_step(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    b.open(Role::Disclosure);
    b.name(&w.say(Word::ForgeProgress, &[]));
    progress_in(b, Some(root), w, row);
    b.close();
}

fn setup_fields(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, number: u16) {
    b.open(Role::Slot);
    for (field, word) in
        [(SetupField::Organization, Word::SetupOrganization), (SetupField::Repository, Word::SetupRepository)]
    {
        setup_field(b, root, w, bindings, field, word, number);
    }
    b.close();
    setup_field(b, root, w, bindings, SetupField::Project, Word::SetupNew, number);
}
fn setup_field(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    field: SetupField,
    word: Word,
    number: u16,
) {
    let draft = root.forge().setup_form();
    let invalid = match root.forge().set_up() {
        Some(last) if last.forge == number => {
            last.answer == Some(SetUpAnswer::Outcome(SetUpOutcome::Refused(forge::forges::SetUpRefusal::Invalid)))
        }
        Some(_) | None => false,
    };
    b.open(Role::Slot);
    b.open(Role::TextField);
    b.name(&w.say(word, &[]));
    b.value(draft.text(field), draft.written(field));
    b.bind(bindings.insert(Meaning::SetupField(field)));
    if draft.required(field) || (invalid && field != SetupField::Project) {
        b.state(State::Invalid);
    }
    b.close();
    if draft.required(field) {
        text(b, w, Role::Alert, Word::SetupRequired);
    }
    b.close();
}
pub(crate) fn project_name(root: &Domain, w: &Words, project: &SetUpProject) -> Box<[u8]> {
    match project {
        SetUpProject::Existing(project) => match crate::pages::project_name(root.pages(), root.client(), *project) {
            Some(name) => copy_of(name),
            None => w.say(Word::Project, &[Hole::Number(u64::from(*project))]),
        },
        SetUpProject::New(name) => {
            if let Some(last) = root.forge().set_up()
                && let Some(starting) = &last.starting
                && last.project == *project
            {
                if starting.value.adoption.project != 0
                    && let Some(name) =
                        crate::pages::project_name(root.pages(), root.client(), starting.value.adoption.project)
                {
                    return copy_of(name);
                }
                return starting.value.name.clone();
            }
            copy_of(name)
        }
    }
}
pub(crate) fn first_line(b: &mut Builder, root: &Domain, w: &Words, row: &Row) {
    let (org, repository, project) = match &row.first {
        Some(first) => (&first.org, &first.repository, &first.project),
        None => match root.forge().set_up() {
            Some(last) if last.forge == row.number => (&last.org, &last.repository, &last.project),
            Some(_) | None => return,
        },
    };
    crate::cards::text(
        b,
        Role::Paragraph,
        &w.say(
            Word::SetupInto,
            &[Hole::Name(forge::words::full_name(org, repository)), Hole::Name(project_name(root, w, project))],
        ),
    );
}
fn adoption_outcome(b: &mut Builder, w: &Words, adoption: &forge::Adoption, answer: &forge::Answer) {
    // Reuse the adoption page's outcome, with the submitted fields.
    let mut form = forge::form(adoption.project);
    form.texts.get_mut(forge::Text::Forge.index()).expect("forge field").text =
        Box::from(skein_lib::Decimal::of(u64::from(adoption.forge)).as_bytes());
    form.texts.get_mut(forge::Text::Owner.index()).expect("owner field").text.clone_from(&adoption.owner);
    form.texts.get_mut(forge::Text::Name.index()).expect("name field").text.clone_from(&adoption.name);
    super::page::outcome(b, &form, answer, w, w.name_bytes);
}

pub(crate) fn app_line(b: &mut Builder, w: &Words, row: &Row) {
    if let Some(slug) = forge::forges::slug(row) {
        b.open(Role::Slot);
        text(b, w, Role::Term, Word::ForgeAppLabel);
        b.open(Role::Description);
        crate::cards::text(b, Role::Slot, &forge::words::app_head(slug, row.installations.is_empty()));
        for (index, installation) in row.installations.iter().enumerate() {
            crate::cards::text(b, Role::Slot, &forge::words::app_installation(installation, index == 0));
        }
        b.close();
        b.close();
    }
}
