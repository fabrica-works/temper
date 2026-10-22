"""Real POSIX shells, with invented ctl and browse answers only."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

WEB = Path(__file__).resolve().parents[2] / 'cluster-web'
LINK = 'http://127.0.0.1:9000/jig/v1/sign-in/link?secret=invented-only'
ADDRESS = '127.0.0.1:9100'
CHAT = 'http://127.0.0.1:9200/chats/42'
REPLY = ['wait', 'text', 'First worker replied.', '--in', 'log', 'Conversation', '--timeout', '120']
CALLS = [
    ['type', 'text area', 'Start a chat'], ['press', 'button', 'Start', '--exact'],
    ['wait', 'log', 'Conversation', '--timeout', '120'], REPLY[:-1] + ['180'], ['address'],
    ['go', '/chats'], ['wait', 'heading', 'Chats', '--exact', '--timeout', '120'],
    ['go', '/chats/42'], REPLY, ['reload'], REPLY, ['shot'], ['back'],
    ['wait', 'heading', 'Chats', '--exact', '--timeout', '120'], ['address']]


class ShellTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def program(self, name, body):
        path = self.root / name
        path.write_text(body)
        path.chmod(0o700)
        return path

    def relink(self, status, message):
        log = self.root / 'ctl.json'
        temper = self.program('temper stand-in', '''#!/usr/bin/python3
import json
from pathlib import Path
import sys
Path(%r).write_text(json.dumps([sys.argv[1:], sys.stdin.read()]))
print('discarded ctl stdout')
print(%r, file=sys.stderr)
sys.exit(%r)
''' % (str(log), message, status))
        result = subprocess.run(['/bin/sh', str(WEB / 'relink'), str(temper), str(self.root / 'spent')],
                                input=LINK + '\n', capture_output=True, text=True, env={})
        self.assertEqual(json.loads(log.read_text()),
                         [['ctl', str(self.root / 'spent'), 'sign-in', '-'], LINK + '\n'])
        self.assertNotIn(LINK, result.stdout + result.stderr)
        self.assertNotIn('discarded ctl stdout', result.stdout + result.stderr)
        return result

    def test_relink_accepts_refusal(self):
        result = self.relink(1, 'temper ctl: the sign-in link was refused')
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, '', ''))

    def test_relink_refuses_success(self):
        result = self.relink(0, '')
        self.assertEqual(result.returncode, 1)
        self.assertIn('relink: expected the spent link refused; ctl exited 0:', result.stderr)

    def test_relink_refuses_other_error(self):
        result = self.relink(1, 'another error')
        self.assertEqual(result.returncode, 1)
        self.assertIn('ctl exited 1: another error', result.stderr)

    def smoke(self, *, fail_at=None, fail_status=1, chat=CHAT, final='http://127.0.0.1:9200/chats'):
        log = self.root / 'browse.jsonl'
        log.write_text('')
        browse = self.program('browse stand-in', '''#!/usr/bin/python3
import json
from pathlib import Path
import sys
log = Path(%r)
number = len(log.read_text().splitlines())
args = sys.argv[1:]
stdin = sys.stdin.read() if args[1] == 'type' else ''
with log.open('a') as output:
    output.write(json.dumps([args, stdin]) + '\\n')
if number == %r:
    print('missed')
    sys.exit(%r)
if args[1] == 'address':
    print(%r if number == 4 else %r)
elif args[1] == 'wait':
    print('met expected part')
else:
    print('answered ' + args[1])
''' % (str(log), fail_at, fail_status, chat, final))
        result = subprocess.run(['/bin/sh', str(WEB / 'smoke'), str(browse), ADDRESS],
                                capture_output=True, text=True, env={})
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        return result, calls

    def test_smoke_good_calls_and_stdin(self):
        result, calls = self.smoke()
        self.assertEqual((result.returncode, result.stderr), (0, ''))
        self.assertEqual(calls, [[[ADDRESS] + args, 'cluster-first-chat\n' if i == 0 else '']
                                 for i, args in enumerate(CALLS)])
        echoes = [line for line in result.stdout.splitlines() if line.startswith('+ browse')]
        self.assertEqual(echoes, ['+ browse ' + ' '.join(args) for args in CALLS])
        self.assertIn(CHAT + '\n', result.stdout)
        self.assertIn('met expected part\n', result.stdout)

    def test_smoke_missed_wait_stops(self):
        result, calls = self.smoke(fail_at=3)
        self.assertEqual(result.returncode, 1)
        self.assertEqual([call[0][1:] for call in calls], CALLS[:4])
        self.assertTrue(result.stdout.endswith('missed\n'))

    def test_smoke_preserves_address_call_status(self):
        result, calls = self.smoke(fail_at=4, fail_status=3)
        self.assertEqual(result.returncode, 3)
        self.assertEqual(len(calls), 5)
        self.assertTrue(result.stdout.endswith('missed\n'))

    def test_smoke_wrong_final_address(self):
        for final in ('http://127.0.0.1:9200/chats/42', 'http://localhost:9200/chats'):
            with self.subTest(final=final):
                result, calls = self.smoke(final=final)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(len(calls), len(CALLS))
                self.assertIn('expected http://127.0.0.1:9200/chats after back', result.stderr)

    def test_smoke_rejects_non_chat_address(self):
        for chat in ('http://localhost:9200/chats/no', 'https://localhost:9200/chats/42',
                     'http://localhost/chats/42', 'http://localhost:9200/chats/42?x=1'):
            with self.subTest(chat=chat):
                result, calls = self.smoke(chat=chat)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(len(calls), 5)


class TroubleCommandTests(unittest.TestCase):
    def test_missing_and_unreadable_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'trouble.jsonl'
            for content in (None, b'\xff'):
                if content is not None:
                    path.write_bytes(content)
                result = subprocess.run(['/usr/bin/python3', '-I', '-B', '-S', str(WEB / 'checks/trouble'),
                                         str(path)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
                self.assertEqual(json.loads(result.stdout), dict(undecided='trouble.jsonl missing'))

    def test_fail_and_undecided_printed_together(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'trouble.jsonl'
            path.write_text(json.dumps(dict(n=1, kind='exception', text='invented error',
                                           address='http://localhost:1/', call=1, during='wait')) + '\n{\n')
            result = subprocess.run(['/usr/bin/python3', '-I', '-B', '-S', str(WEB / 'checks/trouble'),
                                     str(path)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual([json.loads(line) for line in result.stdout.splitlines()], [
                dict(fail='browser/exception/inbox', summary='1 entry; first: invented error'),
                dict(undecided='trouble.jsonl line 2 unreadable')])


if __name__ == '__main__':
    unittest.main()
