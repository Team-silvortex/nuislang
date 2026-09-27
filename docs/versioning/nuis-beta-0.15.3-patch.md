# Beta 0.15.3 Patch

Date: 2026-09-27. Source version: `beta-0.15.3`, following `ad026be0`
(`beta-0.15.2`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol version identifiers,
public function signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.2-patch.md) retain their historical evidence.

## Included Changes

Generated scoped helpers can omit entirely unread flat-record inputs after every
caller proves the complete seed range and exact nominal reconstruction. Complete
initial state, selected initializer work, zero-trip values and output storage
remain intact. Record input demand no longer determines whether state slots exist;
induction, bool and break-control inputs retain their independent identities.

Native helper/callback returns and record inputs now share a checked pure-value
codec. It validates the exact nominal tree, complete field set and scalar kinds
within the existing 1-to-64-leaf bound before emission. Owned-variant prefix
coercion is no longer accepted as pure-value compatibility, and absent fields are
not synthesized as zero. The LLVM transport uses bounded `[N x i64]` values, not
owned aggregate runtime allocation.

The registered CPU domain exposes `cpu.param_value_struct` for compiler-private
record parameters. Generic registration hooks validate function declarations and
runtime argument values; reference execution checks arguments before changing the
call frame, and the host registry forwards the same contracts. Generated non-scoped
helpers whose captures remain wider than 64 parameters after field projection and
bool compaction can use whole-record inputs instead. Source/public/FFI signatures,
callback word layouts and deferred-task admission are not widened.

Generated scoped flat-i64 loops now carry explicit per-trip record input maps.
Complete seed storage remains separate from helper operands, and every scoped
caller must agree on the seed range before the compiler changes the signature.
Reference and LLVM execution reconstruct the record from current scalar, carry
and invariant inputs; dependencies and GLM reads account for actual leaves rather
than descriptor metadata. Native emission prepares all record arguments before
packing any of them. Duplicate, missing or mismatched maps and nominal/kind drift
are rejected without partially changing the candidate or publishing output.

Source-to-native regressions cover a 64-field record plus induction, a 62-field
record with break, and a 61-field record with independent bool/break carries.
Selected arithmetic and work-budget failures preserve output sentinels. CLI
workflows exercise build/cache reuse, artifact tamper rejection and source-free
restoration with byte-identical LLVM and lifecycle states, without interpreter
fallback for a selected native failure.

The [value-return and capture contract](../reference/nuis-native-scalar-value-returns-v1.md)
records the wire forms, admission proofs and dated validation checkpoints. The
[mainline map](../current-mainline-map.md), README and development tensor track this
bounded native CPU slice without claiming general aggregate or resource support.

## Validation Evidence

The latest implementation checkpoint on macOS aarch64 passed 1211 distinct
selected tests:

* 420 core-contract, registered-CPU, verifier and LLVM unit tests
* 674 compiler-lowering unit tests
* 89 selected native/reference bridge regressions, including corrected-test reruns
* two CLI record build/cache/tamper/source-free restoration workflows
* 26 development-tensor tests

Overlapping reruns are excluded. One opt-in LLVM host test was not enabled in
that checkpoint. The preceding generated-record-input checkpoint passed its full
246-test native-bridge suite and an explicit host-clang ABI probe; those historical
results are not a claim that the subsequently expanded full suite was rerun.
Earlier unread-input and shared-codec checkpoints remain separately recorded in
the reference document and must not be added to the latest total.

The rebuilt CLI reports 1492 clean drift checks with clean coverage, hierarchy and
task lineage. These are selected local results, not a full-workspace run, fresh
Linux/GPU acceptance, formal proof or measured speedup. The
[validation checklist](nuis-beta-0.15.0-release-checklist.md) lists broader follow-up
suites; listed commands do not imply that every suite has run for this patch.

Release preparation separately reran all 26 tensor tests, all 20 maintenance-script
tests and the host-absolute-path policy test successfully. The CLI build and fresh
tensor report passed, as did local documentation links, UTF-8, formatting for all
88 changed Rust files and line budgets for all 98 changed source/test/docs files.
These release checks are not additional implementation-test totals. Remote CI
success must be verified for the pushed commit rather than inferred here.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
The next task is to lower whole-record inputs in generated branch helpers without
flattening wide captures or speculating unselected arms. A source regression still
reproduces a separate generated branch helper with 65 scalar parameters; this
patch keeps its rejection instead of increasing the native argument bound.

Mixed/nested scoped carries, deferred record tasks, resource state, provider
dispatch and self-contained application packaging remain separate admission and
implementation work. Compiler self-hosting is not completed by these transport
changes, and this release is neither an ABI freeze nor complete native application
closure.
