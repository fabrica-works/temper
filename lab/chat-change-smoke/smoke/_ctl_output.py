"""S's referee over ctl's JSON renderer, never model prose.

Sources: client/tools.md, 5.4 and 6.2; jig/render/document/{waiting,
read_sets,read_queries,read_watches,chips,tasks,endings,results}.rs.
Historical owner success is ending.task.done, even when result.value is null.
Incomplete pages are refused rather than treated as a complete tree/list.
"""
import json
import re

_OPEN_PHASES = ('waiting', 'idle', 'due', 'preparing', 'claimed', 'running', 'backing_off', 'active')


def _number(value):
    if type(value) is not int or not 0 < value < 1 << 64:
        raise ValueError('invalid task number')
    return value


def _created(text):
    match = re.fullmatch(r'done: task ([1-9][0-9]*), chat\n', text)
    if match is None:
        raise ValueError('expected done: task N, chat')
    return _number(int(match[1]))


def _unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            # tasks.rs::encode_historical_task writes the live TaskResult,
            # then the authoritative historical Result under the same key.
            # Admit only that renderer duplication, with consistent endings.
            prior = result[key]
            historical = (key == 'result' and type(value) is dict and 'ending' in value
                          and (prior is None or prior == {'document': value}
                               or prior == {'ending': value['ending'].get('task')}))
            if not historical:
                raise ValueError('duplicate JSON member')
        result[key] = value
    return result


def _json(text):
    return json.loads(text, object_pairs_hook=_unique)


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _task(document, number):
    task = document['task']
    _require(_number(task['chip']['task']) == number, 'wrong task document')
    _require(type(task['bytes_left']) is int and task['bytes_left'] == 0,
             'task document text is incomplete')
    _require(task['cut'] is False, 'task document is incomplete')
    return task


def _replied(document, root):
    task = _task(document, root)
    _require(document['met'] is True, 'root has not replied')
    _require(task['chip']['executor'] == {'agent': {'charter': 1}}, 'wrong root executor')
    _require(task['chip']['phase'] in _OPEN_PHASES,
             'root must remain Open')


def _tree(document, root, *, final=False):
    task = _task(document, root)
    tree = document['tree']
    _require(_number(tree['root']['task']) == root, 'wrong tree root')
    _require(tree['folded'] == [] and tree['more'] is False, 'incomplete or folded tree')
    _require(type(tree['held_below']) is int and tree['held_below'] == 0, 'held descendant')
    rows = tree['rows']
    _require(type(rows) is list, 'invalid tree rows')
    seen = {root}
    for row in rows:
        number = _number(row['chip']['task'])
        _require(number not in seen, 'duplicate tree task')
        seen.add(number)
        _require(row['cut'] is False, 'incomplete tree row')
        _require(row['chip']['phase'] != 'held', 'held descendant')
        _number(row['parent'])
    if final:
        _replied(dict(document, met=True), root)
        _require(tree['root']['phase'] in _OPEN_PHASES,
                 'tree root must remain Open')
        count = task['chip']['details']['owner']['details']['live_delegates']
        _require(type(count) is int and count == 0, 'live delegates remain')
        _require(all(row['chip']['phase'] in ('ended', 'done', 'failed', 'cancelled') for row in rows),
                 'live descendants remain')
    return rows


def _coding(chip):
    procedure = chip['executor'].get('procedure')
    return (type(procedure) is dict and set(procedure) == {'connector', 'kind'}
            and type(procedure['connector']) is int and procedure['connector'] == 1
            and type(procedure['kind']) is int and procedure['kind'] == 2)


def _child(tree_document, ended_document, root):
    rows = _tree(tree_document, root)
    _require(ended_document['live'] == [] and ended_document['live_after'] is None
             and ended_document['read_on'] is None, 'incomplete ended children page')
    ended = ended_document['ended']['rows']
    _require(ended['cut'] is False and ended['after'] is None, 'incomplete ended children page')
    candidates = set()
    for row in rows:
        if row['parent'] == root and _coding(row['chip']):
            candidates.add(_number(row['chip']['task']))
    for row in ended['rows']:
        _require(row['requester'] == {'task': root}, 'ended child has another parent')
        if _coding(row['chip']):
            candidates.add(_number(row['chip']['task']))
    _require(len(candidates) == 1, 'expected exactly one direct coding procedure')
    return candidates.pop()


def _landed(document, child, root):
    task = _task(document, child)
    _require(document['met'] is True, 'child wait unmet')
    _require(_coding(task['chip']), 'child is not the coding procedure')
    _require(task['requester'] == {'task': root}, 'child has another parent')
    _require(task['chip']['phase'] in ('ended', 'done'), 'child did not end successfully')
    result = task['result']
    _require(result['task']['task'] == child and _coding(result['task']), 'wrong result task')
    _require(result['task']['phase'] in ('ended', 'done'), 'result is not terminal')
    ending = result['ending']
    _require(set(ending) == {'task'} and set(ending['task']) == {'done'}, 'unsuccessful child ending')
    done = ending['task']['done']
    _require(set(done) == {'change'}, 'child result is not a landed change')
    change = done['change']
    _require(type(change['kind']) is int and change['kind'] == 1
             and type(change['part']['connector']) is int and change['part']['connector'] == 1,
             'wrong change kind or connector')
    words = change['words']
    _require(words['cut'] is False and type(words['bytes']) is str
             and re.fullmatch('(?:[0-9a-f]{40}|[0-9a-f]{64})', words['bytes']) is not None,
             'missing complete merge id')
    return words['bytes']


def proof(text):
    """Recheck the retained six-record story transcript at collection time."""
    lines = text.splitlines(keepends=True)
    _require(len(lines) == 6, 'incomplete story evidence')
    root = _created(lines[0])
    replied, tree, ended, settled, final = map(_json, lines[1:])
    _replied(replied, root)
    child = _child(tree, ended, root)
    merge = _landed(settled, child, root)
    _tree(final, root, final=True)
    return dict(root=root, child=child, merge=merge)
