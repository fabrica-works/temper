"""Native canaries only: delegated cgroups/pidfds and the attended IO path.

No temper binary, subject, experiment, network or model is used.
"""
import argparse
from contextlib import redirect_stdout
import importlib.util
import io
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
spec = importlib.util.spec_from_file_location('skein_source', ROOT.parent / 'skein_source.py')
skein_source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(skein_source)
# The test finds the pinned skein; its children run without cargo, so they are handed it
# (--skein), and the emitting children (one argument) never bind the runner.
if '--skein' in sys.argv:
    SKEIN = Path(sys.argv[sys.argv.index('--skein') + 1])
elif __name__ == '__main__' and len(sys.argv) == 2:
    SKEIN = None
else:
    SKEIN = skein_source.root()
if SKEIN is not None:
    host.bind_skein(SKEIN)


def synthetic_link():
    return b'http://127.0.0.1:7601/jig/v1/sign-in/link?secret=' + b'A' * 43 + b'\n'


def emit(mode):
    if mode.startswith('--stop-'):
        def stopped(_sig, _frame):
            if mode == '--stop-signal':
                os.kill(os.getpid(), signal.SIGUSR2)
            os._exit(9 if mode == '--stop-exit-nine' else 0)
        signal.signal(signal.SIGTERM, stopped)
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
    if mode.startswith('--emit-'):
        # Synchronize the two streams without timing assumptions. Both halves
        # still use the manifest's real public lines and private link format.
        signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGUSR1})
        link = synthetic_link()
        public = (b'listening clients 127.0.0.1:7601\n'
                  b'listening workers 127.0.0.1:7602\n'
                  b'listening webhooks 127.0.0.1:7603\n')
        os.write(1, b'canary-before\n')
        if mode == '--emit-link-first':
            for chunk in (link[:7], link[7:41], link[41:]):
                os.write(1, chunk)
            signal.sigwait({signal.SIGUSR1})
            os.write(2, public)
        else:
            os.write(2, public)
            signal.sigwait({signal.SIGUSR1})
            if mode == '--emit-public-first':
                os.write(1, link)
            elif mode == '--emit-malformed':
                os.write(1, b'malformed ' + link)
            elif mode == '--emit-repeated':
                os.write(1, link + link)
            elif mode == '--emit-mismatch':
                os.write(1, link.replace(b'7601', b'7609'))
            elif mode == '--emit-incomplete':
                os.write(1, link[:-1])
                os.close(1)
    else:
        os.write(1, b'canary-ready yes\n')
    signal.pause()
    return 0


def canary_run(lab, unit, case, native_host=None):
    run = host.Run.__new__(host.Run)
    run.lab, run.base = lab, case
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
    run.failures, run.undecided, run.errors = [], [], []
    run.link = run.public = run.interrupted = None
    run.cleaning = False
    run.mkdir('scratch/engine')
    run.mkdir('scratch/waiter')
    run.known['proc.engine.dir'] = str(lab.root / run.base / 'scratch/engine')
    run.known['proc.waiter.dir'] = str(lab.root / run.base / 'scratch/waiter')
    run.host = native_host if native_host is not None else host.native.Host(unit, '/usr/bin/systemctl', run.cost)
    run.wake_r, wake_w = os.pipe2(os.O_CLOEXEC | os.O_NONBLOCK)
    return run, wake_w


def definition(name, mode, ready=()):
    return dict(name=name, argv=[os.path.realpath(sys.executable), '-I', '-B', str(SCRIPT), mode],
                env={}, ready=list(ready), memory_bytes=None)


def worker(unit, root):
    os.umask(0o077)
    with host.store.open_lab(Path(root) / 'lab') as lab, lab.lock():
        # Validate the checked-in S declaration, rather than inventing a link
        # capture for this fixture. The process is synthetic; the IO host is real.
        manifest = json.loads((ROOT / 'experiment.json').read_bytes())
        host.contract(manifest, dict(costs={manifest['cost']: dict(
            ready_seconds=120, command_seconds=240, stop_seconds=20)}))
        rules = next(p['ready'] for p in manifest['processes'] if p['name'] == 'engine')
        modes = ('link-first', 'public-first', 'malformed', 'repeated', 'mismatch', 'incomplete', 'missing')
        native_host = None
        for mode in modes:
            run, wake_w = canary_run(lab, unit, mode, native_host)
            native_host = run.host
            try:
                deadline = time.monotonic() + 120
                run.start_child('processes', definition('engine', '--emit-' + mode, rules), deadline=deadline)
                observe = run.observe
                released = False
                def observe_ordered():
                    nonlocal released
                    observe()
                    first = run.link is not None if mode == 'link-first' else run.records['engine']['ready_ms'] is not None
                    if first and not released:
                        assert not run.startup_ready('engine')
                        released = True
                        run.host.signal_first('proc-engine', signal.SIGUSR1)
                run.observe = observe_ordered
                if mode in ('link-first', 'public-first'):
                    assert run.wait(lambda: run.startup_ready('engine'), deadline)
                    assert run.command('actions', definition('sign-in', '--consume'))
                    assert run.link is None
                    assert 'link' not in run.records['engine']['captures']
                    assert 'proc.engine.ready.link' not in run.known
                    assert len(run.items['proc-engine']['ready']) == len(rules)
                    run.start_child('processes', definition('waiter', '--idle', [
                        dict(stream='stdout', prefix='canary-ready ', capture='ack')]), deadline=time.monotonic() + 120)
                    assert run.wait(lambda: run.startup_ready('waiter'), time.monotonic() + 120)
                elif mode == 'missing':
                    assert run.wait(lambda: released, deadline)
                    assert not run.startup_ready('engine')
                else:
                    try:
                        run.wait(lambda: False, deadline)
                        raise AssertionError('invalid bootstrap was not refused')
                    except host.Halt as error:
                        assert b'AAAA' not in str(error).encode()
                run.observe = observe
                assert run.stop_all()
                assert run.link is None
                assert not run.host.children
                assert all(r['memory']['peak_bytes'] > 0 for r in run.records.values())
                assert all(r['memory']['tree_peak_bytes'] > 0 for r in run.records.values())
                assert all(r['stdin'] is None for r in run.commands)
                assert not list(lab.root.rglob('*stdin*'))
                events = [json.loads(line) for line in lab.read(run.base + '/events.jsonl').splitlines()]
                if mode in ('link-first', 'public-first'):
                    assert [e['name'] for e in events if e['event'] == 'signal'] == ['waiter', 'engine']
                encoded = json.dumps([run.records, run.commands, events]).encode()
                assert b'AAAA' not in encoded
                for file in lab.root.rglob('*'):
                    if file.is_file():
                        assert b'AAAA' not in file.read_bytes()
            finally:
                run.observe = observe
                run.stop_all()
                os.close(run.wake_r)
                os.close(wake_w)
        run, wake_w = canary_run(lab, unit, 'exits', native_host)
        try:
            # A real forked descendant outliving its first process must be
            # observed, killed by cgroup, and awaited through tree emptiness.
            run.cleaning = False
            run.mkdir('scratch/exiting')
            run.known['proc.exiting.dir'] = str(lab.root / run.base / 'scratch/exiting')
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
        finally:
            run.stop_all()
            os.close(run.wake_r)
            os.close(wake_w)
        for mode, suffix in (('exit-zero', None), ('exit-nine', 'exit:9'), ('signal', 'signal:SIGUSR2')):
            run, wake_w = canary_run(lab, unit, 'stop-' + mode, native_host)
            try:
                run.mkdir('scratch/stopping')
                run.known['proc.stopping.dir'] = str(lab.root / run.base / 'scratch/stopping')
                run.start_child('processes', definition('stopping', '--stop-' + mode, [
                    dict(stream='stdout', prefix='canary-ready ', capture='ack')]), deadline=time.monotonic() + 120)
                assert run.wait(lambda: run.startup_ready('stopping'), time.monotonic() + 120)
                assert run.stop_all()
                assert not run.host.children
                assert run.items['proc-stopping']['empty']
                assert run.records['stopping']['exit']['stopped']
                assert not run.records['stopping']['exit']['killed']
                for check in ('end-state', 'promises'):
                    assert host.checks.command(check, 0, b'') == ([], [], [])
                run.check_processes()
                assert [f['signature'] for f in run.failures] == ([] if suffix is None else ['stop/stopping/' + suffix])
                outcome = host.judge.outcome(run.failures, run.errors, run.undecided, None, driver_required=False)
                assert outcome == ('pass' if suffix is None else 'fail')
            finally:
                run.stop_all()
                os.close(run.wake_r)
                os.close(wake_w)
        print(json.dumps(dict(cgroup=str(run.host.root), canaries='pass', graceful_stop_cases=3)))
    return 0


class NativeCanaries(unittest.TestCase):
    def test_actual_bootstrap_acknowledges_and_records_unit_completion(self):
        try:
            host.native.manager('/usr/bin/systemctl', ['show', '--property=Version', '--value'], 240)
        except (OSError, subprocess.SubprocessError, host.native.CapabilityError):
            self.skipTest('delegated systemd user manager unavailable')
        with tempfile.TemporaryDirectory() as directory:
            args = types.SimpleNamespace(host_file=SCRIPT, skein=SKEIN,
                lab=Path(directory) / 'lab', experiment=Path(directory), subject='native-canary', seed='1')
            output = io.StringIO()
            with redirect_stdout(output):
                status = host.launch(args, host.native, host.store, host.host_digest(), host.cli._runner_digest())
            self.assertEqual(status, 0)
            self.assertEqual(output.getvalue(), 'native-bootstrap-canary pass\n')
            result, = args.lab.glob('transport/*/result')
            self.assertEqual(result.stat().st_mode & 0o777, 0o400)
            self.assertIn('cleanup_confirmed=True', result.read_text())
            stdout = result.with_name('stdout').read_bytes()
            self.assertTrue(stdout.startswith(b'admitted\n'))
            self.assertNotIn(synthetic_link().strip(), stdout)

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
                    '-I', '-B', str(SCRIPT), '--worker', unit, directory, '--skein', str(SKEIN)]
            try:
                result = subprocess.run(argv, env=host.native.manager_environment(), stdin=subprocess.DEVNULL,
                                        capture_output=True, timeout=240)
                self.assertEqual(result.returncode, 0, result.stderr.decode('utf-8', 'replace'))
                summary = json.loads(result.stdout)
                self.assertEqual(summary['canaries'], 'pass')
                self.assertEqual(summary['graceful_stop_cases'], 3)
                host.native.await_removed(summary['cgroup'], 20)
            finally:
                # --wait normally observed completion; this also covers an
                # assertion or subprocess timeout in the test's outer process.
                try:
                    host.native.manager('/usr/bin/systemctl', [
                        'kill', '--kill-whom=all', '--signal=SIGKILL', unit], 240)
                except host.native.CapabilityError:
                    pass


def bootstrap_canary():
    # Exercise the actual outer transport, with no subject or experiment.
    parser = argparse.ArgumentParser()
    parser.add_argument('--unit', required=True)
    args, _unused = parser.parse_known_args()
    native = host.native.Host(args.unit, '/usr/bin/systemctl',
                              dict(command_seconds=240, ready_seconds=120, stop_seconds=20))
    print('admitted', flush=True)
    print(json.dumps(dict(run='native-bootstrap-canary', outcome='pass', cgroup=str(native.root))))
    return 0


if __name__ == '__main__':
    if '--unit' in sys.argv:
        sys.exit(bootstrap_canary())
    if len(sys.argv) == 6 and sys.argv[1] == '--worker':
        sys.exit(worker(sys.argv[2], sys.argv[3]))
    if len(sys.argv) == 2 and (sys.argv[1].startswith(('--emit-', '--stop-')) or sys.argv[1] in ('--consume', '--idle', '--early-exit', '--failure')):
        sys.exit(emit(sys.argv[1]))
    unittest.main()
