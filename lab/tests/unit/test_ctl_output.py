import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "chat-change-smoke" / "smoke"))
from _ctl_output import _created, _ended

TOOLS = Path(__file__).resolve().parents[3] / 'crates/temper-client-tools/src'
# The actual renderer golden, adapting only the task number and conversation
# kind for this parser's fixed chat. Preserve its result, indentation and digest.
# Producer: jig/render/document.rs::{task_text,result_text}; result_words.rs::text.
# task_words.rs::a_compound_task_read_and_an_ended_wait_share_result_words
# establishes that an ended wait appends "met" to this same document.
READ = (TOOLS / 'tests/golden/task_report_ended.txt').read_text().replace(
    'task 2 · report ·', 'task 41 · chat ·', 1)
RESULT = READ[READ.index('result: '):READ.index('digest: ')]
# lists.rs::tree prints the root chip first, using the document's phase words.
TREE = 'tree:\n41 · NOTICE · Ended\ndigest: tree fedcba9876543210\n'


class OutputTests(unittest.TestCase):
    def test_creation_is_only_the_documented_done_line(self):
        self.assertEqual(_created("done: task 41, chat\n"), 41)
        for text in ("proposed: p1/1\n", "done: task 41, message\n", "done: task 0, chat\n",
                     "done: task 041, chat\n", "done: task 18446744073709551616, chat\n",
                     "done: task 41, chat", "done: task 41, chat\nextra\n", "41\n"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                _created(text)

    def test_final_wait_and_tree_result_are_read(self):
        _ended(READ + "met\n", 41, tree=False)
        _ended(READ + TREE, 41, tree=True)
        _ended((READ + TREE).replace('· Ended', '· Completed'), 41, tree=True)
        with self.assertRaises(ValueError):
            _ended(READ + TREE.replace('· Ended', '· Completed'), 41, tree=True)

    def test_wait_unmet_live_or_wrong_task_is_rejected(self):
        for text in (READ, READ + "not met\n", (READ + "met\n").replace("· Ended ·", "· Running ·"),
                     (READ + "met\n").replace("task 41", "task 42")):
            with self.subTest(text=text), self.assertRaises(ValueError):
                _ended(text, 41, tree=False)

    def test_missing_malformed_or_wrong_results_are_rejected(self):
        for result in ("", "result: {\n", "result: []\n", RESULT + RESULT,
                       'result: {"ending":"done"}\n', 'result: Completedish\n',
                       'result: Landed · landed it\n'):
            with self.subTest(result=result), self.assertRaises(ValueError):
                _ended(READ.replace(RESULT, result) + "met\n", 41, tree=False)

    def test_missing_or_wrong_tree_and_digest_are_rejected(self):
        for tree in ("", TREE.replace("tree:", "plan:"), TREE.replace("41 ·", "42 ·"),
                     TREE.replace("Ended", "Running"), TREE.replace("fedcba9876543210", "bad"),
                     TREE.replace('41 · NOTICE · Ended', '42 · delegate · Ended\n41 · NOTICE · Ended'),
                     TREE.replace('41 · NOTICE · Ended', '41 · NOTICE · Ended\n41 · NOTICE · Failed')):
            with self.subTest(tree=tree), self.assertRaises(ValueError):
                _ended(READ + tree, 41, tree=True)

    def test_completed_result_words_follow_the_existing_renderer_expectations(self):
        # Exact expectations from tests/jig/task_words.rs::completed_results_print_each_values_words.
        for result in ('result: Completed\n', 'result: Completed · ready\n',
                       'result: Completed\n  one\n  two\n'):
            with self.subTest(result=result):
                read = READ.replace(RESULT, result)
                _ended(read + 'met\n', 41, tree=False)
                _ended(read + TREE, 41, tree=True)

    def test_unsuccessful_endings_and_inconsistent_phases_are_rejected(self):
        # Exact result words from task_words.rs::endings_print_their_states_and_supplied_reasons.
        for result in ('result: Failed\n', 'result: Failed · broken\n',
                       'result: Cancelled\n', 'result: Cancelled · stopped\n',
                       'result: Cancelled\n  one\n  two\n', 'result: Ended\n'):
            for tree in (False, True):
                with self.subTest(result=result, tree=tree), self.assertRaises(ValueError):
                    _ended(READ.replace(RESULT, result) + (TREE if tree else 'met\n'), 41, tree=tree)
        for phase in ('Failed', 'Cancelled', 'Running'):
            for text in ((READ + TREE).replace('· Ended ·', f'· {phase} ·'),
                         READ + TREE.replace('· Ended\n', f'· {phase}\n')):
                with self.subTest(phase=phase, text=text), self.assertRaises(ValueError):
                    _ended(text, 41, tree=True)


if __name__ == "__main__":
    unittest.main()
