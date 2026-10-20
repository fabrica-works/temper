import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "chat-change-smoke" / "smoke"))
from _ctl_output import _created, _ended

# Known ctl text: render/document.rs::result_text and document/outcomes.rs
# encode_result; chips.rs::encode_chip; jig's English phase word Completed.
RESULT = ('result: {"task":{"task":41,"title":"NOTICE","executor":{"agent":{"charter":1}},'
          '"phase":"done","details":{"ended":{"how":"done","at":0}}},"ending":"done","value":null,'
          '"at":null,"details":null,"acts":[]}\n')
READ = ('task 41 · chat · Completed · project 1\n'
        'asked: Add a file NOTICE to widget holding one line: Maintained by temper.\n'
        'requester: you · spent 0.00 of 5.00 · no deadline\n' + RESULT +
        'digest: task 0123456789abcdef\n')
TREE = 'tree:\n41 · NOTICE · Completed\ndigest: tree fedcba9876543210\n'


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

    def test_wait_unmet_live_or_wrong_task_is_rejected(self):
        for text in (READ, READ + "not met\n", (READ + "met\n").replace("Completed", "Running"),
                     (READ + "met\n").replace("task 41", "task 42")):
            with self.subTest(text=text), self.assertRaises(ValueError):
                _ended(text, 41, tree=False)

    def test_missing_malformed_or_wrong_results_are_rejected(self):
        for result in ("", "result: {\n", "result: []\n", RESULT + RESULT,
                       RESULT.replace('"task":41', '"task":42'),
                       RESULT.replace('"ending":"done"', '"ending":true'),
                       RESULT.replace('"phase":"done"', '"phase":"running"'),
                       RESULT.replace('"at":null', '"at":1.5'),
                       RESULT.replace('"at":null', '"at":NaN'),
                       RESULT.replace('"at":null', '"at":null,"at":null')):
            with self.subTest(result=result), self.assertRaises(ValueError):
                _ended(READ.replace(RESULT, result) + "met\n", 41, tree=False)

    def test_missing_or_wrong_tree_and_digest_are_rejected(self):
        for tree in ("", TREE.replace("tree:", "plan:"), TREE.replace("41 ·", "42 ·"),
                     TREE.replace("Completed", "Running"), TREE.replace("fedcba9876543210", "bad")):
            with self.subTest(tree=tree), self.assertRaises(ValueError):
                _ended(READ + tree, 41, tree=True)


if __name__ == "__main__":
    unittest.main()
