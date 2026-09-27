# Beta 0.15.4 Patch

Date: 2026-09-27. Source version: `beta-0.15.4`, following `6e9a27b8`
(`beta-0.15.3`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol identifiers,
public function signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.3-patch.md) retain their historical evidence.

## Included Changes

Generated loop branch helpers now retain proven whole-record inputs instead of
flattening a wide record and its predicate beyond the 64-argument bound. Selected
call/constructor work stays guarded; predicates, complete seeds, induction and
break identities remain independent. Signature rewrites still require agreement
at every scoped caller and exact nominal, source-field and result-slot proofs.

The scoped carry profile now supports flat records containing `bool`, `i32`,
`i64`, `f32` and `f64`, plus independent scalar carries. Bool words are explicitly
encoded and decoded. I32 words preserve sign extension, width-correct wrapping
arithmetic and exact backedge decoding. Typed parameter projections provide total
guard defaults without speculating calls, division or other selected work.

Private `PackF32Word` / `UnpackF32Word` and `PackF64Word` / `UnpackF64Word`
expressions lower through registered CPU operations. These are bit-preserving
transports, not source builtins or numerical float/integer casts. F32 occupies
the low 32 bits of a canonical word; f64 preserves all 64 bits, including the
sign bit. Reference execution and LLVM enforce exact operand kinds and retain
negative zero, NaN payloads, infinities and subnormal bit patterns during transport.
Arithmetic does not promise NaN-payload preservation. The bounded source value-loop
profile adds float add/subtract/multiply, not float division, remainder or comparisons.

Regressions cover zero trips, immutable snapshots, nested iterations, counted
returns, scalar companions, all five kinds in one record and 64-field private
inputs. Selected arithmetic failures and exhausted work budgets preserve output
sentinels and retain zero owned-aggregate allocations/drops in the native probes.
Source/public/FFI/callback signatures, loop metadata and work accounting are unchanged.

Six CLI record shapes exercise build/cache identity, tamper rejection and source-free
restoration with byte-identical LLVM and matching lifecycle states. Native probes
now observe the declared source-helper boundary rather than assuming private
iteration wrappers preserve source parameter order. Field-derived loop bounds in
these carry probes remain named locals under the existing bounded loop profile.

The [value-return and carry contract](../reference/nuis-native-scalar-value-returns-v1.md)
records exact admission rules and dated checkpoints. The
[mainline map](../current-mainline-map.md), README and development tensor track
this bounded native CPU slice separately from general aggregate/resource support.

## Validation Evidence

The latest implementation checkpoint on macOS aarch64 passed 1884 distinct
selected tests after targeted repair reruns:

* 1652 frontend, lowering, walker and NIR-verifier tests
* 123 native/reference bridge regressions
* 24 ordinary native execution tests
* six CLI build/cache/tamper/source-free restoration workflows
* nine CPU scalar tests and 39 LLVM native-session/cast tests
* five reference image/window session tests
* 26 development-tensor tests

Overlapping reruns are excluded. One opt-in LLVM codec probe was not enabled in
this checkpoint; its earlier `-O0`/`-O2` results remain historical. The host-path
policy test passed separately. Intermediate failures and their probe/fixture
repairs are recorded in the reference checkpoint rather than hidden from the total.

The rebuilt CLI reports 1531 clean drift checks with clean coverage, hierarchy
and task lineage. Formatting, source/test/document line budgets, local links and
UTF-8 checks pass. These are selected local results, not a full-workspace run,
fresh Linux/GPU acceptance, formal-safety certification or measured speedup.
The [validation checklist](nuis-beta-0.15.0-release-checklist.md) lists broader
follow-up suites; a listed command does not imply it ran for this patch. Remote
CI success must be checked for the pushed revision, not inferred from local results.

Release preparation separately reran 26 tensor tests, 20 maintenance-script tests
and the host-absolute-path policy test. The live tensor report, full-workspace
format check, local documentation links, UTF-8 and changed-file line budgets also
passed. These release checks do not increase the implementation-test total above.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
The next task is exact field-path seed/backedge mapping for nested pure-scalar
records. Nested loops are covered; nested record carries are not yet admitted.

Sparse mixed backedge projection, resource-bearing state, deferred record tasks,
provider dispatch and self-contained application packaging remain separate work.
This patch does not freeze ABI, complete compiler self-hosting or claim general
native application closure.
