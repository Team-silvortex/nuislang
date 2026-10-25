# Native F32 Field Epoch Print V1

This bounded CPU worktree follow-up continues the [i32 argument proof](nuis-native-i32-field-epoch-print-v1.md)
on Git beta-0.17.8 at `2db5c82c`. It changes no release version, native ABI,
source intrinsic, print policy or backend-specific implementation. Print results
are still exactly i64.

## Exact Typed Grant

The [epoch argument policy](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
adds exact unqualified f32 alongside bool/i32. The completed pure-i64 parameter
query, original argument index, exact nominal field shape/path and initialized
root or genuinely ready child entry are unchanged. Record roots never become
generally available. Neither implicit f64 conversion nor direct unavailable
float printing/word packing, compound arguments, non-i64 or effectful callees,
ordinary conditions/initializers or unavailable ancestors gain permission.

The same 4,096-work/64-depth print-root budget and source-order renderer apply.
Existing sign-bit negation and f32 word codecs are reused; no numeric cast is
substituted for a raw-bit operation, and no new runtime wrapper is added.

Five [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/f32_field_print_epoch_tests.rs)
check exact signature/leaf kinds, stale and qualified roots, single-print depth
and scope, compound/word/whole-record vetoes, ancestor isolation and raw-NIR
selected/skipped failures. Raw NIR injects existing unpack-word nodes for exact
IEEE inputs rather than inventing source literals for arbitrary NaN payloads.

## Raw Callee Words

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/f32_field_epoch_print.ns)
keeps nested f32/i64 records as parent and child carries. Each update flips only
the f32 sign bit through ordinary source negation. A pure i64 observer classifies
negative, zero, NaN and other values with checked division on every branch. A
final pure f32 helper performs an unused checked division before returning the
unchanged float, so State publication cannot silently discard fallible work.

The [independent word oracle](../../tools/nuisc/tests/native_application_bridge/f32_field_epoch_print_cases.rs)
models sign changes by unsigned XOR, not floating arithmetic or compiler codecs.
Fifteen independent healthy sequences cover positive/negative zero, finite and
subnormal values, infinities, quiet/signaling NaN payloads and raw words equal
to observation markers 94/95. The nine failure rows preserve the last accepted
State and exact reached print/callee prefixes before publication.

Test-only LLVM probes read the actual f32 argument using a bitcast and unsigned
extension at the original `observe` and `finish` entries. They do not replace
source computation. The expected native stream pins both raw callee words and
raw returned State words, including NaN payloads and signed zero. Reference
call counts come from structural loop/branch counts, never from scanning marker
values in a payload stream; marker-valued data is therefore not event authority.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/f32_field_epoch_print.rs)
cover normal/reversed YIR, all raw-word sequences, canary outputs, complete
flushed fault prefixes and faulted-session State retention. The success fixture
uses 12 total loop work, 256 helper entries and at most 32 functions. A separate
128-entry NaN case proves protective exhaustion without State publication.
These are explicit fixture bounds, not changes to producer budget rules.
Eight existing [f32 transport checks](../../tools/nuisc/tests/native_application_bridge/f32_scoped_record_carries.rs)
independently cover raw codecs, nested/wide backedges, snapshots, arithmetic,
wrong word-kind rejection and atomic limits.

## IEEE Comparison Repair

The raw probe exposed an existing native mismatch: the callee words were
correct, but NaN classification differed from reference execution. Both
[ordinary equality lowering](../../crates/yir-lower-llvm/src/scalar_equality_lowering.rs)
and [loop comparison lowering](../../crates/yir-lower-llvm/src/loop_scalar.rs)
used ordered `one` for float inequality. They now use unordered `une` for both
f32 and f64, so a comparison involving NaN is unequal. Equality remains ordered
`oeq`; signed zeros compare equal and integer predicates are unchanged.

Two [backend controls](../../crates/yir-lower-llvm/src/tests/float_comparison_tests.rs)
pin ordinary and loop predicate emission at both widths. Two independent
[native comparison matrices](../../tools/nuisc/tests/native_application_bridge/float_ieee_comparisons.rs)
check 81 ordered pairs per width across zero signs, finite values, infinities,
quiet/signaling NaNs and subnormals, through open/event/close and raw State words.
This common lowering repair does not grant pre-update f64 field reads.

## Delivery Boundary

The [finite CLI delivery](../../tools/nuis/tests/native_session_workflow/f32_field_epoch_print.rs)
uses six healthy sequences with exact decimal round trips, including signed
zero, negative finite values and subnormals. Fresh/cache and two independent
source-free images each run those sequences before and after all nine failures,
while retaining binary/container, host-object, envelope, lifecycle and packaging
identities. State text and any flushed trap prefix are checked exactly.

Decimal CLI input is not claimed to preserve custom NaN payloads. That coverage
belongs to the direct raw-word native and reference probes above. Likewise,
classification output alone cannot prove signed-zero or NaN-payload identity;
the raw word checks are independent requirements.

## Validation

Bounded local M2 CPU validation passed 386 distinct targeted tests, including
13 new tests. Repeated focused runs are counted once:

- Compiler: 221 focused/adjacent tests in 68.23 seconds after the LLVM repair,
  including five new f32 controls and existing typed-field/root/child/record/word
  and shared shape controls.
- Native/reference: 77 tests in 171.39 seconds, including three f32 field/budget
  tests, two new IEEE comparison matrices and eight existing f32 codec tests.
  Raw probes cover all fifteen sequences and nine failures in both YIR orders.
- CLI delivery: one new test in 93.52 seconds. Six finite healthy sequences run
  before and after failures in fresh/cache and two independent source-free images.
- LLVM: eight build-policy/literal/comparison tests, including two new scalar
  and loop predicate controls; packaging options: three tests.
- Borrowed-buffer integration: 17 tests in 37.79 seconds; host failure state:
  three; repository maintenance: 26 in 21.000 seconds; host absolute-path policy:
  one; file-line limits: two; tensor tests: 27 in 6.17 seconds.

The final CLI-test compilation and ordinary CLI build passed without warnings
after delivery finished. Formatting, whitespace, UTF-8 across 5,811 files,
4,342 canonical documentation links and locked offline metadata for 29 packages
passed. All 3,828 drift checks and 25,873 required patterns match.

The actual rebuilt CLI matches all six source fields. Complete predecessor
SHA-256 identities and quoted inert history are verified without executing
historical commands. Coverage, hierarchy and lineage are clean, the next task
card is ready and the selected coordinate remains active/99.

These are targeted CPU checks, not a full workspace run or performance evidence.
Earlier CLI delivery families, the complete independent/shared storage matrix,
physical float/performance tests and GPU/Linux execution were not rerun. Custom
NaN payloads are not claimed for the decimal CLI subset. Earlier receipts remain
historical rather than new f32-field acceptance.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is an
exact pre-update f64 field at an exact f64 parameter of a completed pure-i64
print helper, with independent 64-bit IEEE words, signed-zero/non-finite and
selected/skipped-fault evidence. The current policy grants no f64 field reads.
Whole-record readiness, references/resources, general effects/I/O/async capture,
outer writes, stable ABI, GPU/Linux parity and self-hosting remain separate gates.

The subsequent [typed-f64 proof](nuis-native-f64-field-epoch-print-v1.md) advances
the f64 frontier with complete raw callee/State words and a finite CLI subset.
The totals above remain this f32/IEEE-repair checkpoint's own acceptance results.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- f32_field_epoch_print:: i32_field_epoch_print:: bool_field_epoch_print:: field_epoch_print:: typed_f32_scoped_record_carries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- f32_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete i32-field checkpoint fields are retained by SHA-256.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `3d56ccec5926ccb12f45416336cfb8fb0786cf1627aba7bf726a8f2420b0879c` |
| `next_step` | `caa18c1803a4f04f5185679c37fd6c5da90696140a35df46faeeee8e5f81002c` |
| `blocker` | `ec579839562e7b8333026d4f3dff92665c0b0156c053c51107b65d6cc97628d0` |
| `next_action` | `0deb9ec86feaf2cffd812e23808c520182a30e806fd54909332344e7ed6e0b93` |
| `validation_command` | `41e07021f7614a78ae1ad2b29ae6f4d569480366a38837e32a25320210afc71f` |
| `expected_artifact` | `c741303beb9b01448a7e557daeab3c7a9c0b639de536e2ab0fb668ddc95cc9fe` |
