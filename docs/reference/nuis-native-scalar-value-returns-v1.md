# Native Scalar Value Returns V1

Status: private LLVM transport for the experimental
[native scalar session bridge](nuis-native-scalar-session-bridge-v1.md).
This is not a public ABI, a native provider callback contract, or a replacement
for the ordinary owned-aggregate ABI. It does not change application-host selection.

## Layout And Admission

Registered callback roots and ordinary scalar helpers return LLVM `[N x i64]`
values, not pointer bits to heap-owned aggregates. The transport accepts nonempty
nested nominal structs with 1..64 `bool`/`i32`/`i64`/`f32`/`f64` leaves. Empty structs,
resource fields, duplicate sibling names and duplicate flattened paths are rejected.
The shared owned-layout parser retains its byte, field, name and 64-level depth bounds.
The slot bound counts scalar leaves, not only the outer struct's field count.

Callback and helper admission remain distinct:

- Callback layouts bind to the registered session's flattened state signature.
- Each ordinary aggregate call requires exact agreement with its callee's declared
  nominal layout, nested names, field order, kinds and result ownership.
- User aggregate parameters are flattened to exact scalar parameters. General
  owned/resource inputs do not acquire scalar admission or implicit conversions.
- Every reachable function still passes CPU instruction, lane, dependency, parameter,
  graph, node and depth checks. Recursive calls and hidden effects remain rejected.
- Scoped iteration calls retain their separate flat-i64 carry schema; accepting a
  mixed helper return does not make it a valid mixed loop carry.

The producer validates layouts through
[aggregate admission](../../crates/yir-lower-llvm/src/native_session/aggregates.rs),
then selects the private return plan. Both helper and root paths share the existing
YIR scalar schema without requiring a helper to register as a lifecycle callback.
Duplicate nested parent names are checked at every struct level, even when the
resulting leaf paths would be different.

## Value Transport

The [return plan](../../crates/yir-lower-llvm/src/native_session/aggregate_values.rs)
matches runtime values against exact declared nominal layouts before packing them.
Constructor order may differ from declaration order; fields are matched by name,
then published in the declared order. Each leaf is initialized before return.

Packing reuses the existing owned-payload scalar bit rules: bool zero-extends,
i32 sign-extends, f32 retains its raw low 32 bits and f64 retains all 64 bits.
Calls extract typed independent values and reconstruct nested nominal structs.
Earlier results remain independent of subsequent calls, relay helpers and loops.
Callback roots invoked as ordinary helpers use the same private return signature.

Terminal and guarded early returns share this path. A selected early return packs
its own result; an unselected fallible suffix does not execute. Helper entries still
consume the same per-invocation budget as scalar-returning helpers, with no resets
or refunds at aggregate calls. Arithmetic or budget failure remains a process trap,
not a catchable status, transactional rollback or retry.

There are no returned stack pointers, shared scratch globals or aggregate heap
allocations in this admitted path. LLVM still decides target-specific register,
stack and implicit-result-address lowering. Zero aggregate allocation is not a
promise of zero stack traffic, whole-program memory safety or faster wall time.
Ordinary LLVM emission, resource aggregates, external FFI and task thunks retain
their separate ABI and ownership policies.

## Callback Publication

The wrapper checks argument/output shape and canonical scalar words before entry.
It loads every input, calls the root, extracts every result word, then publishes
output with alignment 1. Full overlap and forward/backward partial overlap therefore
preserve the input snapshot. Status 1/2 failures do not enter a helper or modify
output. Native traps do not publish a partial result.

The reference application host separately normalizes named state fields; it does
not become allocation-free merely because native LLVM returns use slot values.
Source evaluation order remains independent of declared field storage order.

## Guarded Local Values

Matching local bindings and rebindings now admit mixed/nested scalar records through
the [typed value profile](../../tools/nuisc/src/lowering/buffer_loop_outline/control_values/layouts.rs).
It resolves nominal layouts from leaves upward, rejecting cycles, unknown/resource
fields, generics, empty records and duplicate fields. New nested neutral initializer
expansion is bounded to 64 record levels and 4096 nodes; flat source admission is
unchanged. This source normalization bound is separate from native slot admission.

The predicate is evaluated once. Only already-bound captures enter the extracted
helper; calls and checked arithmetic remain behind their branch guard. Total field
projections of ready pure records may be moved into private capture transport.
One-sided rebindings preserve the old value, and discarded selected arithmetic is
not erased. Guard defaults independently require exact typed literal zeros, with
no calls or projections. Literal i64-to-i32 narrowing emits a wrapped i32 constant;
it does not widen the native instruction whitelist to dynamic conversions.

Typed helper discovery reuses the proven flat loop catalog without admitting new
mixed loop bodies. Existing direct typed returns do not acquire extra branch helpers
merely because the new profile can describe them. Resource/effectful arms remain
outside this pure-value extraction; i32/floating arithmetic is not newly generalized.

[Local-value probes](../../tools/nuisc/tests/native_application_bridge/typed_local_values.rs)
cover 1/6/63/64-slot records, matching and one-sided choices, reversed constructors and
YIR storage order, independent snapshots, raw floating bits, in-place publication and zero
aggregate allocations/drops, compared with reference open/event/close execution.
The former 64-leaf mixed capture plus predicate now fits through private boolean
capture transport. A helper that actually consumes a whole record with 64 independent
i64 leaves plus its predicate still needs 65 arguments and is rejected by the unchanged bound.

## Private Capture Transport

Only helpers identified by the outliner, not source names or annotations, acquire
the [capture plan](../../tools/nuisc/src/lowering/direct_calls/capture_params.rs).
User functions, callback roots, FFI and scoped iteration signatures stay unchanged.
Generated helpers used as scoped actions are also excluded, using the same target
discovery as scoped-call lowering; their per-trip induction/carry metadata is not packed.
The same plan lowers both the call arguments and the callee's parameters, using
declaration-order scalar leaves and exact nominal reconstruction. The normal owned
return ABI is unchanged; the native value-return path remains heap-free.

Two or more boolean captures share nonnegative i64 words of at most 63 bits.
Singleton tails keep their boolean type, while i32/i64/f32/f64 leaves keep their
original scalar kinds and bits. The first replaced leaf supplies a unique physical
parameter name. Packing uses canonical bool-to-word conversions and bounded
multiply/add; decoding uses positive constant division/remainder and word-to-bool.
The top bit is 62, so an all-true word is exactly i64::MAX, never signed overflow.
These existing YIR operations do not widen the native instruction whitelist.

Arguments are evaluated before packing, including the one-time predicate; no fallible
or effectful branch work is duplicated or hoisted. Decoding is total and precedes the helper guard.
Helper identities, entry charges, loop preflights and failure publication are unchanged.
Physical parameters still undergo the independent native 64-slot check; packing is
not a general large-argument ABI, a new mixed loop carry, or a claim of faster execution.

[Word probes](../../tools/nuisc/tests/native_application_bridge/typed_capture_words.rs)
cover independent boolean patterns, the highest nonnegative bit and a second word,
reversed storage order, in-place lifecycle publication and zero allocation/drop counters.
The [64-slot CLI workflow](../../tools/nuis/tests/native_session_workflow/capture_words.rs)
uses an independent predicate and a mixed record at exactly 64 physical arguments,
with build/run-artifact, malformed input, tamper, cache and source-free restoration checks.
## Field-Selective Captures

The [projection pass](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_projection.rs)
runs before private boolean packing. It processes generated callees before their
callers, so a chain of private helpers does not keep forwarding an entire record
when its terminal users need only a few fields. Function storage order is irrelevant.
Only an actual reduction in scalar leaves changes the signature.

Each selected path becomes a hygienic private parameter. Repeated paths are shared;
using an entire nested subrecord subsumes its child paths and preserves its exact
nominal type. No missing fields are filled with synthetic zeros or partial records.
Definition, every call site and NIR verification move together before YIR lowering.
Unchanged user signatures still flatten in declaration order; projected parameters
use deterministic path order, and each retained subrecord keeps declaration order.

Only ready variable/field paths can be duplicated or discarded at a call boundary.
Computed record arguments veto projection rather than losing effects or failures.
Scalar arguments, including unused checked arithmetic and predicates, are not removed.
Whole-record aliases, returns, forwarding and same-name rebinding conservatively keep
the original capture. Unsupported callers, scoped targets, cycles and their dependent
helpers are not rewritten. The shared read-only expression walk is exhaustive,
including conversions, FFI operands and shader/kernel children.

[Sparse native probes](../../tools/nuisc/tests/native_application_bridge/typed_sparse_captures.rs)
execute a nested 64-i64 state with only four private selection arguments: one predicate
and three fields. The public helper still has 64 parameters. All lifecycle states
match independent expectations and reference execution with reversed YIR storage,
in-place event/close publication, whole-region sentinels and zero allocation/drop counters.
The [sparse CLI workflow](../../tools/nuis/tests/native_session_workflow/capture_fields.rs)
checks the same fixture through build, cache reuse, input/tamper rejection and
source-free materialization with byte-identical LLVM and unchanged lifecycle states.
Mixed sparse guard probes retain raw float bits, skipped division, selected zero/overflow
traps and unchanged output sentinels. Both guarded arms still cost entries: the fixture
needs exactly six, rejects five/zero, and rejects malformed input before helper entry.

The next boundary is propagating field demand through immutable private aliases,
with explicit scope and snapshot rules. This is not unrestricted scalar replacement,
a wider native argument limit, mixed loop carry admission or a measured speedup.

## Evidence And Limits

[Flat-return probes](../../tools/nuisc/tests/native_application_bridge/aggregate_values.rs)
execute 1/2/7/64-field helpers, early returns, reverse constructors and independent
snapshots, while verifying that ordinary LLVM still allocates owned aggregates.
The earlier four-trip nested helper change reduced 13 allocations to 1; callback
value transport subsequently removed that final allocation.

[Callback probes](../../tools/nuisc/tests/native_application_bridge/callback_values.rs)
cover 1/6/64 nested slots, all lifecycle roots, finite/negative-zero/NaN bit patterns,
unaligned full/partial overlap, disjoint buffers and whole-region sentinels.

[Typed helper probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_values.rs)
cover 1/6/64 mixed/nested slots, nested calls, flattened aggregate inputs, a callback
called as a helper, reversed constructors and reversed YIR node/function/body order.
They compare independent slot expectations with reference open/event/close execution
and real native execution, retaining zero aggregate allocations/drops. Two live
helper results contribute different fields to the final state. The open path fits
six shared entries, while event/close start fresh invocation budgets.

[Guard probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_guards.rs)
execute valid/skipped division, selected zero-divisor and signed-overflow traps,
exact three-entry success, two/zero-entry failure, and malformed-input rejection
before a zero entry budget. Trap observation retains the real trap and checks all
five output sentinels plus allocation/drop counters.

The [typed-helper CLI workflows](../../tools/nuis/tests/native_session_workflow.rs)
add mixed/nested State-returning helpers and guarded local rebindings to the multi-carry fixture.
It checks build/run-artifact, manifest identity, tamper rejection, cache reuse,
registration isolation and source-free standalone restoration, including
byte-identical restored LLVM and unchanged lifecycle states.
The guarded-local fixture also closes with a zero reason, proving that its
unselected zero-divisor state construction does not execute in the packaged binary.

[Admission probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_admission.rs)
reject nominal/kind/order/ownership/arity drift, nested resources, empty layouts,
duplicate parents, oversized/deep layouts, spoofed returned values and hidden effects.
The local-rebinding guard probes additionally check exact five-entry success,
four/zero-entry failure, unused selected results and malformed-input rejection,
with unchanged output sentinels and zero aggregate allocations/drops.
General mixed loop carries, resource state, provider dispatch, native image-host
selection, cross-target execution and performance still need separate evidence.

The 2026-09-23 macOS aarch64 capture checkpoint passed 207 compiler/registry-unit,
45 native-bridge, eight ordinary native and five reference image/window cases.
Five CLI workflows passed; exact-64-slot and sparse captures retain cache reuse,
input/tamper rejection and source-free restoration with identical LLVM and states.
All 28 tensor tests passed; 1400 drift checks were clean. No fresh GPU/Linux,
full-workspace, formal safety or performance result is inferred from this checkpoint.
