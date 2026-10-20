//! Temper's frame, pages, bindings and checked phrasebook.
use crate::{
    Limits, forge,
    words::{Word, Words},
};
use skein_lib::Queue;
use skein_ui::{Act, Builder, NodeId, Patch, Role, Tree};
use temper_client_domain::{Action, Domain};
/// The composed semantic tree and jig's bindings for its last build.
#[derive(Debug)]
pub struct View {
    render: skein_ui::View,
    forge_words: Words,
    forge: forge::bindings::Bindings,
    pages: crate::frame::Bindings,
    accounts: crate::accounts::Bindings,
    limits: Limits,
    rows: u32,
    text: u32,
}
impl View {
    #[must_use]
    pub fn new(limits: &Limits, domain: &temper_client_domain::Limits, forge_words: Words, forge: u16) -> View {
        assert!(forge_words.connector == forge, "phrasebook belongs to its forge connector");
        assert!(crate::worst_case(limits, domain, &forge_words).is_some(), "view capacities are checked");
        View {
            render: skein_ui::View::new(&limits.tree),
            forge: forge::bindings::Bindings::new(limits.tree.bindings, domain.forge.checks, forge),
            pages: crate::frame::Bindings::new(limits.tree.bindings),
            accounts: crate::accounts::Bindings::new(domain.accounts),
            forge_words,
            limits: *limits,
            rows: domain.client.domain.rows,
            text: domain.client.domain.text,
        }
    }
    fn task_main(&mut self, b: &mut Builder, root: &Domain) {
        crate::pages::task::main(b, root, &self.forge_words, &mut self.pages, self.rows);
    }
    fn adoption_main(&mut self, b: &mut Builder, root: &Domain) {
        let Some(form) = root.forge().form() else { return };
        let project = crate::pages::project_name(root.pages(), root.client(), form.project);
        b.open(Role::Main);
        b.style(crate::look::MAIN);
        forge::page::page(b, form, project, &self.forge_words, &mut self.forge, self.limits.tree.text_per_node);
        b.close();
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
    let shown = temper_client_domain::shown(root).checked_add(view.accounts.revision()).expect("view revisions fit");
    if !view.render.stale(shown) {
        return;
    }
    view.forge.begin();
    view.pages.begin();
    view.accounts.begin(root.pages().accounts().is_none());
    let mut builder = view.render.builder();
    builder.open(Role::Page);
    crate::frame::banner(&mut builder, root, &view.forge_words, &mut view.pages);
    if let Some(word) = crate::frame::unavailable(root) {
        builder.open(Role::Main);
        builder.style(crate::look::MAIN);
        builder.open(Role::Paragraph);
        view.forge_words.text(
            &mut builder,
            if word == Word::SignedOut { Word::SignedOutMain } else { Word::OutOfDateMain },
            &[],
        );
        builder.close();
        builder.close();
    } else if let temper_client_domain::Address::Inbox { .. } = root.pages().address() {
        crate::pages::inbox::main(&mut builder, root, &view.forge_words, &mut view.pages, view.rows);
    } else if *root.pages().address() == temper_client_domain::Address::Chats {
        crate::pages::chats::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            view.limits.tree.text_per_node,
        );
    } else if let temper_client_domain::Address::Chat { .. } = root.pages().address() {
        crate::pages::chat::main(&mut builder, root, &view.forge_words, &mut view.pages, view.rows, view.text);
    } else if let temper_client_domain::Address::Task { .. } = root.pages().address() {
        view.task_main(&mut builder, root);
    } else if *root.pages().address() == temper_client_domain::Address::Accounts {
        crate::pages::accounts::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.accounts,
            view.limits.tree.text_per_node,
        );
    } else if let temper_client_domain::Address::Adopt { .. } = root.pages().address() {
        view.adoption_main(&mut builder, root);
    }
    builder.close();
    view.render.finish(builder, shown, out);
}
#[must_use]
pub fn act(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
    if let temper_client_domain::Event::Act { action } = event(view, node, act)? { Some(action) } else { None }
}
/// Shell inputs can move a secret directly to the domain without a clonable action.
#[must_use]
pub fn event(view: &mut View, node: NodeId, act: Act) -> Option<temper_client_domain::Event> {
    let bound = view.render.act(node, act)?;
    if bound.binding.owner == crate::accounts::owner() {
        return view.accounts.decode(bound.binding.token, bound.act);
    }
    if bound.binding.owner == crate::frame::owner() {
        return Some(temper_client_domain::Event::Act { action: view.pages.decode(bound.binding.token, bound.act)? });
    }
    if bound.binding.owner == temper_client_domain::forge::words::owner(view.forge_words.connector) {
        Some(temper_client_domain::Event::Act {
            action: forge::bindings::decode(&view.forge, bound.binding.token, bound.act)?,
        })
    } else {
        None
    }
}
