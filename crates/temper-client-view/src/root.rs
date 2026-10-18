//! Keep: frame, owner page and two slot passes (root.md, 9).
//! Replace: Ops connector fragments and phrasebooks with the forge's.
use crate::{
    Limits, forge,
    words::{Word, Words},
};
use jig_client_view::{JigBindings, Owner};
use skein_lib::Queue;
use skein_ui::{Act, Builder, NodeId, Patch, Role, Tree};
use temper_client_domain::{Action, Domain};
/// The composed semantic tree and jig's bindings for its last build.
#[derive(Debug)]
pub struct View {
    render: skein_ui::View,
    jig_words: jig_client_view::words::Words,
    forge_words: Words,
    jig: JigBindings,
    forge: forge::bindings::Bindings,
    pages: crate::frame::Bindings,
    limits: Limits,
}
impl View {
    #[must_use]
    pub fn new(
        limits: &Limits,
        domain: &temper_client_domain::Limits,
        jig_words: jig_client_view::words::Words,
        forge_words: Words,
        forge: u16,
    ) -> View {
        assert!(forge_words.connector == forge, "phrasebook belongs to its forge connector");
        assert!(crate::worst_case(limits, domain, &jig_words, &forge_words).is_some(), "view capacities are checked");
        View {
            render: skein_ui::View::new(&limits.tree),
            jig: JigBindings::new(limits.tree.bindings, domain.jig.text),
            forge: forge::bindings::Bindings::new(limits.tree.bindings, domain.forge.checks, forge),
            pages: crate::frame::Bindings::new(limits.tree.bindings),
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
    view.pages.begin();
    let jig = root.jig().borrow(root.client());
    let mut builder = view.render.builder();
    builder.open(Role::Page);
    crate::frame::banner(&mut builder, root, &view.forge_words, &mut view.pages);
    crate::frame::navigation(&mut builder, root, &view.forge_words, &mut view.pages);
    if let temper_client_domain::Address::Inbox { .. } = root.pages().address() {
        crate::pages::inbox::main(
            &mut builder,
            root,
            &view.forge_words,
            &view.jig_words,
            &mut view.pages,
            &mut view.jig,
        );
    } else if *root.pages().address() == temper_client_domain::Address::Chats {
        crate::pages::chats::main(
            &mut builder,
            root,
            &view.forge_words,
            &view.jig_words,
            &mut view.pages,
            view.jig.text_limit(),
        );
    } else if !jig_client_view::pages::main(&mut builder, &jig, &view.jig_words, &mut view.jig)
        && let Some(form) = root.forge().form()
    {
        let project = forge::page::project_name(&jig, form.project, &view.jig_words);
        builder.open(Role::Main);
        forge::page::page(
            &mut builder,
            form,
            &project,
            &view.forge_words,
            &view.jig_words,
            &mut view.forge,
            view.limits.tree.text_per_node,
        );
        builder.close();
    }
    jig_client_view::after_page(&mut builder, &jig, &view.jig_words, &mut view.jig);
    builder.close();
    for pass in 0..2 {
        for slot in builder.slots(pass) {
            let mut fragment = Builder::new(&view.limits.tree);
            if slot.reference == jig_client_view::signin::SLOT {
                builder.splice(slot.id, fragment);
                continue;
            }
            let filled = match Owner::from_opaque(slot.reference.owner) {
                Some(Owner::Jig) | None => false,
                Some(Owner::Application { number: connector }) => {
                    if connector == root.forge_number() {
                        match view.jig.part(slot.reference.token) {
                            Some(part) => {
                                forge::part(&mut fragment, root.forge(), part, &view.forge_words);
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
                    } else {
                        false
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
    if bound.binding.owner == crate::frame::owner() {
        return view.pages.decode(bound.binding.token, bound.act);
    }
    match Owner::from_opaque(bound.binding.owner) {
        Some(Owner::Jig) => match jig_client_view::decode(&mut view.jig, bound.binding.token, bound.act) {
            Some(action) => Some(Action::Jig(action)),
            None => None,
        },
        Some(Owner::Application { number: connector }) => {
            if connector == view.forge_words.connector {
                forge::bindings::decode(&view.forge, bound.binding.token, bound.act)
            } else {
                None
            }
        }
        None => None,
    }
}
