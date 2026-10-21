//! The demo and fake processes around forge-setup's commands (protocol/setup.md, 7.3).
use crate::demo::Demo;
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, bytes};
use temper_forge_setup::{
    Kind, Step, Steps,
    setup::{Mode, Setup},
};

/// The box's steps for the document (protocol/setup.md, 7.1 and 8.4):
/// the demo's fake LLM, each fake entry's preparation and its fake,
/// forge-setup's steps, then the demo's seed as a link on serve's first line.
#[must_use]
pub fn steps(setup: &Setup, demo: Option<&Demo>, flags: &[Box<[u8]>]) -> Steps {
    let given = temper_forge_setup::steps(setup, flags);
    let demo_count = u32::from(demo.is_some());
    let mut prefix = demo_count;
    for entry in &setup.entries {
        match &entry.mode {
            Mode::Fake(_) => prefix = prefix.checked_add(2).expect("bounded fake steps"),
            Mode::Managed { .. } | Mode::Owned(_) => {}
        }
    }
    if prefix == 0 {
        return given;
    }
    let capacity = u32::try_from(given.always.len())
        .expect("bounded steps")
        .checked_add(prefix)
        .expect("bounded prefix")
        .checked_add(demo_count)
        .expect("bounded seed");
    let mut always = List::with_capacity(capacity);
    if demo.is_some() {
        always.push(command(b"fake llm", &[b"fake", b"llm"], flags, None, Kind::Long)).expect("demo LLM");
    }
    for entry in &setup.entries {
        match &entry.mode {
            Mode::Fake(_) => {
                always
                    .push(command(b"prepare", &[b"forge", b"prepare"], flags, Some(entry.forge), Kind::Once))
                    .expect("fake preparation");
                always
                    .push(command(b"fake forge", &[b"fake", b"forge"], flags, Some(entry.forge), Kind::Long))
                    .expect("fake forge");
            }
            Mode::Managed { .. } | Mode::Owned(_) => {}
        }
    }
    for step in given.always {
        always.push(step).expect("forge-setup steps counted");
    }
    if demo.is_some() {
        always.push(command(b"seed", &[b"fake", b"seed"], flags, None, Kind::Link)).expect("demo seed");
    }
    let mut asked = given.asked;
    for group in &mut asked {
        group.before = group.before.checked_add(prefix).expect("bounded group position");
    }
    Steps { always: always.into_boxed(), asked }
}

fn command(name: &[u8], words: &[&[u8]], flags: &[Box<[u8]>], forge: Option<u16>, kind: Kind) -> Step {
    let capacity = u32::try_from(words.len())
        .expect("command words")
        .checked_add(u32::try_from(flags.len()).expect("bounded flags"))
        .expect("bounded arguments")
        .checked_add(u32::from(forge.is_some()))
        .expect("forge number");
    let mut arguments = List::with_capacity(capacity);
    for word in words {
        arguments.push(bytes::copy_of(word)).expect("command words counted");
    }
    for flag in flags {
        arguments.push(flag.clone()).expect("flags counted");
    }
    let name = match forge {
        Some(number) => {
            let digits = Decimal::of(u64::from(number));
            let number = digits.as_bytes();
            arguments.push(bytes::copy_of(number)).expect("forge number counted");
            let length =
                name.len().checked_add(1).expect("name and space").checked_add(number.len()).expect("bounded name");
            let mut writer = Writer::new(length);
            writer.put(name).expect("name counted");
            writer.put(b" ").expect("space counted");
            writer.put(number).expect("number counted");
            writer.finish()
        }
        None => bytes::copy_of(name),
    };
    Step::new(name, arguments.into_boxed(), kind)
}
