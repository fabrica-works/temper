"""S-only adapter: patch the rendered NOTICE root's result wake before exec.

The canonical template and all finite coding/review scripts stay untouched.
Python is the same lab-tooling exception as the attended S host.
"""
import json
import os
from pathlib import Path
import sys

CUE = 'Add a file NOTICE to widget'


def adapt(document):
    scripts = [s for s in document['cued'] if s['cue'] == CUE]
    if len(scripts) != 1:
        raise ValueError('expected one NOTICE root script')
    turns = scripts[0]['turns']
    expected = {'lines': [{'text': "The change's task has ended; it says how."}],
        'finish': 'stop', 'tokens': 20}
    if len(turns) != 3 or turns[2] != expected:
        raise ValueError('NOTICE root script differs from reviewed contract')
    # This is the last root turn: delegation/reply/wait remain byte-for-byte
    # equivalent JSON. The fallback is text only; no later mutating call exists.
    turns[2] = dict(expected, finish='tool_calls', lines=[expected['lines'][0],
                                   {'call': 'wait', 'arguments': {}}])
    return document


def main(argv):
    try:
        if (len(argv) != 9 or argv[1:4] != ['fake', 'llm', '--document']
                or argv[5:8] != ['--ready', 'listening ', '--seed']):
            raise ValueError('unexpected fake LLM argv')
        source = Path(argv[4])
        document = adapt(json.loads(source.read_bytes()))
        target = source.with_name('llm.s-runtime.json')
        fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'w') as stream:
            json.dump(document, stream, separators=(',', ':'))
            stream.write('\n')
        os.execv(argv[0], [*argv[:4], str(target), *argv[5:]])
    except (ValueError, KeyError, TypeError, OSError):
        print('S runtime LLM document refused', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
