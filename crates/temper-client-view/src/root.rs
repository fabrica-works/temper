//! Keep: frame, owner page and two slot passes (root.md, 9).
//! Replace: Ops connector fragments and phrasebooks with the forge's.
use crate::{
    Limits, forge,
    words::{Word, Words},
};
use jig_view::{Act, Builder, JigBindings, NodeId, Owner, Patch, Role, Tree};
use skein_lib::Queue;
use temper_client_domain::{Action, Domain};
/// The composed semantic tree and jig's bindings for its last build.
#[derive(Debug)]
pub struct View {
    render: jig_view::View,
    jig_words: jig_view::words::Words,
    forge_words: Words,
    jig: JigBindings,
    forge: forge::bindings::Bindings,
    nav: jig_view::SlotRef,
    limits: Limits,
}
impl View {
    #[must_use]
    pub fn new(
        limits: &Limits,
        domain: &temper_client_domain::Limits,
        jig_words: jig_view::words::Words,
        forge_words: Words,
        forge: u16,
    ) -> View {
        assert!(forge_words.connector == forge, "phrasebook belongs to its forge connector");
        assert!(crate::worst_case(limits, domain, &jig_words, &forge_words).is_some(), "view capacities are checked");
        View {
            render: jig_view::View::new(&limits.tree),
            jig: JigBindings::new(limits.tree.bindings, domain.jig.text),
            forge: forge::bindings::Bindings::new(limits.tree.bindings, domain.forge.checks, forge),
            nav: forge::nav(forge),
            jig_words,
            forge_words,
            limits: *limits,
        }
    }
    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.render.tree()
    }
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }
}
pub fn build(view: &mut View, root: &Domain, out: &mut Queue<Patch>) {
    let shown = temper_client_domain::shown(root);
    if !view.render.stale(shown) {
        return;
    }
    view.jig.begin();
    view.forge.begin();
    let mut builder = view.render.builder();
    builder.open(Role::Page);
    if !jig_view::jig(&mut builder, root.jig(), view.nav, &view.jig_words, &mut view.jig) {
        if let Some(form) = root.forge().form() {
            let project = forge::page::project_name(root.jig(), form.project, &view.jig_words);
            forge::page::page(&mut builder, form, &project, &view.forge_words, &view.jig_words, &mut view.forge);
        }
        jig_view::after_page(&mut builder, root.jig(), &view.jig_words, &mut view.jig);
    }
    builder.close();
    for pass in 0..2 {
        for slot in builder.slots(pass) {
            let mut fragment = Builder::new(&view.limits.tree);
            let filled = match slot.reference.owner {
                Owner::Jig | Owner::Other { .. } => false,
                Owner::Side { connector } => {
                    if connector != root.forge_number() {
                        false
                    } else if slot.reference == view.nav {
                        forge::page::navigation(
                            &mut fragment,
                            root.jig().frame_summary().project,
                            &view.forge_words,
                            &mut view.forge,
                        );
                        true
                    } else {
                        match view.jig.part(slot.reference.token) {
                            Some(part) => {
                                forge::part(
                                    &mut fragment,
                                    root.forge(),
                                    part,
                                    root.jig().generation(),
                                    &view.forge_words,
                                );
                                true
                            }
                            None => match view.jig.owner_part(slot.reference.token) {
                                Some(part) => {
                                    forge::owner_part(&mut fragment, part, &view.forge_words);
                                    true
                                }
                                None => false,
                            },
                        }
                    }
                }
            };
            if filled {
                builder.splice(slot.id, fragment);
            } else {
                builder.unshowable(slot.id, &view.forge_words.say(Word::Unshowable, &[]));
            }
        }
    }
    view.render.finish(builder, shown, out);
}
#[must_use]
#[expect(clippy::manual_map, reason = "step code explicitly wraps the owner action")]
pub fn act(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
    let bound = view.render.act(node, act)?;
    match bound.binding.owner {
        Owner::Jig => match jig_view::decode(&mut view.jig, bound.binding.token, bound.act) {
            Some(action) => Some(Action::Jig(action)),
            None => None,
        },
        Owner::Side { connector } => {
            if connector == view.forge_words.connector {
                forge::bindings::decode(&view.forge, bound.binding.token, bound.act)
            } else {
                None
            }
        }
        Owner::Other { .. } => None,
    }
}
