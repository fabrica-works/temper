"""Read only ctl's actual text contract; no model words imply success.

Sources: client/tools.md, 6.1; temper-client-tools/src/jig/render/document.rs,
document/{result_words,waiting,read_sets}.rs and lists.rs. In this fixed chat,
successful historical results print Completed, with optional report words.
Ended is also a generic settled phase; only the result establishes completion.
This helper judges no forge landing (that belongs to checks/_end_state).
"""

import re
import sys

_SUCCESS_PHASE = r"Ended|Completed"


def _number(text: str) -> int:
    if not re.fullmatch(r"[1-9][0-9]*", text) or int(text) > (1 << 64) - 1:
        raise ValueError("invalid task number")
    return int(text)


def _created(text: str) -> int:
    match = re.fullmatch(r"done: task ([1-9][0-9]*), chat\n", text)
    if match is None:
        raise ValueError("expected done: task N, chat")
    return _number(match[1])


def _ended(text: str, task: int, *, tree: bool) -> None:
    if not text.endswith("\n"):
        raise ValueError("unterminated ctl output")
    lines = text.splitlines()
    header = re.fullmatch(rf"task {task} · chat · ({_SUCCESS_PHASE}) · .+", lines[0]) if lines else None
    if header is None:
        raise ValueError("expected the named chat's completed document")
    digests = [i for i, line in enumerate(lines) if re.fullmatch(r"digest: task [0-9a-f]{16}", line)]
    if len(digests) != 1:
        raise ValueError("missing task digest")
    results = [i for i, line in enumerate(lines) if line.startswith("result: ")]
    if len(results) != 1:
        raise ValueError("expected one historical result")
    start, end = results[0], digests[0]
    if (
        start >= end
        or not re.fullmatch(r"result: Completed(?: · .*)?", lines[start])
        or any(not line.startswith("  ") for line in lines[start + 1:end])
    ):
        raise ValueError("expected a successful historical chat result")
    if tree:
        if lines.count("tree:") != 1:
            raise ValueError("missing tree")
        start = lines.index("tree:") + 1
        roots = [line for line in lines[start:] if line.startswith(f"{task} · ")]
        if (start <= end or len(roots) != 1 or lines[start] != roots[0]
                or not re.fullmatch(rf"{task} · .+ · {header[1]}", roots[0])):
            raise ValueError("tree does not show the same completed root")
        if not re.fullmatch(r"digest: tree [0-9a-f]{16}", lines[-1]):
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
