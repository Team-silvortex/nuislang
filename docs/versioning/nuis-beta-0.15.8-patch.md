# Beta 0.15.8 Patch

Date: 2026-10-05. Source revision: `98ca713e` (`beta-0.15.8`), following
`5d6cbc1a` (`beta-0.15.7`) and minor baseline `05951bef` (`beta-0.15.0`).
The user committed this release. Git is authoritative; Cargo package versions,
public signatures and protocol identifiers are not bumped here.

## Included Changes

The release contains the bounded return/capture lowering progression since the
[previous patch record](nuis-beta-0.15.7-patch.md), including pure computed outer
entries, return-arm predicates, stored exit signals and bounded nested logical
trees. Direct `&&`/`||` children preserve source short-circuiting and evaluate
each selected input once; helper construction retains original type/capture
authority and shared node/depth/edge budgets. Evaluated calls and selected checks
are not removed simply because their result is unobserved.

The [computed logical gate reference](../reference/nuis-native-computed-logical-gates-v1.md)
and [scalar snapshot reference](../reference/nuis-native-scalar-loop-snapshots-v1.md)
retain chronological proof boundaries and preceding acceptance receipts.
Public ABI, native carry limits and pure helper admission remain unchanged.

## Release Checkpoint

The preceding logical-tree acceptance recorded 2063 distinct selected passing
tests on local macOS aarch64: 1964 compiler units, five separately selected AOT
units, 58 native bridge tests, 28 tensor tests, two source-free workflows, five
application-session tests and one host-path policy case. Overlapping reruns are
not added to this total. New nested-tree evidence included 672 runtime source
cases, 1344 ordinary/reversed executions and fifteen AOT variants.

The rebuilt CLI recorded `active/94`, 1932/1932 live drift checks and clean
coverage, hierarchy and lineage. Static checks recorded 1930 definitions and
13082 patterns; 4712 UTF-8 files, 3058 documentation links, formatting,
whitespace and changed-file caps passed at that checkpoint.

This is historical selected acceptance included by `98ca713e`, not a claim that
all suites were rerun solely for this release record. Three blocked Unix socket
tests and eleven strict-Clippy findings remained pending. Full-workspace,
fresh Linux/GPU/Windows, performance and formal-safety acceptance were not claimed.

## Post-Release Work

The [leading return-print prefix proof](../reference/nuis-native-return-print-prefixes-v1.md)
is subsequent worktree development, not part of `98ca713e`. Its fresh receipt
and current tensor evidence must not be attributed to this committed release.
The later [computed print arguments](../reference/nuis-native-computed-return-prints-v1.md)
and [interleaved scalar aliases](../reference/nuis-native-return-print-aliases-v1.md)
retain separate worktree receipts as well.
The [staged initializer proof](../reference/nuis-native-staged-return-effects-v1.md)
is a further post-release checkpoint, not part of this committed patch.
The [logical initializer proof](../reference/nuis-native-logical-staged-initializers-v1.md)
also retains its own worktree receipt and does not change this release record.
The mainline remains ns-nova application-led foundation development; neither
compiler self-hosting nor general native application closure is complete.
