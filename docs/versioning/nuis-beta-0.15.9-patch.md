# Beta 0.15.9 Patch

Date: 2026-10-05. Source revision: `585eb409` (`beta-0.15.9`), following
`98ca713e` (`beta-0.15.8`) and minor baseline `05951bef` (`beta-0.15.0`).
The user committed this patch. Git is authoritative; Cargo versions and
protocol identifiers are independent. This document does not create a release.

## Included Changes

The patch includes bounded parent return-print prefixes, selected computed
print arguments, interleaved scalar aliases, ordered checked/call-backed
scalar initialization and direct logical initializer roots. Conditions and
initializers preserve short-circuiting and selected once-only computation,
including unused checks. Parent effects and actual source exits remain ordered;
original scope/type/capture authority and shared proof budgets remain intact.

The [logical initializer proof](../reference/nuis-native-logical-staged-initializers-v1.md)
records the final included checkpoint. Its references retain earlier,
independently verified steps without widening public ABI, native limits or
pure helper admission.

## Recorded Evidence

The included logical checkpoint recorded 2101 distinct selected passing tests:
1994 compiler units, twelve separate native AOT units, 58 native bridge tests,
29 tensor tests, two source-free workflows, five application-session tests
and one host-path policy test. Overlapping focused runs were not added again.

The rebuilt CLI recorded `active/99`, 1981/1981 live drift checks and clean
coverage, hierarchy and task-card lineage. Static checks recorded 1981 literal
definitions/13506 patterns, 58 changed-file caps, 4749 UTF-8 files and 3095
documentation links. Formatting and whitespace passed.

These are historical selected receipts included by this commit, not suites
rerun solely for this version document. Three blocked Unix-socket tests and
eleven strict-Clippy findings remained pending. Full-workspace, fresh
Linux/GPU/Windows, performance and formal-safety acceptance were not claimed.

## Subsequent Work

The [continuing effect-region proof](../reference/nuis-native-continuing-effect-regions-v1.md)
is worktree development after `585eb409`. Its new evidence is separate from
this patch. The mainline remains ns-nova application-led foundation work;
neither general native application closure nor compiler self-hosting is complete.
