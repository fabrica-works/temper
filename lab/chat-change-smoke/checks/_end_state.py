"""Private pure judgment for qa.md, 8.4; deliberately no wire adapter or CLI.

The eventual adapter must select the LAST target Merged.commit and main
Moved.to (including a later deletion), and supply complete objects from the
private git `objects` route. Counts and byte trees match skein's actual
Observation and Object domain, not a proposed observation JSON format.
"""

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
