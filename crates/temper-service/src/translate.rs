//! Stateless application parts between the root and its channel (channel.md, 7).
use crate::settings;
use skein_lib::{List, Secret, Writer};
use temper_engine_domain as engine;
use temper_protocol_worker_wire::{self as wire, v1};

/// Application limits from the engine's forge and checkout bounds.
#[must_use]
pub fn limits(own: &crate::limits::Limits) -> Option<v1::Limits> {
    wire::limits(&wire::Bounds {
        repositories: own.checkout.repositories.min(own.forge.resources_per_task),
        name_bytes: own.checkout.name_bytes.min(own.forge.name_bytes),
        remote_bytes: own.checkout.name_bytes,
        conflicts: own.checkout.conflicts,
        path_bytes: own.checkout.path_bytes,
        token_bytes: v1::CEILINGS.identity_token,
        text_bytes: own.checkout.name_bytes.min(v1::CEILINGS.identity_name),
    })
}

/// A workspace's remotes, made from each configured forge's git base.
pub fn items(
    assignment: &engine::Assignment,
    forges: &[settings::Forge],
    limits: &v1::Limits,
) -> Result<Option<v1::Items>, v1::Problem> {
    if assignment.workspace.repositories.is_empty() {
        return Ok(None);
    }
    let mut repositories = List::with_capacity(limits.items_repositories);
    for repository in &assignment.workspace.repositories {
        let value = repository_item(repository, forges, limits)?;
        repositories.push(value).or(Err(problem(v1::Path::ItemsRepositories)))?;
    }
    Ok(Some(v1::Items::new(limits, v1::ItemsParts { repositories, save: None })?))
}
fn repository_item(
    repository: &engine::ForgeRepository,
    forges: &[settings::Forge],
    limits: &v1::Limits,
) -> Result<v1::Repository, v1::Problem> {
    let mut base = None;
    for forge in forges {
        if forge.number == repository.provider.forge {
            base = Some(forge.git.as_ref());
            break;
        }
    }
    let base = base.ok_or(problem(v1::Path::RepositoryRemote))?;
    let length = base.len().checked_add(repository.owner.len()).ok_or(problem(v1::Path::RepositoryRemote))?;
    let length = length.checked_add(repository.name.len()).ok_or(problem(v1::Path::RepositoryRemote))?;
    let length = length.checked_add(6).ok_or(problem(v1::Path::RepositoryRemote))?;
    if length > usize::try_from(limits.repository_remote).expect("u32 fits usize") {
        return Err(problem(v1::Path::RepositoryRemote));
    }
    let mut remote = Writer::new(length);
    for segment in [base, b"/", &repository.owner, b"/", &repository.name, b".git"] {
        remote.put(segment).expect("measured remote");
    }
    let access = match &repository.push {
        Some(push) => {
            v1::Access::Writable(v1::Writable::new(limits, v1::WritableParts { push: push.clone(), expected: None })?)
        }
        None => v1::Access::ReadOnly,
    };
    v1::Repository::new(
        limits,
        v1::RepositoryParts {
            tag: repository.tag,
            name: repository.name.clone(),
            remote: remote.finish(),
            start: start(&repository.start, limits)?,
            access,
            identity: repository.identity,
        },
    )
}
fn start(start: &engine::ForgeStart, limits: &v1::Limits) -> Result<v1::Start, v1::Problem> {
    Ok(match start {
        engine::ForgeStart::Base(branch) => {
            v1::Start::Base(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Branch(branch) => {
            v1::Start::Branch(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Saved(branch) => {
            v1::Start::Saved(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Merge { branch, base } => {
            v1::Start::Merge(v1::Merge::new(limits, v1::MergeParts { branch: branch.clone(), base: *base })?)
        }
    })
}
fn problem(path: v1::Path) -> v1::Problem {
    v1::Problem { path, reason: skein_codec::Reason::Bound }
}

/// Each decoded optional part reaches its matching root field.
#[must_use]
pub fn parts(left: Option<v1::Left>, saved: Option<v1::Saved>, preparation: Option<v1::Preparation>) -> engine::Parts {
    let mut pushed = List::with_capacity(match &left {
        Some(left) => left.landed().len(),
        None => 0,
    });
    if let Some(left) = left {
        for head in left.into_parts().landed.into_boxed() {
            let head = head.into_parts();
            pushed
                .push(temper_engine_domain_forge::Pushed { tag: head.tag, commit: head.commit })
                .expect("bounded heads");
        }
    }
    let saved = match saved {
        Some(saved) => {
            let saved = saved.into_parts();
            let mut tags = List::with_capacity(saved.repositories.len());
            for repository in saved.repositories.into_boxed() {
                let repository = repository.into_parts();
                let kept = match repository.landing {
                    v1::Landing::Landed(_) => true,
                    v1::Landing::Conflicted(_)
                    | v1::Landing::Explained(_)
                    | v1::Landing::Moved
                    | v1::Landing::Failed
                    | v1::Landing::Refused
                    | v1::Landing::Unchanged => repository.from_saved,
                };
                if kept {
                    tags.push(repository.tag).expect("bounded tags");
                }
            }
            Some(tags.into_boxed())
        }
        None => None,
    };
    let preparation = match preparation {
        Some(v1::Preparation::Missing(missed)) => {
            let missed = missed.into_parts();
            let missing = match missed.missing {
                v1::Missing::Repository => engine::Missing::Repository,
                v1::Missing::Branch => engine::Missing::Branch,
                v1::Missing::Commit => engine::Missing::Commit,
            };
            Some(engine::Preparation::Missing { tag: missed.tag, missing })
        }
        Some(v1::Preparation::Refused(tag)) => Some(engine::Preparation::Refused { tag: tag.tag() }),
        Some(v1::Preparation::Transient) | None => None,
    };
    engine::Parts { pushed: pushed.into_boxed(), saved, preparation }
}

/// Encode one configured identity as the single secret the table holds.
pub fn identity(identity: &settings::Identity, limits: &v1::Limits) -> Result<Secret, v1::Problem> {
    let value = v1::Identity::new(
        limits,
        v1::IdentityParts {
            token: identity.token.clone(),
            login: identity.login.clone(),
            name: identity.author.name.clone(),
            email: identity.author.email.clone(),
        },
    )?;
    Ok(wire::identity(&value))
}
