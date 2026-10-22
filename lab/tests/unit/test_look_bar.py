"""The look bar on small recorded fixtures, without a browser or a real lab."""
import copy
from contextlib import redirect_stdout
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('look_bar', Path(__file__).resolve().parents[2] / 'look_bar.py')
look = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(look)
SIG = 'check/look-style/chat/desktop/style/composer/color'
OTHER = 'check/look-structure/chat/desktop/order/composer'


def file(path, digest='a'):
    return dict(path=path, bytes=1, sha256='sha256:' + digest * 64)


def width():
    return dict(name='desktop', mockup=dict(shot=file('mockup.png')),
                page=dict(shot=file('page.png', 'b')), pair=file('chat/desktop/pair.png'),
                regions=[dict(name='composer', mockup=dict(shot=file('m.png', 'c')),
                              page=dict(shot=file('p.png', 'd')),
                              pair=file('chat/desktop/pairs/composer.png'))])


def judgement(place='mockup', id='color', signature='style/composer/color'):
    d = dict(id=id, width='desktop', signatures=[signature], tag='style', place=place,
             cites='client/mockups/README.md', seen='a different color')
    if place == 'mockup':
        d['finding'] = dict(**{'class': 'look'}, title='Match color', owner='o/web-look')
    elif place == 'silent':
        d['question'] = dict(options=['Keep it', 'Change it'], recommendation='Change it')
    else:
        d['departures'] = ['accepted-color']
    return d


def judgements():
    return dict(format='look-judgements/1', scene='chat',
                widths=[dict(name='desktop', pictures=[dict(digest=look.picture_digest(width()), seen='judged')])],
                differences=[judgement()])


class BarTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.lab = self.root / 'lab'
        self.experiment = self.root / 'experiment'
        self.judged = self.root / 'judgements'
        self.judged.mkdir()
        self.write(self.experiment / 'experiment.json', dict(name='look', planted=[]))
        self.make_key('key', [[SIG], [SIG], [SIG]])

    def write(self, path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(look.canonical(value))

    def make_key(self, key, signatures, undecided=()):
        root = self.lab / 'keys/look' / key
        self.write(root / 'verdict.json', dict(seeds=[dict(seed=str(i), judged=[f'r{i}'])
                                                    for i in range(len(signatures))]))
        for i, sigs in enumerate(signatures):
            run = root / 'runs' / f'r{i}'
            self.write(run / 'run.json', dict(failures=[dict(signature=s) for s in sigs], undecided=list(undecided)))
            capture = run / 'collected/capture/capture'
            self.write(capture / 'capture.json', dict(format='look-capture/1', scenes=[dict(name='chat', widths=[width()])]))
            for path in ('chat/desktop/pair.png', 'chat/desktop/pairs/composer.png'):
                p = capture / path
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(b'picture')

    def judge(self, value=None):
        self.write(self.judged / 'chat.json', judgements() if value is None else value)

    def bar(self, key='key', healthy=None):
        return look.bar(self.lab, self.experiment, key, self.judged, healthy)

    def bar_exit(self):
        with redirect_stdout(io.StringIO()):
            return look.main(['bar', '--lab', str(self.lab), '--key', 'key',
                              '--judgements', str(self.judged), str(self.experiment)])

    def test_bar_exit_without_planted(self):
        self.assertFalse(self.bar()[1])
        self.assertEqual(self.bar_exit(), 1)
        self.judge()
        self.assertTrue(self.bar()[1])
        self.assertEqual(self.bar_exit(), 0)

    def test_picture_digest_refusal_and_missing_part(self):
        w = width()
        w['mockup'] = None
        w['regions'][0]['mockup'] = dict(shot=dict(refused='limit', detail='too large'))
        w['regions'][0]['page'] = dict(found=False)
        expected = [None, 'sha256:' + 'b' * 64, None, None]
        self.assertEqual(look.picture_digest(w), 'sha256:' + hashlib.sha256(look.canonical(expected)[:-1]).hexdigest())

    def test_served_picture_digest_and_bar_unchanged(self):
        expected = ['sha256:' + c * 64 for c in 'abcd']
        self.assertEqual(look.picture_digest(width()),
                         'sha256:' + hashlib.sha256(look.canonical(expected)[:-1]).hexdigest())
        self.judge()
        self.assertEqual(self.bar(), ('chat: r0: fail 1; r1: fail 1; r2: fail 1; same\n'
                                      'unjudged: none\n', True))

    def test_live_digest_uses_only_region_crops(self):
        w = width()
        w['page']['focus'] = dict(role='textbox', name='Message')
        expected = [None, None, 'sha256:' + 'c' * 64, 'sha256:' + 'd' * 64]
        digest = 'sha256:' + hashlib.sha256(look.canonical(expected)[:-1]).hexdigest()
        self.assertEqual(look.picture_digest(w), digest)
        w['mockup']['shot'] = file('another.png', 'e')
        w['page']['shot'] = file('later.png', 'f')
        self.assertEqual(look.picture_digest(w), digest)
        w['regions'][0]['page']['shot'] = file('changed.png', 'e')
        self.assertNotEqual(look.picture_digest(w), digest)

    def test_live_bar_batch_and_gallery_signature(self):
        source = self.lab / 'keys/look/key'
        target = self.lab / 'keys/look-live/key'
        target.parent.mkdir(parents=True)
        source.rename(target)
        self.write(self.experiment / 'experiment.json', dict(name='look-live', planted=[]))
        value = judgements()
        value['widths'][0]['pictures'][0]['digest'] = look.picture_digest(
            dict(width(), page=dict(shot=file('page.png', 'b'), focus=None)))
        value['differences'].append(judgement('mockup', 'gallery', 'gallery/missing/button/Send'))
        gallery_sig = 'check/gallery/chat/desktop/gallery/missing/button/Send'
        for run in sorted((target / 'runs').iterdir()):
            old = run / 'collected/capture/capture'
            live = run / 'collected/look/chat'
            live.parent.mkdir(parents=True)
            old.rename(live)
            capture = json.loads((live / 'capture.json').read_bytes())
            capture['scenes'][0]['widths'][0]['page']['focus'] = None
            self.write(live / 'capture.json', capture)
            # Another capture proves the scene list comes from all records.
            self.write(run / 'collected/look/closing/capture.json', dict(format='look-capture/1',
                scenes=[dict(name='closing', widths=[width()])]))
            self.write(run / 'run.json', dict(failures=[dict(signature=SIG), dict(signature=gallery_sig)],
                                               undecided=[]))
        self.judge(value)
        text, holds = self.bar()
        self.assertTrue(holds)
        self.assertIn('chat: r0: fail 2;', text)
        self.assertIn('closing: r0: pass;', text)
        self.assertIn('unjudged: none', text)
        text = look.batch(self.judged, self.lab, 'look-live', 'key')
        self.assertIn(str(target / 'runs/r0/collected/look/chat/chat/desktop/pair.png'), text)
        self.assertIn('gallery/missing/button/Send', text)

    def test_finding_judges_signature(self):
        self.judge()
        self.assertIn('unjudged: none', self.bar()[0])

    def test_unsigned_finding_judges_nothing_and_prints_none(self):
        value = judgements()
        value['differences'][0]['signatures'] = []
        self.judge(value)
        text, holds = self.bar()
        self.assertIn(SIG + ' — no difference (3 runs)', text)
        self.assertFalse(holds)
        text = look.batch(self.judged, self.lab, 'look', 'key')
        self.assertIn('  Signatures: none\n', text)
        self.assertIn('  Pair: ', text)

    def test_question_judges_signature(self):
        value = judgements()
        value['differences'] = [judgement('silent')]
        self.judge(value)
        self.assertIn('unjudged: none', self.bar()[0])

    def test_unjudged_departure_gone(self):
        self.make_key('key', [['check/look-style/chat/desktop/departure-gone/old']])
        self.assertIn('— departure-gone (1 runs)', self.bar()[0])

    def test_words_signature_is_a_look_check(self):
        self.make_key('key', [['check/look-words/chat/desktop/words/banner/missing/Live']])
        self.assertIn('— not judged (1 runs)', self.bar()[0])

    def test_unjudged_not_a_look_check(self):
        self.make_key('key', [['panic/gallery']])
        self.assertIn('— not a look check (1 runs)', self.bar()[0])

    def test_unjudged_not_judged(self):
        self.assertIn(SIG + ' — not judged (3 runs)', self.bar()[0])

    def test_unjudged_width_not_judged(self):
        value = judgements()
        value['widths'][0]['name'] = 'phone'
        value['differences'] = []
        self.judge(value)
        self.assertIn('— not judged (3 runs)', self.bar()[0])

    def test_unjudged_pictures_moved(self):
        value = judgements()
        value['widths'][0]['pictures'][0]['digest'] = 'sha256:' + '0' * 64
        self.judge(value)
        self.assertIn('— pictures moved (3 runs)', self.bar()[0])

    def test_unjudged_no_difference(self):
        value = judgements()
        value['differences'] = []
        self.judge(value)
        self.assertIn('— no difference (3 runs)', self.bar()[0])

    def test_unjudged_still_failing(self):
        value = judgements()
        value['differences'] = [judgement('page')]
        self.judge(value)
        self.assertIn('— still failing (3 runs)', self.bar()[0])

    def test_unjudged_undecided(self):
        self.make_key('key', [[], []], [dict(check='look-style', reason='shot missing')])
        text, _ = self.bar()
        self.assertEqual(text.count('look-style: shot missing — undecided (2 runs)'), 1)

    def test_undecided_beside_failure_from_same_check(self):
        self.make_key('key', [[SIG]], [dict(check='look-style', reason='shot missing')])
        text, holds = self.bar()
        self.assertIn(SIG + ' — not judged (1 runs)', text)
        self.assertIn('look-style: shot missing — undecided (1 runs)', text)
        self.assertFalse(holds)
        self.assertEqual(self.bar_exit(), 1)

    def test_unjudged_none(self):
        self.make_key('key', [[], [], []])
        self.assertIn('unjudged: none', self.bar()[0])
        self.assertTrue(self.bar()[1])
        self.assertEqual(self.bar_exit(), 0)

    def test_scene_verdict_same(self):
        self.assertIn('chat: r0: fail 1; r1: fail 1; r2: fail 1; same', self.bar()[0])

    def test_scene_verdict_differs(self):
        self.make_key('key', [[SIG], [OTHER], []])
        value = judgements()
        value['differences'].append(judgement('silent', 'order', 'order/composer'))
        self.judge(value)
        self.assertIn('chat: r0: fail 1; r1: fail 1; r2: pass; differs', self.bar()[0])
        self.assertIn('unjudged: none', self.bar()[0])
        self.assertFalse(self.bar()[1])
        self.assertEqual(self.bar_exit(), 1)

    def test_planted_requires_same_scene_signatures(self):
        self.write(self.experiment / 'experiment.json', dict(name='look', planted=[SIG]))
        self.assertTrue(self.bar()[1])
        self.assertEqual(self.bar_exit(), 0)
        self.make_key('key', [[SIG], [SIG, OTHER], [SIG]])
        text, holds = self.bar()
        self.assertEqual(text.count('planted 1/1'), 3)
        self.assertIn('; differs', text)
        self.assertFalse(holds)
        self.assertEqual(self.bar_exit(), 1)

    def test_all_judged_runs_in_seed_order(self):
        path = self.lab / 'keys/look/key/verdict.json'
        self.write(path, dict(seeds=[dict(judged=['r2', 'r0']), dict(judged=['r1'])]))
        self.assertIn('r2: fail 1; r0: fail 1; r1: fail 1', self.bar()[0])

    def test_planted_missing(self):
        self.write(self.experiment / 'experiment.json', dict(name='look', planted=[SIG, OTHER]))
        text, holds = self.bar()
        self.assertFalse(holds)
        self.assertEqual(text.count('planted 1/2'), 3)
        self.assertIn('missing ' + OTHER, text)
        self.assertNotIn('unjudged', text)

    def compare(self):
        self.write(self.experiment / 'experiment.json', dict(name='look', planted=[SIG]))
        self.make_key('healthy', [[OTHER], [OTHER], [OTHER]])
        return self.bar(healthy='healthy')[0]

    def test_planted_minus_healthy(self):
        text = self.compare()
        self.assertEqual(text.count('planted − healthy: 1\n  ' + SIG), 3)

    def test_healthy_minus_planted(self):
        text = self.compare()
        self.assertEqual(text.count('healthy − planted: 1\n  ' + OTHER), 3)

    def test_invalid_judgements_are_errors(self):
        (self.judged / 'chat.json').write_bytes(b'{}\n')
        with self.assertRaisesRegex(ValueError, 'judgements.format'):
            self.bar()

    def test_batch_sections_owner_and_paths(self):
        value = judgements()
        value['differences'] += [judgement('silent', 'order', 'order/composer'), judgement('page', 'extra', 'extra/button')]
        self.judge(value)
        text = look.batch(self.judged, self.lab, 'look', 'key')
        self.assertLess(text.index('## Findings'), text.index('## Questions'))
        self.assertLess(text.index('## Questions'), text.index('## Notes for the user'))
        for expected in ('### o/web-look', 'chat/color', 'desktop; style; look', 'Match color',
                         '1. Keep it', '2. Change it', 'Recommendation: Change it',
                         'chat/order', 'chat/extra', 'client/mockups/README.md', 'style/composer/color'):
            self.assertIn(expected, text)
        paths = [line.removeprefix('  Pair: ') for line in text.splitlines() if line.startswith('  Pair: ')]
        self.assertTrue(paths)
        self.assertTrue(all(Path(p).is_absolute() and Path(p).is_file() for p in paths))
        self.assertTrue(any(p.endswith('/pairs/composer.png') for p in paths))

    def test_canon_rewrites_private(self):
        path = self.root / 'input.json'
        path.write_text(json.dumps(judgements(), indent=2))
        self.assertEqual(look.main(['canon', str(path)]), 0)
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(path.read_bytes(), look.canonical(judgements()))

    def page_capture(self):
        capture = self.lab / 'keys/look/key/runs/r0/collected/capture/capture'
        value = json.loads((capture / 'capture.json').read_bytes())
        w = value['scenes'][0]['widths'][0]
        for record in [w] + w['regions']:
            record['mockup'] = None
            del record['pair']
        w['page']['shot'] = file('chat/desktop/page/page.png')
        w['regions'][0]['page']['shot'] = file('chat/desktop/page/composer.png')
        w['regions'].append(dict(name='unrelated', page=dict(shot=file('missing.png'))))
        for record in [w, w['regions'][0]]:
            path = capture / record['page']['shot']['path']
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'picture')
        return capture, value

    def test_batch_page_pictures_without_mockup(self):
        capture, value = self.page_capture()
        self.write(capture / 'capture.json', value)
        self.judge()
        text = look.batch(self.judged, self.lab, 'look', 'key')
        paths = [line.removeprefix('  Page: ') for line in text.splitlines() if line.startswith('  Page: ')]
        self.assertEqual(paths, [str(capture / 'chat/desktop/page/page.png'),
                                 str(capture / 'chat/desktop/page/composer.png')])
        self.assertTrue(all(Path(p).is_file() for p in paths))
        self.assertNotIn('  Pair: ', text)

    def test_batch_refuses_page_path_outside_capture_or_missing(self):
        capture, value = self.page_capture()
        outside = self.root / 'outside.png'
        outside.write_bytes(b'picture')
        (capture / 'link.png').symlink_to(outside)
        w = value['scenes'][0]['widths'][0]
        self.judge()
        for record in [w, w['regions'][0]]:
            original = record['page']['shot']['path']
            for path in (str(outside), '../../../../../outside.png', 'link.png', 'missing.png', '.'):
                with self.subTest(record=record['name'], path=path):
                    record['page']['shot']['path'] = path
                    self.write(capture / 'capture.json', value)
                    with self.assertRaisesRegex(ValueError, 'page path missing or outside capture'):
                        look.batch(self.judged, self.lab, 'look', 'key')
            record['page']['shot']['path'] = original

    def test_batch_entries_under_finding_and_question(self):
        value = judgements()
        value['differences'].append(judgement('silent', 'order', 'order/composer'))
        for difference, name in zip(value['differences'], ('color', 'order')):
            difference['departures'] = [name + '-departure']
            difference['expectations'] = [name + '-expectation']
        self.judge(value)
        text = look.batch(self.judged, self.lab, 'look', 'key')
        findings, questions = text.split('## Questions')
        self.assertIn('  entries: color-departure, color-expectation\n', findings)
        self.assertIn('  entries: order-departure, order-expectation\n', questions)


class JudgementTests(unittest.TestCase):
    def test_empty_signatures_only_for_mockup(self):
        for place in ('mockup', 'page', 'silent'):
            with self.subTest(place=place):
                value = judgements()
                value['differences'] = [judgement(place)]
                value['differences'][0]['signatures'] = []
                if place == 'mockup':
                    self.assertEqual(look.load_judgements(look.canonical(value)), value)
                else:
                    with self.assertRaisesRegex(ValueError, 'signatures: expected at least 1 entries'):
                        look.load_judgements(look.canonical(value))

    def test_experiment_class_accepted_other_class_refused(self):
        value = judgements()
        value['differences'][0]['finding']['class'] = 'experiment'
        self.assertEqual(look.load_judgements(look.canonical(value)), value)
        value['differences'][0]['finding']['class'] = 'other'
        with self.assertRaisesRegex(ValueError, 'finding.class'):
            look.load_judgements(look.canonical(value))

    def test_entries_on_mockup_and_silent(self):
        for place in ('mockup', 'silent'):
            for member in ('departures', 'expectations'):
                with self.subTest(place=place, member=member):
                    value = judgements()
                    value['differences'] = [judgement(place)]
                    value['differences'][0][member] = ['accepted-part']
                    self.assertEqual(look.load_judgements(look.canonical(value)), value)
                    value['differences'][0][member] = ['Bad']
                    with self.assertRaisesRegex(ValueError, member):
                        look.load_judgements(look.canonical(value))

    def test_load_judgements_refusals(self):
        rows = [
            (('format',), 'other', 'format'), (('scene',), 'Chat', 'scene'),
            (('scene',), 5, 'scene'), (('scene',), None, 'scene'),
            (('widths',), {}, 'widths'), (('widths',), [], 'widths'),
            (('widths', 0, 'name'), 'bad_name', 'widths.0.name'),
            (('widths', 0, 'pictures'), [], 'pictures'),
            (('widths', 0, 'pictures', 0, 'digest'), 'sha256:' + 'A'*64, 'digest'),
            (('widths', 0, 'pictures', 0, 'seen'), '', 'seen'),
            (('differences',), False, 'differences'),
            (('differences', 0, 'id'), 'Bad', '.id'),
            (('differences', 0, 'width'), 'phone', '.width'),
            (('differences', 0, 'signatures'), ['a b'], 'signatures.0'),
            (('differences', 0, 'signatures'), ['/a'], 'signatures.0'),
            (('differences', 0, 'signatures'), [7], 'signatures.0'),
            (('differences', 0, 'tag'), 'unknown', '.tag'),
            (('differences', 0, 'place'), 'unknown', '.place'),
            (('differences', 0, 'cites'), '', '.cites'),
            (('differences', 0, 'seen'), '', '.seen'),
            (('differences', 0, 'finding', 'class'), 'other', 'finding.class'),
            (('differences', 0, 'finding', 'title'), '', 'finding.title'),
            (('differences', 0, 'finding', 'owner'), '', 'finding.owner'),
            (('differences', 0, 'expectations'), [], 'expectations'),
            (('differences', 0, 'expectations'), ['Bad'], 'expectations.0'),
        ]
        for path, replacement, expected in rows:
            with self.subTest(path=path, replacement=replacement):
                value = judgements()
                at = value
                for part in path[:-1]:
                    at = at[part]
                at[path[-1]] = replacement
                with self.assertRaisesRegex(ValueError, expected):
                    look.load_judgements(look.canonical(value))
        for path in (('scene',), ('widths', 0, 'pictures'), ('differences', 0, 'finding', 'owner')):
            with self.subTest(missing=path):
                value = judgements()
                at = value
                for part in path[:-1]:
                    at = at[part]
                del at[path[-1]]
                with self.assertRaisesRegex(ValueError, str(path[-1]) + ': missing'):
                    look.load_judgements(look.canonical(value))
        for member in ('widths', 'differences'):
            value = judgements()
            value[member].append(copy.deepcopy(value[member][0]))
            with self.subTest(repeated=member), self.assertRaisesRegex(ValueError, 'repeated'):
                look.load_judgements(look.canonical(value))
        value = judgements()
        value['widths'][0]['pictures'] *= 2
        with self.assertRaisesRegex(ValueError, 'digest: repeated'):
            look.load_judgements(look.canonical(value))
        value = judgements()
        value['differences'].append(judgement(id='second'))
        with self.assertRaisesRegex(ValueError, 'signatures.0: repeated'):
            look.load_judgements(look.canonical(value))
        value = judgements()
        value['extra'] = 1
        with self.assertRaisesRegex(ValueError, 'extra: unknown'):
            look.load_judgements(look.canonical(value))
        with self.assertRaisesRegex(ValueError, 'not canonical'):
            look.load_judgements(json.dumps(judgements()).encode())

    def test_place_members_refused(self):
        for place, extra in (('page', dict(finding={})), ('silent', dict(finding={})),
                             ('mockup', dict(question={})), ('page', dict(departures=[])),
                             ('silent', dict(question=dict(options=['One'], recommendation='')))):
            value = judgements()
            value['differences'] = [judgement(place)]
            value['differences'][0].update(extra)
            with self.subTest(place=place, extra=extra), self.assertRaises(ValueError):
                look.load_judgements(look.canonical(value))

    def test_member_shapes_and_required_fields(self):
        for place in ('mockup', 'page', 'silent'):
            base = judgements()
            base['differences'] = [judgement(place)]
            paths = [(), ('widths', 0), ('widths', 0, 'pictures', 0), ('differences', 0)]
            if place in ('mockup', 'silent'):
                paths.append(('differences', 0, 'finding' if place == 'mockup' else 'question'))
            for path in paths:
                obj = base
                for part in path:
                    obj = obj[part]
                required = set(obj) - {'departures', 'expectations'}
                for field in sorted(required | {'unexpected'}):
                    with self.subTest(place=place, path=path, field=field):
                        value = copy.deepcopy(base)
                        at = value
                        for part in path:
                            at = at[part]
                        if field == 'unexpected':
                            at[field] = 'extra'
                        else:
                            del at[field]
                        with self.assertRaisesRegex(ValueError, field):
                            look.load_judgements(look.canonical(value))
                for replacement in (False, [], 'text'):
                    with self.subTest(place=place, path=path, replacement=replacement):
                        value = copy.deepcopy(base)
                        if not path:
                            value = replacement
                        else:
                            at = value
                            for part in path[:-1]:
                                at = at[part]
                            at[path[-1]] = replacement
                        with self.assertRaises(ValueError):
                            look.load_judgements(look.canonical(value))

    def test_question_text_rules(self):
        for field, replacement in (('options', 'two'), ('options', ['one', '']),
                                   ('recommendation', 7), ('recommendation', '')):
            value = judgements()
            value['differences'] = [judgement('silent')]
            value['differences'][0]['question'][field] = replacement
            with self.subTest(field=field, replacement=replacement), self.assertRaisesRegex(ValueError, field):
                look.load_judgements(look.canonical(value))

    def test_page_lists_need_one_name_between_them(self):
        value = judgements()
        value['differences'] = [judgement('page')]
        value['differences'][0]['expectations'] = []
        self.assertEqual(look.load_judgements(look.canonical(value)), value)
        value['differences'][0]['departures'] = []
        with self.assertRaisesRegex(ValueError, 'at least one name'):
            look.load_judgements(look.canonical(value))

    def test_all_problems_reported(self):
        value = judgements()
        value['scene'] = 'Bad'
        value['differences'][0]['cites'] = ''
        value['differences'][0]['finding']['owner'] = ''
        with self.assertRaises(ValueError) as raised:
            look.load_judgements(look.canonical(value))
        for member in ('judgements.scene', 'differences.0.cites', 'finding.owner'):
            self.assertIn(member, str(raised.exception))


class BlindTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.directory = self.root / 'views'
        self.directory.mkdir()
        (self.directory / 'desktop.png').write_bytes(b'picture')
        self.transcript = self.root / 'stream.jsonl'

    def check(self, name='Read', path=None, cwd=None, tools=None):
        events = [dict(type='system', subtype='init', cwd=str(cwd or self.directory),
                       tools=['Read'] if tools is None else tools),
                  dict(type='assistant', message=dict(role='assistant', content=[
                      dict(type='tool_use', id='toolu_1', name=name,
                           input=dict(file_path=str(path or self.directory / 'desktop.png')))])),
                  dict(type='user', message=dict(role='user', content=[
                      dict(type='tool_result', tool_use_id='toolu_1', content='picture')])),
                  dict(type='result', subtype='success', result='a difference')]
        self.transcript.write_bytes(b''.join(look.canonical(e) for e in events))
        return look.blind(self.directory, self.transcript, ['planted'])

    def test_blind_passes(self):
        text, holds = self.check()
        self.assertTrue(holds)
        self.assertIn('Read: ' + str(self.directory / 'desktop.png'), text)

    def test_blind_forbidden_name(self):
        (self.directory / 'PlAnTeD.png').write_bytes(b'x')
        text, holds = self.check()
        self.assertFalse(holds)
        self.assertIn('forbidden word planted', text)

    def test_blind_directory_own_path(self):
        self.check()
        self.assertFalse(look.blind(self.directory, self.transcript, ['views'])[1])

    def test_blind_other_tool(self):
        text, holds = self.check(name='Bash')
        self.assertFalse(holds)
        self.assertIn('tool Bash', text)

    def test_blind_path_outside(self):
        self.assertFalse(self.check(path=self.root / 'outside.png')[1])

    def test_blind_symlink_outside(self):
        (self.directory / 'link').symlink_to(self.root / 'outside.png')
        self.assertFalse(self.check(path=self.directory / 'link')[1])

    def test_blind_wrong_cwd(self):
        self.assertFalse(self.check(cwd=self.root)[1])

    def test_blind_wrong_tools(self):
        self.assertFalse(self.check(tools=['Read', 'Bash'])[1])

    def test_blind_missing_init(self):
        self.transcript.write_text('')
        self.assertFalse(look.blind(self.directory, self.transcript, ['planted'])[1])


if __name__ == '__main__':
    unittest.main()
