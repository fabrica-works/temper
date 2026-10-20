"""Real shell and canary executables only; no counterfeit temper/harness."""

import subprocess
import tempfile
import unittest
from pathlib import Path

EXPERIMENT = Path(__file__).resolve().parents[2] / "chat-change-smoke"
WRAPPER = EXPERIMENT / "tools" / "ctl"
STORY = EXPERIMENT / "smoke" / "story"
LINK = "http://127.0.0.1:4000/jig/v1/sign-in/link?secret=run-only"
ALLOW = "me inbox tasks task chat activity turn run wait message close adopt answer decide release stop cancel pending help".split()


class CtlTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.client = Path(self.scratch.name) / "client with spaces"
        self.client.mkdir()
        self.address = self.client / "address"
        self.address.write_bytes(b"http://127.0.0.1:4000\n")

    def call(self, *args, binary="/bin/echo", link=LINK, stdin=b""):
        return subprocess.run(["/bin/sh", str(WRAPPER), binary, str(self.client), link, *args],
                              input=stdin, capture_output=True, check=False)

    def refused(self, result):
        self.assertEqual(result.returncode, 1, result)
        self.assertEqual(result.stdout, b"")
        self.assertIn(b"ctl: refused: ", result.stderr)

    def test_each_area_one_tool_is_forwarded_with_fixed_directory_except_help(self):
        for tool in ALLOW:
            with self.subTest(tool=tool):
                result = self.call(tool)
                expected = ["ctl"] + ([] if tool == "help" else [str(self.client)]) + [tool]
                self.assertEqual((result.returncode, result.stdout, result.stderr),
                                 (0, (" ".join(expected) + "\n").encode(), b""))

    def test_help_and_json_prefix_follow_actual_host_shape(self):
        for args, expected in (
            (("help", "task"), b"ctl help task\n"),
            (("--json", "help", "task"), b"ctl help task --json\n"),
            (("--json", "me"), f"ctl {self.client} --json me\n".encode()),
        ):
            with self.subTest(args=args):
                self.assertEqual(self.call(*args).stdout, expected)

    def test_tools_outside_area_one_are_refused(self):
        for tool in ("sign-in", "sign-out", "add-key", "accounts", "models", "connect", "forges",
                     "forge-connect", "repositories", "goal", "project", "compound", "--call", "", "bogus"):
            with self.subTest(tool=tool):
                self.refused(self.call(tool))

    def test_hidden_call_and_sign_in_out_are_refused_in_every_position(self):
        for word in ("--call", "--call=-", "--call=/tmp/call", "sign-in", "sign-out"):
            for args in ((word,), ("me", word), ("help", word),
                         ("chat", "--project", "1", "--", word),
                         ("--json", "chat", "words", word)):
                with self.subTest(args=args):
                    self.refused(self.call(*args))

    def test_literal_words_are_neither_split_nor_evaluated(self):
        marker = Path(self.scratch.name) / "must-not-exist"
        words = [f"$(touch {marker})", f"`touch {marker}`", "*", "--callish", "two words", "line\nbreak"]
        result = self.call("message", "41", *words)
        expected = subprocess.run(["/bin/echo", "ctl", str(self.client), "message", "41", *words],
                                  capture_output=True, check=False)
        self.assertEqual((result.returncode, result.stdout, result.stderr),
                         (expected.returncode, expected.stdout, expected.stderr))
        self.assertFalse(marker.exists())

    def test_invalid_address_files_are_refused_independently_of_link_match(self):
        for address in (
            b"", b"http://127.0.0.1:4000", b"http://127.0.0.1:4000\n\n",
            b"http://127.0.0.1:4000\nextra", b"http://127.0.0.1:4000\r\n",
            b"https://127.0.0.1:4000\n", b"http://localhost:4000\n",
            b"http://192.0.2.1:4000\n", b"http://127.0.0.1:65536\n",
            b"http://127.0.0.1:-1\n", b"http://127.0.0.1:1e3\n",
            b"http://127.0.0.1:999999999999999999999\n", b"http://127.0.0.1\n",
            b"http://127.256.0.1:4000\n", b"http://127.00.0.1:4000\n",
            b"http://127.0.0.1.:4000\n", b"http://127..0.1:4000\n",
            b"http://127.0.0.1:4000/\n", b"http://127.0.0.1:4000?x\n",
            b"http://user@127.0.0.1:4000\n", b"http://[::2]:4000\n",
            b"http://[::ffff:127.0.0.1]:4000\n", b"http://[:::1]:4000\n",
            b"http://[:0:0:0:0:0:0:1]:4000\n", b"http://[::1%lo]:4000\n",
        ):
            with self.subTest(address=address):
                self.address.write_bytes(address)
                link = address.decode().rstrip("\n") + "/jig/v1/sign-in/link?secret=run-only"
                self.refused(self.call("me", link=link))

    def test_numeric_loopback_spellings_are_accepted(self):
        for host in ("127.0.0.1", "127.3.4.5", "[::1]", "[0:0:0:0:0:0:0:1]",
                     "[0000::0001]", "[0:0:0:0:0::1]"):
            with self.subTest(host=host):
                authority = host + ":4000"
                self.address.write_text("http://" + authority + "\n")
                self.assertEqual(self.call("me", link="http://" + authority + "/jig/v1/sign-in/link?secret=run-only").returncode, 0)

    def test_address_is_pinned_for_every_call_including_help(self):
        for address in (b"http://127.0.0.1:4001\n", b"http://127.0.0.2:4000\n"):
            self.address.write_bytes(address)
            for tool in ("me", "help"):
                with self.subTest(address=address, tool=tool):
                    self.refused(self.call(tool))
        self.address.unlink()
        self.refused(self.call("me"))

    def test_malformed_or_other_run_links_are_refused(self):
        for link in ("127.0.0.1:4000", "https://127.0.0.1:4000/path", "http://127.0.0.1:4000",
                     "http://user@127.0.0.1:4000/path", "http://127.0.0.1:4000.evil/path"):
            with self.subTest(link=link):
                self.refused(self.call("me", link=link))

    def test_real_canary_stderr_and_status_survive_exec(self):
        for binary in ("/bin/true", "/bin/false", "/bin/cat"):
            with self.subTest(binary=binary):
                direct = subprocess.run([binary, "ctl", str(self.client), "me"], input=b"stdin\n",
                                        capture_output=True, check=False)
                wrapped = self.call("me", binary=binary, stdin=b"stdin\n")
                self.assertEqual((wrapped.returncode, wrapped.stdout, wrapped.stderr),
                                 (direct.returncode, direct.stdout, direct.stderr))

    def test_stdin_reaches_real_cat_through_help(self):
        # cat reads '-' after reporting its missing 'ctl' and 'help' files.
        direct = subprocess.run(["/bin/cat", "ctl", "help", "-"], input=b"untouched stdin\n",
                                capture_output=True, check=False)
        wrapped = self.call("help", "-", binary="/bin/cat", stdin=b"untouched stdin\n")
        self.assertEqual(wrapped.stdout, b"untouched stdin\n")
        self.assertEqual((wrapped.returncode, wrapped.stdout, wrapped.stderr),
                         (direct.returncode, direct.stdout, direct.stderr))

    def test_missing_tool_is_refused(self):
        self.refused(self.call())

    def test_story_refuses_echo_output_instead_of_fabricating_completion(self):
        result = subprocess.run(["/bin/sh", str(STORY), "/bin/echo", str(self.client), LINK],
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b"--key smoke-chat", result.stdout)
        self.assertIn(b"Add a file NOTICE to widget holding one line: Maintained by temper.", result.stdout)
        self.assertIn(b"story: expected done: task N, chat", result.stderr)

    def test_story_stops_on_real_command_failure(self):
        result = subprocess.run(["/bin/sh", str(STORY), "/bin/false", str(self.client), LINK],
                                capture_output=True, check=False)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (1, b"", b""))


if __name__ == "__main__":
    unittest.main()
