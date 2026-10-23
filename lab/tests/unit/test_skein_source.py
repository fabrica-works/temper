"""The pinned skein's root from a fake `cargo metadata` document; cargo itself never runs."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

LAB = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('skein_source', LAB / 'skein_source.py')
skein_source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(skein_source)

REV = '6b5934ded61cda47d25d00cb4e92887ace80b3ce'
SOURCE = 'git+https://github.com/fabrig-dev/skein.git#' + REV


class FromMetadata(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.checkout = Path(directory.name) / 'checkouts/skein-0123/6b5934d'
        (self.checkout / 'scripts/lab/runner').mkdir(parents=True)

    def package(self, name, source=SOURCE, manifest=None):
        manifest = manifest or self.checkout / 'crates' / name / 'Cargo.toml'
        return dict(name=name, source=source, manifest_path=str(manifest))

    def metadata(self, *packages):
        return dict(packages=[dict(name='temper', source=None, manifest_path='/temper/Cargo.toml'),
                              dict(name='libc', source='registry+https://github.com/rust-lang/crates.io-index',
                                   manifest_path='/registry/libc/Cargo.toml'),
                              *packages])

    def refused(self, metadata, why):
        with self.assertRaises(skein_source.Unresolved) as raised:
            skein_source.from_metadata(metadata)
        self.assertIn(why, str(raised.exception))

    def test_the_checkout_holding_the_runner_is_the_root(self):
        nested = self.checkout / 'testing/skein-browser/Cargo.toml'
        metadata = self.metadata(self.package('skein-io'), self.package('skein-browser', manifest=nested))
        self.assertEqual(skein_source.from_metadata(metadata), self.checkout)

    def test_a_query_in_the_source_is_still_one_revision(self):
        source = 'git+https://github.com/fabrig-dev/skein.git?rev=6b5934d#' + REV
        self.assertEqual(skein_source.from_metadata(self.metadata(self.package('skein-io', source))), self.checkout)

    def test_no_skein_package_fails(self):
        self.refused(self.metadata(), 'names no skein package')

    def test_a_path_dependency_fails(self):
        self.refused(self.metadata(self.package('skein-io'), self.package('skein-lib', None)), 'a path')

    def test_a_branch_or_short_revision_fails(self):
        for source in ('git+https://github.com/fabrig-dev/skein.git?branch=main',
                       'git+https://github.com/fabrig-dev/skein.git#6b5934d',
                       'registry+https://github.com/rust-lang/crates.io-index'):
            with self.subTest(source=source):
                self.refused(self.metadata(self.package('skein-io', source)), 'full revision')

    def test_two_revisions_fail(self):
        other = 'git+https://github.com/fabrig-dev/skein.git#' + '0' * 40
        self.refused(self.metadata(self.package('skein-io'), self.package('skein-lib', other)), 'more than one')

    def test_a_tree_without_the_runner_fails(self):
        (self.checkout / 'scripts/lab/runner').rmdir()
        self.refused(self.metadata(self.package('skein-io')), 'holds no scripts/lab/runner')

    def test_packages_in_two_checkouts_fail(self):
        elsewhere = self.checkout.parent / 'other/crates/skein-lib/Cargo.toml'
        self.refused(self.metadata(self.package('skein-io'), self.package('skein-lib', manifest=elsewhere)),
                     'outside one checkout')


class Root(unittest.TestCase):
    def test_a_failing_cargo_fails_and_names_its_last_line(self):
        failed = subprocess.CompletedProcess([], 101, b'', b'warning: x\nerror: the lock file needs to be updated\n')
        skein_source.root.cache_clear()
        self.addCleanup(skein_source.root.cache_clear)
        with patch.object(skein_source.subprocess, 'run', return_value=failed):
            with self.assertRaises(skein_source.Unresolved) as raised:
                skein_source.root()
        self.assertEqual(str(raised.exception), 'cargo metadata failed: error: the lock file needs to be updated')


if __name__ == '__main__':
    unittest.main()
