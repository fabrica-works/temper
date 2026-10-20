"""Known byte trees, not observations in a guessed external encoding."""

import sys
import unittest
from pathlib import Path

EXPERIMENT = Path(__file__).resolve().parents[2] / "chat-change-smoke"
sys.path.insert(0, str(EXPERIMENT / "checks"))
from _end_state import _CHANGED, _CONTENT, _NOT_LANDED, _Object, _Status, _judge

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


class EndStateTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
