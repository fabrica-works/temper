"""Private pure judgment for qa.md, 8.4; deliberately no wire adapter or CLI.

The observation projection reads the landed skein writer's JSON lines.
The eventual adapter must supply an owner-confirmed flush/stop boundary and
complete objects from the private git `objects` route.
"""

import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from enum import IntEnum

_NOTICE = b"Maintained by temper.\n"
_NOT_LANDED = "not-landed/fake/widget/main"
_CONTENT = "content/fake/widget/main/NOTICE"
_CHANGED = "changed/fake/widget/main"


class _Status(IntEnum):
    PASS = 0
    FAIL = 1
    UNDECIDED = 2


@dataclass(frozen=True)
class _Object:
    """Private projection of the fake's Object, with a whole byte tree."""

    parent: int | None
    merge_parent: int | None
    tree: Mapping[bytes, bytes] | None


@dataclass(frozen=True)
class _Verdict:
    status: _Status
    signatures: tuple[str, ...] = ()
    reason: str = ""
    evidence: tuple[str, ...] = ()


def _count(value: object) -> bool:
    return type(value) is int and 0 < value <= (1 << 64) - 1


def _observation_members(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("repeated observation member")
        result[key] = value
    return result


def _observation_number(text):
    value = int(text)
    if text.startswith("-") or value >= 1 << 64:
        raise ValueError("number is not a u64")
    return value


def _project_observations(data: bytes | None) -> dict:
    """Pure end-state inputs for _judge; neither EOF nor a checkpoint proves flush.

    Source: skein fbc771fde2a784d3b95e7a03460c4ccc56829c9d,
    crates/skein-fake-forge-protocol/src/observations{.rs,/read.rs,/writer.rs},
    fake-forge.md, 2.4. Validate the moved/deleted/merged records used here;
    other official tags are ignored, without decoding their content or judging
    promises. Content is base64, names UTF-8, numbers full u64 JSON integers.
    """
    complete = data is not None and data.endswith(b"\n")
    checkpoint = False
    losses = []
    merged = moved = None
    tags = {
        "moved", "deleted", "opened", "closed", "reopened", "labelled",
        "revised", "depends", "requested", "defined", "commented", "edited",
        "removed", "reviewed", "reported", "merged", "enqueued", "group",
        "dequeued", "refused", "rejected", "wiki",
    }
    fields = {
        "moved": {"observation", "repository", "branch", "from", "to", "by", "forward"},
        "deleted": {"observation", "repository", "branch", "at", "by"},
        "merged": {"observation", "repository", "number", "base", "head", "commit", "by"},
    }
    try:
        for line in (data or b"").split(b"\n")[:-1]:
            checkpoint = False
            # Explicit UTF-8 avoids json.loads(bytes)' UTF-16/32 auto-detection.
            # int rejects fractional/exponent numbers and non-JSON constants.
            record = json.loads(line.decode("utf-8"), object_pairs_hook=_observation_members,
                                parse_int=_observation_number, parse_float=int, parse_constant=int)
            if type(record) is not dict:
                raise ValueError("observation is not an object")
            if "observations_lost" in record:
                loss = record["observations_lost"]
                if set(record) != {"observations_lost"} or type(loss) is not int or not 0 <= loss < 1 << 64:
                    raise ValueError("unknown loss")
                losses.append(loss)
                checkpoint = True
                continue
            tag = record.get("observation")
            repository = record.get("repository")
            if type(tag) is not str or tag not in tags or type(repository) is not str:
                raise ValueError("unknown observation")
            repository.encode("utf-8")
            if tag not in fields:
                continue
            if set(record) != fields[tag]:
                raise ValueError("malformed end-state observation")
            branch = record["base" if tag == "merged" else "branch"]
            if type(branch) is not str:
                raise ValueError("branch is not a name")
            branch.encode("utf-8")
            numbers = {"merged": ("number", "head", "commit", "by"),
                       "moved": ("to", "by"), "deleted": ("at", "by")}[tag]
            if tag == "moved":
                if record["from"] is not None:
                    numbers += ("from",)
                if record["forward"] is not None and type(record["forward"]) is not bool:
                    raise ValueError("forward is not a flag")
            if any(type(record[key]) is not int or not 0 <= record[key] < 1 << 64 for key in numbers):
                raise ValueError("count is not a u64")
            if repository == "fake/widget" and branch == "main":
                if tag == "merged":
                    merged = record["commit"]
                else:
                    moved = record["to"] if tag == "moved" else None
    except (ValueError, UnicodeError, RecursionError):
        complete = False
    return dict(complete_lines=complete and checkpoint, loss_counts=tuple(losses),
                last_merged=merged, last_moved=moved)


def _tree_valid(tree: Mapping[bytes, bytes] | None) -> bool:
    return isinstance(tree, Mapping) and all(
        type(path) is bytes and type(content) is bytes for path, content in tree.items()
    )


def _judge(
    *,
    complete_lines: bool | None,
    flushed: bool | None,
    loss_counts: Sequence[int] | None,
    last_merged: int | None,
    last_moved: int | None,
    objects: Mapping[int, _Object] | None,
    notice: bytes,
) -> _Verdict:
    """No IO, time, credentials or temper documents; unknown never means pass.

    Unavailable trees/objects must stay `None`, never filled with
    empty maps on read errors. Empty trees here mean known empty trees.
    `flushed` means the owner's complete pre-stop boundary was obtained;
    an EOF alone cannot establish it. Loss zero must be explicit.
    """
    if loss_counts is not None and any(type(n) is int and n > 0 for n in loss_counts):
        return _Verdict(_Status.UNDECIDED, reason="observations lost")
    if (
        complete_lines is not True
        or flushed is not True
        or not loss_counts
        or any(type(n) is not int or n != 0 for n in loss_counts)
    ):
        return _Verdict(_Status.UNDECIDED, reason="observations missing")
    # A broken reference file is an unavailable oracle, never a new expectation.
    if notice != _NOTICE:
        return _Verdict(_Status.UNDECIDED, reason="NOTICE reference invalid")
    if last_merged is None or last_moved is None:
        return _Verdict(_Status.FAIL, (_NOT_LANDED,))
    if not _count(last_merged) or not _count(last_moved):
        return _Verdict(_Status.UNDECIDED, reason="observations missing")
    if last_merged != last_moved:
        return _Verdict(_Status.FAIL, (_NOT_LANDED,))
    commit = last_merged
    evidence = (f"C={commit}",)
    if objects is None or commit not in objects:
        return _Verdict(_Status.UNDECIDED, reason="store unreadable", evidence=evidence)
    current = objects[commit]
    parent = current.parent
    evidence += (f"P={parent}",)
    if parent is None:
        return _Verdict(_Status.FAIL, (_CHANGED,), evidence=evidence)
    if not _count(parent) or parent not in objects:
        return _Verdict(_Status.UNDECIDED, reason="store unreadable", evidence=evidence)
    previous = objects[parent]
    if not _tree_valid(current.tree) or not _tree_valid(previous.tree):
        return _Verdict(_Status.UNDECIDED, reason="store unreadable", evidence=evidence)
    evidence += (
        f"P paths={sorted(previous.tree)!r}",
        f"C paths={sorted(current.tree)!r}",
        f"P NOTICE={previous.tree.get(b'NOTICE')!r}",
        f"C NOTICE={current.tree.get(b'NOTICE')!r}",
    )
    failures = []
    if current.tree.get(b"NOTICE") != notice:
        failures.append(_CONTENT)
    # The fake's squash has one parent. No permissions or file kinds are
    # invented: its actual Tree maps byte paths directly to byte contents.
    if (
        parent == commit
        or current.merge_parent is not None
        or b"NOTICE" in previous.tree
        or b"NOTICE" not in current.tree
        or {p: b for p, b in current.tree.items() if p != b"NOTICE"} != previous.tree
    ):
        failures.append(_CHANGED)
    return _Verdict(_Status.FAIL if failures else _Status.PASS, tuple(failures), evidence=evidence)
