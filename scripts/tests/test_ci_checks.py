"""Exercise the actual CI entry scripts in small, isolated repository fixtures."""

from pathlib import Path
import os
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class CiChecks(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="nuis-ci-checks-")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "scripts").mkdir()
        (self.root / "docs").mkdir()
        (self.root / "bin").mkdir()
        (self.root / "README.md").write_text("# Fixture\n", encoding="utf-8")
        for name in ["check-doc-links.sh", "check-host-absolute-paths.sh"]:
            shutil.copyfile(ROOT / "scripts" / name, self.root / "scripts" / name)

    def run_script(self, name):
        env = os.environ.copy()
        env["PATH"] = str(self.root / "bin") + os.pathsep + env["PATH"]
        return subprocess.run(
            ["bash", str(self.root / "scripts" / name)],
            cwd=self.root / "docs",
            env=env,
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )

    def stub(self, name, source):
        path = self.root / "bin" / name
        path.write_text("#!/usr/bin/env bash\n" + source, encoding="utf-8")
        path.chmod(0o700)

    def test_links_resolve_from_source_not_invocation_directory(self):
        (self.root / "docs" / "guide.md").write_text(
            "[root](../README.md#fixture)\n", encoding="utf-8"
        )
        (self.root / "README.md").write_text(
            "[guide](docs/guide.md)\n[dir](docs)\n"
            "[web](https://example.invalid/absent)\n[anchor](#fixture)\n",
            encoding="utf-8",
        )
        result = self.run_script("check-doc-links.sh")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("3 local links checked", result.stdout)

    def test_missing_inline_link_fails_instead_of_silent_success(self):
        (self.root / "README.md").write_text(
            "[missing](docs/missing.md)\n[code](crates/missing.rs)\n", encoding="utf-8"
        )
        result = self.run_script("check-doc-links.sh")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing target 'docs/missing.md'", result.stdout)
        self.assertIn("missing target 'crates/missing.rs'", result.stdout)

    def test_absolute_markdown_link_fails(self):
        (self.root / "README.md").write_text(
            "[host](/machine-specific/docs.md)\n", encoding="utf-8"
        )
        result = self.run_script("check-doc-links.sh")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("absolute markdown link", result.stdout)

    def test_extractor_failure_is_not_reported_as_success(self):
        self.stub("perl", "exit 23\n")
        result = self.run_script("check-doc-links.sh")
        self.assertEqual(result.returncode, 23)
        self.assertNotIn("verification: ok", result.stdout)

    def test_missing_documentation_tree_is_not_reported_as_success(self):
        (self.root / "docs").rmdir()
        result = subprocess.run(
            ["bash", str(self.root / "scripts" / "check-doc-links.sh")],
            cwd=self.root, text=True, capture_output=True, check=False, timeout=30,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("verification: ok", result.stdout)

    def test_path_policy_supports_cold_cache_and_targets_only_its_integration_test(self):
        self.stub("cargo", "printf '%s\\n' \"$@\"\n")
        result = self.run_script("check-host-absolute-paths.sh")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        arguments = result.stdout.splitlines()
        self.assertNotIn("--offline", arguments)
        self.assertIn("--locked", arguments)
        self.assertIn("--test", arguments)
        self.assertIn("examples_mainline_compile", arguments)
        self.assertIn("--exact", arguments)
        self.assertIn("--test-threads=1", arguments)

    def test_path_test_failure_is_not_reported_as_success(self):
        self.stub("cargo", "exit 101\n")
        result = self.run_script("check-host-absolute-paths.sh")
        self.assertEqual(result.returncode, 101)
        self.assertNotIn("policy check: ok", result.stdout)

    def test_workflow_fetches_locked_dependencies_before_cargo_validation(self):
        workflow = (ROOT / ".github" / "workflows" / "build.yml").read_text(encoding="utf-8")
        self.assertLess(workflow.index("cargo fetch --locked"), workflow.index("bash scripts/check-host-absolute-paths.sh"))
        self.assertIn("cargo build --workspace --locked", workflow)
        self.assertNotIn("continue-on-error", workflow)

    def test_repository_links_do_not_depend_on_ignored_build_outputs(self):
        paths = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            cwd=ROOT, capture_output=True, check=True, timeout=30,
        ).stdout.split(b"\0")
        for raw in paths:
            if not raw:
                continue
            relative = Path(os.fsdecode(raw))
            source = ROOT / relative
            if not source.is_file():
                continue
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            if relative == Path("README.md") or relative.parts[0] == "docs":
                shutil.copyfile(source, destination)
            elif not destination.exists():
                destination.touch()
        self.assertFalse((self.root / "target").exists())
        result = self.run_script("check-doc-links.sh")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
