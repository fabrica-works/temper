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
const NAMES: [&str; 6] = [
    "actions-report-each-job-as-a-status-on-its-head",
    "actions-status-link-is-runs-n-jobs-i",
    "actions-status-link-names-no-attempt",
    "run-number-is-index-in-repo",
    "run-jobs-come-in-index-order",
    "job-log-range-answers-206",
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
    capture(f, out, mask, secrets, "actions-job-log", &log, None)?;
    let response = f.exchange("GET", &log, None, Some(&f.writer.token), Some((0, 3)))?;
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
    out: &Path,
    mask: &mut Mask,
    secrets: &Secrets,
    name: &str,
    target: &str,
    range: Option<(u64, u64)>,
) -> Result<Value> {
    let response = f.exchange("GET", target, None, Some(&f.writer.token), range)?;
    record(out, mask, secrets, name, &response)?;
    Ok(serde_json::from_slice(&response.body).unwrap_or(Value::Null))
}
fn record(
    out: &Path,
    mask: &mut Mask,
    secrets: &Secrets,
    name: &str,
    response: &temper_throwaway_forgejo::Exchange,
) -> Result<()> {
    write::check(name, &response.sent, &response.body, secrets)?;
    let request = mask::request(mask, response);
    let response = mask::response(mask, response);
    write::case(out, name, &request, &response, secrets)
}
fn start(f: &Fixture, binary: &Path, secrets: &mut Secrets) -> Result<(Runner, String)> {
    let scratch = f.instance.scratch().join("runner");
    fs::create_dir_all(scratch.join("home"))?;
    let output = command(binary, &scratch).arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || !version.contains("12.13.2") {
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
