"""The subject's gallery tar depends on names and bytes, not host metadata."""
import hashlib
import io
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from look_pack import pack


class PackTests(unittest.TestCase):
    def test_pack_ignores_disk_times_and_modes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            # Create in a different order from the archive's names.
            (root / 'z.txt').write_bytes(b'last\x00\xff')
            (root / 'a').mkdir()
            (root / 'a' / 'page.html').write_bytes(b'<main>page</main>\n')
            (root / 'empty').mkdir()
            paths = [root, *root.rglob('*')]
            for path in paths:
                path.chmod(0o700 if path.is_dir() else 0o600)
                os.utime(path, (1000, 1000))
            first = pack(root)
            for path in paths:
                path.chmod(0o755 if path.is_dir() else 0o644)
                os.utime(path, (2000, 2000))
            second = pack(root)
        self.assertEqual(first, second)
        self.assertEqual(hashlib.sha256(first).digest(), hashlib.sha256(second).digest())
        with tarfile.open(fileobj=io.BytesIO(first), mode='r:') as archive:
            self.assertEqual(archive.getnames(), ['a', 'a/page.html', 'empty', 'z.txt'])
            for member in archive:
                self.assertTrue(member.isdir() or member.isfile())
                self.assertEqual((member.mtime, member.uid, member.gid), (0, 0, 0))
                self.assertEqual((member.uname, member.gname), ('', ''))
                self.assertEqual(member.mode, 0o755 if member.isdir() else 0o644)
                self.assertEqual(member.pax_headers, {})
            self.assertEqual(archive.extractfile('a/page.html').read(), b'<main>page</main>\n')
            self.assertEqual(archive.extractfile('z.txt').read(), b'last\x00\xff')

    def test_links_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'file').write_bytes(b'page')
            (root / 'directory').mkdir()
            for target in ('file', 'directory', 'missing'):
                with self.subTest(target=target):
                    link = root / 'link'
                    link.symlink_to(target)
                    with self.assertRaisesRegex(ValueError, 'not a regular file or directory: link'):
                        pack(root)
                    link.unlink()

    def test_special_files_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            os.mkfifo(root / 'fifo')
            with self.assertRaisesRegex(ValueError, 'not a regular file or directory: fifo'):
                pack(root)

    def test_names_outside_ustar_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ('x' * 101)).write_bytes(b'page')
            with self.assertRaises(ValueError):
                pack(root)


if __name__ == '__main__':
    unittest.main()
