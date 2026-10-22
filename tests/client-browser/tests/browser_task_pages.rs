//! Real task addresses, a fresh open and the lineage bar, on a procedure's plan (client/README.md,
//! 5.10), through `temper box`'s demo and its own fakes. The terminal-side proofs are tests/client's
//! `tests/task.rs`, `crates/temper-client-shell-terminal/src/tests/golden/agent_task.txt`, and
//! `crates/temper/tests/qa_delegation.rs`'s Task 2 in a box. The browser alone proves the addresses
//! and the first read after a full page load at `/tasks/3`.
#![expect(
    clippy::disallowed_types,
    reason = "the real browser tier measures the box's startup, outside a replaying world's clock"
)]
use skein_fake_person::{Find, Next, Step as PersonStep, Target};
use skein_fake_person_browser::{BrowserFace, Harness, Outcome, Play, Script, Step};
use skein_lib::Time;
use skein_ui::Role;
use std::time::Instant;
use temper_client_browser_world::{Tier, WITHIN};

/// Measure from the harness's start through the live Chats page, before starting the story.
struct Started {
    began: Instant,
    chats: Option<Script>,
    story: Script,
}
impl Play for Started {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        if let Some(chats) = &mut self.chats {
            match chats.next(face, now) {
                Next::Done => {
                    eprintln!("box start to live Chats: {} ms", self.began.elapsed().as_millis());
                    self.chats = None;
                }
                other @ (Next::Do(_) | Next::Wait { .. } | Next::Failed { .. }) => return other,
            }
        }
        self.story.next(face, now)
    }
    fn at(&self) -> u32 {
        self.chats.as_ref().map_or_else(|| self.story.at(), Script::at)
    }
}

#[test]
fn a_procedures_plan_reaches_its_agent_and_back_by_address() {
    let tier = Tier::demo("procedure-plan-agent-addresses");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |find| Step::Person(PersonStep::Press { find });
    let heading = |level, name: &[u8]| Find::exactly_named(Role::Heading { level }, name);
    let task = |number: &[u8]| Find::exactly_named(Role::Link, number);
    let change = b"Export widget's table as CSV";
    let agent = "T3 · Export widget's table as CSV".as_bytes();
    let plan_agent = || {
        task(b"Task 3")
            .within(Box::new([Target::ExactRole { role: Role::TreeItem, name: b"Task 3".as_slice().into() }]))
    };
    let chats = Script::new(Box::new([
        see(tier.states().live),
        press(Find::exactly_named(Role::Link, b"Chats")),
        see(heading(1, b"What shall we change?")),
    ]));
    let story = Script::new(Box::new([
        see(Find::exactly_named(Role::Button, b"Add CSV export to widget")),
        press(Find::exactly_named(Role::Button, b"Add CSV export to widget")),
        // 2: qa_delegation::running_card_names_the_change_and_exposes_its_steps opens this procedure.
        see(Find::exactly_named(Role::Region, b"Task 2")),
        see(task(b"Task 2")
            .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }]))),
        press(
            task(b"Task 2")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }])),
        ),
        Step::Address(b"/tasks/2".as_slice().into()),
        see(heading(1, change)),
        see(heading(2, b"Its tasks")),
        // 3: task::a_delegates_page_shows_its_brief_run_and_report follows a plan to its agent.
        see(Find::exactly_named(Role::TreeItem, b"Task 3")),
        see(plan_agent()),
        press(plan_agent()),
        Step::Address(b"/tasks/3".as_slice().into()),
        see(heading(1, agent)),
        see(Find::exactly_named(Role::Region, b"Task brief")),
        // 4: task::a_delegates_page_shows_its_brief_run_and_report returns through the lineage.
        press(task(b"Task 2")),
        Step::Address(b"/tasks/2".as_slice().into()),
        see(heading(2, b"Its tasks")),
        // 5: no terminal match: a fresh browser client opens at its address and receives its first read.
        Step::Person(PersonStep::Go { address: b"/tasks/3".as_slice().into() }),
        see(heading(1, agent)),
        see(Find::exactly_named(Role::Region, b"Task brief")),
        Step::Address(b"/tasks/3".as_slice().into()),
    ]));
    let mut harness = Harness::new(tier.setup());
    let mut measured = Started { began: Instant::now(), chats: Some(chats), story };
    let outcome = harness.run(&mut [&mut measured]);
    eprintln!("box task-page story: {} ms", measured.began.elapsed().as_millis());
    assert_eq!(outcome, Outcome::Done);
}
