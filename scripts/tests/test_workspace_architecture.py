"""Architecture policies are generic across current and future domain packages."""

from contextlib import redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "workspace_architecture", ROOT / "scripts/check-workspace-architecture.py",
)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class WorkspaceArchitecture(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="nuis-architecture-tests-")
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)

    def metadata(self, definitions):
        paths = {name: self.root / directory for name, directory, _ in definitions}
        packages = []
        for name, directory, dependencies in definitions:
            edges = []
            for destination, kind in dependencies:
                edges.append({"name": "alias-is-not-package-identity", "rename": "local_alias",
                              "path": str(paths[destination]), "kind": kind})
            packages.append({"id": name, "name": name,
                             "manifest_path": str(self.root / directory / "Cargo.toml"),
                             "dependencies": edges})
        return {"workspace_members": list(paths), "packages": packages}

    def report(self, definitions):
        return AUDIT.audit_workspace(self.metadata(definitions), self.root)

    def test_future_domains_share_contracts_without_an_enumerated_allowlist(self):
        report = self.report([
            ("yir-core", "crates/yir-core", []),
            ("yir-domain-future-hardware", "crates/future", [("yir-core", None)]),
            ("yir-domain-cpu", "crates/cpu", [("yir-core", None)]),
        ])
        self.assertEqual(report["violations"], [])
        self.assertEqual(report["domain_count"], 2)

    def test_transitive_cross_domain_dependency_is_rejected(self):
        report = self.report([
            ("yir-domain-a", "crates/a", [("bridge", None)]),
            ("bridge", "crates/bridge", [("yir-domain-b", None)]),
            ("yir-domain-b", "crates/b", []),
        ])
        self.assertIn({"rule": "domain-reaches-another-domain",
                       "path": ["yir-domain-a", "bridge", "yir-domain-b"]},
                      report["violations"])

    def test_build_dependencies_cannot_hide_tool_coupling(self):
        report = self.report([
            ("library", "crates/library", [("compiler", "build")]),
            ("compiler", "tools/compiler", []),
        ])
        self.assertEqual(report["dependency_edge_count"], 1)
        self.assertEqual(report["violations"][0]["rule"], "implementation-or-domain-reaches-tool")

    def test_dev_only_edges_and_cycles_are_not_production_dependencies(self):
        report = self.report([
            ("yir-domain-a", "crates/a", [("yir-domain-b", "dev")]),
            ("yir-domain-b", "crates/b", [("yir-domain-a", "dev")]),
        ])
        self.assertEqual(report["violations"], [])
        self.assertEqual(report["dependency_edge_count"], 0)
        self.assertEqual(report["dev_edge_count"], 2)

    def test_foundations_cannot_reach_domain_implementations_through_a_bridge(self):
        report = self.report([
            ("yir-core", "crates/yir-core", [("bridge", None)]),
            ("bridge", "crates/bridge", [("yir-domain-new", None)]),
            ("yir-domain-new", "crates/new", []),
        ])
        self.assertTrue(any(v["rule"] == "foundation-reaches-domain-implementation"
                            for v in report["violations"]))

    def test_composition_roots_and_tool_library_edges_are_not_falsely_forbidden(self):
        report = self.report([
            ("yir-verify", "crates/verify", [("yir-domain-new", None)]),
            ("yir-domain-new", "crates/new", []),
            ("nsld", "tools/nsld", [("nuisc", None)]),
            ("nuisc", "tools/nuisc", [("yir-verify", None)]),
        ])
        self.assertEqual(report["violations"], [])

    def test_missing_or_external_members_are_not_silently_accepted(self):
        with self.assertRaises(ValueError):
            AUDIT.audit_workspace({"workspace_members": ["missing"], "packages": []}, self.root)
        metadata = self.metadata([("outside", "../outside", [])])
        with self.assertRaises(ValueError):
            AUDIT.audit_workspace(metadata, self.root)

    def test_cli_failure_is_nonzero_and_json_has_only_relative_manifest_paths(self):
        metadata = self.metadata([
            ("yir-domain-a", "crates/a", [("yir-domain-b", None)]),
            ("yir-domain-b", "crates/b", []),
        ])
        output = io.StringIO()
        with patch.object(AUDIT, "ROOT", self.root), patch.object(AUDIT, "load_metadata", return_value=metadata), redirect_stdout(output):
            status = AUDIT.main(["--json"])
        self.assertEqual(status, 1)
        report = json.loads(output.getvalue())
        self.assertTrue(report["violations"])
        self.assertTrue(all(not Path(p["manifest"]).is_absolute() for p in report["packages"]))

    def test_cli_is_read_only_and_resolves_root_without_an_installed_rust_toolchain(self):
        before = (ROOT / "Cargo.lock").read_bytes()
        metadata = {"workspace_members": ["contract"], "packages": [{
            "id": "contract", "name": "contract",
            "manifest_path": str(ROOT / "Cargo.toml"), "dependencies": [],
        }]}
        binary = self.root / "bin"
        binary.mkdir()
        arguments = self.root / "cargo-args.txt"
        invocation = self.root / "cargo-cwd.txt"
        stub = binary / "cargo"
        stub.write_text(
            "#!/bin/sh\n"
            f"printf '%s\\n' \"$@\" > {shlex.quote(str(arguments))}\n"
            f"pwd > {shlex.quote(str(invocation))}\n"
            f"printf '%s\\n' {shlex.quote(json.dumps(metadata))}\n",
            encoding="utf-8",
        )
        stub.chmod(0o700)
        environment = os.environ.copy()
        environment.update(PATH=str(binary) + os.pathsep + environment["PATH"],
                           CARGO_HOME=str(self.root / "empty-cargo-home"),
                           RUSTUP_TOOLCHAIN="deliberately-not-installed")
        result = subprocess.run(
            ["python3", str(ROOT / "scripts/check-workspace-architecture.py"), "--json"],
            cwd=self.root, env=environment, capture_output=True, text=True,
            check=False, timeout=30,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["violations"], [])
        self.assertEqual((ROOT / "Cargo.lock").read_bytes(), before)
        self.assertEqual(invocation.read_text().strip(), str(ROOT))
        self.assertEqual(arguments.read_text().splitlines(), [
            "metadata", "--manifest-path", str(ROOT / "Cargo.toml"),
            "--offline", "--locked", "--no-deps", "--format-version", "1",
        ])

    def test_workflow_runs_architecture_audit_after_locked_fetch(self):
        workflow = (ROOT / ".github/workflows/build.yml").read_text(encoding="utf-8")
        command = "python3 scripts/check-workspace-architecture.py"
        self.assertLess(workflow.index("cargo fetch --locked"), workflow.index(command))
        self.assertLess(workflow.index(command), workflow.index("cargo build --workspace --locked"))

    def test_developer_profiles_keep_incremental_cache_opt_in(self):
        source = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        for profile in ["dev", "test"]:
            section = re.search(r"(?m)^\[profile\." + profile + r"\]\s*\n([^\[]*)", source)
            self.assertIsNotNone(section)
            self.assertRegex(section.group(1), r"(?m)^incremental\s*=\s*false\s*$")


if __name__ == "__main__":
    unittest.main()
