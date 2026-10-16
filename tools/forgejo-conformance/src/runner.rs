use crate::{
    Result,
    facts::{self, Fact, Outcome},
    fixture::{Fixture, R},
    mask::{self, Mask},
    write::{self, SecretKind, Secrets},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command},
    time::Duration,
};
use temper_throwaway_forgejo::{Exchange, Range};
const NAMES: [&str; 13] = [
    "actions-report-each-job-as-a-status-on-its-head",
    "actions-status-link-is-runs-n-jobs-i",
    "actions-status-link-names-no-attempt",
    "run-number-is-index-in-repo",
    "run-jobs-come-in-index-order",
    "job-log-range-answers-206",
    "job-steps-number-from-0-set-up-first-complete-last",
    "failed-workflow-step-and-completion-step-are-failure",
    "step-log-holds-that-steps-lines-alone",
    "step-log-suffix-range-answers-206-with-its-last-bytes",
    "step-log-suffix-content-range-names-its-first-byte",
    "empty-log-suffix-range-answers-206-empty",
    "step-past-the-last-is-404",
];
struct Runner {
    child: Option<Child>,
}
impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _kill = child.kill();
            let _wait = child.wait();
        }
    }
}
fn command(binary: &Path, scratch: &Path) -> Command {
    let mut cmd = Command::new(binary);
    cmd.env_clear().env("HOME", scratch.join("home")).env("PATH", "/usr/bin:/bin").current_dir(scratch);
    cmd
}
pub fn run(f: &Fixture, out: &Path, mask: &mut Mask, secrets: &mut Secrets) -> Result<(String, Vec<Fact>)> {
    run_with(f, Some(out), mask, secrets)
}
pub fn component(f: &Fixture, secrets: &mut Secrets) -> Result<()> {
    run_with(f, None, &mut Mask::new(f.instance.port()), secrets)?;
    Ok(())
}
fn run_with(f: &Fixture, out: Option<&Path>, mask: &mut Mask, secrets: &mut Secrets) -> Result<(String, Vec<Fact>)> {
    let Some(binary) = std::env::var_os("TEMPER_FORGEJO_RUNNER") else {
        return Ok((
            "none".into(),
            NAMES
                .into_iter()
                .map(|name| Fact { name, outcome: Outcome::Unchecked("TEMPER_FORGEJO_RUNNER not set".into()) })
                .collect(),
        ));
    };
    let binary = PathBuf::from(binary);
    let (_runner, version) = start(f, &binary, secrets)?;
    f.api("PATCH", R, &json!({"has_actions":true}))?;
    f.branch(R, "actions", "landing")?;
    f.put(R,"actions",".forgejo/workflows/conformance.yaml","name: conformance\non: push\njobs:\n  pass:\n    runs-on: temper-host\n    steps:\n      - run: exit 0\n  fail:\n    runs-on: temper-host\n    steps:\n      - run: |\n          printf 'TEMPER_CONFORMANCE_FAILED_JOB\\n'\n          exit 1\n")?;
    let head =
        f.get(&format!("{R}/branches/actions"))?["commit"]["id"].as_str().ok_or("actions head missing")?.to_owned();
    let target = format!("{R}/commits/{head}/status?page=1&limit=2");
    let mut settled = None;
    for _ in 0..120 {
        let row = f.get(&target)?;
        if row["statuses"]
            .as_array()
            .is_some_and(|s| s.len() == 2 && s.iter().all(|s| s["status"] == "success" || s["status"] == "failure"))
        {
            settled = Some(row);
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    let combined = settled.ok_or("Actions jobs did not settle in 60 seconds")?;
    capture(f, out, mask, secrets, "actions-combined-status", &target, None)?;
    let statuses = combined["statuses"].as_array().ok_or("statuses missing")?;
    let link = statuses[0]["target_url"].as_str().ok_or("status link missing")?;
    let number =
        link.split("/actions/runs/").nth(1).and_then(|s| s.split('/').next()).ok_or("status link run missing")?;
    let runs = capture(
        f,
        out,
        mask,
        secrets,
        "actions-runs",
        &format!("{R}/actions/runs?head_sha={head}&run_number={number}&page=1&limit=1"),
        None,
    )?;
    let run = rows(&runs, "workflow_runs").first().ok_or("Actions run missing")?;
    let id = run["id"].as_u64().ok_or("run id missing")?;
    capture(f, out, mask, secrets, "actions-run", &format!("{R}/actions/runs/{id}"), None)?;
    let jobs = capture(f, out, mask, secrets, "actions-jobs", &format!("{R}/actions/runs/{id}/jobs"), None)?;
    let jobs = rows(&jobs, "jobs");
    let job = jobs
        .iter()
        .find(|j| j["name"] == "fail" || j["status"] == "failure" || j["conclusion"] == "failure")
        .ok_or("failed job missing")?;
    let job_id = job["id"].as_u64().ok_or("job id missing")?;
    let attempt = job["attempt"].as_u64().ok_or("job attempt missing")?;
    let log = format!("{R}/actions/jobs/{job_id}/logs?attempt={attempt}");
    let whole = exchange(f, out, mask, secrets, "actions-job-log", &log, None)?;
    let response =
        f.exchange("GET", &log, None, Some(&f.writer.token), Some(temper_throwaway_forgejo::Range::Head(3)))?;
    record(out, mask, secrets, "actions-job-log-range", &response)?;
    let mut facts = Vec::new();
    facts::add(
        &mut facts,
        NAMES[0],
        combined["sha"] == head
            && statuses.iter().all(|s| s["url"].as_str().is_some_and(|url| url.ends_with(&head)))
            && statuses.iter().any(|s| s["status"] == "success")
            && statuses.iter().any(|s| s["status"] == "failure"),
        "job statuses did not name the workflow head and both outcomes",
    );
    facts::add(&mut facts, NAMES[1], statuses.iter().all(|s| s["target_url"].as_str().is_some_and(valid_link)), link);
    facts::add(
        &mut facts,
        NAMES[2],
        statuses.iter().all(|s| s["target_url"].as_str().is_some_and(|s| !s.contains('?') && !s.contains("attempt"))),
        link,
    );
    facts::add(
        &mut facts,
        NAMES[3],
        run["run_number"].as_u64() == number.parse().ok() || run["index_in_repo"].as_u64() == number.parse().ok(),
        &run.to_string(),
    );
    facts::add(
        &mut facts,
        NAMES[4],
        jobs_in_index_order(statuses, jobs),
        &jobs.iter().map(|j| j["id"].to_string()).collect::<Vec<_>>().join(","),
    );
    facts::add(&mut facts, NAMES[5], response.status == 206, &format!("HTTP {}", response.status));
    facts.extend(job_steps(f, out, mask, secrets, jobs, &whole)?);
    Ok((version, facts))
}
fn jobs_in_index_order(statuses: &[Value], jobs: &[Value]) -> bool {
    jobs.len() == 2
        && statuses.iter().all(|s| {
            s["target_url"]
                .as_str()
                .and_then(|s| s.rsplit('/').next())
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|i| {
                    jobs.get(i).is_some_and(|job| {
                        s["context"] == format!("conformance / {} (push)", job["name"].as_str().unwrap_or_default())
                    })
                })
        })
}
fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value.as_array().or_else(|| value[key].as_array()).map_or(&[], Vec::as_slice)
}
fn valid_link(link: &str) -> bool {
    let Some(tail) = link.split("/actions/runs/").nth(1) else {
        return false;
    };
    let parts: Vec<_> = tail.split('/').collect();
    parts.len() == 3 && parts[0].parse::<u64>().is_ok() && parts[1] == "jobs" && parts[2].parse::<u64>().is_ok()
}
fn capture(
    f: &Fixture,
    out: Option<&Path>,
    mask: &mut Mask,
    secrets: &Secrets,
    name: &str,
    target: &str,
    range: Option<temper_throwaway_forgejo::Range>,
) -> Result<Value> {
    Ok(serde_json::from_slice(&exchange(f, out, mask, secrets, name, target, range)?.body).unwrap_or(Value::Null))
}
fn record(
    out: Option<&Path>,
    mask: &mut Mask,
    secrets: &Secrets,
    name: &str,
    response: &temper_throwaway_forgejo::Exchange,
) -> Result<()> {
    write::check(name, &response.sent, &response.body, secrets)?;
    let request = mask::request(mask, response);
    let response = mask::response(mask, response);
    if let Some(out) = out {
        write::case(out, name, &request, &response, secrets)?;
    }
    Ok(())
}
fn start(f: &Fixture, binary: &Path, secrets: &mut Secrets) -> Result<(Runner, String)> {
    let scratch = f.instance.scratch().join("runner");
    fs::create_dir_all(scratch.join("home"))?;
    let output = command(binary, &scratch).arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or_default().to_owned();
    if !output.status.success() || !temper_throwaway_forgejo::names_runner(&version) {
        return Err(format!("runner {}: {version}", binary.display()).into());
    }
    let registration = f.get(&format!("{R}/actions/runners/registration-token"))?["token"]
        .as_str()
        .ok_or("runner registration token missing")?
        .to_owned();
    secrets.add(SecretKind::RegistrationToken, &registration);
    let config = scratch.join("config.yaml");
    fs::write(
        &config,
        format!(
            "log:\n  level: info\nrunner:\n  file: {}/registration.json\n  capacity: 1\n  timeout: 1m\n  shutdown_timeout: 1s\n  fetch_interval: 100ms\n  report_interval: 100ms\n  labels: [temper-host:host]\ncache:\n  enabled: false\nhost:\n  workdir_parent: {}/jobs\n",
            scratch.display(),
            scratch.display()
        ),
    )?;
    let mut register = command(binary, &scratch);
    register.arg("--config").arg(&config).args(["register", "--no-interactive"]);
    register.args(["--instance", &f.instance.base(), "--token", &registration]);
    register.args(["--name", "temper-conformance", "--labels", "temper-host:host"]);
    if !register.output()?.status.success() {
        return Err("throwaway runner registration failed".into());
    }
    let log = File::create(scratch.join("runner.log"))?;
    let child = command(binary, &scratch)
        .arg("--config")
        .arg(&config)
        .arg("daemon")
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    Ok((Runner { child: Some(child) }, version))
}

fn exchange(
    f: &Fixture,
    out: Option<&Path>,
    mask: &mut Mask,
    secrets: &Secrets,
    name: &str,
    target: &str,
    range: Option<Range>,
) -> Result<Exchange> {
    let response = f.exchange("GET", target, None, Some(&f.writer.token), range)?;
    record(out, mask, secrets, name, &response)?;
    Ok(response)
}
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|part| part == needle)
}
fn content_range(exchange: &Exchange) -> Option<String> {
    String::from_utf8_lossy(&exchange.head)
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.eq_ignore_ascii_case("content-range"))
        .map(|(_, value)| value.trim().to_owned())
}
fn job_steps(
    f: &Fixture,
    out: Option<&Path>,
    mask: &mut Mask,
    secrets: &Secrets,
    jobs: &[Value],
    whole: &Exchange,
) -> Result<Vec<Fact>> {
    const SUFFIX: u64 = 10;
    let job = jobs.iter().find(|j| j["name"] == "fail").ok_or("failed job missing")?;
    let id = job["id"].as_u64().ok_or("job id missing")?;
    let attempt = job["attempt"].as_u64().ok_or("job attempt missing")?;
    let read = capture(f, out, mask, secrets, "actions-job", &format!("{R}/actions/jobs/{id}"), None)?;
    let steps = read["steps"].as_array().ok_or("job steps missing")?;
    let failed = steps
        .iter()
        .find(|step| step["status"] == "failure")
        .and_then(|s| s["number"].as_u64())
        .ok_or("failed step missing")?;
    let last = steps.iter().filter_map(|step| step["number"].as_u64()).max().ok_or("last step missing")?;
    let log = format!("{R}/actions/jobs/{id}/logs?attempt={attempt}&step=");
    let passing = jobs.iter().find(|j| j["name"] == "pass").ok_or("passing job missing")?;
    let targets = [
        ("actions-job-log-step", format!("{log}{failed}"), None),
        ("actions-job-log-step-suffix", format!("{log}{failed}"), Some(Range::Tail(SUFFIX))),
        ("actions-job-log-past-last-step", format!("{log}{}", last + 1), None),
        (
            "actions-job-log-empty-suffix",
            format!("{R}/actions/jobs/{}/logs?attempt={}&step=1", passing["id"], passing["attempt"]),
            Some(Range::Tail(SUFFIX)),
        ),
    ];
    let mut captured = Vec::new();
    for (name, target, range) in targets {
        captured.push(exchange(f, out, mask, secrets, name, &target, range)?);
    }
    let [step, suffix, past, empty] = captured.as_slice() else { unreachable!("four captures") };
    let n = step.body.len();
    let tail = usize::try_from(SUFFIX)?;
    let range = content_range(suffix);
    let empty_range = content_range(empty);
    let lengths = (whole.body.len(), n, suffix.body.len(), empty.body.len());
    let statuses = (step.status, suffix.status, empty.status, past.status);
    let seen =
        format!("steps={steps:?}; lengths={lengths:?}; statuses={statuses:?}; suffix={range:?}; empty={empty_range:?}");
    let numbered = steps.iter().enumerate().all(|(i, step)| step["number"].as_u64() == u64::try_from(i).ok());
    let checks = [
        steps.len() >= 3
            && numbered
            && steps.first().is_some_and(|s| s["name"] == "Set up job")
            && steps.last().is_some_and(|s| s["name"] == "Complete job"),
        steps.first().is_some_and(|s| s["status"] == "success")
            && 0 < failed
            && failed < last
            && steps.last().is_some_and(|s| s["status"] == "failure"),
        step.status == 200
            && contains(&step.body, b"TEMPER_CONFORMANCE_FAILED_JOB")
            && n < whole.body.len()
            && contains(&whole.body, &step.body)
            && !contains(&step.body, b"Run Main")
            && !contains(&step.body, b"Job failed"),
        suffix.status == 206 && n > tail && step.body.get(n.saturating_sub(tail)..) == Some(suffix.body.as_slice()),
        n > tail && range == Some(format!("bytes {}-{}/{n}", n.saturating_sub(tail), n.saturating_sub(1))),
        empty.status == 206 && empty.body.is_empty() && empty_range.as_deref() == Some("bytes 0--1/0"),
        past.status == 404,
    ];
    let mut facts = Vec::new();
    for (name, holds) in NAMES[6..].iter().zip(checks) {
        facts::add(&mut facts, name, holds, &seen);
    }
    Ok(facts)
}
