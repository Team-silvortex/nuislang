"""Keep source checkouts clean and their current entrypoints discoverable."""

from pathlib import Path
import os
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]
ENTRY_ROUTERS = ("README.md", "docs/README.md", "docs/current-mainline-map.md")


def local_markdown_targets(relative):
    source = ROOT / relative
    targets = set()
    for target in re.findall(r"\[[^\]]*\]\(([^)\s]+)\)", source.read_text(encoding="utf-8")):
        target = target.split("#", 1)[0]
        if target and ":" not in target:
            targets.add((source.parent / target).resolve())
    return targets


class RepositoryHygiene(unittest.TestCase):
    def test_aot_packer_frontdoor_does_not_absorb_source_generators(self):
        root = ROOT / "tools/yir-pack-aot/src"
        source = (root / "main.rs").read_text(encoding="utf-8")
        for module in ("backend_artifacts", "shader_codegen", "host_ffi", "host_objc", "host_compile"):
            self.assertIn("mod " + module + ";", source)
            self.assertTrue((root / (module + ".rs")).is_file())
        for implementation in (
            "fn objc_host_source(", "fn render_shader_glsl_scaffold(",
            "fn collect_host_ffi_symbols(", "PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES",
            "#include <",
        ):
            self.assertNotIn(implementation, source,
                             "the packager entrypoint must delegate backend and host details")

    def test_entry_routers_stay_short_and_link_to_the_proof_catalog(self):
        catalog = (ROOT / "docs/reference/nuis-native-session-proof-index.md").resolve()
        for relative in ENTRY_ROUTERS:
            with self.subTest(path=relative):
                text = (ROOT / relative).read_text(encoding="utf-8")
                self.assertLessEqual(len(text.splitlines()), 300,
                                     "proof narratives belong in focused references, not routers")
                self.assertIn(catalog, local_markdown_targets(relative))

    def test_native_receipts_remain_reachable_from_the_proof_catalog(self):
        relative = "docs/reference/nuis-native-session-proof-index.md"
        catalog = (ROOT / relative).resolve()
        receipts = {p.resolve() for p in catalog.parent.glob("nuis-native-*.md")} - {catalog}
        self.assertTrue(receipts)
        missing = receipts - local_markdown_targets(relative)
        self.assertEqual(sorted(p.name for p in missing), [],
                         "keep native receipts discoverable when shortening entrypoints")

    def test_current_series_routers_agree_on_snapshot_and_checklist(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        series = re.search(r"repository is on `([^`\s]+)\.\*`", readme)
        self.assertIsNotNone(series, "README must identify the current Git minor series")
        prefix = "nuis-" + series.group(1) + ".0-"
        expected = {(ROOT / "docs/versioning" / (prefix + suffix)).resolve()
                    for suffix in ("snapshot.md", "release-checklist.md")}
        for path in expected:
            self.assertTrue(path.is_file(), path.name)
        for relative in (*ENTRY_ROUTERS, "docs/reference/README.md", "docs/versioning/README.md"):
            with self.subTest(path=relative):
                self.assertTrue(expected <= local_markdown_targets(relative),
                                "current routers must not point at an earlier minor's operating gates")

    def test_source_checkout_does_not_ship_generated_outputs(self):
        paths = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            cwd=ROOT, capture_output=True, check=True, timeout=30,
        ).stdout.split(b"\0")
        rejected = []
        for raw in paths:
            if not raw:
                continue
            relative = Path(os.fsdecode(raw))
            if not (ROOT / relative).is_file():
                continue
            generated = any(part in {"target", ".nuis", "__pycache__"} for part in relative.parts)
            generated |= relative.name == ".DS_Store" or relative.suffix in {".pyc", ".pyo"}
            generated |= relative.parts[:3] == ("tools", "yir-preview-macos", "build")
            generated |= relative.parts[:2] == ("examples", "bins") and relative != Path("examples/bins/README.md")
            if generated:
                rejected.append(relative.as_posix())
        self.assertEqual(rejected, [], "generated outputs belong in ignored local build directories")


if __name__ == "__main__":
    unittest.main()
