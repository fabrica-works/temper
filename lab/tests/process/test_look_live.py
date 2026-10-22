"""Real /bin/sh steps, with invented browse responses and recorded stdin."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

LIVE = Path(__file__).resolve().parents[2] / 'look-live'
ADDRESS = '127.0.0.1:9100'
REPLY = ['wait', 'text', 'First reply.', '--in', 'log', 'Conversation', '--timeout', '60']
PHASES = {
    'send': [(['type', 'text area', 'Start a chat'], 'look-live-chat\n'),
             (['press', 'button', 'Start', '--exact'], ''),
             (['wait', 'log', 'Conversation', '--timeout', '60'], ''),
             (['type', 'text area', 'Message'], 'look-live-sent\n'),
             (['key', 'Enter'], ''), (REPLY[:-1] + ['90'], '')],
    'reload': [(['reload'], ''), (REPLY, '')],
    'close': [(['press', 'button', 'Close', '--exact'], ''),
              (['wait', 'dialog', 'Close', '--exact', '--timeout', '60'], '')]}


class StepsTests(unittest.TestCase):
    def run_phase(self, phase, fail_at=None, status=1):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / 'browse.jsonl'
            log.write_text('')
            browse = root / 'browse stand-in'
            browse.write_text('''#!/usr/bin/python3
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
print('met expected part' if args[1] == 'wait' else 'answered ' + args[1])
''' % (str(log), fail_at, status))
            browse.chmod(0o700)
            result = subprocess.run(['/bin/sh', str(LIVE / 'steps'), phase, str(browse), ADDRESS],
                                    capture_output=True, text=True, env={})
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            return result, calls

    def test_each_phase_calls_in_order_and_stdin(self):
        for phase, expected in PHASES.items():
            with self.subTest(phase=phase):
                result, calls = self.run_phase(phase)
                self.assertEqual((result.returncode, result.stderr), (0, ''))
                self.assertEqual(calls, [[[ADDRESS] + args, stdin] for args, stdin in expected])
                echoes = [line for line in result.stdout.splitlines() if line.startswith('+ browse')]
                self.assertEqual(echoes, ['+ browse ' + ' '.join(args) for args, _ in expected])
                self.assertIn('met expected part\n', result.stdout)

    def test_each_wait_missed_stops_immediately(self):
        for phase, expected in PHASES.items():
            for index, (args, _) in enumerate(expected):
                if args[0] != 'wait':
                    continue
                with self.subTest(phase=phase, wait=index):
                    result, calls = self.run_phase(phase, fail_at=index)
                    self.assertEqual(result.returncode, 1)
                    self.assertEqual(calls, [[[ADDRESS] + args, stdin] for args, stdin in expected[:index+1]])
                    self.assertTrue(result.stdout.endswith('missed\n'))

    def test_type_and_key_failures_preserve_status(self):
        for index in (0, 3, 4):
            result, calls = self.run_phase('send', fail_at=index, status=3)
            self.assertEqual(result.returncode, 3)
            self.assertEqual(len(calls), index + 1)

    def test_unknown_phase(self):
        result, calls = self.run_phase('unknown')
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])


if __name__ == '__main__':
    unittest.main()
