//! Application landing rules use jig's pattern and project vocabulary.
use super::{Problems, What, limits, read};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::Document;
use skein_lib::{List, Map};
use temper_engine_domain::{self as engine, Approval, Freshness, Gate, LandingRule};

fn freshness(
    object: &mut config::read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Option<Freshness> {
    let value = read::text(object, document, b"freshness", path, problems)?;
    match value.as_ref() {
        b"exact" => Some(Freshness::Exact),
        b"clean" => Some(Freshness::Clean),
        _ => {
            read::form(false, path.member(b"freshness"), problems);
            None
        }
    }
}
fn pattern(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &jig_domain_authority::Limits,
    problems: &mut Problems,
) -> Option<jig_domain_authority::Pattern> {
    let mut object = read::object(document, at, path, problems)?;
    let ats = read::list(&mut object, document, b"segments", limits.segments, path, problems);
    let last = read::text(&mut object, document, b"last", path, problems);
    let value = read::text(&mut object, document, b"value", path, problems);
    read::finish(object, document, path, problems);
    let mut segments = List::with_capacity(limits.segments);
    if let Some(ats) = ats {
        for (index, at) in ats.iter().enumerate() {
            match config::read::text(document, *at, limits.segment_bytes) {
                Ok(segment) => segments.push(segment).expect("bounded segments"),
                Err(what) => problems.push(read::index(&path.member(b"segments"), index), What::Jig(what)),
            }
        }
    }
    let value = value?;
    if value.len() > usize::try_from(limits.segment_bytes).expect("segment limit") {
        problems.push(path.member(b"value"), What::Jig(config::What::Bound));
    }
    let last = match last?.as_ref() {
        b"exact" => jig_domain_authority::Last::Exact(value),
        b"open" => jig_domain_authority::Last::Open(value),
        _ => {
            read::form(false, path.member(b"last"), problems);
            return None;
        }
    };
    Some(jig_domain_authority::Pattern { segments: segments.into_boxed(), last })
}
fn rule(
    document: &Document,
    at: u32,
    path: &Path,
    jig: &config::Engine,
    limits: &limits::Limits,
    problems: &mut Problems,
) -> Option<LandingRule> {
    let mut object = read::object(document, at, path, problems)?;
    let connector = read::short(&mut object, document, b"connector", path, problems);
    let kind = read::short(&mut object, document, b"kind", path, problems);
    let pattern_at = read::required(&mut object, document, b"pattern", path, problems);
    let enforce = read::boolean(&mut object, document, b"enforce", path, problems);
    let ci = read::boolean(&mut object, document, b"ci", path, problems);
    let up_to_date = read::boolean(&mut object, document, b"up_to_date", path, problems);
    let gate_ats = read::list(&mut object, document, b"gates", limits.forge.change_policy.gates, path, problems);
    let approval_ats = read::list(&mut object, document, b"approvals", jig.limits.core.authority.roles, path, problems);
    read::finish(object, document, path, problems);
    if let Some(connector) = connector
        && !jig.connectors.contains(&connector)
    {
        problems.push(path.member(b"connector"), What::Jig(config::What::Refers));
    }
    if let Some(kind) = kind {
        read::form(kind == 4, path.member(b"kind"), problems);
    }
    let pattern = match pattern_at {
        Some(at) => pattern(document, at, &path.member(b"pattern"), &jig.limits.core.authority, problems),
        None => None,
    };
    let mut gates = List::with_capacity(limits.forge.change_policy.gates);
    if let Some(ats) = gate_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"gates"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let number = read::number(&mut object, document, b"number", &path, problems);
            let blocking = read::boolean(&mut object, document, b"blocking", &path, problems);
            let freshness = freshness(&mut object, document, &path, problems);
            read::finish(object, document, &path, problems);
            if let Some(number) = number
                && let Some(blocking) = blocking
                && let Some(freshness) = freshness
            {
                gates.push(Gate { number, blocking, freshness }).expect("bounded gates");
            }
        }
    }
    let mut approvals = List::with_capacity(jig.limits.core.authority.roles);
    if let Some(ats) = approval_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"approvals"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let role = read::number(&mut object, document, b"role", &path, problems);
            let people = read::number(&mut object, document, b"people", &path, problems);
            let freshness = freshness(&mut object, document, &path, problems);
            read::finish(object, document, &path, problems);
            if let Some(role) = role
                && let Some(people) = people
                && let Some(freshness) = freshness
            {
                approvals.push(Approval { role, people, freshness }).expect("bounded approvals");
            }
        }
    }
    Some(LandingRule {
        connector: connector?,
        kind: kind?,
        pattern: pattern?,
        enforce: enforce?,
        ci: ci?,
        up_to_date: up_to_date?,
        gates: gates.into_boxed(),
        approvals: approvals.into_boxed(),
    })
}
fn rules(
    document: &Document,
    ats: List<u32>,
    path: &Path,
    jig: &config::Engine,
    limits: &limits::Limits,
    project: Option<u32>,
    problems: &mut Problems,
) -> Box<[LandingRule]> {
    let mut rules = List::with_capacity(ats.len());
    for (index, at) in ats.iter().enumerate() {
        let path = read::index(path, index);
        if let Some(rule) = rule(document, *at, &path, jig, limits, problems) {
            for (index, approval) in rule.approvals.iter().enumerate() {
                for configured in &jig.projects {
                    if let Some(project) = project
                        && configured.number != project
                    {
                        continue;
                    }
                    let mut found = false;
                    for role in &configured.policy.roles {
                        if role.number == approval.role {
                            found = true;
                        }
                    }
                    if !found {
                        problems.push(
                            read::index(&path.member(b"approvals"), index).member(b"role"),
                            What::Jig(config::What::Refers),
                        );
                    }
                }
            }
            rules.push(rule).expect("bounded rules");
        }
    }
    rules.into_boxed()
}
pub(super) fn parse(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    limits: &limits::Limits,
    problems: &mut Problems,
) -> Option<engine::LandingPolicy> {
    let path = Path::root().member(b"application").member(b"landing");
    let mut object = read::object(document, at, &path, problems)?;
    let deployment_ats =
        read::list(&mut object, document, b"deployment", jig.limits.core.authority.requirements, &path, problems);
    let project_ats =
        read::list(&mut object, document, b"projects", jig.limits.core.authority.projects, &path, problems);
    read::finish(object, document, &path, problems);
    let deployment = rules(document, deployment_ats?, &path.member(b"deployment"), jig, limits, None, problems);
    let mut projects = Map::with_capacity(jig.limits.core.authority.projects);
    if let Some(ats) = project_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"projects"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let project = read::number(&mut object, document, b"project", &path, problems);
            let ats =
                read::list(&mut object, document, b"rules", jig.limits.core.authority.requirements, &path, problems);
            read::finish(object, document, &path, problems);
            if let Some(project) = project
                && let Some(ats) = ats
            {
                let mut found = false;
                for configured in &jig.projects {
                    if configured.number == project {
                        found = true;
                    }
                }
                if !found {
                    problems.push(path.member(b"project"), What::Jig(config::What::Refers));
                }
                if projects.contains_key(&project) {
                    problems.push(path.member(b"project"), What::Jig(config::What::Unique));
                }
                let rules = rules(document, ats, &path.member(b"rules"), jig, limits, Some(project), problems);
                projects.insert(project, rules).expect("bounded projects");
            }
        }
    }
    Some(engine::LandingPolicy { deployment, projects })
}
