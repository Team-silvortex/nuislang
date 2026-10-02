# Beta 0.15.5 Patch

Date: 2026-10-02. Source version: `beta-0.15.5`, following `b02829ac`
(`beta-0.15.4`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol identifiers,
public function signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.4-patch.md) retain their historical evidence.

## Included Changes

Scoped loop carries now admit nested pure-scalar records containing bool, i32,
i64, f32 and f64. Complete field-segment identities and nominal reconstruction
keep same-named leaves distinct. Shared shape proofs connect capture planning,
source lowering and exact native word transport without changing public ABI.

Sparse typed capture projection retains complete seed storage, selected
initializer work, canonical codecs and explicit per-trip slot maps. Lexical
aliases, subrecord snapshots and identity-preserving writes retain version
proofs. Backward field demand crosses materialized joins and nested loops through
bounded fixed points with lexical break/continue targets. Unknown origins and
whole escapes remain conservative rather than silently dropping state.

Nested early returns may reuse a proven initialized mutable record, remove only
unobserved induction recovery and share a compiler-minted return/break signal.
Original source admission and whole-function revalidation remain mandatory.
Ordinary breaks, observed exit values, source ordering and selected traps keep
their independent obligations; rejected candidates retain the original plan.

Invariant promotion now reaches outer return-owned loops. A monotone fixed point
joins zero trips and every intermediate child write, not merely final backedges.
Proven unchanged leaves stay outside the backedge while changing leaves use a
private typed record. Work/depth exhaustion is conservative. At most two
independently bounded attempts are made: nested promotion, then an established
inner-only fallback from the original plan. Failed attempts cannot install
partial private definitions or discard an admitted inner plan.

The mixed nested-return fixture now executes 9/30/31/57/58/59/60/61/62/63/64-word
states with outer N and inner N-1 carries. The former 65/63 boundary at width 64
is now 64/63 by proving an unchanged outer tag, not raising the 64-word limit or
deleting the return signal. Changing that tag still requires 65 outer words and
rejects. This is a proven profile, not a claim that every 64-word program fits.

Regressions preserve exact work budgets, raw floating-point bits, malformed-input
rejection before work, immutable snapshots and atomic output publication for
disjoint or overlapping buffers. Native probes retain zero owned-aggregate
allocation/drop calls. Thirteen CLI workflows exercise build/cache identity,
tamper rejection before opening a session, and restoration after removing source
and build inputs. The 9/64-word return fixtures retain complete state and identical
LLVM with private iteration arities of 12/16 and 5/11 respectively.

The [value-return and carry contract](../reference/nuis-native-scalar-value-returns-v1.md)
records exact admission rules and dated checkpoints. The
[mainline map](../current-mainline-map.md), README and development tensor keep
this native CPU work separate from provider dispatch and resource-bearing state.

## Validation Evidence

The latest implementation checkpoint on local macOS aarch64 passed 1985 distinct
selected tests after targeted repair reruns:

* 1729 frontend, lowering, walker and NIR-verifier tests
* 151 native/reference bridge regressions
* 34 ordinary native execution tests
* thirteen CLI build/cache/tamper/source-free restoration workflows
* 26 native LLVM unit tests
* five reference image/window session tests
* 26 development-tensor tests and one host-path policy case

Overlapping reruns are excluded. The 9/64-word CLI return pair reran after the
fallback guard. One optional LLVM probe remained ignored. The rebuilt CLI reports
1611 clean drift checks with clean coverage, hierarchy and task lineage.
Formatting, local links, UTF-8 and changed-file line limits passed at that
checkpoint. This is selected local evidence, not a full-workspace run, fresh
Linux/GPU acceptance, formal-safety certification or measured speedup.

The [validation checklist](nuis-beta-0.15.0-release-checklist.md) lists broader
follow-up suites; a listed command does not imply it ran for this patch. Remote
CI success must be checked for the pushed revision, not inferred from local results.

Release preparation separately reran 26 tensor tests, 20 maintenance-script tests
and the host-path policy case. The live tensor report, full-workspace format
check, 2961 local documentation links, 4589 UTF-8 files and line caps for 103
changed/new Rust and Markdown files passed. These release checks do not increase
the implementation-test total above; the full native suites were not repeated
solely for the version/documentation update.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
Next is broader outer invariant admission across ordinary child exits, preserving
ordered reads, observed exit values and the independently admitted inner plan.
Further return-snapshot reductions must keep the same bounded proof discipline.

Resource-bearing state, deferred record tasks, native provider dispatch and
self-contained application packaging remain separate work. This patch does not
freeze ABI, complete compiler self-hosting or claim general native application
closure. Independent Nustars and Galaxies remain behind their existing contracts.
