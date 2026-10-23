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
        forges::{Connection, Kind, Mode, Offer, Row, SetUpAnswer, SetUpOutcome, SetUpProject, SetupField},
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
                ready_card(b, root, w, bindings, row);
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
        Shown::Choose => choose(b, root, w, bindings, place),
        Shown::Details(choice) => {
            link(b, w, bindings, Word::ForgeAll, place.address(None));
            match choice {
                Choice::Elsewhere => elsewhere(b, root, w),
                Choice::Forge(number) => {
                    if let Some(row) = connect::row(number, root.forges(), root.forge().demo()) {
                        match row.kind {
                            Kind::GitHub => github(b, w, row),
                            Kind::Forgejo => here(b, root, w, forge, row),
                        }
                    }
                }
            }
        }
        Shown::SettingUp(_) | Shown::Ready(_) => {}
    }
}
fn choose(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, place: Place) {
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
}
fn github(b: &mut Builder, w: &Words, row: &Row) {
    flow::card(b, &w.say(Word::ForgeGitHub, &[]));
    text(b, w, Role::Heading { level: 3 }, Word::ForgeConnectGitHub);
    text(b, w, Role::Paragraph, Word::ForgeGitHubCreate);
    field(b, w, Word::ForgeOrganization, false);
    flow::control(b, Control::Disabled { name: &w.say(Word::ForgeConnectGitHub, &[]) });
    text(b, w, Role::Heading { level: 3 }, Word::ForgeConnectApp);
    text(b, w, Role::Paragraph, Word::ForgeAppDescription);
    crate::cards::text(b, Role::Paragraph, &w.say(Word::ForgeSetupUrl, &[Hole::Number(u64::from(row.number))]));
    field(b, w, Word::ForgeAppId, false);
    field(b, w, Word::ForgePrivateKey, true);
    flow::control(b, Control::Disabled { name: &w.say(Word::ForgeConnectApp, &[]) });
    text(b, w, Role::Paragraph, Word::ForgePasted);
    text(b, w, Role::Paragraph, Word::ForgeGitHubUnavailable);
    b.close();
}
fn field(b: &mut Builder, w: &Words, word: Word, secret: bool) {
    b.open(Role::TextField);
    if secret {
        b.entry(skein_ui::EntryKind::Secret);
    }
    b.name(&w.say(word, &[]));
    b.state(State::Disabled);
    b.close();
}
fn here(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    let terminal = row.offer == Offer::NotUnderTheBox;
    flow::card(b, &w.say(if terminal { Word::ForgeTerminal } else { Word::ForgeHere }, &[]));
    if terminal {
        text(b, w, Role::Paragraph, Word::ForgeTerminalDetails);
        let commands = forge::words::set_up_commands(row.number);
        let mut rest = commands.as_ref();
        // The shared forge words delimit exactly the four terminal commands with backticks.
        for _ in 0_u32..4 {
            let Some(start) = skein_lib::bytes::find(rest, b"`") else { break };
            let tail = rest.get(start..).expect("command start").strip_prefix(b"`").expect("backtick");
            let Some(end) = skein_lib::bytes::find(tail, b"`") else { break };
            code(b, tail.get(..end).expect("command end"));
            rest = tail.get(end..).expect("command end").strip_prefix(b"`").expect("backtick");
        }
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
pub(crate) fn progress(b: &mut Builder, w: &Words, row: &Row) {
    let current = match row.connection {
        Connection::SettingUp { step: Some(forge::forges::Step::Prepare) } => 1,
        Connection::SettingUp { step: Some(forge::forges::Step::Run) } => 2,
        Connection::Connecting => 3,
        Connection::Connected { .. } => {
            if connect::underway(row) {
                3
            } else {
                4
            }
        }
        Connection::SettingUp { step: None | Some(forge::forges::Step::Fetch) }
        | Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => 0,
    };
    let fetch = w.say(Word::ForgeFetch, &[]);
    let prepare = w.say(Word::ForgePrepare, &[]);
    let run = w.say(Word::ForgeRun, &[]);
    let connecting = match &row.first {
        Some(first) => w.say(Word::SetupMaking, &[Hole::Name(forge::words::full_name(&first.org, &first.repository))]),
        None => w.say(Word::ForgeConnecting, &[]),
    };
    flow::progress(
        b,
        w,
        &w.say(Word::ForgeProgress, &[]),
        &[
            (&fetch, mark(current, 0)),
            (&prepare, mark(current, 1)),
            (&run, mark(current, 2)),
            (&connecting, mark(current, 3)),
        ],
    );
}
fn mark(current: usize, index: usize) -> Mark {
    match index.cmp(&current) {
        core::cmp::Ordering::Less => Mark::Done,
        core::cmp::Ordering::Equal => Mark::UnderWay,
        core::cmp::Ordering::Greater => Mark::ToCome,
    }
}
pub(crate) fn setting_up(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, row: &Row) {
    flow::card(b, &w.say(Word::ForgeSettingUpTitle, &[]));
    first_line(b, root, w, row);
    if connect::failed(row) {
        crate::cards::text(b, Role::Alert, &forge::words::forge_connection(row));
        setup_control(b, root, w, bindings, row.number, Word::ForgeAgain);
    } else {
        progress(b, w, row);
        text(b, w, Role::Paragraph, Word::ForgeLeave);
    }
    b.close();
}
pub(crate) fn ready_card(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, row: &Row) {
    if let Some(first) = &row.first {
        let repository = forge::words::full_name(&first.org, &first.repository);
        let project = project_name(root, w, &first.project);
        flow::card(b, &w.say(Word::SetupReady, &[Hole::Name(repository)]));
        crate::cards::text(
            b,
            Role::Paragraph,
            &w.say(Word::SetupIn, &[Hole::Name(project.clone()), Hole::Name(name(w, row, root))]),
        );
        progress(b, w, row);
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
        } else if let SetUpProject::Existing(project) = first.project {
            flow::control(b, Control::Link { name: &open, to: bindings.insert(Act::OpenProject { project }) });
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
    } else {
        flow::card(b, &w.say(Word::ForgeConnectedTitle, &[Hole::Name(name(w, row, root))]));
        if let Connection::Connected { login, .. } = &row.connection {
            crate::cards::text(b, Role::Paragraph, &w.say(Word::ForgeConnectedAs, &[Hole::Name(copy_of(login))]));
        }
        adopt(b, root, w, bindings);
    }
    b.open(Role::Paragraph);
    w.text(b, Word::ForgeOnForges, &[Hole::Name(name(w, row, root))]);
    link(b, w, bindings, Word::ForgesTitle, Address::Forges);
    w.text(b, Word::ForgeOnForgesTail, &[]);
    b.close();
    b.close();
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
pub(crate) fn each_step(b: &mut Builder, w: &Words, row: &Row) {
    b.open(Role::Disclosure);
    b.name(&w.say(Word::ForgeProgress, &[]));
    progress(b, w, row);
    b.close();
}

fn setup_fields(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, number: u16) {
    let draft = root.forge().setup_form();
    for (field, word) in
        [(SetupField::Organization, Word::SetupOrganization), (SetupField::Repository, Word::SetupRepository)]
    {
        setup_field(b, root, w, bindings, field, word, number);
    }
    b.open(Role::ChoiceGroup);
    b.name(&w.say(Word::SetupProject, &[]));
    let project = project_name(root, w, &SetUpProject::Existing(root.pages().project().unwrap_or(1)));
    for (new, label) in
        [(false, w.say(Word::SetupExisting, &[Hole::Name(project)])), (true, w.say(Word::SetupNew, &[]))]
    {
        b.open(Role::Choice);
        b.name(&label);
        b.bind(bindings.insert(Meaning::SetupNew(new)));
        if draft.new == new {
            b.state(State::Selected);
        }
        b.close();
    }
    b.close();
    if draft.new {
        setup_field(b, root, w, bindings, SetupField::Project, Word::SetupNew, number);
    }
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
        SetUpProject::New(name) => copy_of(name),
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
