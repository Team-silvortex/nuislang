#!/usr/bin/env python3

"""Validate the encoding contract for tracked Nuis repository text files."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


TEXT_SUFFIXES = {
    ".bnf",
    ".c",
    ".gitignore",
    ".json",
    ".ll",
    ".lock",
    ".m",
    ".md",
    ".ns",
    ".pest",
    ".rs",
    ".sh",
    ".swift",
    ".toml",
    ".txt",
    ".yir",
    ".yaml",
    ".yml",
}
TEXT_FILENAMES = {
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    "LICENSE",
}


def tracked_text_paths(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    paths = []
    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        relative = Path(raw_path.decode("utf-8"))
        if relative.name in TEXT_FILENAMES or relative.suffix.lower() in TEXT_SUFFIXES:
            paths.append(relative)
    return paths


def forbidden_codepoint(character: str) -> bool:
    value = ord(character)
    if value < 0x20 and character not in {"\t", "\n"}:
        return True
    if value == 0x7F:
        return True
    if 0x200B <= value <= 0x200F:
        return True
    if 0x202A <= value <= 0x202E:
        return True
    if 0x2060 <= value <= 0x206F:
        return True
    return value == 0xFEFF


def validate_file(root: Path, relative: Path) -> list[str]:
    data = (root / relative).read_bytes()
    failures = []
    if data.startswith(b"\xef\xbb\xbf"):
        failures.append(f"{relative}: UTF-8 BOM is not allowed")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        return [f"{relative}: invalid UTF-8 at byte {error.start}: {error.reason}"]
    if "\r" in text:
        line = text.count("\n", 0, text.index("\r")) + 1
        failures.append(f"{relative}:{line}: CR/CRLF line ending is not allowed; use LF")
    for offset, character in enumerate(text):
        if not forbidden_codepoint(character):
            continue
        line = text.count("\n", 0, offset) + 1
        column = offset - text.rfind("\n", 0, offset)
        failures.append(
            f"{relative}:{line}:{column}: forbidden control character U+{ord(character):04X}"
        )
    return failures


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    paths = tracked_text_paths(root)
    failures = [
        failure
        for relative in paths
        for failure in validate_file(root, relative)
    ]
    if failures:
        print("text encoding contract: failed", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print(f"text encoding contract: ok ({len(paths)} tracked UTF-8 files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
