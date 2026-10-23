"""The pinned skein's root: cargo's checkout of the revision temper's Cargo.lock names.

temper's lab tests bind skein's runner (`scripts/lab/runner`) from that checkout, so a
skein change reaches them only by a repin, whose gate runs them. Nothing here skips: a
root that cannot be found is an error.
CLI: lab/skein-source (prints the root)
"""
import functools
import json
from pathlib import Path
import re
import subprocess
import sys

TEMPER = Path(__file__).resolve().parents[1]
GIT = re.compile(r'git\+https://github\.com/[^/?#]+/skein\.git(\?[^#]*)?#([0-9a-f]{40})')


class Unresolved(RuntimeError):
    """The pinned skein's root could not be found; the message says why."""


def from_metadata(metadata):
    """The root holding scripts/lab/runner, from `cargo metadata`'s document."""
    revisions, manifests = set(), []
    for package in metadata.get('packages', []):
        source = package.get('source') or ''
        if 'skein.git' not in source and not package['name'].startswith('skein-'):
            continue
        found = GIT.fullmatch(source)
        if found is None:
            raise Unresolved(package['name'] + ' is not a git dependency on skein at a full revision: '
                             + (source or 'a path'))
        revisions.add(found.group(2))
        manifests.append(Path(package['manifest_path']))
    if not manifests:
        raise Unresolved('Cargo.lock names no skein package')
    if len(revisions) != 1:
        raise Unresolved('Cargo.lock names more than one skein revision: ' + ', '.join(sorted(revisions)))
    root = next((d for d in manifests[0].parents if (d / 'scripts/lab/runner').is_dir()), None)
    if root is None:
        raise Unresolved('the checkout of skein ' + revisions.pop() + ' holds no scripts/lab/runner')
    if any(root not in manifest.parents for manifest in manifests):
        raise Unresolved('skein packages lie outside one checkout: ' + str(root))
    return root


@functools.cache
def root():
    """The pinned skein's root, read once per process (cargo fetches the lock's sources when absent)."""
    result = subprocess.run(['cargo', 'metadata', '--format-version', '1', '--locked'],
                            cwd=TEMPER, stdin=subprocess.DEVNULL, capture_output=True, check=False)
    if result.returncode != 0:
        lines = result.stderr.decode(errors='replace').strip().splitlines()
        raise Unresolved('cargo metadata failed: ' + (lines[-1] if lines else 'exit ' + str(result.returncode)))
    return from_metadata(json.loads(result.stdout))


def main():
    try:
        print(root())
    except Unresolved as error:
        print('skein-source: ' + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
