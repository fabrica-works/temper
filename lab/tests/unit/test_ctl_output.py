"""S referee cases using the actual ctl renderer's owner JSON encodings."""
import copy
import json
from pathlib import Path
import re
import subprocess
import sys
import types
import unittest
from unittest.mock import patch

SMOKE = Path(__file__).resolve().parents[2] / 'chat-change-smoke' / 'smoke'
sys.path.insert(0, str(SMOKE))
from _ctl_output import _child, _created, _json, _landed, proof
from _story import CallFailed, Story, main

# The real landed-owner fixture in task_words.rs, not a model report.
TOOLS = Path(__file__).resolve().parents[3] / 'crates/temper-client-tools/src'
MERGE = re.search(r'const MERGE:.*?b"([0-9a-f]+)"',
                 (TOOLS / 'tests/jig/task_words.rs').read_text())[1]
ROOT, CHILD = 41, 42


def chip(number, *, coding=False, phase='running', delegates=0):
    return dict(task=number, executor=({'procedure': dict(connector=1, kind=2)} if coding
                 else {'agent': dict(charter=1)}), phase=phase,
                details={'owner': dict(details=dict(live_delegates=delegates))})


def task(chip):
    return dict(chip=chip, requester={'task': ROOT}, bytes_left=0, cut=False)


def evidence(*, fast=True):
    # Project only the referee's fields, with exactly their renderer nesting.
    root = task(chip(ROOT))
    replied = dict(task=copy.deepcopy(root), met=True)
    row = dict(chip=chip(CHILD, coding=True), parent=ROOT, cut=False)
    tree = dict(task=copy.deepcopy(root), tree=dict(root=chip(ROOT),
        rows=[] if fast else [row], folded=[], more=False, held_below=0))
    ended = dict(live=[], live_after=None, read_on=None, ended={'rows': dict(
        rows=[dict(chip=chip(CHILD, coding=True, phase='ended'), requester={'task': ROOT})]
             if fast else [], after=None, cut=False)})
    child = task(chip(CHILD, coding=True, phase='ended'))
    child['result'] = dict(task=copy.deepcopy(child['chip']), ending={'task': {'done': {'change': dict(
        kind=1, part=dict(connector=1, kind='resource', key={'path': ['0001']}),
        words=dict(bytes=MERGE, cut=False))}}}, value=None)
    return replied, tree, ended, dict(task=child, met=True), copy.deepcopy(dict(
        task=root, tree=dict(root=chip(ROOT), rows=[], folded=[], more=False, held_below=0)))


def transcript(documents=None):
    return 'done: task 41, chat\n' + ''.join(json.dumps(d) + '\n' for d in (documents or evidence()))


class OutputTests(unittest.TestCase):
    def test_creation_is_only_the_documented_done_line(self):
        self.assertEqual(_created('done: task 41, chat\n'), ROOT)
        for text in ('41\n', 'done: task 0, chat\n', 'done: task 041, chat\n',
                     'done: task 18446744073709551616, chat\n', 'done: task 41, chat'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                _created(text)

    def test_live_and_fast_ended_discovery_keep_historical_landing(self):
        for fast in (True, False):
            docs = evidence(fast=fast)
            self.assertEqual(_child(docs[1], docs[2], ROOT), CHILD)
            self.assertEqual(proof(transcript(docs)), dict(root=ROOT, child=CHILD, merge=MERGE))
        docs = evidence(fast=False)
        docs[2]['ended']['rows']['rows'] = evidence()[2]['ended']['rows']['rows']
        self.assertEqual(_child(docs[1], docs[2], ROOT), CHILD)  # one child seen twice
        # A complete tree may retain a terminal row, or omit it entirely.
        docs[4]['tree']['rows'] = [dict(parent=ROOT, cut=False,
                                       chip=chip(CHILD, coding=True, phase='ended'))]
        self.assertEqual(proof(transcript(docs))['child'], CHILD)

    def test_missing_ambiguous_and_wrong_parent_procedures_are_rejected(self):
        for mutation in ('missing', 'ambiguous', 'nested', 'producer', 'other-connector'):
            _, tree, ended, _, _ = evidence()
            rows = ended['ended']['rows']['rows']
            if mutation == 'missing':
                rows.clear()
            elif mutation == 'ambiguous':
                rows.append(dict(rows[0], chip=chip(43, coding=True)))
            elif mutation == 'nested':
                rows[0]['requester'] = {'task': 40}
            elif mutation == 'producer':
                rows[0]['chip']['executor'] = {'agent': {'charter': 2}}
            else:
                rows[0]['chip']['executor']['procedure']['connector'] = 2
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                _child(tree, ended, ROOT)

    def test_incomplete_discovery_and_final_tree_never_pass(self):
        mutations = [lambda d: d[1]['tree'].update(more=True),
            lambda d: d[1]['tree'].update(folded=[dict(parent=ROOT)]),
            lambda d: d[2].update(read_on='tasks --after cursor'),
            lambda d: d[2]['ended']['rows'].update(cut=True),
            lambda d: d[2]['ended']['rows'].update(after={'value': '00'}),
            lambda d: d[4]['task'].update(bytes_left=1),
            lambda d: d[4]['task'].update(cut=True),
            lambda d: d[4]['tree']['root'].update(task=99),
            lambda d: d[4]['tree'].update(held_below=1),
            lambda d: d[4]['task']['chip']['details']['owner']['details'].update(live_delegates=1),
            lambda d: d[4]['tree'].update(rows=[dict(parent=ROOT, cut=False, chip=chip(55, phase='held'))]),
            lambda d: d[4]['task']['chip'].update(phase='ended')]
        for mutate in mutations:
            docs = list(evidence())
            mutate(docs)
            with self.subTest(mutate=mutate), self.assertRaises(ValueError):
                proof(transcript(docs))
        docs = list(evidence(fast=False))
        docs[1]['tree']['rows'][0]['cut'] = True
        with self.assertRaises(ValueError):
            proof(transcript(docs))

    def test_owner_landing_requires_complete_merge_and_successful_procedure(self):
        mutations = [lambda d: d.update(met=False),
            lambda d: d['task'].update(requester={'task': 99}),
            lambda d: d['task']['chip'].update(task=99),
            lambda d: d['task']['chip'].update(executor={'agent': {'charter': 2}}),
            lambda d: d['task']['chip'].update(phase='failed'),
            lambda d: d['task']['result'].update(ending={'task': {'failed': {'bytes': 'broken', 'cut': False}}}),
            lambda d: d['task']['result'].update(ending={'task': {'cancelled': {'result': CHANGE}}}),
            lambda d: d['task']['result'].update(ending='deleted'),
            lambda d: d['task']['result'].update(ending='done', value=CHANGE),
            lambda d: d['task']['result'].update(ending={'task': {'done': {'report': {'bytes': MERGE, 'cut': False}}}}),
            lambda d: d['task']['result']['ending']['task']['done']['change'].update(kind=2),
            lambda d: d['task']['result']['ending']['task']['done']['change']['part'].update(connector=2),
            lambda d: d['task']['result']['ending']['task']['done']['change']['words'].update(cut=True),
            lambda d: d['task']['result']['ending']['task']['done']['change']['words'].update(bytes=MERGE[:12])]
        for mutate in mutations:
            doc = evidence()[3]
            mutate(doc)
            with self.subTest(mutate=mutate), self.assertRaises((ValueError, TypeError)):
                _landed(doc, CHILD, ROOT)

    def test_actual_historical_renderer_duplicate_result_is_consistent(self):
        doc = evidence()[3]
        result = doc['task']['result']
        for prior in (None, {'document': result}, {'ending': result['ending']['task']}):
            # tasks.rs writes TaskResult first, then historical Result.
            encoded = json.dumps(doc).replace('"result": ', '"result": ' + json.dumps(prior) + ', "result": ', 1)
            self.assertEqual(_landed(_json(encoded), CHILD, ROOT), MERGE)
        with self.assertRaises(ValueError):
            _json('{"met":false,"met":true}')
        with self.assertRaises(ValueError):
            _json('{"result":{"ending":{"failed":{}}},"result":' + json.dumps(result) + '}')
        with self.assertRaises(ValueError):
            proof(transcript() + '{}\n')


CHANGE = evidence()[3]['task']['result']['ending']['task']['done']


class StoryDeadline(unittest.TestCase):
    def test_empty_failed_response_keeps_stdout_empty(self):
        output = bytearray()
        with patch('sys.stdout', types.SimpleNamespace(buffer=types.SimpleNamespace(write=output.extend, flush=lambda: None))), \
                patch('_story.subprocess.run', return_value=types.SimpleNamespace(stdout=b'', returncode=1)):
            with self.assertRaises(CallFailed):
                Story(['/temper', '/client', 'http://127.0.0.1:7601/']).call('chat', json=False)
        self.assertEqual(output, b'')

    def drive(self, *, failed=False):
        now = [100.0]
        calls = []
        replies = iter([b'done: task 41, chat\n', *[json.dumps(d).encode() for d in evidence()]])
        output = bytearray()
        def run(argv, **kwargs):
            calls.append((argv, kwargs['timeout']))
            now[0] += 20
            return types.SimpleNamespace(stdout=next(replies), returncode=3 if failed else 0)
        with patch('sys.stdout', types.SimpleNamespace(buffer=types.SimpleNamespace(write=output.extend, flush=lambda: None))), \
                patch('_story.time.monotonic', side_effect=lambda: now[0]), \
                patch('_story.subprocess.run', side_effect=run):
            status = main(['/temper', '/client', 'http://127.0.0.1:7601/'])
        if not failed:
            self.assertEqual(proof(output.decode()), {'root': ROOT, 'child': CHILD, 'merge': MERGE})
        return status, calls

    def test_all_calls_and_waits_share_the_original_absolute_deadline(self):
        status, calls = self.drive()
        self.assertEqual(status, 0)
        self.assertEqual([t for _, t in calls], [220, 200, 180, 160, 140, 120])
        waits = [a for a, _ in calls if 'wait' in a]
        self.assertEqual([a[a.index('--timeout') + 1] for a in waits], ['200', '140'])
        self.assertEqual([a[a.index('--until') + 1] for a in waits], ['replied', 'ended'])
        self.assertEqual(waits[1][waits[1].index('task') + 1], str(CHILD))
        self.assertEqual(calls[-1][0][-4:], ['task', str(ROOT), '--tree', '--json'])

    def test_nonzero_exit_timeout_and_exhausted_allowance_fail(self):
        self.assertEqual(self.drive(failed=True)[0], 1)
        with patch('_story.time.monotonic', side_effect=[0, 219.1]), \
                patch('_story.subprocess.run') as run:
            with self.assertRaises(ValueError):
                Story(['/temper', '/client', 'http://127.0.0.1:1/']).call('chat')
            run.assert_not_called()
        with patch('_story.subprocess.run', side_effect=subprocess.TimeoutExpired('ctl', 1)):
            self.assertEqual(main(['/temper', '/client', 'http://127.0.0.1:1/']), 1)


if __name__ == '__main__':
    unittest.main()
