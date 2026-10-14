//! Outside branch moves and completed shell writes, independent of either root.
use crate::{
    fixtures,
    world::{Proc, Pushed, Story},
};
use std::collections::BTreeMap;
use temper_fake_forge_domain::{Observation, api};

pub(crate) struct Forge {
    transport: temper_fake_git::Transport,
    tree: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) pushed: Vec<Pushed>,
    merge: bool,
    edited: bool,
    prepared: bool,
}
impl Forge {
    pub(crate) fn new(transport: temper_fake_git::Transport, story: Story) -> Self {
        let content: &[u8] = if matches!(story, Story::Merge) { b"topic\n" } else { b"base\n" };
        Self {
            transport,
            tree: BTreeMap::from([(b"conflict.txt".to_vec(), content.to_vec())]),
            pushed: Vec::new(),
            merge: matches!(story, Story::Merge),
            edited: false,
            prepared: false,
        }
    }
    pub(crate) fn adopted(&mut self) {
        if self.merge && !self.prepared {
            crate::stories::prepare_merge_branch(&self.transport);
            self.prepared = true;
        }
    }
    pub(crate) fn observe(&mut self, procs: &[Proc]) {
        for proc in procs {
            if let Proc::Shell(shell) = proc
                && !shell.written.is_empty()
            {
                self.tree.insert(shell.target.clone(), shell.written.clone());
                self.edited = true;
            }
        }
        let mut forge = self.transport.forge.borrow_mut();
        while let Some(observed) = forge.domain.pop_observation() {
            if let Observation::Merged { repository, head, .. } = &observed {
                green(&forge, repository, *head);
            }
            let Observation::Moved { repository, branch, from, to, by: 1 } = observed else { continue };
            let metadata = forge.domain.metadata(&repository).expect("observed repository");
            if branch == metadata.default_branch {
                green(&forge, &repository, to);
            }
            // Git may create the remote branch at the editor's first committed change.
            // Earlier preparation has no completed agent write.
            if !self.edited || from == Some(to) || branch == metadata.default_branch {
                continue;
            }
            let object = forge.domain.object(to).expect("observed pushed object");
            let tree: BTreeMap<_, _> =
                object.tree.iter().map(|(path, bytes)| (path.to_vec(), bytes.to_vec())).collect();
            assert_eq!(tree, self.tree, "pushed tree is the completed agent edits");
            let parents = object.parent.into_iter().chain(object.merge_parent).collect();
            self.pushed.push(Pushed { branch, commit: to, parents });
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.pushed.iter().any(|push| !self.merge || push.parents == [fixtures::BRANCH_HEAD, fixtures::MOVED_BASE])
    }
}
fn green(forge: &temper_fake_git::Forge, repository: &[u8], head: u64) {
    let api::Answer::Checks(checks) = forge
        .domain
        .inspect(&forge.config, repository, &api::Read::Checks { commit: head })
        .expect("landing CI statuses")
    else {
        panic!("CI status answer")
    };
    assert!(
        !checks.is_empty() && checks.iter().all(|check| check.status.state == api::Check::Passed),
        "nothing lands on a default branch without green CI"
    );
}
