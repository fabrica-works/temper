#!/bin/sh
# The tiers the gate leaves out that main must keep green: skein's `land`
# runs this after each code landing, through heavy --gate, and a failure
# goes to the director (skein's owners.md, 4.5). Process and full QA run
# independently on this checkout; the outer runner owns queueing and cleanup.
set -eu
umask 077
cd -P "$(dirname "$0")/.."
exec python3 -I -B - <<'PY'
import datetime
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import time


def emit(event, **fields):
    print("after-landing " + json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


statuses = {}
measurements = []


def phase(name, kind, command):
    start = stamp()
    clock = time.monotonic()
    emit("phase-start", phase=name, kind=kind, start=start)
    try:
        code = subprocess.call(command)
    except OSError as error:
        print(f"{name}: {error}", flush=True)
        code = 127
    statuses[name] = code
    result = dict(phase=name, kind=kind, start=start, end=stamp(),
                  command_wall_seconds=round(time.monotonic() - clock, 3), exit_code=code)
    measurements.append(result)
    emit("phase-end", **result)
    return code == 0


def blocked(name):
    statuses[name] = "blocked"
    emit("phase-blocked", phase=name, reason="failed prerequisite")


root = Path.cwd()
sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
started = stamp()
emit("run-start", sha=sha, host=socket.gethostname(), start=started)
# The bundle builder uses target/ too. Refuse a different configured target
# rather than mixing outputs or touching a target the outer runner cannot clean.
target = root / "target"
configured = Path(os.environ.get("CARGO_TARGET_DIR", str(target)))
if (target.is_symlink() or (target.exists() and not target.is_dir())
        or configured.resolve() != target.resolve()):
    statuses["setup"] = 1
    emit("setup-failed", reason="after-landing requires the worktree's physical target directory")
    emit("run-end", sha=sha, host=socket.gethostname(), start=started, end=stamp(),
         process_exit_code=1, qa_exit_code=1, phases=statuses, exit_code=1)
    raise SystemExit(1)
os.environ.update(CARGO_TARGET_DIR=str(target), TEMPER_BUNDLE="none",
                  TEMPER_TEST_BUNDLE=str(target / "temper-bundle"),
                  TEMPER_TEST_CLAUDE=str(target / "debug/fake-claude"),
                  NEXTEST_DOUBLE_SPAWN="0", NEXTEST_USER_CONFIG_FILE="none")

programs = ["cargo", "build", "-p", "temper", "--bin", "temper",
            "-p", "temper-fake-claude", "--bin", "fake-claude"]
# Keep these process commands aligned with proof-one-process.sh. Separate
# build/discovery from command wall time; nextest still reports execution time.
process = ["cargo", "nextest", "run", "-p", "temper", "--profile", "process",
           "--no-fail-fast", "--failure-output", "immediate-final"]
qa = ["cargo", "nextest", "run", "--workspace", "--profile", "qa",
      "--no-fail-fast", "--retries", "0", "--failure-output", "immediate-final"]
phase("process-lab", "test", ["python3", "-I", "-B", "-m", "unittest", "discover",
                              "-s", "lab/tests/process", "-v"])
programs_ok = phase("process-programs", "build", programs)
if programs_ok and phase("process-build", "build-discovery", process + ["--no-run"]):
    phase("process-tests", "test", process)
else:
    if not programs_ok:
        blocked("process-build")
    blocked("process-tests")
process_ok = all(code == 0 for code in statuses.values())
emit("process-end", end=stamp(), exit_code=0 if process_ok else 1)

# This entire interval is the QA extension, including prep and build. Join its
# timestamps to the outer heavy receipt/TSV; raw queue wait is not added wait.
extension = stamp()
clock = time.monotonic()
emit("qa-extension-start", start=extension)
bundle_ok = phase("qa-bundle", "build", ["scripts/bundle.sh", "--loopback", "target/temper-bundle"])
if not programs_ok:
    programs_ok = phase("qa-programs", "build", programs)
# Use the same designated local-owned Forgejo/runner as the process tier.
# The existing fixtures own pin validation and scratch services; never fetch here.
chromium = os.environ.get("SKEIN_TEST_CHROMIUM") or shutil.which("chromium")
required = [chromium, os.environ.get("TEMPER_FORGEJO"), os.environ.get("TEMPER_FORGEJO_RUNNER")]
environment_ok = all(path and Path(path).is_file() and os.access(path, os.X_OK) for path in required)
statuses["qa-environment"] = 0 if environment_ok else 1
emit("qa-environment", end=stamp(), exit_code=statuses["qa-environment"],
     requirement="Chromium and designated pinned TEMPER_FORGEJO/TEMPER_FORGEJO_RUNNER executables")
if bundle_ok and programs_ok and environment_ok:
    if phase("qa-build", "build-discovery", qa + ["--no-run"]):
        phase("qa-tests", "test", qa)
    else:
        blocked("qa-tests")
else:
    blocked("qa-build")
    blocked("qa-tests")
qa_ok = all(code == 0 for name, code in statuses.items() if name.startswith("qa-"))
emit("qa-extension-end", start=extension, end=stamp(),
     command_wall_seconds=round(time.monotonic() - clock, 3), exit_code=0 if qa_ok else 1)
# Repeat timings at the end so the outer heavy tail retains them too.
for result in measurements:
    emit("phase-summary", **result)
code = 0 if process_ok and qa_ok else 1
emit("run-end", sha=sha, host=socket.gethostname(), start=started, end=stamp(),
     process_exit_code=0 if process_ok else 1, qa_exit_code=0 if qa_ok else 1,
     phases=statuses, exit_code=code)
raise SystemExit(code)
PY
