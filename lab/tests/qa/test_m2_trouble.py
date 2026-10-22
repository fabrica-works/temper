"""Black-box trouble-check contract: lab README, qa-web.md 8.1."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

CHECK = Path(__file__).resolve().parents[2] / 'cluster-web/checks/trouble'
ORIGIN = 'http://127.0.0.1:32109'


def entry(address=ORIGIN + '/chats', *, kind='exception', text='invented trouble',
          during=None, call=0, n=1):
    return dict(n=n, kind=kind, text=text, address=address, call=call, during=during)


def line(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode() + b'\n'


class TroubleQA(unittest.TestCase):
    def setUp(self):
        os.umask(0o077)
        self.temp = tempfile.TemporaryDirectory(prefix='m2-trouble-')
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / 'trouble.jsonl'

    def check(self, raw):
        if raw is not None:
            self.path.write_bytes(raw)
        result = subprocess.run(['/usr/bin/python3', '-I', '-B', '-S', str(CHECK), str(self.path)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)
        return result, [json.loads(row) for row in result.stdout.splitlines()]

    def test_empty_and_seed_only_logs_pass(self):
        for raw in (b'', line(entry(during='sign-in', call=1))):
            with self.subTest(raw=raw):
                result, rows = self.check(raw)
                self.assertEqual((result.returncode, rows), (0, []), result.stderr)

    def test_each_route_has_stable_page_signature(self):
        cases = [('/', 'inbox'), ('/chats', 'chats'), ('/accounts', 'accounts'),
                 ('/chats/17', 'chat'), ('/chats/901', 'chat'),
                 ('/tasks/23', 'task'), ('/tasks/902', 'task'),
                 ('/projects/31/adopt', 'adopt'), ('/projects/903/adopt', 'adopt'),
                 ('/jig/v1/sign-in/link?secret=invented', 'sign-in'), ('/invented', 'other'),
                 ('/chats/not-a-number', 'other'), ('/tasks/23/more', 'other'),
                 ('/projects/31/adopt/more', 'other')]
        for path, page in cases:
            with self.subTest(path=path):
                result, rows = self.check(line(entry(ORIGIN + path)))
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual([r['fail'] for r in rows], ['browser/exception/' + page])

    def test_blank_and_off_origin_addresses_are_classified(self):
        for address, page in [('about:blank', 'blank'),
                              ('http://invented.example.invalid/chats/17', 'off-origin')]:
            with self.subTest(address=address):
                result, rows = self.check(line(entry(address)))
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual([r['fail'] for r in rows], ['browser/exception/' + page])

    def test_each_hand_kind_keeps_its_name(self):
        for kind in ('exception', 'console', 'log', 'crash', 'dialog', 'network'):
            with self.subTest(kind=kind):
                result, rows = self.check(line(entry(kind=kind)))
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual([r['fail'] for r in rows], ['browser/' + kind + '/chats'])

    def test_grouping_counts_entries_and_keeps_first_text(self):
        raw = b''.join(line(e) for e in [entry(text='invented FIRST'),
            entry(ORIGIN + '/chats', n=2, text='invented SECOND'),
            entry(ORIGIN + '/tasks/45', n=3, text='invented THIRD')])
        result, rows = self.check(raw)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(len(rows), 2)
        by_signature = {r['fail']: r['summary'] for r in rows}
        summary = by_signature['browser/exception/chats']
        self.assertIn('2', summary)
        self.assertIn('invented FIRST', summary)
        self.assertNotIn('invented SECOND', summary)
        self.assertIn('browser/exception/task', by_signature)

    def test_only_seed_sign_in_call_is_excluded(self):
        raw = b''.join(line(e) for e in [entry(during='sign-in', call=1),
            entry(n=2, during='go', call=2), entry(n=3, during=None, call=0)])
        result, rows = self.check(raw)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(len(rows), 1)
        self.assertIn('2', rows[0]['summary'])

    def test_missing_and_unreadable_inputs_are_undecided(self):
        for raw in (None, b'{broken\n', b'\xff\n', b'[]\n', b'{}\n'):
            with self.subTest(raw=raw):
                result, rows = self.check(raw)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertTrue(rows)
                self.assertTrue(all(set(r) == {'undecided'} for r in rows), rows)

    def test_bad_line_preserves_failures_beside_undecided(self):
        result, rows = self.check(line(entry()) + b'{broken\n' + line(entry(ORIGIN + '/accounts', n=3)))
        self.assertEqual({r['fail'] for r in rows if 'fail' in r},
                         {'browser/exception/chats', 'browser/exception/accounts'})
        self.assertTrue(any('undecided' in r for r in rows), rows)
        self.assertEqual(result.returncode, 1, (result.stderr, rows))
