#!/usr/bin/env python3
"""Audit declared workspace boundaries without compiling or modifying files."""

import argparse
from collections import deque
import json
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
FOUNDATIONS = {"yir-core", "nuis-artifact", "nuis-semantics"}


def audit_workspace(metadata, root):
    root = Path(root).resolve()
    members = set(metadata["workspace_members"])
    packages = {p["id"]: p for p in metadata["packages"] if p["id"] in members}
    if set(packages) != members:
        raise ValueError("workspace metadata omits a declared member")
    by_manifest = {}
    by_name = {}
    for package in packages.values():
        manifest = Path(package["manifest_path"]).resolve()
        try:
            relative = manifest.relative_to(root).as_posix()
        except ValueError as error:
            raise ValueError("workspace member is outside the repository") from error
        name = package["name"]
        if name in by_name or manifest in by_manifest:
            raise ValueError("workspace members must have unique names and manifests")
        by_name[name] = {"name": name, "manifest": relative}
        by_manifest[manifest] = name

    edges = set()
    dev_edges = set()
    for package in packages.values():
        for dependency in package["dependencies"]:
            if not dependency.get("path"):
                continue
            manifest = (Path(dependency["path"]) / "Cargo.toml").resolve()
            destination = by_manifest.get(manifest)
            if destination is None:
                continue
            kind = dependency.get("kind") or "normal"
            edge = (package["name"], destination, kind)
            (dev_edges if kind == "dev" else edges).add(edge)

    graph = {name: set() for name in by_name}
    for source, destination, _ in edges:
        graph[source].add(destination)

    violations = []
    for source in sorted(graph):
        pending = deque([(source,)])
        seen = {source}
        while pending:
            path = pending.popleft()
            for destination in sorted(graph[path[-1]]):
                if destination in seen:
                    continue
                seen.add(destination)
                route = path + (destination,)
                source_crate = by_name[source]["manifest"].startswith("crates/")
                destination_tool = by_name[destination]["manifest"].startswith("tools/")
                source_domain = source.startswith("yir-domain-")
                destination_domain = destination.startswith("yir-domain-")
                rule = None
                if (source_crate or source_domain) and destination_tool:
                    rule = "implementation-or-domain-reaches-tool"
                elif source in FOUNDATIONS and destination_domain:
                    rule = "foundation-reaches-domain-implementation"
                elif source_domain and destination_domain and source != destination:
                    rule = "domain-reaches-another-domain"
                if rule:
                    violations.append({"rule": rule, "path": list(route)})
                pending.append(route)

    return {
        "package_count": len(by_name),
        "domain_count": sum(name.startswith("yir-domain-") for name in by_name),
        "dependency_edge_count": len(edges),
        "dev_edge_count": len(dev_edges),
        "packages": [by_name[name] for name in sorted(by_name)],
        "edges": [{"from": a, "to": b, "kind": kind} for a, b, kind in sorted(edges)],
        "violations": violations,
    }


def load_metadata():
    result = subprocess.run(
        ["cargo", "metadata", "--manifest-path", str(ROOT / "Cargo.toml"),
         "--offline", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True, timeout=60,
    )
    return json.loads(result.stdout)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit repository-relative metadata")
    options = parser.parse_args(argv)
    try:
        report = audit_workspace(load_metadata(), ROOT)
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        print(f"workspace architecture audit failed: {error}", file=sys.stderr)
        return 2
    if options.json:
        print(json.dumps(report, indent=2))
    else:
        state = "ok" if not report["violations"] else "FAILED"
        print(f"workspace architecture: {state} ({report['package_count']} packages, "
              f"{report['domain_count']} domains, {report['dependency_edge_count']} "
              "normal/build edges; dev-only edges excluded)")
        for violation in report["violations"]:
            print(f"{violation['rule']}: {' -> '.join(violation['path'])}")
    return int(bool(report["violations"]))


if __name__ == "__main__":
    sys.exit(main())
