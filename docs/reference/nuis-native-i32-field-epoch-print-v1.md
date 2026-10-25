# Native I32 Field Epoch Print V1

This bounded CPU worktree follow-up continues the [bool argument proof](nuis-native-bool-field-epoch-print-v1.md)
on Git beta-0.17.8 at `2db5c82c`. It changes no release version, native ABI,
print policy or backend-specific implementation. Printed values remain i64.

## Shared Typed Argument

The [completed signature owner](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs)
now exposes the exact parameter type at an original argument index only for a
completed pure helper returning i64. It replaces the bool-only query rather
than adding another per-kind copy. Full argument arity/type validation remains
mandatory after checking all arguments.

The [epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
accepts exact unqualified bool or i32 parameter types for this bounded profile.
It reuses the original initialized/writable/changed root or genuinely ready
child entry and the [same field-path/shape certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs).
Whole-record roots never enter the print view's available set. The
[expression checker](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
queries parameter types lazily only when a print certificate exists, retaining
the original recursion depth boundary. Ordinary expressions do not acquire a
new lookup path or new readiness from this certificate.

No implicit widening or whole-record call is admitted. Direct unavailable i32
prints, widening casts around unavailable i32 fields, compound i32 arguments,
wrong parameter/result types, non-pure callees, stale/nonowned/malformed roots,
ordinary assignments/conditions/initializers and unavailable ancestors keep
their vetoes. Existing explicit narrowing of an independently permitted i64
value remains legal; it is not confused with a new i32 field grant. Existing
bool argument controls and direct i64 field permission are retained.

The 4,096-work/64-depth print-root budget, exact complete layout/path proof,
source-order renderer, backedge word codecs and i64-v1 policy are unchanged.
No new source intrinsic, foreign shim, runtime allocation or fixture-name case
is introduced.

## Signed Execution Witness

Five new [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_field_print_epoch_tests.rs)
check exact parameter/leaf types, qualifier/stale-layout rejection, single-print
isolation, depth, cast/result/whole-record vetoes and ancestor isolation. Raw
NIR executes both induction orders, selected/skipped print work, zero trips,
MIN/MAX, negative/zero and truncation inputs without simplification.

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/i32_field_epoch_print.ns)
uses nested i32 total/i64 tag records as parent and child carries and publishes
an i32 State. Arithmetic explicitly narrows the delta and wraps at i32 width.
The pure i64 observer returns the current i32 sign, with checked division on
each sign branch; the final checked i32 helper runs before State publication.
No missing source widening intrinsic is invented just for this test.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_field_epoch_print_cases.rs)
uses i32 truncation/wrapping and sign-extended i64 expected words. One normal
sequence keeps 64 -> 72 -> -2; five additional independent sequences start from
MIN, -1, zero, MAX and `4294967295`, then exercise wrapped updates and skipped
poison. Its nine failure rows retain the last accepted State and exact reached
callee/print prefixes rather than publishing a partially computed result.

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/i32_field_epoch_print.rs)
pin signed State words at open/events, complete flushed native fault prefixes,
callee entries, normal/reversed YIR and canary outputs. The fixture uses 12
total loop work, 192 helper entries and at most 32 functions. The complete
three-by-three sign-observer path exhausts 128 entries; a separate negative
control retains that 128-entry rejection and proves an exact flushed source
prefix with no State publication. These are explicit test bounds, not widened
backend admission. Seven existing
[i32 transport checks](../../tools/nuisc/tests/native_application_bridge/i32_scoped_record_carries.rs)
cover signed wide backedges, snapshots, nested iterations, atomic limits and
wrong cast-kind rejection independently of the new print profile.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/i32_field_epoch_print.rs)
runs all six healthy sequences before and after failures in fresh/cache and two
independent source-free images. It pins exact `i32` State rendering and retains
binary/container, host-object, envelope, lifecycle and packaging identities.
Native probes flush and prove complete fault prefixes; CLI checks only stdout
actually flushed by a trapping child.

## Validation

Bounded local M2 CPU validation passed 363 distinct targeted tests, including
nine new tests. Repeated focused runs are counted once:

- Compiler: 216 focused/adjacent tests in 63.23 seconds, including five new i32
  controls and the migrated bool controls, existing field/root/child/record/word
  checks and shared scalar-record shapes.
- Native/reference: 64 tests in 185.27 seconds. Three new tests cover signed
  State/print equivalence, faulted reference sessions and 128-entry exhaustion.
  The matrix retains seven i32 codec tests and prior literal entry-budget guards.
- CLI delivery: one new test in 101.72 seconds. All six healthy sequences run
  before and after nine failures in fresh/cache and two source-free images.
- Producer policy: three LLVM build-policy and three packaging-option tests.
- Borrowed-buffer integration: 17 tests in 38.39 seconds; host failure state:
  three; repository maintenance: 26 in 21.311 seconds; host absolute-path policy:
  one; file-line limits: two; tensor tests: 27 in 6.36 seconds.

The first native run exposed configured helper-entry exhaustion in the full
three-by-three sign-observer path, not a word-value mismatch. Success now uses
an explicit 192-entry fixture bound; a new 128-entry negative test proves the
original protective rejection rather than removing that case or changing the
producer's budget rules. Both print-policy branches of the shared test probe
retain their independently selected gates.

The final CLI-test compilation and ordinary CLI build passed without warnings
after delivery finished. Formatting, whitespace, UTF-8 across 5,802 files,
4,325 canonical documentation links and locked offline metadata for 29 packages
passed. All 3,812 drift checks and 25,795 required patterns match.

The rebuilt CLI matches all six source fields. Complete predecessor SHA-256
identities and quoted inert history are verified without executing historical
commands. Coverage, hierarchy and lineage are clean, the next task card is
ready and the selected coordinate remains active/99.

These are targeted CPU checks, not a full workspace run or performance evidence.
Earlier CLI delivery families, the full independent/shared storage matrix,
physical float/performance tests and GPU/Linux execution were not rerun.
Earlier receipts remain historical rather than new i32-field acceptance.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
an exact pre-update f32 field at an exact f32 parameter of a completed pure-i64
print helper. It needs independent raw IEEE-word and selected/skipped-fault
controls, including signed zero and non-finite payloads. The current bool/i32
policy does not authorize float fields. References/resources, whole-record
readiness, general effects/I/O/async capture, outer writes, stable ABI,
GPU/Linux parity and self-hosting remain separate gates.

The subsequent [typed-f32 proof](nuis-native-f32-field-epoch-print-v1.md) advances
the f32 frontier with raw callee/State words and an explicitly finite CLI subset.
The totals above remain this i32 checkpoint's own acceptance results.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_field_epoch_print:: bool_field_epoch_print:: field_epoch_print:: typed_i32_scoped_record_carries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete bool-field checkpoint fields are retained by SHA-256.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `5a3a95013c76952765bb72638954301854e3130cecb40cf8ce325068693012fb` |
| `next_step` | `0e096c1294cf02589f80ba32885ea18c4f3af2f6630d9a296db9dd184ff384d3` |
| `blocker` | `8b3b5bcb7953be72d4b2095d61b858b47da389e3fbd1e3564c7bbcd9a4a200ea` |
| `next_action` | `6af52f88b6806dee83629d135fe35d040d60f2a7b732a723fa0f9ae7f10e5f6d` |
| `validation_command` | `fb87a63b557f875a8c2f9988d393bb42c7f2042ca772e74d14957149390dc884` |
| `expected_artifact` | `2671a87cdb1e207904a690c638da099f47511e0eccbff698f66c9e8a3cbde541` |
