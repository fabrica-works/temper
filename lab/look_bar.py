"""Read the look's bar and the judge's batches; never alter lab records."""
import argparse
from collections import defaultdict
import hashlib
import json
import os
from pathlib import Path
import re
import sys

NAME = re.compile(r'[a-z][a-z0-9-]*\Z')
DIGEST = re.compile(r'sha256:[0-9a-f]{64}\Z')
TAGS = ('layout', 'navigation', 'content', 'naming', 'style', 'behaviour')
LOOK = re.compile(r'check/look-(?:structure|style)/([^/]+)/([^/]+)/(.+)\Z')


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'),
                       ensure_ascii=False, allow_nan=False) + '\n').encode('utf-8')


def _file(value):
    return (type(value) is dict and set(value) == {'path', 'bytes', 'sha256'}
            and type(value['path']) is str and type(value['bytes']) is int
            and value['bytes'] >= 0 and type(value['sha256']) is str
            and DIGEST.fullmatch(value['sha256']))


def picture_digest(width):
    """Hash viewport then region shot digests, canonical JSON WITHOUT a newline.

    Missing parts, null shots and refusals contribute None, in record order.
    """
    def shot(side):
        value = side.get('shot') if type(side) is dict else None
        return value['sha256'] if _file(value) else None
    shots = [shot(width.get(side)) for side in ('mockup', 'page')]
    for region in width['regions']:
        shots.extend(shot(region.get(side)) for side in ('mockup', 'page'))
    return 'sha256:' + hashlib.sha256(canonical(shots)[:-1]).hexdigest()


def _parse(data):
    def constant(value):
        raise ValueError('invalid JSON constant: ' + value)
    return json.loads(data, parse_constant=constant)


def load_judgements(data):
    """Validate canonical look-judgements/1; report all member errors together."""
    value = _parse(data)
    errors = []

    def error(path, message):
        errors.append(path + ': ' + message)

    def shape(obj, required, optional, path):
        if type(obj) is not dict:
            error(path, 'expected object')
            return {}
        for key in sorted(set(required) - obj.keys()):
            error(path + '.' + key, 'missing member')
        for key in sorted(obj.keys() - set(required) - set(optional)):
            error(path + '.' + key, 'unknown member')
        return obj

    def text(item, path, pattern=None):
        if type(item) is not str or not item.strip():
            error(path, 'expected nonempty text')
        elif pattern and not pattern.fullmatch(item):
            error(path, 'invalid value')

    def items(item, path, minimum=0):
        if type(item) is not list:
            error(path, 'expected list')
            return []
        if len(item) < minimum:
            error(path, f'expected at least {minimum} entries')
        return item

    def names(item, path, minimum=1):
        for i, name in enumerate(items(item, path, minimum)):
            text(name, f'{path}.{i}', NAME)

    def unique(item, seen, path):
        if type(item) is str:
            if item in seen:
                error(path, 'repeated value')
            seen.add(item)

    if data != canonical(value):
        error('judgements', 'bytes not canonical')
    obj = shape(value, ('format', 'scene', 'widths', 'differences'), (), 'judgements')
    if obj.get('format') != 'look-judgements/1':
        error('judgements.format', 'expected look-judgements/1')
    text(obj.get('scene'), 'judgements.scene', NAME)
    widths = set()
    for i, width in enumerate(items(obj.get('widths'), 'judgements.widths', 1)):
        path = f'judgements.widths.{i}'
        width = shape(width, ('name', 'pictures'), (), path)
        text(width.get('name'), path + '.name', NAME)
        unique(width.get('name'), widths, path + '.name')
        digests = set()
        for j, picture in enumerate(items(width.get('pictures'), path + '.pictures', 1)):
            p = f'{path}.pictures.{j}'
            picture = shape(picture, ('digest', 'seen'), (), p)
            text(picture.get('digest'), p + '.digest', DIGEST)
            unique(picture.get('digest'), digests, p + '.digest')
            text(picture.get('seen'), p + '.seen')
    ids, signatures = set(), defaultdict(set)
    for i, difference in enumerate(items(obj.get('differences'), 'judgements.differences')):
        path = f'judgements.differences.{i}'
        place = difference.get('place') if type(difference) is dict else None
        extra = {'mockup': ('finding',), 'page': (), 'silent': ('question',)}.get(place, ()) if type(place) is str else ()
        optional = ('departures', 'expectations')
        d = shape(difference, ('id', 'width', 'signatures', 'tag', 'place', 'cites', 'seen') + extra,
                  optional, path)
        for key in ('id', 'width'):
            text(d.get(key), path + '.' + key, NAME)
        unique(d.get('id'), ids, path + '.id')
        width = d.get('width')
        if type(width) is not str or width not in widths:
            error(path + '.width', 'not among widths')
        for j, signature in enumerate(items(d.get('signatures'), path + '.signatures', 1)):
            p = f'{path}.signatures.{j}'
            text(signature, p)
            if type(signature) is str:
                if signature.startswith('/') or any(c.isspace() for c in signature):
                    error(p, 'expected relative signature without spaces')
                if type(width) is str:
                    unique(signature, signatures[width], p)
        for key, allowed in (('tag', TAGS), ('place', ('mockup', 'page', 'silent'))):
            if d.get(key) not in allowed:
                error(path + '.' + key, 'unknown value')
        for key in ('cites', 'seen'):
            text(d.get(key), path + '.' + key)
        for key in ('departures', 'expectations'):
            if key in d:
                names(d[key], path + '.' + key, 0 if place == 'page' else 1)
        if place == 'mockup':
            finding = shape(d.get('finding'), ('class', 'title', 'owner'), (), path + '.finding')
            if finding.get('class') not in ('world', 'look', 'experiment'):
                error(path + '.finding.class', 'expected world, look or experiment')
            for key in ('title', 'owner'):
                text(finding.get(key), path + '.finding.' + key)
        elif place == 'page':
            if not any(type(d.get(k)) is list and d[k] for k in ('departures', 'expectations')):
                error(path, 'page needs departures and/or expectations with at least one name')
        elif place == 'silent':
            question = shape(d.get('question'), ('options', 'recommendation'), (), path + '.question')
            for j, option in enumerate(items(question.get('options'), path + '.question.options', 2)):
                text(option, f'{path}.question.options.{j}')
            text(question.get('recommendation'), path + '.question.recommendation')
    if errors:
        raise ValueError('; '.join(errors))
    return value


def _read(path):
    return _parse(Path(path).read_bytes())


def _judgements(directory):
    result = {}
    try:
        paths = sorted(Path(directory).iterdir())
    except FileNotFoundError:
        return result
    for path in paths:
        if path.suffix != '.json':
            continue
        try:
            value = load_judgements(path.read_bytes())
            if value['scene'] != path.stem:
                raise ValueError('scene does not match filename')
            result[value['scene']] = value
        except ValueError as error:
            raise ValueError(f'{path}: {error}') from error
    return result


def _runs(lab, name, key):
    root = Path(lab) / 'keys' / name / key
    verdict = _read(root / 'verdict.json')
    groups = []
    for seed in verdict['seeds']:
        runs = []
        for run in seed['judged']:
            directory = root / 'runs' / run
            record = _read(directory / 'run.json')
            capture_dir = directory / 'collected/capture/capture'
            capture = _read(capture_dir / 'capture.json')
            if capture['format'] != 'look-capture/1':
                raise ValueError(f'{capture_dir}: expected look-capture/1')
            runs.append(dict(id=run, record=record, capture=capture,
                             directory=capture_dir,
                             signatures={f['signature'] for f in record['failures']}))
        groups.append(runs)
    return groups


def _reason(signature, run, judgements):
    match = LOOK.fullmatch(signature)
    if match and match[3].startswith('departure-gone/'):
        return 'departure-gone'
    if not match:
        return 'not a look check'
    scene, width, tail = match.groups()
    judgement = judgements.get(scene)
    judged_width = next((w for w in judgement['widths'] if w['name'] == width), None) if judgement else None
    if judged_width is None:
        return 'not judged'
    captured = next((w for s in run['capture']['scenes'] if s['name'] == scene
                     for w in s['widths'] if w['name'] == width), None)
    if captured is None or picture_digest(captured) not in {p['digest'] for p in judged_width['pictures']}:
        return 'pictures moved'
    difference = next((d for d in judgement['differences'] if d['width'] == width
                       and tail in d['signatures']), None)
    if difference is None:
        return 'no difference'
    return 'still failing' if difference['place'] == 'page' else None


def bar(lab, experiment_dir, key, judgements_dir, healthy_key=None):
    experiment = _read(Path(experiment_dir) / 'experiment.json')
    groups = _runs(lab, experiment['name'], key)
    runs = [run for group in groups for run in group]
    judgements = _judgements(judgements_dir)
    lines, holds = [], True
    scenes = list(dict.fromkeys(s['name'] for run in runs for s in run['capture']['scenes']))
    for scene in scenes:
        sets = [{s for s in run['signatures'] if (m := LOOK.fullmatch(s)) and m[1] == scene}
                for run in runs]
        verdicts = [f"{run['id']}: " + (f'fail {len(s)}' if s else 'pass')
                    for run, s in zip(runs, sets)]
        same = all(s == sets[0] for s in sets)
        holds = holds and same
        lines.append(f"{scene}: " + '; '.join(verdicts) + ('; same' if same else '; differs'))
    planted = experiment['planted']
    if planted:
        for run in runs:
            missing = [s for s in planted if s not in run['signatures']]
            holds = holds and not missing
            lines.append(f"{run['id']}: planted {len(planted) - len(missing)}/{len(planted)}")
            lines.extend('  missing ' + s for s in missing)
        if not runs:
            holds = False
        if healthy_key:
            healthy = _runs(lab, experiment['name'], healthy_key)
            if len(groups) != len(healthy) or any(not a or not b for a, b in zip(groups, healthy)):
                raise ValueError('healthy comparison needs a first judged run for each seed of both keys')
            for a, b in zip(groups, healthy):
                for title, difference in (('planted − healthy', a[0]['signatures'] - b[0]['signatures']),
                                          ('healthy − planted', b[0]['signatures'] - a[0]['signatures'])):
                    lines.append(f"{a[0]['id']} / {b[0]['id']}: {title}: {len(difference)}")
                    lines.extend('  ' + s for s in sorted(difference))
    else:
        unjudged = defaultdict(set)
        for run in runs:
            for signature in run['signatures']:
                reason = _reason(signature, run, judgements)
                if reason:
                    unjudged[signature, reason].add(run['id'])
            for item in run['record']['undecided']:
                unjudged[item['check'] + ': ' + item['reason'], 'undecided'].add(run['id'])
        lines.append('unjudged:' if unjudged else 'unjudged: none')
        holds = holds and not unjudged
        for (signature, reason), ids in sorted(unjudged.items()):
            lines.append(f'  {signature} — {reason} ({len(ids)} runs)')
    return '\n'.join(lines) + '\n', holds


def _pair_paths(run, scene, difference):
    width = next((w for s in run['capture']['scenes'] if s['name'] == scene
                  for w in s['widths'] if w['name'] == difference['width']), None)
    if width is None:
        raise ValueError(f"{scene}/{difference['width']}: absent from first judged run")
    regions = {parts[1] for signature in difference['signatures']
               if len(parts := signature.split('/')) >= 2 and parts[0] in ('style', 'order')}
    files = [width['pair']] + [r['pair'] for r in width['regions'] if r['name'] in regions]
    paths = []
    for file in files:
        if file is None:
            continue
        path = (run['directory'] / file['path']).resolve()
        if not path.is_relative_to(run['directory'].resolve()) or not path.is_file():
            raise ValueError(f'pair path missing or outside capture: {path}')
        paths.append(str(path))
    return paths


def batch(judgements_dir, lab, name, key):
    judgements = _judgements(judgements_dir)
    runs = [r for group in _runs(lab, name, key) for r in group]
    if not runs:
        raise ValueError('batch needs a judged run')
    findings, questions, notes = defaultdict(list), [], []
    for scene, judgement in judgements.items():
        for d in judgement['differences']:
            finding = d.get('finding')
            title = finding['title'] if finding else d['seen']
            item = [f"- {scene}/{d['id']} — {d['width']}; {d['tag']}" +
                    (f"; {finding['class']}" if finding else '') + ': ' + title,
                    '  Signatures: ' + ', '.join(d['signatures']), '  Cites: ' + d['cites']]
            item.extend('  Pair: ' + p for p in _pair_paths(runs[0], scene, d))
            entries = d.get('departures', []) + d.get('expectations', [])
            if d['place'] in ('mockup', 'silent') and entries:
                item.append('  entries: ' + ', '.join(entries))
            if finding:
                findings[finding['owner']].append('\n'.join(item))
            elif d['place'] == 'silent':
                item.extend(f'  {i}. {option}' for i, option in enumerate(d['question']['options'], 1))
                item.append('  Recommendation: ' + d['question']['recommendation'])
                questions.append('\n'.join(item))
            else:
                notes.append('\n'.join(item))
    lines = ['## Findings']
    for owner in sorted(findings):
        lines.extend(['', '### ' + owner, '', '\n\n'.join(findings[owner])])
    lines.extend(['', '## Questions', '', '\n\n'.join(questions), '',
                  '## Notes for the user', '', '\n\n'.join(notes)])
    return '\n'.join(lines) + '\n'


def _tool_uses(value):
    """Walk stream messages, including tool blocks inside partial stream events."""
    if type(value) is dict:
        if value.get('type') == 'tool_use':
            yield value
        for member in value.values():
            yield from _tool_uses(member)
    elif type(value) is list:
        for member in value:
            yield from _tool_uses(member)


def blind(directory, transcript, forbidden):
    directory = Path(directory)
    if not directory.is_absolute() or not directory.is_dir():
        raise ValueError('blind directory must be an absolute directory')
    root = directory.resolve()
    lines, failures = [], []
    for path in [directory] + sorted(directory.rglob('*')):
        if path != directory and not path.is_file() and not path.is_symlink():
            continue
        for word in forbidden:
            if word.casefold() in str(path).casefold():
                failures.append(f'{path}: forbidden word {word}')
    initialized = False
    for number, line in enumerate(Path(transcript).read_bytes().splitlines(), 1):
        event = _parse(line)
        if type(event) is not dict:
            raise ValueError(f'transcript line {number}: expected object')
        if event.get('type') == 'system' and event.get('subtype') == 'init':
            initialized = True
            if event.get('cwd') != str(directory):
                failures.append(f"init cwd {event.get('cwd')}: expected {directory}")
            if event.get('tools') != ['Read']:
                failures.append('init tools: expected exactly ["Read"]')
        for block in _tool_uses(event):
            if block.get('name') != 'Read':
                failures.append(f"tool {block.get('name')}: only Read allowed")
                continue
            inputs = block.get('input')
            path = inputs.get('file_path') if type(inputs) is dict else None
            if type(path) is not str or not path:
                failures.append('Read: missing file_path')
                continue
            resolved = Path(os.path.realpath(directory / path))
            lines.append('Read: ' + path)
            if not resolved.is_relative_to(root):
                failures.append(f'{path}: Read outside directory ({resolved})')
    if not initialized:
        failures.append('missing system init')
    lines.extend(failures)
    lines.append('blind: fail' if failures else 'blind: pass')
    return '\n'.join(lines) + '\n', not failures


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for command in ('bar', 'batch'):
        sub = commands.add_parser(command)
        sub.add_argument('--key', required=True)
        sub.add_argument('--lab', type=Path, default=Path.home() / 'src/rust/tmp/lab')
        sub.add_argument('--judgements', type=Path)
        if command == 'bar':
            sub.add_argument('--healthy')
        sub.add_argument('experiment', type=Path)
    sub = commands.add_parser('blind')
    sub.add_argument('--dir', type=Path, required=True)
    sub.add_argument('--transcript', type=Path, required=True)
    sub.add_argument('--forbid', nargs='+', required=True)
    sub = commands.add_parser('canon')
    sub.add_argument('files', nargs='+', type=Path)
    args = parser.parse_args(argv)
    try:
        if args.command == 'canon':
            for path in args.files:
                data = canonical(_read(path))
                load_judgements(data)
                path.chmod(0o600)
                path.write_bytes(data)
            return 0
        if args.command == 'blind':
            output, holds = blind(args.dir, args.transcript, args.forbid)
        else:
            name = _read(args.experiment / 'experiment.json')['name']
            judgements = args.judgements or args.lab / 'judgements' / name
            if args.command == 'bar':
                output, holds = bar(args.lab, args.experiment, args.key, judgements, args.healthy)
            else:
                output, holds = batch(judgements, args.lab, name, args.key), True
        print(output, end='')
        return 0 if holds else 1
    except (OSError, ValueError, KeyError, TypeError, UnicodeError) as error:
        print(f'look_bar: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
