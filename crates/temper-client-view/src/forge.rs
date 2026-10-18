//! Keep: owner parts and parts held for the page (root.md, 6–9).
//! Replace: Ops subjects with the forge's resource words and fixed paragraph.
use crate::words::{Word, Words};
use alloc::boxed::Box;
use jig_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::{Reader, Writer, bytes::copy_of};
use skein_ui::{Builder, Role, phrases::Hole};
pub mod bindings;
pub mod page;
pub mod words;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource<'a> {
    Repository { host: &'a [u8], owner: &'a [u8], name: &'a [u8] },
    Branch { host: &'a [u8], owner: &'a [u8], name: &'a [u8], branch: &'a [Box<[u8]>] },
    Pull { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    Issue { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    Adopted { forge: u16, repository: u32 },
    Other,
}
#[must_use]
pub fn resource(path: &[Box<[u8]>]) -> Resource<'_> {
    match path {
        [forge, repository] if forge.len() == 2 && repository.len() == 4 => Resource::Adopted {
            forge: Reader::new(forge).u16().expect("two bytes"),
            repository: Reader::new(repository).u32().expect("four bytes"),
        },
        [prefix, host, owner, name] if prefix.as_ref() == b"forge" => Resource::Repository { host, owner, name },
        [prefix, host, owner, name, tag, branch @ ..]
            if prefix.as_ref() == b"forge" && tag.as_ref() == b"branch" && !branch.is_empty() =>
        {
            Resource::Branch { host, owner, name, branch }
        }
        [prefix, host, owner, name, tag, number] if prefix.as_ref() == b"forge" && tag.as_ref() == b"pull" => {
            Resource::Pull { host, owner, name, number }
        }
        [prefix, host, owner, name, tag, number] if prefix.as_ref() == b"forge" && tag.as_ref() == b"issue" => {
            Resource::Issue { host, owner, name, number }
        }
        _ => Resource::Other,
    }
}
fn joined(segments: &[Box<[u8]>]) -> Box<[u8]> {
    let mut bytes = segments.len().saturating_sub(1);
    for segment in segments {
        bytes = bytes.checked_add(segment.len()).expect("bounded resource path fits");
    }
    let mut writer = Writer::new(bytes);
    for (index, segment) in segments.iter().enumerate() {
        if index != 0 {
            writer.put(b"/").expect("separator counted");
        }
        writer.put(segment).expect("segment counted");
    }
    writer.finish()
}
fn full_name(owner: &[u8], name: &[u8]) -> Box<[u8]> {
    let size =
        owner.len().checked_add(name.len()).expect("bounded full name fits").checked_add(1).expect("separator fits");
    let mut writer = Writer::new(size);
    writer.put(owner).expect("owner counted");
    writer.put(b"/").expect("separator counted");
    writer.put(name).expect("name counted");
    writer.finish()
}
fn path_text(builder: &mut Builder, path: &[Box<[u8]>], words: &Words) {
    match resource(path) {
        Resource::Repository { host, owner, name } => {
            words.text(builder, Word::Repository, &[Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))]);
        }
        Resource::Branch { host, owner, name, branch } => words.text(
            builder,
            Word::Branch,
            &[Hole::Name(joined(branch)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))],
        ),
        Resource::Pull { host, owner, name, number } => words.text(
            builder,
            Word::Pull,
            &[Hole::Name(copy_of(number)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))],
        ),
        Resource::Issue { host, owner, name, number } => words.text(
            builder,
            Word::Issue,
            &[Hole::Name(copy_of(number)), Hole::Name(full_name(owner, name)), Hole::Name(copy_of(host))],
        ),
        Resource::Adopted { forge, repository } => {
            words.text(builder, Word::Adopted, &[Hole::Number(u64::from(repository)), Hole::Number(u64::from(forge))]);
        }
        Resource::Other => words.text(builder, Word::Path, &[Hole::Name(joined(path))]),
    }
}
fn shown(builder: &mut Builder, words: &Words) {
    builder.open(Role::Paragraph);
    words.text(builder, Word::Shown, &[]);
    builder.close();
}
pub fn part(builder: &mut Builder, side: &temper_client_domain::forge::Side, part: &Part, words: &Words) {
    if !side.contains(part) {
        builder.open(Role::Status);
        words.text(builder, Word::Unshowable, &[]);
        builder.close();
        return;
    }
    match part {
        Part::Connector { .. } => words.text(builder, Word::Connector, &[]),
        Part::Resource { path, .. } => path_text(builder, path.as_slice(), words),
        Part::Kind { kind, .. } => match crate::words::kind(*kind) {
            Some(word) => words.text(builder, word, &[]),
            None => words.text(builder, Word::OtherKind, &[Hole::Number(u64::from(*kind))]),
        },
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::OwnerEffect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => shown(builder, words),
    }
}
pub fn owner_part(builder: &mut Builder, part: &OwnerPart, words: &Words) {
    match part.kind {
        OwnerPartKind::Resource => match &part.key {
            OwnerPartKey::Path(path) => path_text(builder, path.as_slice(), words),
            OwnerPartKey::Number(_) => shown(builder, words),
        },
        OwnerPartKind::Pool
        | OwnerPartKind::Result
        | OwnerPartKind::Procedure
        | OwnerPartKind::Effect
        | OwnerPartKind::Outbox
        | OwnerPartKind::Hold => shown(builder, words),
    }
}
