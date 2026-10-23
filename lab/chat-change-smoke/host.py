"""Attended S (qa.md): canonical inputs, native trees and private sign-in.

Python is the approved lab-tooling exception to programming-model.md, 1.
One poll loop owns every child's IO and lifecycle; runner.host owns cgroups.
CLI: host --skein SOURCE --lab PRIVATE_TMP_LAB --experiment DIRECTORY
          --subject sha256:... --seed DECIMAL
One explicit seed per invocation. No retries, driver, faults or campaign.
"""
import argparse
from datetime import datetime, timezone
import importlib
import json
import os
from pathlib import Path
import secrets
import signal
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _host_output import Output, credential, origin
from _host_native import launch

HOST_FILES = ('host', 'host.py', '_host_output.py', '_host_native.py')
ACTION_NAMES = ['sign-in', 'add-key', 'usable', 'forge', 'adopt', 'story']


class Halt(RuntimeError):
    """Nonsecret control failure, safe to retain by its fixed message."""


def bind_skein(source):
    global canonical, checks, cli, draws, evidence, experiment, judge, native, store
    scripts = source / 'scripts/lab'
    sys.path.insert(0, str(scripts))
    modules = [importlib.import_module('runner.' + name) for name in
               ('canonical', 'checks', 'cli', 'draws', 'evidence', 'experiment', 'judge', 'host', 'store')]
    canonical, checks, cli, draws, evidence, experiment, judge, native, store = modules
    if Path(native.__file__).resolve() != (scripts / 'runner/host.py').resolve():
        raise Halt('selected skein source differs from loaded runner')


def utc():
    return datetime.now(timezone.utc).isoformat(timespec='milliseconds').replace('+00:00', 'Z')


def host_files():
    root = Path(__file__).resolve().parent
    return [dict(path='.attended/' + name, data=store.read_regular(root / name)[0],
                 exec=bool((root / name).stat().st_mode & 0o100)) for name in HOST_FILES]


def host_digest():
    return canonical.tree_digest(store.entries(host_files()))


def require(condition, message):
    if not condition:
        raise Halt(message)


def contract(exp, settings):
    """An intentionally narrow contract over the canonical S manifest."""
    require(exp.get('driver') is None and exp.get('faults') == [] and exp.get('planted') == [],
            'attended host accepts S only (no driver, faults or planted signatures)')
    require([a['name'] for a in exp['actions']] == ACTION_NAMES, 'S seed/story order differs')
    require({p['name'] for p in exp['processes']} == {'forge', 'llm', 'engine', 'w1', 'w2'},
            'S requires forge, llm, engine and two workers')
    for process in exp['processes']:
        name = process['name']
        if name in ('engine', 'w1', 'w2'):
            argv = ['{artifact.temper}', 'serve', '--config', '{proc.' + name + '.dir}/config',
                    '--data', '{proc.' + name + '.dir}/data', '--cache', '{proc.' + name + '.dir}/cache']
        else:
            argv = ['{artifact.temper}', 'fake', name, '--document', '{proc.' + name + '.dir}/' + name + '.json',
                    '--ready', 'listening ', '--seed', '{seed.' + name + '}']
            if name == 'forge':
                argv += ['--observations', '{proc.forge.dir}/observations.jsonl']
        require(process['argv'] == argv, 'S process argv differs from the approved cluster contract')
        if name == 'engine':
            ready = [dict(stream='stderr', prefix='listening ' + listener + ' ', capture=listener)
                     for listener in ('clients', 'workers', 'webhooks')]
        elif name in ('w1', 'w2'):
            ready = [dict(stream='stderr', prefix='linked ' + name + ' ', capture='engine')]
        else:
            ready = [dict(stream='stdout', prefix='listening ', capture='address')]
        require(process['ready'] == ready, 'S requires only the declared public readiness rules')
    require([c['name'] for c in exp['collect']] == ['observations', 'store'], 'S collection order differs')
    require({c['name'] for c in exp['checks']} == {'end-state', 'promises'}, 'S checks differ')
    require('client' in exp['dirs'], 'S client directory missing')
    require(exp['cost'] == 'temper-m1-calibration', 'S cost class differs from approved calibration')
    cost = settings['costs'][exp['cost']]
    require(tuple(cost[k] for k in ('ready_seconds', 'command_seconds', 'stop_seconds')) == (120, 240, 20),
            'S requires approved ready120/command240/stop20 calibration')
    for group in ('processes', 'actions', 'collect', 'checks'):
        for item in exp[group]:
            require('stdin' not in item, 'S declares no stdin; sign-in uses only the private host pipe')
            script = (group, item['name']) in (('actions', 'story'), ('checks', 'end-state'))
            require(item['env'] == {} or script and item['env'] == {'PATH': '/usr/bin:/bin'},
                    'S environment must be empty except explicit script PATH')
    signin = exp['actions'][0]
    require(signin['argv'] == ['{artifact.temper}', 'ctl', '{dir.client}', 'sign-in', '-']
            and 'stdin' not in signin, 'S sign-in declaration differs')
    prefix = ['{artifact.temper}', 'ctl', '{dir.client}']
    require(exp['actions'][1]['argv'] == prefix + ['add-key', '1', '{proc.llm.dir}/llm.key',
            '--deployment', '--key', 'lab-llm'], 'S add-key declaration differs')
    for index, tool, condition in ((2, 'accounts', 'usable'), (3, 'forges', 'connected:1')):
        argv = exp['actions'][index]['argv']
        require(argv[:7] == prefix + ['wait', tool, '--until', condition]
                and len(argv) == 9 and argv[7] == '--timeout' and argv[8].isdigit()
                and 0 < int(argv[8]) < 240, 'S wait declaration differs')
    require(exp['actions'][4]['argv'] == prefix + ['adopt', '--project', '1', '--forge', '1',
            '--owner', 'fake', '--name', 'widget', '--landing', 'main', '--key', 'lab-adopt'],
            'S adopt declaration differs')
    require(all('await' not in a for a in exp['actions']), 'S has no action await protocol')
    require('proc.engine.ready.link' not in json.dumps(exp), 'S has no credential placeholders')
    require(exp['actions'][-1]['argv'] == ['/bin/sh', '{experiment}/smoke/story',
            '{artifact.temper}', '{dir.client}', 'http://{proc.engine.ready.clients}/'],
            'story must use the complete public clients origin')
    observations, copied = exp['collect']
    require(observations['argv'] == ['/bin/cp', '{proc.forge.dir}/observations.jsonl',
            '{collect.dir}/observations.jsonl'] and observations['scan'] == [], 'S observations copy differs')
    require(copied['argv'] == ['/bin/cp', '-R', '--', '{proc.engine.dir}/data/state/store/.',
            '{collect.dir}/.'] and copied['scan'] == [], 'S stopped store copy differs')
    end = next(c for c in exp['checks'] if c['name'] == 'end-state')
    require(end['argv'] == ['{experiment}/checks/end-state',
            '{collected.observations}/observations.jsonl', '{proc.forge.ready.address}'],
            'end-state declares its three static argv; attended host supplies boundary/deadline/evidence/result')
    promises = next(c for c in exp['checks'] if c['name'] == 'promises')
    require(promises['argv'] == ['{artifact.temper}', 'fake', 'forge', 'check', '--config',
            '{proc.engine.dir}/config', '--data', '{proc.engine.dir}/data', '--cache',
            '{proc.engine.dir}/cache', '1', '{collected.observations}/observations.jsonl', '{collected.store}'],
            'S promises must read stopped engine configuration and copied store')


def prepare(lab, args):
    evidence.require_recovered(lab)
    exp, files = cli._experiment_input(args.experiment)
    settings = canonical.loads(lab.read('settings.json'))
    contract(exp, settings)
    require(not any(f['path'].startswith('.attended/') for f in files), 'reserved attended input namespace')
    files += host_files()
    identity, problems = cli.validate_command(lab, args.experiment, args.subject, inputs=(exp, files))
    require(not problems, 'canonical validation refused; use safe lab validate for input diagnostics')
    path = f"keys/{exp['name']}/{identity}"
    previous = evidence.read(lab, path)
    # Every executable shell/interpreter dependency of the fixed story is keyed.
    key = canonical.loads(lab.read(path + '/key.json'))
    programs = {p['path']: p['sha256'] for p in key['lab']['programs']}
    for program in ('/bin/sh', '/usr/bin/python3', '/usr/bin/env', '/usr/bin/systemd-run', '/usr/bin/systemctl'):
        info = cli._program(program)
        programs[info['path']] = info['sha256']
    key['lab']['programs'] = [dict(path=p, sha256=programs[p]) for p in sorted(programs)]
    identity = canonical.digest(key)
    path = f"keys/{exp['name']}/{identity}"
    lab.write_once(path + '/key.json', canonical.record_bytes(key))
    previous = evidence.read(lab, path)
    require(judge.seed_admitted(previous, args.seed), 'a fourth seed is forbidden')
    return exp, key, path, 1 + sum(r['seed'] == args.seed for r in previous)


def checkpoint_prefix(raw):
    """A complete explicit zero-loss prefix, never EOF as completion evidence."""
    end, offset, losses = 0, 0, []
    for line in raw.splitlines(keepends=True):
        if not line.endswith(b'\n'):
            break
        offset += len(line)
        try:
            record = json.loads(line.decode('utf-8'), object_pairs_hook=canonical._pairs,
                                parse_float=canonical._no_number, parse_constant=canonical._no_number)
        except (ValueError, UnicodeError):
            return b'', False
        if type(record) is not dict:
            return b'', False
        if 'observations_lost' in record:
            n = record['observations_lost']
            if set(record) != {'observations_lost'} or type(n) is not int or not 0 <= n < 1 << 64:
                return b'', False
            losses.append(n)
            end = offset
    return raw[:end], bool(losses) and all(n == 0 for n in losses)


class Run:
    def __init__(self, lab, args, exp, key, key_path, attempt):
        self.lab, self.args, self.exp, self.key = lab, args, exp, key
        now = datetime.now(timezone.utc)
        self.run_id = now.strftime('%Y%m%dT%H%M%S') + f'{now.microsecond // 1000:03d}Z-' + secrets.token_hex(3)
        self.key_path, self.attempt = key_path, attempt
        self.base = key_path + '/runs/' + self.run_id
        self.cost = key['lab']['cost']
        self.start, self.started = time.monotonic(), utc()
        self.records, self.commands, self.items = {}, [], {}
        self.failures, self.undecided, self.errors, self.bounds = [], [], [], []
        self.host, self.link, self.public = None, None, None
        self.interrupted, self.cleaning = None, False
        self.order = draws.start_order(args.seed, {p['name']: p.get('after', []) for p in exp['processes']})
        self.schedule = dict(start_order=[], faults=[], draws={})
        subject, subject_path = cli._subject(lab, key['subject'])
        exp_path = lab.root / 'store/experiments' / key['experiment']
        self.known = {'experiment': str(exp_path), 'source': str(subject_path / 'source'),
                      'runner': str(cli._runner_root()), 'run.seed': args.seed, 'run.id': self.run_id}
        self.known.update({'artifact.' + n: str(subject_path / 'artifacts' / n) for n in subject['artifacts']})
        self.known.update({'proc.' + p['name'] + '.dir': str(lab.root / self.base / 'scratch' / p['name']) for p in exp['processes']})
        self.known.update({'dir.' + n: str(lab.root / self.base / 'dirs' / n) for n in exp['dirs']})
        self.known.update({'collected.' + c['name']: str(lab.root / self.base / 'collected' / c['name']) for c in exp['collect']})
        self.texts = {f['path']: f['data'].decode('utf-8') for f in store.snapshot(exp_path)
                      if any(f['path'] == x.get('from') for p in exp['processes'] for x in p.get('files', []))}
        self.wake_r = None

    def ms(self):
        return int((time.monotonic() - self.start) * 1000)

    def event(self, event, **fields):
        fields.setdefault('ms', self.ms())
        self.lab.append_line(self.base + '/events.jsonl', dict(format='lab-event/1', event=event, **fields))

    def render(self, text):
        for placeholder, name in experiment._parts(text):
            if placeholder and name.startswith('seed.') and name not in self.known:
                value = draws.draw(self.args.seed, name[5:])
                self.schedule['draws'][name[5:]] = value
                self.known[name] = value
        return experiment.render(text, self.known)

    def write(self, path, data):
        self.lab.write_once(self.base + '/' + path, data)

    def mkdir(self, path):
        os.close(self.lab.directory(self.base + '/' + path, create=True))

    def start_child(self, group, definition, *, deadline, extra=()):
        name = definition['name']
        identity = ('proc-' if group == 'processes' else group + '-') + name
        argv = [self.render(x) for x in definition['argv']] + list(extra)
        # Recheck mutable host executables against the key before every spawn.
        for program in self.key['lab']['programs']:
            info = cli._program(program['path'])
            require(info['sha256'] == program['sha256'], 'keyed host program changed')
        programs = {p['path']: p['sha256'] for p in self.key['lab']['programs']}
        aliases = ['/bin/sh', '/usr/bin/python3', '/usr/bin/env']
        if argv[0].startswith(('/bin/', '/usr/bin/')):
            aliases.append(argv[0])
        for path in aliases:
            info = cli._program(path)
            require(programs.get(info['path']) == info['sha256'], 'keyed executable alias changed')
        require(cli._runner_digest() == self.key['lab']['runner'], 'selected runner changed')
        # A public endpoint is sufficient for the wrapper; no synthetic link.
        if name == 'story' and group == 'actions':
            try:
                origin(argv[-1])
            except ValueError:
                raise Halt('story requires an exact public loopback origin') from None
        env = {k: self.render(v) for k, v in definition['env'].items()}
        if argv[0] in ('/bin/sh',) or argv[0].endswith('/checks/end-state'):
            env = {'PATH': '/usr/bin:/bin'}
        prefix = 'proc/' + name + '/' if group == 'processes' else 'commands/' + group + '-' + name + '/'
        record = dict(name=name, group=group, argv=argv, env=env, stdin=None, started_ms=self.ms(),
                      ready_ms=None, captures={}, exit=None, output={},
                      memory=dict(peak_bytes=None, source=None, tree_peak_bytes=None,
                                  bound_bytes=definition.get('memory_bytes')))
        if group == 'processes':
            self.records[name] = record
            for entry in definition.get('files', []):
                target = 'scratch/' + name + '/' + entry['path']
                if 'tree' in entry:
                    self.lab.unpack_tree(self.base + '/' + target,
                        self.lab.read('store/subjects/' + self.key['subject'] + '/artifacts/' + entry['tree']))
                else:
                    self.write(target, self.render(self.texts[entry['from']]).encode('utf-8'))
        else:
            self.commands.append(record)
        outputs = {s: dict(path=prefix + s, fd=self.lab.create_file(self.base + '/' + prefix + s), filter=Output())
                   for s in ('stdout', 'stderr')}
        stdin_fd = os.open(os.devnull, os.O_RDONLY | os.O_CLOEXEC)
        pipe_w = None
        try:
            if group == 'actions' and name == 'sign-in':
                require(self.link is not None, 'private engine sign-in unavailable')
                os.close(stdin_fd)
                stdin_fd, pipe_w = os.pipe2(os.O_CLOEXEC)
                # The producer's fixed link is smaller than PIPE_BUF; no event
                # loop needs to keep a credential-bearing pending write.
                require(len(self.link) <= os.fpathconf(pipe_w, 'PC_PIPE_BUF'), 'private sign-in line too long')
                record['about'] = {'stdin': 'private engine link delivered once through an anonymous pipe; no stdin evidence'}
            cwd = self.known['proc.' + name + '.dir'] if group == 'processes' else str(self.lab.root / self.base)
            child = self.host.spawn(identity, argv, env, stdin_fd, cwd=cwd)
            self.items[identity] = dict(record=record, definition=definition, outputs=outputs, deadline=deadline,
                                        stopped=False, timed_out=False, empty=False, ready=set(), reaped=False)
            self.event('start', name=name, group=group, pid=child['pid'], argv=argv, env=env)
            if pipe_w is not None:
                require(os.write(pipe_w, self.link) == len(self.link), 'private sign-in pipe write incomplete')
                self.event('private_stdin', name=name, evidence=None)
        finally:
            if pipe_w is not None:
                os.close(pipe_w)
                self.link = None
            os.close(stdin_fd)
            if identity not in self.items:
                for output in outputs.values():
                    os.close(output['fd'])
        return identity

    def line(self, identity, stream, safe, secret, complete):
        item = self.items[identity]
        record = item['record']
        name = record['name']
        if secret is not None:
            if name == 'engine' and record['group'] == 'processes' and stream == 'stdout':
                require(self.public is None, 'engine emitted more than one sign-in line')
                try:
                    self.link, self.public = credential(secret)
                except ValueError:
                    raise Halt('engine emitted a malformed sign-in line') from None
                self.match_origin(item)
            return
        if not complete or record['group'] != 'processes' or record['ready_ms'] is not None:
            return
        text = safe[:-1].decode('utf-8', 'replace').removesuffix('\r')
        for rule in item['definition']['ready']:
            token = (rule['stream'], rule['prefix'], rule['capture'])
            if stream == rule['stream'] and text.startswith(rule['prefix']):
                item['ready'].add(token)
                if rule['capture'] is not None:
                    value = text[len(rule['prefix']):]
                    record['captures'][rule['capture']] = value
        self.mark_ready(item)

    def mark_ready(self, item):
        record = item['record']
        name = record['name']
        if len(item['ready']) == len(item['definition']['ready']) and record['ready_ms'] is None:
            if name == 'engine':
                self.match_origin(item)
            record['ready_ms'] = self.ms()
            for capture, value in record['captures'].items():
                self.known[f'proc.{name}.ready.{capture}'] = value
            self.event('ready', name=name, captures=record['captures'])

    def match_origin(self, item):
        clients = item['record']['captures'].get('clients')
        if clients is not None:
            try:
                public = origin('http://' + clients + '/')
            except ValueError:
                raise Halt('engine public client endpoint is invalid') from None
            require(self.public is None or public == self.public,
                    'engine public client endpoint differs from private link origin')

    def startup_ready(self, name):
        return self.records[name]['ready_ms'] is not None and (name != 'engine' or self.link is not None)

    def output(self, identity, stream):
        child, item = self.host.children[identity], self.items[identity]
        fd = child['streams'][stream]
        try:
            chunk = os.read(fd, 65536)
        except BlockingIOError:
            return
        output = item['outputs'][stream]
        for safe, secret, complete in output['filter'].feed(chunk, eof=not chunk):
            if safe:
                view = memoryview(safe)
                while view:
                    view = view[os.write(output['fd'], view):]
            self.line(identity, stream, safe, secret, complete)
        if not chunk:
            os.close(fd)
            del child['streams'][stream]

    def observe(self):
        early = False
        for identity, child in list(self.host.children.items()):
            item, record = self.items[identity], self.items[identity]['record']
            if child['exec_ack'] is not None:
                try:
                    ack = os.read(child['exec_ack'], 65536)
                except BlockingIOError:
                    ack = None
                if ack is not None:
                    require(not ack, 'child exec failed (diagnostic suppressed)')
                    os.close(child['exec_ack'])
                    child['exec_ack'] = None
            for stream in list(child['streams']):
                self.output(identity, stream)
            if not child['reaped'] and self.host.exited(identity):
                result = self.host.reap(identity)
                if result is not None:
                    record['exit'] = dict(ms=self.ms(), stopped=item['stopped'], killed=child['killed'], **result)
                    record['memory'].update(peak_bytes=child['peak_bytes'], source=child['source'])
                    self.event('exit', name=record['name'], group=record['group'], **record['exit'])
                    if record['group'] == 'processes' and not item['stopped']:
                        early = True
                    counts = self.host.counts(identity)
                    self.failures.extend(checks.survivors(record['name'], counts))
                    if counts['populated']:
                        self.event('survivors', name=record['name'], pids=counts['pids'])
                        if not child['killed']:
                            self.host.kill_tree(identity)
                            item['deadline'] = time.monotonic() + self.cost['stop_seconds']
                    if not child['killed']:
                        item['deadline'] = time.monotonic() + self.cost['stop_seconds']
            counts = self.host.counts(identity)
            record['memory']['tree_peak_bytes'] = counts['tree_peak_bytes']
            if child['reaped'] and not counts['populated'] and not child['streams'] and child['exec_ack'] is None:
                for stream, output in item['outputs'].items():
                    os.fsync(output['fd'])
                    os.close(output['fd'])
                    data = self.lab.read(self.base + '/' + output['path'])
                    record['output'][stream] = dict(path=output['path'], bytes=len(data), sha256=store.file_digest(data))
                    self.failures.extend(checks.panics(record['name'], data.decode('utf-8', 'replace'), [output['path']]))
                self.host.remove(identity)
                item['empty'] = True
                self.event('tree_empty', name=record['name'], group=record['group'])
            elif (child['reaped'] or child['killed']) and time.monotonic() >= item['deadline']:
                self.errors.append(dict(what='containment/' + record['name'], detail='reaped tree failed to empty within stop_seconds'))
                raise Halt('reaped tree failed to empty within stop_seconds')
        if early and not self.cleaning:
            raise Halt('service exited before requested stop')

    def wait(self, predicate, deadline):
        while True:
            self.observe()
            require(self.interrupted is None or self.cleaning, 'interrupted')
            if predicate():
                return True
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return False
            cleanup = [item['deadline'] - time.monotonic() for identity, item in self.items.items()
                       if identity in self.host.children and (self.host.children[identity]['reaped']
                                                                            or self.host.children[identity]['killed'])]
            if cleanup:
                remaining = min(remaining, min(cleanup))
            self.host.poll(max(1, int(remaining * 1000)), self.wake_r)
            try:
                os.read(self.wake_r, 4096)
            except BlockingIOError:
                pass

    def command(self, group, definition, *, extra=(), deadline=None):
        if deadline is None:
            deadline = time.monotonic() + self.cost['command_seconds']
        if group == 'collect':
            destination = self.known['collected.' + definition['name']]
            if definition['name'] == 'observations':
                self.mkdir('collected/observations/raw')
                destination += '/raw'
            self.known['collect.dir'] = destination
        identity = self.start_child(group, definition, deadline=deadline, extra=extra)
        item = self.items[identity]
        if not self.wait(lambda: item['empty'], deadline):
            item['timed_out'] = True
            self.bounds.append(dict(name='command_seconds', who=definition['name']))
            self.failures.append(checks.failure('bound/command_seconds/' + definition['name'], 'bound', 'command deadline passed'))
            self.stop_child(identity, kill=True)
            raise Halt('command deadline passed')
        record = item['record']
        if group == 'checks':
            failures, unknown, errors = checks.command(definition['name'], record['exit'].get('status'),
                self.lab.read(self.base + '/' + record['output']['stdout']['path']))
            self.failures.extend(failures)
            self.undecided.extend(unknown)
            self.errors.extend(errors)
        else:
            self.failures.extend(checks.action(definition['name'], record))
        return record['exit'].get('status') == 0

    def stop_child(self, identity, *, kill=False):
        if identity not in self.host.children:
            return
        item, child = self.items[identity], self.host.children[identity]
        item['stopped'] = True
        if child['killed']:
            deadline = item['deadline']
        else:
            deadline = time.monotonic() + self.cost['stop_seconds']
            if kill:
                self.host.kill_tree(identity)
                item['deadline'] = deadline
                self.event('kill_tree', name=item['record']['name'])
            else:
                self.host.signal_first(identity, signal.SIGTERM)
                self.event('signal', name=item['record']['name'], signal='SIGTERM', target='first')
        if self.wait(lambda: item['empty'], deadline):
            return
        self.bounds.append(dict(name='stop_seconds', who=item['record']['name']))
        self.failures.append(checks.failure('bound/stop_seconds/' + item['record']['name'], 'bound', 'stop deadline passed'))
        require(not child['killed'], 'killed tree did not empty; retained for unit cleanup')
        self.host.kill_tree(identity)
        item['deadline'] = time.monotonic() + self.cost['stop_seconds']
        self.event('kill_tree', name=item['record']['name'])
        # Match native runner: one grace, then one kill/empty allowance. A kill
        # that was already requested never receives a restarted allowance.
        require(self.wait(lambda: item['empty'], item['deadline']),
                'killed tree did not empty; retained for unit cleanup')

    def stop_all(self):
        self.cleaning = True
        self.link = None
        if self.host is not None:
            for identity in reversed(list(self.host.children)):
                try:
                    self.stop_child(identity)
                except BaseException as error:
                    self.errors.append(dict(what='cleanup', detail=type(error).__name__))
                    self.event('cleanup_failed', name=self.items[identity]['record']['name'])
        return self.host is not None and not self.host.children

    def collect_live(self):
        copied = self.command('collect', self.exp['collect'][0])
        require(copied, 'live observation copy failed')
        raw = self.lab.read(self.base + '/collected/observations/raw/observations.jsonl')
        prefix, zero_loss = checkpoint_prefix(raw)
        self.write('collected/observations/observations.jsonl', prefix)
        story = self.commands[-2]
        confirmed = (zero_loss and story['name'] == 'story' and story['exit'].get('status') == 0
                     and not story['exit']['stopped'] and not story['exit']['killed'])
        self.event('observation_boundary', confirmed=confirmed, loss_zero=zero_loss,
                   command='story', exit=story['exit'], final_read='ctl task --tree',
                   proof=story['output']['stdout'], prefix_bytes=len(prefix), original_bytes=len(raw),
                   producer='protocol/setup.md 8.4; fake-forge.md 2.4: flush precedes replies; jig domain/tasks.md 1: ended follows settled delegates/effects',
                   subject=self.key['subject'], runner=self.key['lab']['runner'])
        self.mkdir('collected/end-state')
        deadline = time.monotonic() + self.cost['command_seconds']
        end = next(c for c in self.exp['checks'] if c['name'] == 'end-state')
        directory = str(self.lab.root / self.base / 'collected/end-state')
        self.command('checks', end, deadline=deadline, extra=(
            '--boundary', 'confirmed' if confirmed else 'unknown', '--deadline-monotonic', str(deadline),
            '--evidence-dir', directory, '--result', directory + '/result.jsonl'))

    def stopped_check(self):
        require(not self.host.children and self.items['proc-engine']['empty'], 'store copy requires exited engine and all trees empty')
        self.event('store_copy_admitted', engine_empty=True, all_trees_empty=True)
        # Reject links in the stopped source before cp; never inspect a live store.
        source = self.known['proc.engine.dir'] + '/data/state/store'
        store.walk(source)
        if not self.command('collect', self.exp['collect'][1]):
            self.undecided.append(dict(check='promises', reason='stopped store copy failed; check skipped'))
            self.event('check_skipped', name='promises', reason='store copy failed')
            return
        self.event('store_copy_complete')
        self.command('checks', next(c for c in self.exp['checks'] if c['name'] == 'promises'))

    def finish(self):
        if self.host is not None and not self.host.children:
            for name in ('observations', 'end-state', 'store'):
                path = self.base + '/collected/' + name
                try:
                    files = store.snapshot(self.lab.root / path)
                    self.event('collection', name=name, files=store.entries(files))
                    for file in files:
                        fd = os.open(self.lab.root / path / file['path'], store.FILE_FLAGS)
                        try:
                            os.fchmod(fd, 0o500 if file['exec'] else 0o400)
                            os.fsync(fd)
                        finally:
                            os.close(fd)
                    fd = self.lab.directory(path)
                    try:
                        self.lab._freeze(fd)
                    finally:
                        os.close(fd)
                except FileNotFoundError:
                    pass
                except BaseException as error:
                    self.errors.append(dict(what='freeze/' + name, detail=type(error).__name__))
        for name, record in self.records.items():
            panic = [f for f in self.failures if f['kind'] == 'panic' and f['signature'].startswith('panic/' + name + '/')]
            failures, unknown = checks.process(name, record, panic)
            self.failures.extend(f for f in failures if f not in self.failures)
            self.undecided.extend(unknown)
        for i, failure in enumerate(self.failures):
            failure['id'] = 'failure-' + str(i + 1)
        if getattr(self, 'interrupted', None) is not None and not any(e['what'] == 'interrupted' for e in self.errors):
            self.errors.append(dict(what='interrupted', detail=self.interrupted))
        result = judge.outcome(self.failures, self.errors, self.undecided, None, driver_required=False)
        record = dict(format='lab-run/1', run=self.run_id, key=self.key_path.rsplit('/', 1)[1],
                      experiment=self.exp['name'], seed=self.args.seed, attempt=self.attempt, campaign=None,
                      started=self.started, ended=utc(), outcome=result, schedule=self.schedule,
                      processes=self.records, commands=self.commands, driver=None, failures=self.failures,
                      undecided=self.undecided, errors=self.errors, bounds=self.bounds,
                      usage=dict(elapsed_ms=self.ms(), longest_ms=None, driver=None, triage=[]),
                      about=dict(host='attended S', unit=self.args.unit, stdin='private sign-in has null stdin evidence',
                                 observation_boundary='causal S terminal read plus explicit checkpoint prefix'))
        self.event('outcome', outcome=result)
        evidence.publish(self.lab, self.key_path, record)
        if self.host is not None and not self.host.children:
            self.lab.remove_live(self.base + '/live.json')
        return record

    def run(self, admitted=None):
        parent, leaf = self.lab._parent(self.base, create=True)
        try:
            os.mkdir(leaf, 0o700, dir_fd=parent)
        finally:
            os.close(parent)
        self.mkdir('proc')
        self.write('live.json', canonical.record_bytes(dict(format='lab-live/1', unit=self.args.unit)))
        for path in ['scratch/' + p['name'] for p in self.exp['processes']] + ['dirs/' + n for n in self.exp['dirs']] + [
                'collected/' + c['name'] for c in self.exp['collect']]:
            self.mkdir(path)
        wake_r, wake_w = os.pipe2(os.O_CLOEXEC | os.O_NONBLOCK)
        self.wake_r = wake_r
        saved = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM)}
        for sig in saved:
            signal.signal(sig, lambda sig, _frame: setattr(self, 'interrupted', signal.Signals(sig).name))
        old_wake = signal.set_wakeup_fd(wake_w)
        try:
            try:
                self.host = native.Host(self.args.unit, '/usr/bin/systemctl', self.cost)
                self.event('containment', cgroup=str(self.host.root), controllers=['memory', 'pids'])
                if admitted is not None:
                    admitted()
                for name in self.order:
                    definition = next(p for p in self.exp['processes'] if p['name'] == name)
                    deadline = time.monotonic() + self.cost['ready_seconds']
                    label = 'start-order.' + str(len(self.schedule['start_order']))
                    self.schedule['draws'][label] = draws.draw(self.args.seed, label)
                    identity = self.start_child('processes', definition, deadline=deadline)
                    self.schedule['start_order'].append(name)
                    # Public readiness is recorded independently. Bootstrap must
                    # also arrive within this same ready120 window, in either
                    # stream order; no second wait or deadline is introduced.
                    if not self.wait(lambda: self.startup_ready(name), deadline):
                        self.bounds.append(dict(name='ready_seconds', who=name))
                        self.failures.append(checks.failure('bound/ready_seconds/' + name, 'bound', 'readiness deadline passed'))
                        raise Halt('readiness deadline passed')
                for action in self.exp['actions']:
                    require(self.command('actions', action), 'S seed/story action failed')
                self.collect_live()
            except BaseException as error:
                # Never interpolate arbitrary exceptions; a child may have
                # supplied a credential-looking line to any parser.
                detail = str(error) if isinstance(error, Halt) else type(error).__name__
                self.errors.append(dict(what='interrupted' if self.interrupted else 'host', detail=self.interrupted or detail))
                self.event('host_error', cause=type(error).__name__)
            empty = self.stop_all()
            if empty and 'proc-engine' in self.items and self.items['proc-engine']['empty'] and not self.interrupted:
                self.cleaning = False
                try:
                    self.stopped_check()
                except BaseException as error:
                    self.errors.append(dict(what='stopped_collection', detail=type(error).__name__))
                    self.event('check_skipped', name='promises', reason='stopped collection unavailable')
                    self.stop_all()
            elif self.interrupted:
                self.undecided.append(dict(check='promises', reason='interrupted before stopped collection'))
            elif not empty:
                self.undecided.append(dict(check='promises', reason='native tree emptiness unconfirmed; store copy skipped'))
            return self.finish()
        finally:
            self.link = None
            signal.set_wakeup_fd(old_wake)
            for sig, handler in saved.items():
                signal.signal(sig, handler)
            os.close(wake_r)
            os.close(wake_w)


def main(argv=None):
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--skein', type=Path, required=True)
    parser.add_argument('--lab', type=Path, required=True)
    parser.add_argument('--experiment', type=Path, required=True)
    parser.add_argument('--subject', required=True)
    parser.add_argument('--seed', required=True)
    for private in ('unit', 'expected-host', 'expected-runner'):
        parser.add_argument('--' + private, help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    try:
        for name in ('skein', 'lab', 'experiment'):
            setattr(args, name, getattr(args, name).absolute())
        args.host_file = Path(__file__).resolve()
        bind_skein(args.skein)
        canonical.u64(args.seed)
        if args.unit is None:
            return launch(args, native, store, host_digest(), cli._runner_digest())
        require(host_digest() == args.expected_host and cli._runner_digest() == args.expected_runner,
                'host or selected skein changed during bootstrap')
        require(native.manager('/usr/bin/systemctl', ['show', '--property=MainPID', '--value', args.unit], 240)
                == str(os.getpid()), 'worker is not the delegated unit main')
        with store.open_lab(args.lab) as lab, lab.lock() as lock_fd:
            exp, key, key_path, attempt = prepare(lab, args)
            os.pwrite(lock_fd, (args.unit + '\n').encode('ascii'), 0)
            os.fsync(lock_fd)
            run = Run(lab, args, exp, key, key_path, attempt)
            record = run.run(admitted=lambda: print('admitted', flush=True))
            print(json.dumps(dict(run=record['run'], outcome=record['outcome'],
                                  cgroup=str(run.host.root) if run.host else '')))
            if run.interrupted is not None:
                return 128 + int(getattr(signal, run.interrupted))
            return {'pass': 0, 'fail': 1, 'incomplete': 3, 'error': 4}[record['outcome']]
    except BaseException as error:
        detail = str(error) if isinstance(error, Halt) else type(error).__name__
        print('attended host: ' + detail, file=sys.stderr)
        return 4


if __name__ == '__main__':
    sys.exit(main())
