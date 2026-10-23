"""Attended S's credential boundary and causal/stopped collection ordering."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[2] / 'chat-change-smoke'
sys.path.insert(0, str(ROOT))
spec = importlib.util.spec_from_file_location('smoke_host', ROOT / 'host.py')
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)
host.bind_skein(Path('/home/free/src/rust/skein'))
from _host_output import Output, credential, origin
from test_ctl_output import transcript

LINK = b'http://127.0.0.1:7601/jig/v1/sign-in/link?secret=' + b'A' * 43 + b'\n'


class Filtering(unittest.TestCase):
    def test_every_split_and_single_byte_chunks(self):
        source = b'before\xff\n' + LINK + b'after\n'
        for chunks in ([source[:i], source[i:]] for i in range(len(source) + 1)):
            framed = Output()
            result = [record for chunk in chunks for record in framed.feed(chunk)]
            result += framed.feed(b'', eof=True)
            self.assertEqual(b''.join(r[0] for r in result), b'before\xff\nafter\n')
            self.assertEqual([r[1] for r in result if r[1] is not None], [LINK])
        framed = Output()
        result = [record for byte in source for record in framed.feed(bytes([byte]))]
        self.assertEqual(b''.join(r[0] for r in result), b'before\xff\nafter\n')

    def test_malformed_and_incomplete_links_never_persist(self):
        variants = [LINK[:-1], b'warning ' + LINK, LINK.replace(b'http://', b'HTtP://'),
                    LINK.replace(b'secret=', b'secret%3D'), LINK.replace(b'/link?', b'/broken?'),
                    LINK.replace(b'127.0.0.1', b'bad address'), LINK.replace(b'secret=', b'secret=\xff'),
                    LINK.replace(b'sign-in', b'sign-\x00in'), LINK.replace(b'sign-in', b'sign%2Din')]
        for value in variants:
            with self.subTest(value=value[:20]):
                framed = Output()
                result = framed.feed(value) + framed.feed(b'', eof=True)
                self.assertEqual(b''.join(r[0] for r in result), b'')
        for value in (LINK[:-1], b'warning ' + LINK, LINK.replace(b'/link?', b'/broken?')):
            with self.assertRaises(ValueError) as error:
                credential(value)
            self.assertNotIn('AAAA', str(error.exception))

    def test_complete_link_yields_only_public_origin(self):
        private, public = credential(LINK)
        self.assertEqual(private, LINK)
        self.assertEqual(public, 'http://127.0.0.1:7601/')
        self.assertEqual(origin('http://[::1]:7602/'), 'http://[::1]:7602/')
        for value in ('https://127.0.0.1:7601/', 'http://example.test:7601/', 'http://u:p@127.0.0.1:7601/',
                      'http://127.0.0.1:7601', 'http://127.0.0.1:7601/path',
                      'http://127.0.0.1:7601/?x=1', 'http://127.0.0.1:7601/#',
                      'http://[::1%lo]:7601/',
                      'http://127.0.0.1:7601/\n', LINK.decode().strip()):
            with self.assertRaises(ValueError):
                origin(value)

    def test_real_pipe_output_is_filtered_before_command_file(self):
        with tempfile.TemporaryDirectory() as directory:
            for stream in ('stdout', 'stderr'):
                read_fd, write_fd = os.pipe2(os.O_NONBLOCK | os.O_CLOEXEC)
                output_fd = os.open(directory + '/' + stream, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
                run = host.Run.__new__(host.Run)
                run.host = types.SimpleNamespace(children={'command': {'streams': {stream: read_fd}}})
                run.items = {'command': dict(record=dict(name='sign-in', group='actions'),
                    outputs={stream: dict(fd=output_fd, filter=Output())})}
                try:
                    for chunk in (b'kept\n' + LINK[:17], LINK[17:48], LINK[48:] + b'tail'):
                        os.write(write_fd, chunk)
                        run.output('command', stream)
                    os.close(write_fd)
                    write_fd = None
                    run.output('command', stream)
                finally:
                    os.close(output_fd)
                    if write_fd is not None:
                        os.close(write_fd)
                self.assertEqual(Path(directory, stream).read_bytes(), b'kept\ntail')

    def engine(self):
        run = host.Run.__new__(host.Run)
        run.public = run.link = None
        run.known = {}
        run.ms = lambda: 5
        run.events = []
        run.event = lambda event, **fields: run.events.append(dict(event=event, **fields))
        manifest = json.loads((ROOT / 'experiment.json').read_bytes())
        definition = next(p for p in manifest['processes'] if p['name'] == 'engine')
        record = dict(name='engine', group='processes', captures={}, ready_ms=None)
        run.records = {'engine': record}
        run.items = {'proc-engine': dict(record=record, definition=definition, ready=set())}
        return run

    def public_ready(self, run):
        for listener, port in (('clients', 7601), ('workers', 7602), ('webhooks', 7603)):
            run.line('proc-engine', 'stderr',
                     f'listening {listener} 127.0.0.1:{port}\n'.encode(), None, True)

    def test_public_readiness_and_private_link_in_either_order(self):
        for link_first in (True, False):
            run = self.engine()
            if link_first:
                run.line('proc-engine', 'stdout', b'', LINK, True)
                self.assertIsNone(run.records['engine']['ready_ms'])
                self.assertFalse(run.startup_ready('engine'))
            self.public_ready(run)
            self.assertEqual(run.records['engine']['ready_ms'], 5)
            self.assertEqual(run.startup_ready('engine'), link_first)
            if not link_first:
                run.line('proc-engine', 'stdout', b'', LINK, True)
            self.assertTrue(run.startup_ready('engine'))
            self.assertEqual(run.link, LINK)
            self.assertEqual(len(run.items['proc-engine']['ready']), 3)
            self.assertEqual(set(run.records['engine']['captures']), {'clients', 'workers', 'webhooks'})
            self.assertNotIn('proc.engine.ready.link', run.known)
            saved = json.dumps([run.records, run.events, run.known]).encode()
            self.assertNotIn(b'AAAA', saved)
            self.assertEqual(len(run.events), 1)

    def test_invalid_mismatched_repeated_and_incomplete_links_fail_safely(self):
        for value in (LINK[:-1], b'warning ' + LINK,
                      LINK.replace(b'7601', b'7609'), LINK.replace(b'http://', b'HTtP://')):
            run = self.engine()
            self.public_ready(run)
            with self.assertRaises(host.Halt) as error:
                run.line('proc-engine', 'stdout', b'', value, value.endswith(b'\n'))
            self.assertNotIn('AAAA', str(error.exception))
        run = self.engine()
        self.public_ready(run)
        run.line('proc-engine', 'stdout', b'', LINK, True)
        run.link = None  # Already delivered: repetition is still forbidden.
        with self.assertRaisesRegex(host.Halt, 'more than one'):
            run.line('proc-engine', 'stdout', b'', LINK, True)
        run = self.engine()
        run.line('proc-engine', 'stdout', b'', LINK.replace(b'7601', b'7609'), True)
        with self.assertRaisesRegex(host.Halt, 'differs'):
            self.public_ready(run)

    def test_missing_link_uses_original_startup_deadline_and_cleans_up(self):
        run = self.engine()
        with tempfile.TemporaryDirectory() as directory, host.store.open_lab(directory + '/lab') as lab:
            run.lab, run.base = lab, 'missing'
            run.args = types.SimpleNamespace(seed='1', unit='inert-unit')
            run.exp = dict(processes=[run.items['proc-engine']['definition']], dirs=[], collect=[], actions=[])
            run.order = ['engine']
            run.cost = dict(ready_seconds=120, command_seconds=240, stop_seconds=20)
            run.schedule = dict(start_order=[], draws={})
            run.bounds, run.failures, run.errors, run.undecided = [], [], [], []
            run.interrupted = None
            run.start_child = Mock(return_value='proc-engine')
            run.finish = lambda: run.errors
            run.stop_all = Mock(return_value=False)
            run.collect_live = Mock()
            def wait(predicate, deadline):
                self.assertEqual(deadline, 130)
                self.public_ready(run)
                self.assertFalse(predicate())
                return False
            run.wait = wait
            with patch.object(host.time, 'monotonic', return_value=10), patch.object(
                    host.native, 'Host', return_value=types.SimpleNamespace(root='/inert')):
                errors = run.run()
            self.assertIn('readiness deadline passed', errors[0]['detail'])
            self.assertEqual(run.bounds, [dict(name='ready_seconds', who='engine')])
            run.stop_all.assert_called_once()
            run.collect_live.assert_not_called()


class Collection(unittest.TestCase):
    def test_checkpoint_is_required_and_loss_never_becomes_complete(self):
        prefix = b'{"observation":"moved"}\n{"observations_lost":0}\n'
        raw = prefix + b'{"unfinished":'
        self.assertEqual(host.checkpoint_prefix(raw), (prefix, True))
        self.assertEqual(host.checkpoint_prefix(b'{"observation":"moved"}\n'), (b'', False))
        self.assertFalse(host.checkpoint_prefix(prefix + b'{"observations_lost":1}\n')[1])
        self.assertFalse(host.checkpoint_prefix(prefix + b'{"observations_lost":true}\n')[1])
        self.assertFalse(host.checkpoint_prefix(prefix + b'{"observations_lost":0,"observations_lost":0}\n')[1])

    def stopped(self, copy_ok):
        run = host.Run.__new__(host.Run)
        run.host = types.SimpleNamespace(children={})
        run.items = {'proc-engine': dict(empty=True)}
        run.known = {'proc.engine.dir': '/stopped-engine'}
        run.exp = dict(collect=[dict(name='observations'), dict(name='store')], checks=[dict(name='promises')])
        run.undecided = []
        calls = []
        run.event = lambda event, **fields: calls.append(event)
        run.command = lambda group, definition: calls.append(group + '/' + definition['name']) or copy_ok
        return run, calls

    def test_store_copy_requires_all_trees_empty(self):
        run, calls = self.stopped(True)
        run.host.children['worker'] = object()
        with patch.object(host.store, 'walk') as read_store:
            with self.assertRaises(host.Halt):
                run.stopped_check()
            read_store.assert_not_called()
        self.assertEqual(calls, [])

    def test_store_read_and_check_follow_empty_and_successful_copy(self):
        run, calls = self.stopped(True)
        with patch.object(host.store, 'walk', side_effect=lambda path: calls.append('read-stopped-source')):
            run.stopped_check()
        self.assertEqual(calls, ['store_copy_admitted', 'read-stopped-source', 'collect/store',
                                 'store_copy_complete', 'checks/promises'])

    def test_failed_copy_skips_promises(self):
        run, calls = self.stopped(False)
        with patch.object(host.store, 'walk'):
            run.stopped_check()
        self.assertNotIn('checks/promises', calls)
        self.assertEqual(run.undecided[0]['check'], 'promises')
        self.assertIn('check_skipped', calls)

    def test_boundary_requires_rechecked_delegate_evidence_and_zero_loss(self):
        for story_text, zero_loss, expected in ((transcript(), True, True),
                ('model says landed\n', True, False), (transcript(), False, False),
                (transcript().replace('"live_delegates": 0', '"live_delegates": 1'), True, False)):
            run = host.Run.__new__(host.Run)
            run.base = 'run'
            raw = b'{"observations_lost":' + (b'0' if zero_loss else b'1') + b'}\n'
            run.lab = types.SimpleNamespace(root=Path('/private'), read=lambda path: (
                raw if path.endswith('observations.jsonl') else story_text.encode()))
            run.commands = [dict(name='story', exit=dict(status=0, stopped=False, killed=False),
                output=dict(stdout=dict(path='commands/actions-story/stdout'))), dict(name='observations')]
            run.exp = dict(collect=[dict(name='observations')], checks=[dict(name='end-state')])
            run.key = dict(subject='subject', lab=dict(runner='runner'))
            run.cost = dict(command_seconds=240)
            run.event, run.write, run.mkdir = Mock(), Mock(), Mock()
            calls = []
            run.command = lambda group, definition, **kwargs: calls.append((group, definition, kwargs)) or True
            run.collect_live()
            event = run.event.call_args.kwargs
            self.assertEqual(event['confirmed'], expected)
            self.assertEqual(event['endpoint'], 'landed coding delegate settled; root Open with zero live delegates')
            if expected:
                self.assertEqual(event['settlement']['child'], 42)
            extra = calls[-1][2]['extra']
            self.assertEqual(extra[:2], ('--boundary', 'confirmed' if expected else 'unknown'))
            self.assertEqual(float(extra[3]), calls[-1][2]['deadline'])
            self.assertEqual(calls[0][0], 'collect')
            self.assertEqual(calls[-1][0], 'checks')

    def test_live_collection_and_end_state_precede_stop(self):
        run = host.Run.__new__(host.Run)
        with tempfile.TemporaryDirectory() as directory, host.store.open_lab(directory + '/lab') as lab:
            run.lab, run.base = lab, 'test-run'
            run.args = types.SimpleNamespace(seed='1', unit='inert-unit')
            run.exp = dict(processes=[], dirs=[], collect=[], actions=[])
            run.order, run.records, run.items = [], {}, {'proc-engine': dict(empty=True)}
            run.cost = dict(stop_seconds=20, ready_seconds=120, command_seconds=240)
            run.errors, run.undecided = [], []
            run.host, run.link, run.interrupted = None, None, None
            calls = []
            run.event = lambda *args, **kwargs: None
            run.collect_live = lambda: calls.extend(['live-observations', 'live-end-state-objects'])
            run.stop_all = lambda: calls.append('reverse-stop-and-await-empty') or True
            run.stopped_check = lambda: calls.extend(['stopped-copy', 'promises'])
            run.finish = lambda: calls
            with patch.object(host.native, 'Host', return_value=types.SimpleNamespace(root='/owned-canary')):
                self.assertEqual(run.run(admitted=lambda: calls.append('admitted')), ['admitted', 'live-observations', 'live-end-state-objects',
                    'reverse-stop-and-await-empty', 'stopped-copy', 'promises'])


class StopBudgets(unittest.TestCase):
    def run_state(self, killed=False):
        run = host.Run.__new__(host.Run)
        child = dict(killed=killed)
        run.items = {'child': dict(record=dict(name='canary'), empty=False, deadline=19)}
        run.host = types.SimpleNamespace(children={'child': child})
        run.host.signal_first = Mock()
        run.host.kill_tree = Mock(side_effect=lambda identity: child.update(killed=True))
        run.event = Mock()
        run.cost = dict(stop_seconds=20)
        run.bounds, run.failures = [], []
        run.wait = Mock(return_value=False)
        return run

    def test_timeout_kill_has_one_empty_allowance(self):
        run = self.run_state()
        with patch.object(host.time, 'monotonic', return_value=10):
            with self.assertRaises(host.Halt):
                run.stop_child('child', kill=True)
        self.assertEqual(run.wait.call_count, 1)
        self.assertEqual(run.wait.call_args.args[1], 30)
        run.host.kill_tree.assert_called_once()
        self.assertEqual(run.items['child']['deadline'], 30)

    def test_prior_survivor_kill_does_not_restart_cap(self):
        run = self.run_state(killed=True)
        with patch.object(host.time, 'monotonic', return_value=20):
            with self.assertRaises(host.Halt):
                run.stop_child('child')
        self.assertEqual(run.wait.call_count, 1)
        self.assertEqual(run.wait.call_args.args[1], 19)
        run.host.kill_tree.assert_not_called()

    def test_reaping_a_killed_parent_preserves_the_original_empty_cap(self):
        run = self.run_state(killed=True)
        child = run.host.children['child']
        child.update(reaped=False, exec_ack=None, streams={}, peak_bytes=1, source='wait4')
        run.items['child'].update(stopped=True, outputs={})
        run.items['child']['record'].update(group='actions', memory={})
        run.host.exited = Mock(return_value=True)
        run.host.reap = Mock(side_effect=lambda identity: child.update(reaped=True) or dict(status=0))
        run.host.counts = Mock(return_value=dict(populated=True, tree_peak_bytes=1, pids=1))
        run.ms, run.cleaning, run.errors = lambda: 1, True, []
        with patch.object(host.time, 'monotonic', return_value=10):
            run.observe()
        self.assertEqual(run.items['child']['deadline'], 19)
        run.host.kill_tree.assert_not_called()

    def test_grace_then_kill_matches_native_runner(self):
        run = self.run_state()
        with patch.object(host.time, 'monotonic', side_effect=[10, 30]):
            with self.assertRaises(host.Halt):
                run.stop_child('child')
        self.assertEqual([call.args[1] for call in run.wait.call_args_list], [30, 50])
        run.host.signal_first.assert_called_once()
        run.host.kill_tree.assert_called_once()


class Manifest(unittest.TestCase):
    def manifest(self):
        exp = json.loads((ROOT / 'experiment.json').read_bytes())
        settings = dict(costs={exp['cost']: dict(ready_seconds=120, command_seconds=240, stop_seconds=20)})
        return exp, settings

    def test_only_the_declared_S_contract_is_accepted(self):
        exp, settings = self.manifest()
        host.contract(exp, settings)
        mutations = [
            lambda e: e.update(driver={}),
            lambda e: e['processes'][0].update(argv=['/usr/bin/true']),
            lambda e: e['processes'][1]['argv'].__setitem__(3, '/tmp/other.py'),
            lambda e: e['processes'][1].update(argv=e['processes'][1]['argv'][4:]),
            lambda e: e['actions'].reverse(),
            lambda e: e['actions'][0].update(stdin='credential'),
            lambda e: e['actions'][0].update(stdin='{proc.engine.ready.link}\n'),
            lambda e: e['processes'][0].update(stdin='/tmp/link'),
            lambda e: e['actions'][1].update(stdin=''),
            lambda e: e['collect'][0].update(stdin=''),
            lambda e: e['checks'][0].update(stdin=''),
            lambda e: e['actions'][0].update({'await': {}}),
            lambda e: e['processes'][2]['ready'].append(
                dict(stream='stdout', prefix='http://', capture='link')),
            lambda e: e['processes'][2]['ready'][0].update(capture='link'),
            lambda e: e['processes'][2]['files'].append(
                dict(path='{proc.engine.ready.link}', **{'from': 'engine.json'})),
            lambda e: e['actions'][-1]['argv'].__setitem__(-1, '{proc.engine.ready.clients}'),
            lambda e: e['actions'][-1]['argv'].__setitem__(-1, '{proc.engine.ready.link}'),
            lambda e: e['checks'][1]['argv'].__setitem__(-1, '{proc.engine.dir}/data/state/store'),
            lambda e: e['actions'][1].update(env={'HOME': '/tmp'})]
        for mutate in mutations:
            changed = copy.deepcopy(exp)
            mutate(changed)
            with self.assertRaises(host.Halt):
                host.contract(changed, settings)
        settings['costs'][exp['cost']]['command_seconds'] = 241
        with self.assertRaises(host.Halt):
            host.contract(exp, settings)


class StoryOrigin(unittest.TestCase):
    def test_complete_origin_reaches_spawn_unchanged_and_invalid_origin_never_spawns(self):
        with tempfile.TemporaryDirectory() as directory, host.store.open_lab(directory + '/lab') as lab:
            run = host.Run.__new__(host.Run)
            run.lab, run.base = lab, 'story-origin'
            programs = [dict(path=p, sha256='keyed') for p in ('/bin/sh', '/usr/bin/python3', '/usr/bin/env')]
            run.key = dict(lab=dict(programs=programs, runner='selected'))
            run.host = types.SimpleNamespace(spawn=Mock(return_value=dict(pid=1)))
            run.ms = lambda: 1
            run.event = Mock()
            run.items, run.commands = {}, []
            run.render = lambda text: {'{experiment}': '/s', '{artifact.temper}': '/temper',
                                     '{dir.client}': '/client'}.get(text, text)
            definition = dict(name='story', env={'PATH': '/usr/bin:/bin'}, argv=[
                '/bin/sh', '/s/smoke/story', '/temper', '/client', 'http://127.0.0.1:7601/'])
            with patch.object(host.cli, '_program', side_effect=lambda path: dict(path=path, sha256='keyed')), patch.object(
                    host.cli, '_runner_digest', return_value='selected'):
                run.start_child('actions', definition, deadline=240)
                self.assertEqual(run.host.spawn.call_args.args[1], definition['argv'])
                for value in ('http://http://127.0.0.1:7601/', 'http://127.0.0.1:7601/path',
                              LINK.decode().strip()):
                    definition['argv'][-1] = value
                    with self.assertRaises(host.Halt):
                        run.start_child('actions', definition, deadline=240)
                run.host.spawn.assert_called_once()
            for output in run.items['actions-story']['outputs'].values():
                os.close(output['fd'])


class Records(unittest.TestCase):
    def test_stop_checks_keep_existing_causes_and_memory_observations(self):
        for stopped, killed, prior, expected in (
            (False, False, [], ['stop/engine/exit:9']),
            (True, False, [], ['stop/engine/exit:9']),
            (True, True, ['bound/stop_seconds/engine'], ['bound/stop_seconds/engine']),
            (True, True, ['survivor/engine'], ['survivor/engine']),
            (True, False, ['panic/engine/source.rs:1:1'], ['panic/engine/source.rs:1:1']),
            # A killed flag by itself must never hide an unsuccessful exit.
            (True, True, [], ['stop/engine/exit:9']),
        ):
            with self.subTest(stopped=stopped, killed=killed, prior=prior):
                run = host.Run.__new__(host.Run)
                run.records = {'engine': dict(exit=dict(status=9, stopped=stopped, killed=killed),
                    memory=dict(bound_bytes=10, peak_bytes=11))}
                run.failures = [host.checks.failure(signature, signature.split('/')[0], 'observed') for signature in prior]
                run.undecided = []
                run.check_processes()
                self.assertCountEqual([f['signature'] for f in run.failures], expected + ['memory/engine'])
                self.assertEqual(run.undecided, [])

    def test_requested_stop_must_exit_zero_even_with_empty_trees_and_passing_checks(self):
        for ending, expected in ((dict(status=0), 'pass'), (dict(status=9), 'fail'),
                                 (dict(signal='SIGUSR2'), 'fail')):
            with self.subTest(ending=ending), tempfile.TemporaryDirectory() as directory, host.store.open_lab(directory + '/lab') as lab:
                run = host.Run.__new__(host.Run)
                run.lab, run.base = lab, 'stop-proof'
                run.host = types.SimpleNamespace(children={})
                run.records = {'engine': dict(name='engine', group='processes',
                    exit=dict(ms=1, stopped=True, killed=False, **ending),
                    memory=dict(bound_bytes=None, peak_bytes=1))}
                run.failures, run.undecided, run.errors, run.bounds = [], [], [], []
                run.commands = []
                run.interrupted = None
                run.exp = dict(name='smoke')
                run.key_path = 'keys/smoke/key'
                run.run_id, run.attempt, run.schedule, run.started = 'stop-proof', 1, {}, host.utc()
                run.args = types.SimpleNamespace(seed='1', unit='canary-unit')
                run.ms = lambda: 1
                # Both real checker contracts report pass; only service exit can fail.
                self.assertEqual(host.checks.command('end-state', 0, b''), ([], [], []))
                self.assertEqual(host.checks.command('promises', 0, b''), ([], [], []))
                with patch.object(host.evidence, 'publish'), patch.object(lab, 'remove_live'):
                    record = run.finish()
                self.assertEqual(record['outcome'], expected)
                if expected == 'fail':
                    reason = 'exit:9' if 'status' in ending else 'signal:SIGUSR2'
                    self.assertEqual([f['signature'] for f in record['failures']], ['stop/engine/' + reason])

    def test_failed_admission_keeps_canonical_error_evidence_and_live_marker(self):
        with tempfile.TemporaryDirectory() as directory, host.store.open_lab(directory + '/lab') as lab, lab.lock():
            run = host.Run.__new__(host.Run)
            run.lab = lab
            run.key = dict(format='lab-key/1', experiment='sha256:' + '1' * 64,
                           subject='sha256:' + '2' * 64, lab=dict(runner='sha256:' + '3' * 64,
                               python='3.11', programs=[], driver=None, cost=dict(name='calibration',
                                   ready_seconds=120, command_seconds=240, stop_seconds=20,
                                   driver_seconds=240, driver_actions=1, driver_input_tokens=1)))
            run.key_path = 'keys/smoke/' + host.canonical.digest(run.key)
            lab.write_once(run.key_path + '/key.json', host.canonical.record_bytes(run.key))
            run.run_id = '20261023T001000000Z-abcdef'
            run.base = run.key_path + '/runs/' + run.run_id
            run.exp = dict(name='smoke')
            run.args = types.SimpleNamespace(seed='1', unit='canary-unit')
            run.attempt, run.schedule, run.started = 1, {}, host.utc()
            run.host, run.records, run.commands = None, {}, []
            run.failures, run.undecided, run.bounds = [], [], []
            run.errors = [dict(what='host', detail='CapabilityError')]
            run.interrupted = 'SIGTERM'
            run.ms = lambda: 1
            run.write('live.json', host.canonical.record_bytes(dict(format='lab-live/1', unit='canary-unit')))
            record = run.finish()
            self.assertEqual(record['outcome'], 'error')
            self.assertIn(dict(what='interrupted', detail='SIGTERM'), record['errors'])
            self.assertIsNone(record['usage']['longest_ms'])
            self.assertEqual(host.evidence.read(lab, run.key_path)[0]['outcome'], 'error')
            verdict = host.canonical.loads(lab.read(run.key_path + '/verdict.json'))
            self.assertEqual(verdict['verdict'], 'not enough evidence')
            # The real publication helper's additive/write-once behavior stays
            # authoritative: identical publication is idempotent, edits refuse.
            host.evidence.publish(lab, run.key_path, record)
            self.assertEqual(len(host.evidence.read(lab, run.key_path)), 1)
            with self.assertRaises(host.canonical.Refused):
                lab.write_once(run.base + '/run.json', b'changed')
            self.assertTrue((lab.root / run.base / 'live.json').is_file())


if __name__ == '__main__':
    unittest.main()
