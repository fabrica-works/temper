"""Read only ctl's actual text contract; no model words imply success.

Sources: client/tools.md, 6.1; temper-client-tools/src/jig/render/{document,
lists}.rs and document/outcomes.rs; jig-client/src/words/english.rs.
This private helper validates the fixed story's creation, ended read and
tree/result. It judges no forge landing (that belongs to checks/_end_state).
"""

import json
import re
import sys

_ENDED = r"Ended|Completed|Failed|Cancelled"


def _number(text: str) -> int:
    if not re.fullmatch(r"[1-9][0-9]*", text) or int(text) > (1 << 64) - 1:
        raise ValueError("invalid task number")
    return int(text)


def _created(text: str) -> int:
    match = re.fullmatch(r"done: task ([1-9][0-9]*), chat\n", text)
    if match is None:
        raise ValueError("expected done: task N, chat")
    return _number(match[1])


def _unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate result field")
        result[key] = value
    return result


def _not_integer(value):
    raise ValueError("non-integer JSON number")


def _ended(text: str, task: int, *, tree: bool) -> None:
    if not text.endswith("\n"):
        raise ValueError("unterminated ctl output")
    lines = text.splitlines()
    if not lines or not re.fullmatch(rf"task {task} · chat · ({_ENDED}) · .+", lines[0]):
        raise ValueError("expected the named chat's ended document")
    if not any(re.fullmatch(r"digest: task [0-9a-f]{16}", line) for line in lines):
        raise ValueError("missing task digest")
    results = [line[8:] for line in lines if line.startswith("result: ")]
    if len(results) != 1:
        raise ValueError("expected one historical result")
    result = json.loads(results[0], object_pairs_hook=_unique,
                        parse_float=_not_integer, parse_constant=_not_integer)
    if (
        not isinstance(result, dict)
        or set(result) != {"task", "ending", "value", "at", "details", "acts"}
        or not isinstance(result["task"], dict)
        or type(result["task"].get("task")) is not int
        or result["task"]["task"] != task
        or result["task"].get("phase") not in ("ended", "done", "failed", "cancelled")
        or not (
            result["ending"] == "done"
            or isinstance(result["ending"], dict)
            and len(result["ending"]) == 1
            and next(iter(result["ending"])) in ("task", "failed", "cancelled", "withdrawn")
        )
        or result["at"] is not None and type(result["at"]) is not int
        or not isinstance(result["acts"], list)
    ):
        raise ValueError("malformed historical result")
    if tree:
        if lines.count("tree:") != 1:
            raise ValueError("missing tree")
        start = lines.index("tree:") + 1
        if not any(re.fullmatch(rf"{task} · .+ · ({_ENDED})", line) for line in lines[start:]):
            raise ValueError("tree does not show the ended root")
        if not any(re.fullmatch(r"digest: tree [0-9a-f]{16}", line) for line in lines[start:]):
            raise ValueError("missing tree digest")
    elif lines[-1] != "met":
        raise ValueError("wait condition unmet")


def _main(args: list[str]) -> int:
    try:
        text = sys.stdin.read()
        if args == ["created"]:
            print(_created(text))
        elif len(args) == 2 and args[0] in ("ended", "tree"):
            _ended(text, _number(args[1]), tree=args[0] == "tree")
        else:
            raise ValueError("invalid parser invocation")
    except (ValueError, UnicodeError) as error:
        print(f"story: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(_main(sys.argv[1:]))
