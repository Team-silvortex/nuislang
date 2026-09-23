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
- Source aggregate parameters are flattened to exact scalar parameters. General
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
helper; calls, projections and checked arithmetic remain behind their branch guard.
One-sided rebindings preserve the old value, and discarded selected arithmetic is
not erased. Guard defaults independently require exact typed literal zeros, with
no calls or projections. Literal i64-to-i32 narrowing emits a wrapped i32 constant;
it does not widen the native instruction whitelist to dynamic conversions.

Typed helper discovery reuses the proven flat loop catalog without admitting new
mixed loop bodies. Existing direct typed returns do not acquire extra branch helpers
merely because the new profile can describe them. Resource/effectful arms remain
outside this pure-value extraction; i32/floating arithmetic is not newly generalized.

[Local-value probes](../../tools/nuisc/tests/native_application_bridge/typed_local_values.rs)
cover 1/6/63-slot records, matching and one-sided choices, reversed constructors and
YIR storage order, independent snapshots, raw floating bits, in-place publication and zero
aggregate allocations/drops, compared with reference open/event/close execution.
The full 64-leaf capture plus a separate predicate needs 65 helper arguments and
is explicitly rejected by the unchanged 64-argument native bound. Reducing capture
pressure is the next boundary, not a reason to silently raise that bound.

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
