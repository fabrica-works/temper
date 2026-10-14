//! Forgejo Actions identities, bounded by the retained CI statuses.
use crate::api::{Answer, Error, JobRef, What};
use crate::domain::Domain;
use crate::store::Repository;
use alloc::boxed::Box;
use skein_lib::{Id, List};

/// One CI run per repository and commit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Run {
    pub id: u64,
    pub number: u64,
    pub commit: u64,
}
/// One job per configured context, in its context's position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Job {
    pub id: u64,
    pub run: u64,
    pub index: u32,
    pub attempt: u32,
}
#[derive(Debug)]
pub(crate) struct Kept {
    pub(crate) run: Run,
    pub(crate) jobs: List<Job>,
}

pub(crate) fn start(domain: &mut Domain, id: Id<Repository>, commit: u64) {
    let repository = domain.repositories.get_mut(id).expect("a CI repository");
    domain.runs = domain.runs.checked_add(1).expect("run ids fit");
    repository.run_number = repository.run_number.checked_add(1).expect("run numbers fit");
    let run = Run { id: domain.runs, number: repository.run_number, commit };
    let capacity = u32::try_from(repository.checks.contexts.len()).expect("bounded contexts");
    let mut jobs = List::with_capacity(capacity);
    for index in 0..capacity {
        domain.jobs = domain.jobs.checked_add(1).expect("job ids fit");
        jobs.push(Job { id: domain.jobs, run: run.id, index, attempt: 1 }).expect("one job per context");
    }
    repository.runs.insert(commit, Kept { run, jobs }).expect("room for each retained CI commit");
}

pub(crate) fn rerun(domain: &mut Domain, id: Id<Repository>, commit: u64, context: u32) {
    let repository = domain.repositories.get_mut(id).expect("a CI repository");
    let run = repository.runs.get_mut(&commit).expect("a CI run");
    let job = run.jobs.get_mut(context).expect("a CI job");
    job.attempt = 2;
}

pub(crate) fn runs(repository: &Repository, commit: u64, number: u64) -> Answer {
    let mut runs = List::with_capacity(1);
    if let Some(kept) = repository.runs.get(&commit)
        && kept.run.number == number
    {
        runs.push(kept.run).expect("one matching run");
    }
    Answer::Runs(runs.into_boxed())
}

pub(crate) fn run(repository: &Repository, id: u64) -> Result<&Kept, Error> {
    for (_, run) in &repository.runs {
        if run.run.id == id {
            return Ok(run);
        }
    }
    Err(Error::Missing(What::Job))
}

pub(crate) fn jobs(repository: &Repository, id: u64) -> Result<Answer, Error> {
    Ok(Answer::Jobs(copy_of_jobs(&run(repository, id)?.jobs)))
}
fn copy_of_jobs(jobs: &List<Job>) -> Box<[Job]> {
    let mut answer = List::with_capacity(jobs.len());
    for job in jobs {
        answer.push(*job).expect("one entry per job");
    }
    answer.into_boxed()
}

pub(crate) fn job(repository: &Repository, id: u64, attempt: u32) -> Result<(u64, &[u8]), Error> {
    for (&commit, kept) in &repository.runs {
        for job in &kept.jobs {
            if job.id == id && job.attempt == attempt {
                let index = usize::try_from(job.index).expect("a bounded context");
                let name = repository.checks.contexts.get(index).expect("a configured context");
                return Ok((commit, name));
            }
        }
    }
    Err(Error::Missing(What::Job))
}

pub(crate) fn reference(repository: &Repository, commit: u64, context: &[u8]) -> Option<JobRef> {
    let run = repository.runs.get(&commit)?;
    for job in &run.jobs {
        let index = usize::try_from(job.index).ok()?;
        if *repository.checks.contexts.get(index)?.as_ref() == *context {
            return Some(JobRef { run: run.run.id, job: job.id, attempt: job.attempt });
        }
    }
    None
}

pub(crate) fn link(repository: &Repository, commit: u64, context: &[u8]) -> Box<[u8]> {
    let Some(reference) = reference(repository, commit, context) else { return Box::new([]) };
    let kept = repository.runs.get(&commit).expect("the reference's run");
    let mut link = List::with_capacity(
        u32::try_from(repository.name.len()).expect("bounded name").checked_add(51).expect("bounded link"),
    );
    append(&mut link, b"/");
    append(&mut link, &repository.name);
    append(&mut link, b"/actions/runs/");
    decimal(&mut link, kept.run.number);
    append(&mut link, b"/jobs/");
    for job in &kept.jobs {
        if job.id == reference.job {
            decimal(&mut link, u64::from(job.index));
        }
    }
    link.into_boxed()
}
fn append(out: &mut List<u8>, bytes: &[u8]) {
    for &byte in bytes {
        out.push(byte).expect("link capacity");
    }
}
fn decimal(out: &mut List<u8>, number: u64) {
    append(out, skein_lib::Decimal::of(number).as_bytes());
}
