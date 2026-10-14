//! The native client's terminal face; ordinary transport notices remain local.
use super::{Client, Notice};
use crate::{
    client::{NativeClient, Shown},
    route,
};
pub(crate) fn event(event: &temper_client_domain::Event) -> Option<Notice> {
    use jig_domain_client::{
        Event as Jig,
        contract::{ReadNone, ReadResult, WatchEnd},
    };
    use temper_client_domain::Event;
    if matches!(
        event,
        Event::SignedOut
            | Event::Answer { envelope: skein_lib::RequestEnvelope::SignedOut, .. }
            | Event::Jig(
                Jig::Ended { end: WatchEnd::SignedOut, .. }
                    | Jig::Read { result: ReadResult::None(ReadNone::SignedOut), .. }
            )
    ) {
        Some(Notice::SignedOut)
    } else {
        None
    }
}
pub(crate) fn signing(signing: &jig_shell_native::Signing, out: bool) -> Option<Notice> {
    use jig_shell_native::Signing;
    match signing {
        Signing::Again { .. } | Signing::Lost { .. } if !out => Some(Notice::SignInAgain),
        Signing::Refused { .. } if !out => Some(Notice::SignInRefused),
        Signing::SignedIn { .. }
        | Signing::SignedOut { .. }
        | Signing::Again { .. }
        | Signing::Lost { .. }
        | Signing::Refused { .. } => None,
    }
}
impl Client for NativeClient {
    fn tree(&self) -> &jig_view::Tree {
        Self::tree(self)
    }
    fn built(&self) -> u64 {
        self.seen().builds
    }
    fn act(&mut self, node: jig_view::NodeId, act: jig_view::Act) -> bool {
        Self::act(self, node, act)
    }
    fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_shell_native::Unlinked> {
        Self::sign_in(self, link)
    }
    fn notice(&mut self) -> Option<Notice> {
        while self.signing().is_some() {}
        while let Some(shown) = self.notices() {
            match shown {
                Shown::Terminal(notice) => return Some(notice),
                Shown::SignInAsked => return Some(Notice::SignedOut),
                Shown::Notice(route::Notice::Credential { party, credential }) => {
                    return Some(Notice::Credential { party, token: credential });
                }
                Shown::Notice(
                    route::Notice::Dropped { .. }
                    | route::Notice::Unread { .. }
                    | route::Notice::Served { .. }
                    | route::Notice::Unasked
                    | route::Notice::Fact(_),
                ) => {}
            }
        }
        None
    }
    fn close(&mut self) {
        Self::close(self);
    }
}
