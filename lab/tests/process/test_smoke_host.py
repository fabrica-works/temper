"""Native canaries only: delegated cgroups/pidfds and the attended IO path.

No temper binary, subject, experiment, network or model is used.
"""
import importlib.util
import json
import os
from pathlib import Path
import secrets
import signal
import subprocess
import sys
import tempfile
import time
import types
import unittest

SCRIPT = Path(__file__).resolve()
ROOT = SCRIPT.parents[2] / 'chat-change-smoke'
sys.path.insert(0, str(ROOT))
spec = importlib.util.spec_from_file_location('smoke_host', ROOT / 'host.py')
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)
host.bind_skein(Path('/home/free/src/rust/skein'))


def synthetic_link():
    return b'http://127.0.0.1:7601/jig/v1/sign-in/link?secret=' + b'A' * 43 + b'\n'


def emit(mode):
    if mode == '--consume':
        data = sys.stdin.buffer.read()
        if data != synthetic_link():
            return 1
        print('private pipe consumed')
        return 0
    if mode == '--failure':
        return 9
    if mode == '--early-exit':
        if os.fork() == 0:
            signal.pause()
            os._exit(0)
        return 7
    if mode == '--emit':
        os.write(2, b'listening clients 127.0.0.1:7601\n')
        os.write(1, b'canary-before\n')
        link = synthetic_link()
        for chunk in (link[:7], link[7:41], link[41:]):
            os.write(1, chunk)
        os.write(2, b'malformed ' + link)
    else:
        os.write(1, b'canary-ready yes\n')
    signal.pause()
    return 0


def canary_run(lab, unit):
    run = host.Run.__new__(host.Run)
    run.lab, run.base = lab, 'canary'
    run.args = types.SimpleNamespace(seed='1', unit=unit)
    run.cost = dict(command_seconds=240, ready_seconds=120, stop_seconds=20)
    programs = {}
    for path in ('/bin/sh', '/usr/bin/python3', '/usr/bin/env', os.path.realpath(sys.executable)):
        info = host.cli._program(path)
        programs[info['path']] = dict(path=info['path'], sha256=info['sha256'])
    run.key = dict(subject='unused', lab=dict(programs=list(programs.values()), runner=host.cli._runner_digest()))
    run.exp = dict(processes=[])
    run.known, run.texts = {}, {}
    run.schedule = dict(draws={})
    run.start, run.started = time.monotonic(), host.utc()
    run.records, run.commands, run.items = {}, [], {}
    run.failures, run.undecided, run.errors, run.bounds = [], [], [], []
    run.link = run.public = run.interrupted = None
    run.cleaning = False
    run.mkdir('scratch/engine')
    run.mkdir('scratch/waiter')
    run.known['proc.engine.dir'] = str(lab.root / run.base / 'scratch/engine')
    run.known['proc.waiter.dir'] = str(lab.root / run.base / 'scratch/waiter')
    run.host = host.native.Host(unit, '/usr/bin/systemctl', run.cost)
    run.wake_r, wake_w = os.pipe2(os.O_CLOEXEC | os.O_NONBLOCK)
    return run, wake_w


def definition(name, mode, ready=()):
    return dict(name=name, argv=[os.path.realpath(sys.executable), '-I', '-B', str(SCRIPT), mode],
                env={}, ready=list(ready), memory_bytes=None)


def worker(unit, root):
    os.umask(0o077)
    with host.store.open_lab(Path(root) / 'lab') as lab, lab.lock():
        run, wake_w = canary_run(lab, unit)
        try:
            rules = [dict(stream='stdout', prefix='http://', capture='link'),
                     dict(stream='stderr', prefix='listening clients ', capture='clients')]
            run.start_child('processes', definition('engine', '--emit', rules), deadline=time.monotonic() + 120)
            assert run.wait(lambda: run.records['engine']['ready_ms'] is not None, time.monotonic() + 120)
            assert run.command('actions', definition('sign-in', '--consume'))
            assert run.link is None
            run.start_child('processes', definition('waiter', '--idle', [
                dict(stream='stdout', prefix='canary-ready ', capture='ack')]), deadline=time.monotonic() + 120)
            assert run.wait(lambda: run.records['waiter']['ready_ms'] is not None, time.monotonic() + 120)
            assert run.stop_all()
            assert not run.host.children
            assert all(r['memory']['peak_bytes'] > 0 for r in run.records.values())
            assert all(r['memory']['tree_peak_bytes'] > 0 for r in run.records.values())
            assert run.commands[0]['stdin'] is None
            assert not list(lab.root.rglob('*stdin*'))
            events = [json.loads(line) for line in lab.read('canary/events.jsonl').splitlines()]
            assert [e['name'] for e in events if e['event'] == 'signal'] == ['waiter', 'engine']
            encoded = json.dumps([run.records, run.commands]).encode()
            assert synthetic_link().strip() not in encoded
            assert b'AAAA' not in encoded
            for file in lab.root.rglob('*'):
                if file.is_file():
                    assert synthetic_link().strip() not in file.read_bytes()
                    assert b'AAAA' not in file.read_bytes()
            # A real forked descendant outliving its first process must be
            # observed, killed by cgroup, and awaited through tree emptiness.
            run.cleaning = False
            run.mkdir('scratch/exiting')
            run.known['proc.exiting.dir'] = str(lab.root / 'canary/scratch/exiting')
            run.start_child('processes', definition('exiting', '--early-exit'), deadline=time.monotonic() + 120)
            try:
                run.wait(lambda: False, time.monotonic() + 120)
                raise AssertionError('early exit was not observed')
            except host.Halt:
                pass
            assert run.stop_all()
            assert any(f['signature'] == 'survivor/exiting' for f in run.failures)
            assert run.records['exiting']['exit']['status'] == 7
            assert not run.command('actions', definition('failing-canary', '--failure'))
            assert not run.host.children
            assert any(f['signature'] == 'action/failing-canary/exit:9' for f in run.failures)
            print(json.dumps(dict(cgroup=str(run.host.root), canaries='pass')))
        finally:
            run.stop_all()
            os.close(run.wake_r)
            os.close(wake_w)
    return 0


class NativeCanaries(unittest.TestCase):
    def test_containment_filter_private_pipe_reverse_stop_and_early_exit(self):
        try:
            host.native.manager('/usr/bin/systemctl', ['show', '--property=Version', '--value'], 240)
        except (OSError, subprocess.SubprocessError, host.native.CapabilityError):
            self.skipTest('delegated systemd user manager unavailable')
        with tempfile.TemporaryDirectory() as directory:
            unit = 'lab-smoke-host-canary-' + secrets.token_hex(6) + '.service'
            argv = ['/usr/bin/systemd-run', '--user', '--no-ask-password', '--quiet', '--wait', '--pipe',
                    '--collect', '--expand-environment=no', '--service-type=exec', '--unit=' + unit,
                    '--property=Delegate=yes', '--property=KillMode=control-group',
                    '--property=TimeoutStartSec=120', '--property=TimeoutStopSec=20',
                    '/usr/bin/env', '-i', 'PATH=/usr/bin:/bin', os.path.realpath(sys.executable),
                    '-I', '-B', str(SCRIPT), '--worker', unit, directory]
            try:
                result = subprocess.run(argv, env=host.native.manager_environment(), stdin=subprocess.DEVNULL,
                                        capture_output=True, timeout=240)
                self.assertEqual(result.returncode, 0, result.stderr.decode('utf-8', 'replace'))
                summary = json.loads(result.stdout)
                self.assertEqual(summary['canaries'], 'pass')
                host.native.await_removed(summary['cgroup'], 20)
            finally:
                # --wait normally observed completion; this also covers an
                # assertion or subprocess timeout in the test's outer process.
                try:
                    host.native.manager('/usr/bin/systemctl', [
                        'kill', '--kill-whom=all', '--signal=SIGKILL', unit], 240)
                except host.native.CapabilityError:
                    pass


if __name__ == '__main__':
    if len(sys.argv) == 4 and sys.argv[1] == '--worker':
        sys.exit(worker(sys.argv[2], sys.argv[3]))
    if len(sys.argv) == 2 and sys.argv[1] in ('--emit', '--consume', '--idle', '--early-exit', '--failure'):
        sys.exit(emit(sys.argv[1]))
    unittest.main()
