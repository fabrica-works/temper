"""Known byte trees and the landed skein observation writer's JSON lines."""

import base64
import copy
import json
import sys
import unittest
from pathlib import Path

EXPERIMENT = Path(__file__).resolve().parents[2] / "chat-change-smoke"
sys.path.insert(0, str(EXPERIMENT / "checks"))
from _end_state import _CHANGED, _CONTENT, _NOT_LANDED, _Object, _Status, _decode_objects, _judge, _project_observations

NOTICE = b"Maintained by temper.\n"
PARENT = {b"README.md": b"widget\n", b"src/a": b"\x00\xff\n"}
CURRENT = {**PARENT, b"NOTICE": NOTICE}


def judge(**changes):
    args = dict(
        complete_lines=True,
        flushed=True,
        loss_counts=(0,),
        last_merged=12,
        last_moved=12,
        objects={11: _Object(None, None, PARENT), 12: _Object(11, None, CURRENT)},
        notice=NOTICE,
    )
    args.update(changes)
    return _judge(**args)


def object_reply(commit=12, parent=11):
    def binary(raw):
        return base64.b64encode(raw).decode("ascii")

    def record(number, first_parent, tree):
        # Intentionally different member order from the supplied contract.
        tree = {**tree, b"\x00\xff/path": b"\xff\x00"}
        return dict(message=binary(b"\x00\xffmessage\n"),
                    tree=[dict(content=binary(value), path=binary(path)) for path, value in tree.items()],
                    merge_parent=None, parent=first_parent, commit=number)

    return dict(graph=[], objects=[record(commit, parent, CURRENT), record(parent, None, PARENT)])


class EndStateTests(unittest.TestCase):
    def test_object_reply_binary_u64_and_exact_notice_delta(self):
        for commit, parent in ((12, 11), ((1 << 64) - 1, (1 << 64) - 2)):
            for change, status, signatures in (
                (None, _Status.PASS, ()),
                (dict(path="b3RoZXI=", content="AP8="), _Status.FAIL, (_CHANGED,)),
                (dict(path="Tk9USUNF", content="d3Jvbmc="), _Status.FAIL, (_CONTENT,)),
            ):
                with self.subTest(commit=commit, change=change):
                    reply = object_reply(commit, parent)
                    if change:
                        entries = reply["objects"][0]["tree"]
                        entries[:] = [entry for entry in entries if entry["path"] != change["path"]]
                        entries.append(change)
                    objects = _decode_objects(json.dumps(reply).encode(), [commit, parent])
                    self.assertIsNotNone(objects)
                    self.assertEqual(objects[parent].tree, {**PARENT, b"\x00\xff/path": b"\xff\x00"})
                    verdict = judge(last_merged=commit, last_moved=commit, objects=objects)
                    self.assertEqual((verdict.status, verdict.signatures), (status, signatures))
        initial = object_reply()
        initial["objects"].pop()
        initial["graph"] = [dict(commit=11, parent=None, merge_parent=None)]
        self.assertEqual(set(_decode_objects(json.dumps(initial).encode(), [12])), {12})

    def test_partial_or_malformed_object_reply_never_supplies_evidence(self):
        cases = []
        for dropped in (0, 1):
            reply = object_reply()
            record = reply["objects"].pop(dropped)
            cases.append((f"dropped asked {dropped}", json.dumps(reply).encode()))
            reply["graph"].append({key: record[key] for key in ("commit", "parent", "merge_parent")})
            cases.append((f"asked only in graph {dropped}", json.dumps(reply).encode()))
        for field in ("commit", "parent", "merge_parent", "tree", "message"):
            reply = object_reply()
            del reply["objects"][0][field]
            cases.append(("missing " + field, json.dumps(reply).encode()))
        for value in (True, 12.0, "12", -1, 1 << 64):
            for field in ("commit", "parent", "merge_parent"):
                reply = object_reply()
                reply["objects"][0][field] = value
                cases.append((f"non-u64 {field} {value}", json.dumps(reply).encode()))
        for field in ("path", "content", "message"):
            for value in ("%%%", "YQ", "YQ===", "YR==", "_w==", "YQ==\n", 3, None):
                reply = object_reply()
                target = reply["objects"][0]
                (target if field == "message" else target["tree"][0])[field] = value
                cases.append((f"base64 {field} {value!r}", json.dumps(reply).encode()))
        reply = object_reply()
        reply["objects"].append(copy.deepcopy(reply["objects"][0]))
        cases.append(("duplicate object", json.dumps(reply).encode()))
        reply = object_reply()
        reply["objects"][0]["tree"].append(copy.deepcopy(reply["objects"][0]["tree"][0]))
        cases.append(("duplicate decoded path", json.dumps(reply).encode()))
        good = json.dumps(object_reply()).encode()
        cases.extend((
            ("truncated", good[:-1]), ("invalid UTF8", b"\xff"),
            ("repeated member", good.replace(b'"commit": 12', b'"commit": 12, "commit": 12')),
            ("missing tree entry content", good.replace(b'"content": "d2lkZ2V0Cg==", ', b"")),
        ))
        for name, wire in cases:
            with self.subTest(name=name):
                objects = _decode_objects(wire, [12, 11])
                self.assertIsNone(objects)
                self.assertEqual(judge(objects=objects).status, _Status.UNDECIDED)

    def test_notice_file_is_exactly_22_bytes(self):
        self.assertEqual(len(NOTICE), 22)
        self.assertEqual((EXPERIMENT / "NOTICE").read_bytes(), NOTICE)

    def test_exact_addition_passes_with_deterministic_evidence(self):
        verdict = judge()
        self.assertEqual(verdict.status, _Status.PASS)
        self.assertEqual(verdict.signatures, ())
        self.assertEqual(verdict.evidence, (
            "C=12", "P=11", "P paths=[b'README.md', b'src/a']",
            "C paths=[b'NOTICE', b'README.md', b'src/a']",
            "P NOTICE=None", "C NOTICE=b'Maintained by temper.\\n'",
        ))

    def test_known_empty_parent_is_distinct_from_unavailable_parent(self):
        self.assertEqual(judge(objects={
            11: _Object(None, None, {}), 12: _Object(11, None, {b"NOTICE": NOTICE}),
        }).status, _Status.PASS)
        self.assertEqual(judge(objects={12: _Object(11, None, CURRENT)}).status, _Status.UNDECIDED)

    def test_exact_bytes_not_line_substring(self):
        for content in (b"Maintained by temper.", NOTICE + b"\n", NOTICE.replace(b"\n", b"\r\n"),
                        NOTICE + b"extra", b"Maintained by somebody else.\n", b"\xff"):
            with self.subTest(content=content):
                verdict = judge(objects={11: _Object(None, None, PARENT),
                                         12: _Object(11, None, {**PARENT, b"NOTICE": content})})
                self.assertEqual((verdict.status, verdict.signatures), (_Status.FAIL, (_CONTENT,)))

    def test_added_removed_modified_and_renamed_other_files_fail(self):
        trees = (
            {**CURRENT, b"other": b"new"},
            {b"src/a": PARENT[b"src/a"], b"NOTICE": NOTICE},
            {**CURRENT, b"src/a": b"different"},
            {b"renamed": PARENT[b"README.md"], b"src/a": PARENT[b"src/a"], b"NOTICE": NOTICE},
        )
        for tree in trees:
            with self.subTest(tree=tree):
                self.assertEqual(judge(objects={11: _Object(None, None, PARENT),
                                               12: _Object(11, None, tree)}).signatures, (_CHANGED,))

    def test_preexisting_notice_and_merge_parent_are_not_one_squash_addition(self):
        for objects in (
            {11: _Object(None, None, CURRENT), 12: _Object(11, None, CURRENT)},
            {11: _Object(None, None, PARENT), 12: _Object(11, 10, CURRENT)},
            {12: _Object(None, None, CURRENT)},
        ):
            with self.subTest(objects=objects):
                self.assertEqual(judge(objects=objects).signatures, (_CHANGED,))

    def test_missing_notice_has_content_and_delta_failures(self):
        self.assertEqual(judge(objects={11: _Object(None, None, PARENT),
                                       12: _Object(11, None, PARENT)}).signatures, (_CONTENT, _CHANGED))

    def test_refused_or_unmerged_or_moved_away_story_fails(self):
        for merge, move in ((None, None), (None, 11), (12, None), (12, 13), (13, 12)):
            with self.subTest(merge=merge, move=move):
                verdict = judge(last_merged=merge, last_moved=move)
                self.assertEqual((verdict.status, verdict.signatures), (_Status.FAIL, (_NOT_LANDED,)))

    def test_explicit_loss_always_undecided(self):
        for changes in (dict(loss_counts=(1,)), dict(loss_counts=(0, 2, 0)),
                        dict(loss_counts=(4,), flushed=False, last_merged=None)):
            with self.subTest(changes=changes):
                verdict = judge(**changes)
                self.assertEqual((verdict.status, verdict.reason), (_Status.UNDECIDED, "observations lost"))

    def test_incomplete_or_unknown_never_passes_even_with_perfect_tree(self):
        for changes in (
            dict(complete_lines=False), dict(complete_lines=None), dict(flushed=False), dict(flushed=None),
            dict(loss_counts=None), dict(loss_counts=()), dict(loss_counts=(-1,)), dict(loss_counts=(False,)),
        ):
            with self.subTest(changes=changes):
                verdict = judge(**changes)
                self.assertEqual((verdict.status, verdict.reason), (_Status.UNDECIDED, "observations missing"))

    def test_missing_store_and_incomplete_objects_are_undecided(self):
        for objects in (None, {}, {11: _Object(None, None, PARENT)},
                        {12: _Object(11, None, CURRENT)},
                        {11: _Object(None, None, PARENT), 12: _Object(11, None, None)},
                        {11: _Object(None, None, None), 12: _Object(11, None, CURRENT)},
                        {11: _Object(None, None, {"text-path": b"wrong type"}),
                         12: _Object(11, None, CURRENT)}):
            with self.subTest(objects=objects):
                verdict = judge(objects=objects)
                self.assertEqual((verdict.status, verdict.reason), (_Status.UNDECIDED, "store unreadable"))

    def test_reference_corruption_is_not_an_alternate_success(self):
        self.assertEqual(judge(notice=NOTICE.rstrip()).status, _Status.UNDECIDED)

    def test_counts_are_domain_u64s_not_sha_strings_or_booleans(self):
        for count in ("12", True, -1, 0, 1 << 64):
            with self.subTest(count=count):
                self.assertEqual(judge(last_merged=count, last_moved=count).status, _Status.UNDECIDED)

    def test_writer_projection_needs_whole_lossless_observations_and_owner_boundary(self):
        commit = (1 << 64) - 1  # Distinct integers above 2**53 must not round together.
        merge = dict(observation="merged", repository="fake/widget", number=1,
                     base="main", head=commit - 1, commit=commit, by=2)
        move = dict(observation="moved", repository="fake/widget", branch="main",
                    **{"from": commit - 2, "to": commit, "by": 2, "forward": True})
        delete = dict(observation="deleted", repository="fake/widget", branch="main", at=commit, by=2)
        zero = {"observations_lost": 0}
        objects = {commit - 2: _Object(None, None, PARENT),
                   commit: _Object(commit - 2, None, CURRENT)}

        def wire(records):
            return b"".join((json.dumps(r, ensure_ascii=False) + "\n").encode("utf-8") for r in records)

        story = [dict(merge, commit=commit - 1), dict(move, to=commit - 1), zero, merge, move]
        complete = wire(story + [zero])
        ignored = [dict(move, repository="別/widget", to=1), dict(delete, branch="topic"),
                   dict(merge, base="other", commit=1),
                   dict(observation="commented", repository="fake/widget", number=1, id=1,
                        body="AP8K", by=2)]  # Opaque base64 content, never a text/tree substitute.
        cases = [
            ("last target, repeated checkpoints", complete + wire([zero]), _Status.PASS, commit, commit),
            ("other repository/branch and content", wire(story + ignored + [zero]), _Status.PASS, commit, commit),
            ("delete", wire(story + [delete, zero]), _Status.FAIL, commit, None),
            ("recreate", wire(story + [delete, dict(move, **{"from": None}), zero]), _Status.PASS, commit, commit),
            ("moved away", wire(story + [dict(move, to=commit - 1), zero]), _Status.FAIL, commit, commit - 1),
            ("merge does not imply move", wire([merge, zero]), _Status.FAIL, commit, None),
            ("loss is sticky", complete + wire([{"observations_lost": 2}, zero]), _Status.UNDECIDED, commit, commit),
        ]
        for name, data, status, merged, moved in cases:
            with self.subTest(name=name):
                facts = _project_observations(data)
                self.assertEqual((facts["last_merged"], facts["last_moved"]), (merged, moved))
                self.assertEqual(_judge(**facts, flushed=True, objects=objects, notice=NOTICE).status, status)

        malformed = [
            None, b"", wire(story), complete.rstrip(b"\n"), complete + b'{"observations_lost":',
            complete + b"\n", complete + b"[]\n", complete + b"{bad}\n",
            complete + b'{"observation":"new_tag","repository":"fake/widget"}\n' + wire([zero]),
            complete + b'{"observations_lost":0,"observations_lost":0}\n',
            complete + b'{"observations_lost":0,"extra":1}\n',
            complete + b'{"observations_lost":NaN}\n',
            complete + b'{"observations_lost":-0}\n',
            complete + b'{"observations_lost":1e0}\n',
            complete + b'{"observations_lost":0.0}\n',
            complete + wire([dict(move, forward=1), zero]),
            complete + wire([{k: v for k, v in merge.items() if k != "head"}, zero]),
            complete + wire([dict(delete, at=True), zero]),
            complete + wire([dict(merge, commit="12"), zero]),
            complete + wire([dict(move, to=1 << 64), zero]),
            complete + wire([dict(move, **{"from": -1}), zero]),
            complete + wire([dict(move, extra=1), zero]),
            complete + b'{"observations_lost":false}\n',
            complete + b'{"observations_lost":null}\n',
            complete + b'{"observations_lost":18446744073709551616}\n',
            complete + b'{"observation":"moved","repository":"\xff"}\n',
            complete + wire([dict(move, branch="surrogate"), zero]).replace(b"surrogate", b"\\ud800"),
        ]
        for index, data in enumerate(malformed):
            with self.subTest(malformed=index):
                self.assertEqual(_judge(**_project_observations(data), flushed=True,
                                        objects=objects, notice=NOTICE).status, _Status.UNDECIDED)
        for boundary in (None, False, 1):
            with self.subTest(boundary=boundary):
                self.assertEqual(_judge(**_project_observations(complete), flushed=boundary,
                                        objects=objects, notice=NOTICE).status, _Status.UNDECIDED)
        self.assertEqual(_judge(**_project_observations(complete), flushed=True,
                                objects=None, notice=NOTICE).status, _Status.UNDECIDED)


if __name__ == "__main__":
    unittest.main()
