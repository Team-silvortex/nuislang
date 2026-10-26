# Native Binary Float Field Epoch Print V1

This CPU worktree follow-up continues the
[sign-chain proof](nuis-native-nested-unary-float-field-epoch-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. It changes no release version, native ABI, source
intrinsic or runtime wrapper. It is not a new release or heterogeneous completion.

## Typed Binary Plan

The [shared plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
admits same-kind f32/f64 addition, subtraction, multiplication and division at
exact float arguments of completed pure-i64 print helpers. It uses a bounded
work stack, not an enumeration of expression shapes. Both operands are checked
left before right; the original tree is rendered without reassociation,
duplication or synthetic ready variables. Canonical sign operations reuse the
[shared sign recognizer](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_unary_float.rs).

Every leaf must be an original certified field or independently ready scalar
literal/variable/Var-rooted field of the exact expected kind. The
[epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
retains entry, ownership, original nominal layout/path and the complete pure-i64
parameter-index query. The ready-leaf callback uses ordinary read validation,
not the epoch extension. Neither arithmetic nor a print makes a record ambiently
ready. Both the plan and aggregate print roots retain bounded depth/work checks.

Mixed kinds, reference roots, stale layouts, casts, remainder, hidden calls,
whole-record leaves, ordinary initializers/conditions and unavailable ancestors
do not gain new permission. Ready expressions keep their existing independent
routes. The shared typed-value policy now recognizes float division, not float
remainder; this is distinct from pre-update read authority.

## Checked Division

The CPU reference contract rejects either zero sign as a divisor. Previously,
LLVM float division emitted an unchecked `fdiv`, unlike reference execution.
The [shared LLVM guard](../../crates/yir-lower-llvm/src/checked_float_division.rs)
now uses ordered zero equality and traps before scalar or loop float division.
NaN divisors are not zero; permitted nonzero operations retain IEEE arithmetic.
No infinity-returning zero-divide policy is introduced by this read grant.

Strict native generic division accepts only two exact matching f32/f64 operands
or the existing i64 pair; remainder remains i64. Typed division opcodes outside
the native bridge whitelist remain outside it. Normal and explicitly typed LLVM
scalar emitters share the same guard. Register-derived labels remain unique
without changing loop block counters. Two
[LLVM controls](../../crates/yir-lower-llvm/src/tests/float_division_tests.rs)
cover scalar generic/typed paths, zero signs, NaN, guard placement and loop labels.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/binary_float_print_epoch_tests.rs)
cover all four operators, both operand orders, exact ready reads and unary
composition, mixed-kind/cast/call/ancestor vetoes, source-order traversal and
cumulative depth/work limits. Raw NIR and source execution retain both induction
orders, selected/skipped work, IEEE classes and reached positive/negative zero
failures. Legacy source rejection cases now exercise hidden float-return calls,
not arithmetic that this later proof legitimately admits.

## Execution And Delivery

The [Nuis program](../../tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print.ns)
combines all four operations over parent/child fields and an invariant float
parameter, with distinct noncommutative expressions and source negation.
Carry updates remain separate sign operations; final State publication retains
the independent checked tail helper.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print_cases.rs)
performs each f32 operation at f32 precision and each f64 operation at f64
precision. Raw probes inspect the actual ready input, computed callee arguments
and final State. Only arithmetic NaNs normalize in the test observer: finite,
infinite and signed-zero result words stay exact. There is no arithmetic NaN
payload promise. Input/carry NaN payloads remain checked as raw words, independently
of argument arithmetic. Production computation is never normalized by a probe.

[Native/reference execution](../../tools/nuisc/tests/native_application_bridge/binary_float_field_epoch_print.rs)
uses 21 healthy sequences per width and thirteen failure rows: nine prior integer
or tail failures plus four reached float-zero cases, with a corresponding skipped
child poison. Normal/reversed YIR, exact reached prefixes, canaries, last accepted
State and faulted-session behavior remain checked. Extra sequences exercise
fractional coefficients, maximal finite values, normal/subnormal underflow and
finite overflow. Scalar reference-call counts
are structural. Owned-struct calls do not emit the same scalar-call event; their
native input-entry probe is pinned once in the complete word stream instead.

Success uses 12 loop work, 256 helper entries and at most 32 functions. Separate
128-entry NaN controls retain atomic rejection without widening producer budgets.
The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/binary_float_field_epoch_print.rs)
runs seven healthy sequences per width before and after all thirteen failures on
fresh/cache and two independent source-free images. Binary/container, host-object,
envelope, lifecycle and packaging identities remain pinned. Decimal arguments
are not used as arbitrary NaN-payload evidence.

## Validation

The serial, locked validation chain passed 618 distinct tests, including twelve
new controls: six compiler, two LLVM, three native/reference and one CLI delivery
test. This comprises 255 targeted compiler, 105 targeted native/reference, one
CLI, all 178 LLVM library tests, three packaging, 17 buffer-loop, three host-failure,
26 maintenance-script, one absolute-path policy, two file-limit and 27 tensor
tests. Repeated focused runs are counted only once. The LLVM suite includes the
normally ignored host-clang record round trip at both O0 and O2; this is not a
claim that every workspace suite ran.

Both widths passed fresh/cache and two independent source-free deliveries with
seven healthy sequences before and after all thirteen failures. Formatting,
whitespace, delivery-test compilation, the ordinary CLI build, locked offline
metadata for 29 packages, 4,397 local documentation links and 5,840 UTF-8 files
also passed. The iteration registration base was extracted without changing
existing check IDs or order, keeping the source files within their line limits.

The rebuilt CLI reports all 3,880 drift checks clean, together with coverage,
manifest, milestone, hierarchy and task-card lineage statuses. All six selected
fields exactly match source and retain their complete predecessor values and
SHA-256 digests. Historical validation commands remain quoted here-document
data consumed by a no-op, checked without executing prior chains. These results
cover bounded CPU execution only, not GPU/Linux parity, stable ABI, performance
targets or self-hosting.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Pure
float-return helper calls as operands, such as `observe(scale(record.value) +
ready, divisor)`, still need a completed-call/argument-read proof; scalar output
type alone cannot authorize them. General effects/resources/I/O/async capture,
stable ABI, GPU/Linux parity and self-hosting remain separate gates.
The later [pure float-call proof](nuis-native-float-call-field-epoch-print-v1.md)
advances the call boundary without changing this checkpoint's historical totals.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- binary_float_print_epoch control_values::typed_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- binary_float_field_epoch_print division_admission:: division_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- binary_float_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `df7f58b9c9986231d178d70f77d37220b8d7b8b5ed52dca7c569b15a77479a4a` |
| `next_step` | `012cf07fa0994d0b3bee1031af3928bfdf27552403758af821b92556103f9bd4` |
| `blocker` | `488d0a4a07b84501061ff7cbc0f3d6754bb10aea231be01f4d58673b70a18cb7` |
| `next_action` | `09b7fe8aa43ac4db948162eb65e8c408390fbb690025647ec3293a1e64ece318` |
| `validation_command` | `a8bf943f82cc03a5dc7983e06f90107e02abf7484b7ad660a555ca239ef30fcb` |
| `expected_artifact` | `aa72b68a3f0f9d93d678c52ddad10c0be0cca7011668acbba2fc955cd936043c` |
