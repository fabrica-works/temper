//! Git reaches the same forge domain the HTTP service owns in the world.
use crate::check::Credential;
use skein_lib::{Env, Queue, ReplyTo, Secret, Time, Token, Wall};
use std::{cell::RefCell, rc::Rc};
use temper_fake_checkout::git::{Created, Fault, Pushed, Remote, Tree, Want, What};
use temper_fake_forge_domain::{self as domain, api};

/// The world's one forge, shared by HTTP and every hosted git invocation.
#[derive(Debug)]
pub struct Forge {
    pub domain: domain::Domain,
    pub config: domain::Config,
    pub now: Time,
    /// Other replies and hooks remain the HTTP world's to route.
    pub pending: Vec<domain::Request>,
    serial: u64,
}
impl Forge {
    #[must_use]
    pub fn new(domain: domain::Domain, config: domain::Config) -> Self {
        Self { domain, config, now: Time::ZERO, pending: Vec::new(), serial: 0 }
    }
}
/// The known tokens and logins stay solely on this side of the boundary.
#[derive(Debug, Clone)]
pub struct Transport {
    pub forge: Rc<RefCell<Forge>>,
    tokens: Rc<Vec<(Secret, u64)>>,
    users: Rc<Vec<(u64, Box<[u8]>)>>,
    user: Option<u64>,
}
impl Transport {
    /// Copy the protocol scenario's identity tables before it takes ownership.
    #[must_use]
    pub fn new(forge: Rc<RefCell<Forge>>, table: &temper_fake_forge_protocol::Config) -> Self {
        Self {
            forge,
            tokens: Rc::new(table.tokens.iter().cloned().collect()),
            users: Rc::new(table.users.iter().cloned().collect()),
            user: None,
        }
    }
    pub fn authenticate(&mut self, credential: Option<&Credential>) {
        self.user = credential.and_then(|(login, token)| {
            self.tokens.iter().find_map(|(known, user)| {
                (known == token && self.users.iter().any(|(id, name)| id == user && name == login)).then_some(*user)
            })
        });
    }
    #[must_use]
    pub const fn authenticated(&self) -> bool {
        self.user.is_some()
    }
    pub fn commit_tree(&mut self, parent: u64, merging: Option<u64>, tree: Tree, message: &[u8]) -> u64 {
        let mut forge = self.forge.borrow_mut();
        let config = forge.config;
        let files =
            tree.into_iter().map(|(path, content)| api::File { path: path.into(), content: content.into() }).collect();
        domain::commit_tree(&mut forge.domain, &config, parent, merging, files, message).expect("scenario commit room")
    }
    fn call(&mut self, remote: &[u8], operation: api::Git) -> Result<api::Answer, Fault> {
        let user = self.user.ok_or(Fault::Refused)?;
        let remote = repository(remote);
        let mut forge = self.forge.borrow_mut();
        forge.serial += 1;
        let token = Token::new((7 << 60) + forge.serial);
        let env = Env { now: forge.now, wall: Wall::EPOCH, limits: forge.config };
        let mut out = Queue::with_capacity(domain::MAX_OUT);
        domain::step(
            &mut forge.domain,
            &env,
            domain::Event::Call {
                reply_to: ReplyTo::new(token),
                user,
                repository: remote.into(),
                op: api::Op::Git(operation),
            },
            &mut out,
        );
        domain::fire(&mut forge.domain, &env, &mut out);
        forge.domain.reclaim();
        let mut terminal = None;
        while let Some(request) = out.pop() {
            match request {
                domain::Request::Reply { to, result } => {
                    let replied = to.into_token();
                    if replied == token {
                        terminal = Some(result.map_err(fault));
                    } else {
                        forge.pending.push(domain::Request::Reply { to: ReplyTo::new(replied), result });
                    }
                }
                request @ domain::Request::Hook { .. } => forge.pending.push(request),
            }
        }
        terminal.expect("the world's git transport uses an immediate forge domain")
    }
    #[must_use]
    pub fn branch(&self, remote: &[u8], branch: &[u8]) -> Option<u64> {
        let forge = self.forge.borrow();
        match forge.domain.inspect(&forge.config, repository(remote), &api::Read::Branch { branch: branch.into() }) {
            Ok(api::Answer::Commit(commit)) => Some(commit),
            Ok(_) | Err(_) => None,
        }
    }
    #[must_use]
    pub fn is_ancestor(&self, ancestor: u64, commit: u64) -> bool {
        self.forge.borrow().domain.is_ancestor(ancestor, commit)
    }
}
impl Remote for Transport {
    fn heads(&mut self, remote: &[u8]) -> Result<Vec<u64>, Fault> {
        let api::Answer::Cloned { branches, .. } = self.call(remote, api::Git::Clone)? else {
            panic!("clone answer");
        };
        Ok(branches.iter().map(|head| head.commit).collect())
    }
    fn fetch(&mut self, remote: &[u8], want: Want<'_>) -> Result<u64, Fault> {
        let want = match want {
            Want::Branch(branch) => api::Want::Branch(branch.into()),
            Want::Commit(commit) => api::Want::Commit(commit),
            Want::Default => api::Want::Default,
        };
        let api::Answer::Commit(commit) = self.call(remote, api::Git::Fetch { want })? else {
            panic!("fetch answer");
        };
        Ok(commit)
    }
    fn create(&mut self, remote: &[u8], branch: &[u8], commit: u64) -> Result<Created, Fault> {
        let api::Answer::Branch(created) = self.call(remote, api::Git::Create { branch: branch.into(), commit })?
        else {
            panic!("create answer");
        };
        Ok(match created {
            api::Created::Created => Created::Created,
            api::Created::Exists => Created::Exists,
        })
    }
    fn push(&mut self, remote: &[u8], branch: &[u8], commit: u64, expected: Option<u64>) -> Result<Pushed, Fault> {
        let api::Answer::Pushed(pushed) =
            self.call(remote, api::Git::Push { branch: branch.into(), commit, expected })?
        else {
            panic!("push answer");
        };
        Ok(match pushed {
            api::Pushed::Pushed => Pushed::Pushed,
            api::Pushed::Rejected => Pushed::Rejected,
        })
    }
    fn parent(&self, commit: u64) -> Option<u64> {
        self.forge.borrow().domain.object(commit).expect("stored commit").parent
    }
    fn merge_parent(&self, commit: u64) -> Option<u64> {
        self.forge.borrow().domain.object(commit).expect("stored commit").merge_parent
    }
    fn tree(&self, commit: u64) -> Tree {
        self.forge
            .borrow()
            .domain
            .object(commit)
            .expect("stored commit")
            .tree
            .iter()
            .map(|(path, content)| (path.to_vec(), content.to_vec()))
            .collect()
    }
    fn message(&self, commit: u64) -> Vec<u8> {
        self.forge.borrow().domain.object(commit).expect("stored commit").message.to_vec()
    }
    fn store(&mut self, parent: u64, merging: Option<u64>, tree: Tree, message: &[u8]) -> Option<u64> {
        let mut forge = self.forge.borrow_mut();
        let config = forge.config;
        let files =
            tree.into_iter().map(|(path, content)| api::File { path: path.into(), content: content.into() }).collect();
        match merging {
            Some(second) => Some(
                domain::merge_commit(&mut forge.domain, &config, parent, second, files, message)
                    .expect("scenario commit room"),
            ),
            None => domain::commit(&mut forge.domain, &config, parent, files, message).expect("scenario commit room"),
        }
    }
}
fn repository(url: &[u8]) -> &[u8] {
    let rest = url.strip_prefix(b"https://").or_else(|| url.strip_prefix(b"http://")).expect("admitted URL");
    let slash = rest.iter().position(|b| *b == b'/').expect("repository URL path");
    rest[slash + 1..].strip_suffix(b".git").unwrap_or(&rest[slash + 1..])
}
fn fault(error: api::Error) -> Fault {
    match error {
        api::Error::Missing(api::What::Repository) => Fault::Missing(What::Repository),
        api::Error::Missing(api::What::Branch) => Fault::Missing(What::Branch),
        api::Error::Missing(api::What::Commit) => Fault::Missing(What::Commit),
        api::Error::Refused | api::Error::Forbidden | api::Error::Protected => Fault::Refused,
        api::Error::Unreachable | api::Error::Unavailable | api::Error::Timeout | api::Error::RateLimited { .. } => {
            Fault::Unreachable
        }
        api::Error::Missing(
            api::What::Item
            | api::What::Pull
            | api::What::Comment
            | api::What::Label
            | api::What::File
            | api::What::Page
            | api::What::Review
            | api::What::Job,
        )
        | api::Error::TooLarge
        | api::Error::Full
        | api::Error::Exists
        | api::Error::Circular
        | api::Error::NothingToMerge
        | api::Error::Empty
        | api::Error::Closed
        | api::Error::Stale
        | api::Error::Conflict => panic!("unexpected git fault: {error:?}"),
    }
}
