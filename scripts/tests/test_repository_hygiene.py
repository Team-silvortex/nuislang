"""Keep regeneratable outputs out of a source-only checkout."""

from pathlib import Path
import os
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]


class RepositoryHygiene(unittest.TestCase):
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
