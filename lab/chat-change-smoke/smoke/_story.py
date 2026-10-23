"""Fixed S calls; one deadline, product watches only, no polling."""
from pathlib import Path
import subprocess
import sys
import time

from _ctl_output import _child, _created, _json, _landed, _replied, _tree

STORY_SECONDS = 220


class CallFailed(Exception):
    """ctl already supplied its failure diagnostics on stderr."""


class Story:
    def __init__(self, prefix):
        self.deadline = time.monotonic() + STORY_SECONDS
        self.prefix = ['/bin/sh', str(Path(__file__).resolve().parents[1] / 'tools/ctl'), *prefix]

    def remaining(self):
        remaining = self.deadline - time.monotonic()
        if remaining < 1:
            raise ValueError('story deadline exhausted')
        return remaining

    def call(self, *args, condition=None, json=True):
        argv = self.prefix + list(args)
        if condition is not None:
            argv += ['--until', condition, '--timeout', str(int(self.remaining()))]
        if json:
            argv += ['--json']
        answer = subprocess.run(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                timeout=self.remaining(), check=False)
        # Keep each public response intact and delimit responses for rechecking;
        # ctl's JSON renderer does not append a newline.
        sys.stdout.buffer.write(answer.stdout)
        if not answer.stdout.endswith(b'\n'):
            sys.stdout.buffer.write(b'\n')
        sys.stdout.buffer.flush()
        self.remaining()
        if answer.returncode != 0:
            raise CallFailed
        text = answer.stdout.decode('utf-8')
        return _json(text) if json else text

    def run(self):
        root = _created(self.call('chat', '--project', '1', '--key', 'smoke-chat',
            'Add a file NOTICE to widget holding one line: Maintained by temper.', json=False))
        _replied(self.call('wait', 'task', str(root), condition='replied'), root)
        tree = self.call('task', str(root), '--tree')
        ended = self.call('tasks', '--project', '1', '--parent', str(root), '--ended')
        child = _child(tree, ended, root)
        _landed(self.call('wait', 'task', str(child), condition='ended'), child, root)
        _tree(self.call('task', str(root), '--tree'), root, final=True)
        self.remaining()


def main(args):
    try:
        if len(args) != 3:
            raise ValueError('expected temper, client directory and public origin')
        Story(args).run()
    except CallFailed:
        return 1
    except ValueError as error:
        if error.args == ('expected done: task N, chat',):
            print('story: expected done: task N, chat', file=sys.stderr)
        else:
            print('story: incomplete or unsuccessful landed-delegate evidence', file=sys.stderr)
        return 1
    except (KeyError, TypeError, AttributeError, OSError, subprocess.TimeoutExpired):
        print('story: incomplete or unsuccessful landed-delegate evidence', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
