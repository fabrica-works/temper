//! The reference case's cast, accounts and first kit tree (domain/testing.md, 5.3).
use crate::{fixtures, forge, landing};
use serde_json::json;
use skein_fake_forge_domain::{self as domain, api};

pub mod acts;
pub mod oracle;
pub mod scripts;

// Fresh deployment numbers; the oracle checks them against the person's documents
// and A's question's tool output and the answer delivered to A.
pub const PROJECT: u64 = 1;
pub const DIRECTOR: u64 = 1;
pub const A: u64 = 2;
pub const DESIGN: u64 = 3;
pub const QUESTION: u64 = 6;
pub const FIRST_ACCOUNT: u64 = 1;
pub const SECOND_ACCOUNT: u64 = 3;
pub const SECOND_KEY_FILE: &str = "second.key";
pub const DEFAULT_MODEL: &str = "fake";
pub const DEFAULT_MODEL_NUMBER: u8 = 0;
pub const SURVEY: u64 = 3;
pub const PACE_QUESTION_MESSAGE: u64 = 1;
pub const PACE_PERSON: &str = "@case_director Ask about the pace and keep my answer.";
pub const PACE_QUESTION: &str = "Claude spends eight times what the week can sustain. I recommend moving what needs no taste to Codex and switching s2 to the second subscription. Keep the pace?";
pub const PACE_ANSWER: &str = "Keep the pace while the hardware allows; move what needs no taste.";
pub const PACE_NOTE: &str = "Pace";
pub const PACE_PENDING: &str = "The pace question is waiting for your answer.";
pub const PACE_A_SPEC: &str =
    "@case_a Ask the director whether to keep the pace; on its direction, have a survey delegated and report.";
pub const REFUSED_PERSON: &str = "@case_director Delegate A's report and wait for it.";
pub const REFUSED_A_SPEC: &str = "@case_a Report when your account allows the work to continue.";
pub const SURVEY_SPEC: &str = "@case_survey Survey the work that needs no taste and report.";
pub const RESUMED_REPORT: &str = "The work continued on its account.";
pub const PACE_REPORT: &str = "The pace is kept and the survey is done.";
pub const SURVEY_REPORT: &str = "Work needing no taste can move.";
// The fake forge's one repository stands as kit, since forge.rs names it,
// until the fixture holds two.
pub const KIT: &str = "acme/app";
pub const OWNER_MODEL: &str = "owner";
pub const OWNER_EFFORT: &str = "xhigh";
// Model and effort numbers are their configuration order, after the default model.
pub const OWNER_MODEL_NUMBER: u8 = 1;
pub const OWNER_EFFORT_NUMBER: u8 = 0;
pub const DRAFT: &str = "docs/drafts/export.md";
pub const DESIGN_DOC: &str = "docs/design/export.md";
pub const OWNERS_DOC: &str = "docs/process/owners.md";
pub const DRAFT_WORDS: &str = "# Export draft\n\nExport the application's data.\n";
pub const OWNERS_WORDS: &str = "# Owners\n\nAn owner designs its draft, reports, then implements it.\n";
pub const DESIGN_WORDS: &str = "# Export\n\nDesigned by A from the draft at its commit.\n";
pub const DESIGN_LINE: &str = "Designed by A from the draft at its commit.";
pub const PERSON: &str = "@case_director Export's draft has landed; schedule it, priority 1.";
pub const A_SPEC_PREFIX: &str = "@case_a Own the draft Export: docs/drafts/export.md at kit's commit ";
pub const A_SPEC_SUFFIX: &str = ". Design it as a change of Markdown alone, send me your design report as a question, and finish on my green light.";
pub const SCHEDULED: &str = "Scheduled: A owns Export at priority 1.";
pub const GREEN_LIGHT: &str = "Green light: go on.";
pub const ANSWERED: &str = "Answered A.";
pub const DIRECTOR_REPORT: &str = "A designed Export and has its green light.";
pub const DESIGN_SPEC: &str = "@case_design Write Export's design at docs/design/export.md, Markdown alone.";
pub const DESIGN_DELEGATED: &str = "Design delegated.";
pub const DESIGN_REPORT: &str =
    "Export's design has landed on kit's main at docs/design/export.md: one Markdown file. Green light to go on?";
pub const ASKED: &str = "Asked the director.";
pub const A_REPORT: &str = "Export's design landed, and the director gave its green light.";
pub const DESIGN_TITLE: &str = "Export's design";
pub const REVIEWED: &str = "Checked";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Director,
    A,
    Design,
    Review,
    Survey,
}
impl Agent {
    #[must_use]
    pub fn cue(self) -> &'static [u8] {
        match self {
            Self::Director => b"@case_director",
            Self::A => b"@case_a ",
            Self::Design => b"@case_design",
            Self::Survey => b"@case_survey",
            Self::Review => b"Review this change at the named head",
        }
    }
}

#[must_use]
pub fn kit_tree() -> Box<[api::File]> {
    let mut tree = (forge::fixture().setup)().tree.into_vec();
    for (path, words) in [(DRAFT, DRAFT_WORDS), (OWNERS_DOC, OWNERS_WORDS)] {
        tree.push(api::File { path: path.as_bytes().into(), content: words.as_bytes().into() });
    }
    tree.into_boxed_slice()
}

#[must_use]
pub fn draft_commit() -> String {
    // State::new uses this configuration and seed. repository stores the first
    // tree without a timestamp; real-loop wall skew cannot change its identity.
    let config = temper_fake_git_hosted::fixtures::CONFIG;
    let mut domain = domain::Domain::new(&config, fixtures::SEED);
    let first = domain::repository(&mut domain, &config, (fixture().setup)());
    let object = skein_fake_forge_protocol::routes::object(first);
    let mut words = String::new();
    for byte in object.bytes {
        use std::fmt::Write;
        write!(words, "{byte:02x}").expect("commit hex");
    }
    words
}

#[must_use]
pub fn fixture() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (landing::fixture().setup)();
            setup.tree = kit_tree();
            setup.checks.cue =
                Some(api::Cue::File { path: DESIGN_DOC.as_bytes().into(), green: DESIGN_LINE.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (landing::fixture().configure)(value, agent, forge);
            // The direct service fixture has no binary-added deployment project.
            // Its owner role in project zero makes deployment accounts readable.
            let mut deployment = value["engine"]["projects"][0].clone();
            deployment["number"] = 0.into();
            value["engine"]["projects"].as_array_mut().expect("projects").push(deployment);
            value["engine"]["limits"]["overrides"]["people.projects"] = 2.into();
            value["engine"]["limits"]["overrides"]["authority.projects"] = 2.into();
            value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
            // A runs on the director's explicit choice, distinct from the default.
            let mut model = value["engine"]["llm"]["models"][0].clone();
            model["name"] = OWNER_MODEL.into();
            model["efforts"] = json!([OWNER_EFFORT]);
            value["engine"]["llm"]["models"].as_array_mut().expect("models").push(model);
            value["engine"]["llm"]["providers"][0]["models"] = json!(["fake", OWNER_MODEL]);
            value["engine"]["charters"][0]["alternatives"] = json!([OWNER_MODEL]);
            let mut model = agent["endpoints"][0]["models"][0].clone();
            model["name"] = OWNER_MODEL.into();
            model["efforts"] = json!([OWNER_EFFORT]);
            agent["endpoints"][0]["models"].as_array_mut().expect("agent models").push(model);
            // The director delegates, messages, amends and keeps project notes.
            value["engine"]["rules"]["ceiling"]["tools"] = 391.into();
            value["engine"]["rules"]["ceiling"]["notes"] = 2.into();
            // The extra owner adds one delegation level before the change procedure.
            value["engine"]["rules"]["ceiling"]["delegation"]["depth"] = 5.into();
            for project in value["engine"]["projects"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["tools"] = 391.into();
                project["policy"]["ceiling"]["notes"] = 2.into();
                project["policy"]["roles"][0]["authority"]["notes"] = 2.into();
                project["policy"]["ceiling"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["tools"] = 391.into();
            }
            // A tracked owner reserves core news, landing news, an answer and its
            // delegate's result together; the landing fixture's inbox holds 4 KiB.
            value["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = 16384.into();
            // The director's prescribed final report exceeds the chat's 32 bytes.
            value["engine"]["templates"][0]["contract"]["report"]["words"] = 128.into();
            // The brief includes the full draft commit and its instructions.
            value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 512.into();
            landing::admit(value);
        },
    }
}
