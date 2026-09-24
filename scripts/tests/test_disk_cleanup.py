"""Run maintenance scripts against disposable, isolated Git workspaces."""

from pathlib import Path
import os
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class DiskCleanup(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="nuis-cleanup-tests-")
        self.addCleanup(self.scratch.cleanup)
        self.base = Path(self.scratch.name)
        self.root = self.base / "workspace with spaces"
        self.home = self.base / "home"
        self.home.mkdir()
        (self.root / "scripts").mkdir(parents=True)
        for name in ["disk-clean-safe.sh", "disk-audit.sh"]:
            shutil.copyfile(ROOT / "scripts" / name, self.root / "scripts" / name)
        self.write("Cargo.toml", "[workspace]\n")
        self.write(".gitignore", "target/\n**/.nuis/\n__pycache__/\n/examples/bins/*\n!/examples/bins/README.md\n/tools/yir-preview-macos/build/\n")
        self.git("init", "-q")
        self.git("add", "Cargo.toml", ".gitignore", "scripts")

    def git(self, *args):
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            capture_output=True, text=True, check=True, timeout=30,
        )

    def write(self, relative, text="keep", executable=False):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        if executable:
            path.chmod(0o700)
        return path

    def run_script(self, *args, name="disk-clean-safe.sh"):
        env = os.environ.copy()
        env.update(HOME=str(self.home), TMPDIR=str(self.base))
        return subprocess.run(
            ["bash", str(self.root / "scripts" / name), *args],
            cwd=self.home, env=env, text=True, capture_output=True,
            check=False, timeout=30,
        )

    def assert_ok(self, result):
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def fixtures(self):
        return [self.write(path) for path in [
            "target/debug/incremental/session/state",
            "target/release/incremental/session/state",
            "tools/nuis/target/nuis-build/old/binary",
            "crates/domain/target/local/cache",
            "examples/projects/nested/demo with spaces/.nuis/cache/object",
            "examples/bins/local-run/binary",
            "tools/yir-preview-macos/build/PreviewFrame",
            "scripts/tests/__pycache__/old.pyc",
        ]]

    def test_default_is_read_only_and_resolves_workspace_from_script(self):
        paths = self.fixtures()
        result = self.run_script("--verbose")
        self.assert_ok(result)
        self.assertIn("Dry-run only", result.stdout)
        self.assertIn("Selected 8 workspace outputs", result.stdout)
        self.assertIn("demo with spaces/.nuis/cache", result.stdout)
        self.assertTrue(all(path.exists() for path in paths))

    def test_apply_preserves_source_project_state_and_external_resources(self):
        paths = self.fixtures()
        keep = [self.write(path) for path in [
            "target/debug/nuis", "target/debug/deps/libnuis.rlib",
            "examples/projects/nested/demo with spaces/main.ns",
            "examples/projects/nested/demo with spaces/.nuis/settings.toml",
            "examples/bins/README.md", "subprojects/future/target/keep",
        ]]
        for path in [
            self.base / "kyuubiki/target/keep", self.base / "sibling/target/keep",
            self.home / ".cargo/registry/cache/keep", self.home / ".npm/_cacache/keep",
            self.base / "nuis_old_shared_temp/keep",
        ]:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("keep", encoding="utf-8")
            keep.append(path)
        result = self.run_script("--apply")
        self.assert_ok(result)
        self.assertTrue(all(not path.exists() for path in paths))
        self.assertTrue(all(path.exists() for path in keep))
        self.assertTrue((self.root / ".git/index").exists())

    def test_build_binaries_retains_cli_libraries_metadata_and_unknown_files(self):
        remove = [self.write(path, executable=True) for path in [
            "target/debug/deps/check-0123456789abcdef",
            "target/release/deps/check-0123456789abcdef.exe",
        ]]
        keep = [self.write(path, executable=True) for path in [
            "target/debug/nuis", "target/debug/deps/libmacro-0123456789abcdef.dylib",
            "target/debug/deps/libnuis-0123456789abcdef.rlib",
            "target/debug/deps/check-0123456789abcdef.d",
            "target/debug/deps/manual-inspection-file",
        ]]
        self.assert_ok(self.run_script("--apply", "--build-binaries"))
        self.assertTrue(all(not path.exists() for path in remove))
        self.assertTrue(all(path.exists() for path in keep))

    def test_workspace_removal_is_explicit(self):
        output = self.write("target/debug/nuis")
        self.assert_ok(self.run_script("--workspace"))
        self.assertTrue(output.exists())
        self.assert_ok(self.run_script("--workspace", "--apply"))
        self.assertFalse((self.root / "target").exists())
        self.assertTrue((self.root / "Cargo.toml").exists())

    def test_tracked_candidate_vetoes_every_deletion(self):
        paths = self.fixtures()
        self.git("add", "-f", "tools/nuis/target/nuis-build/old/binary")
        result = self.run_script("--apply")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing tracked cleanup path", result.stderr)
        self.assertTrue(all(path.exists() for path in paths))

    def test_non_ignored_candidate_is_not_assumed_disposable(self):
        paths = self.fixtures()
        self.write(".gitignore", "target/\n")
        result = self.run_script("--apply")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing non-ignored cleanup path", result.stderr)
        self.assertTrue(all(path.exists() for path in paths))

    def test_symlinked_candidate_and_parent_veto_every_deletion(self):
        for parent in [False, True]:
            with self.subTest(parent=parent):
                incremental = self.write("target/debug/incremental/session/state")
                external = self.base / f"external-{parent}"
                (external / "target").mkdir(parents=True)
                sentinel = external / "target/keep"
                sentinel.write_text("keep", encoding="utf-8")
                link = self.root / ("tools/linked" if parent else "tools/linked/target")
                link.parent.mkdir(parents=True, exist_ok=True)
                link.symlink_to(external if parent else external / "target", target_is_directory=True)
                try:
                    result = self.run_script("--apply")
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("refusing symlinked cleanup path", result.stderr)
                    self.assertTrue(incremental.exists() and sentinel.exists())
                finally:
                    link.unlink()
                    if not parent:
                        link.parent.rmdir()

    def test_empty_workspace_is_a_successful_noop(self):
        result = self.run_script("--apply", "--build-binaries")
        self.assert_ok(result)
        self.assertIn("Selected 0 workspace outputs", result.stdout)

    def test_removed_global_options_fail_without_cleanup(self):
        paths = self.fixtures()
        for option in ["--cargo-cache", "--docker", "--unknown"]:
            result = self.run_script("--apply", option)
            self.assertEqual(result.returncode, 2)
            self.assertTrue(all(path.exists() for path in paths))

    def test_audit_is_read_only_and_refuses_apply(self):
        paths = self.fixtures()
        self.assert_ok(self.run_script(name="disk-audit.sh"))
        result = self.run_script("--apply", name="disk-audit.sh")
        self.assertEqual(result.returncode, 2)
        self.assertTrue(all(path.exists() for path in paths))


if __name__ == "__main__":
    unittest.main()
