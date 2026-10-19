//! Temper's browser shell: platform facts enter, the root's requests leave.
#![cfg(target_arch = "wasm32")]
#![forbid(unsafe_code)]
use jig_client_shell_web::sign_in;
use skein_lib::Queue;
use skein_ui::{Act, NodeId};
use skein_ui_web::{
    Look,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform, wasm_bindgen,
};
use std::{cell::RefCell, rc::Rc};
mod host;
mod limits;
enum Report {
    Start,
    Act(NodeId, Act),
}
struct Shell {
    host: host::Host,
    page: Rc<RefCell<Page<'static>>>,
    dom: Dom,
}
#[wasm_bindgen(start)]
pub fn start() {
    if !skein_ui_web::restart::Restart::new(
        3,
        skein_lib::Duration::from_secs(60),
        "temper-restarts",
        "The application failed.",
    )
    .install("app")
    {
        return;
    }
    // Replace the printed secret's address before starting anything that can send.
    let secret = sign_in::web::take_from_page(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/" });
    let window = web_sys::window().expect("window");
    let mount = window.document().expect("document").get_element_by_id("app").expect("bundle mount");
    let facts = platform::facts();
    let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
    #[cfg(debug_assertions)]
    assert_ne!(path.split('?').next(), Some("/test/panic"), "the test address panics at start");
    let inbox = Inbox::new();
    let mut host = host::Host::new(facts.seed, facts.now, facts.wall);
    host.start(path.as_bytes(), secret);
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], draw: |_, _, _, _, _| false, components: &[], elements: &[] },
        &host.bounds.tree,
        b"temper",
        facts.primary,
    )));
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut shell = Shell { host, page, dom };
    inbox.start(Box::new(move |inbox| shell.turn(inbox)));
    inbox.raise(Report::Start);
}
impl Shell {
    fn turn(&mut self, inbox: &Inbox<Report>) {
        let (now, wall) = platform::clocks();
        self.host.clocks(now, wall);
        while let Some(report) = inbox.next() {
            match report {
                Report::Start => {}
                Report::Act(node, act) => self.host.act(node, act),
            }
            self.host.requests();
        }
        let mut patches = Queue::with_capacity(self.host.bounds.tree.patches);
        self.host.build(&mut patches);
        let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
        let mut ops = Vec::new();
        self.page.borrow_mut().apply(&patches, &mut ops);
        self.dom.apply(&ops);
    }
}
