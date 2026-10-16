# Native Stateful Bounded Per-Event Loops V1

This bounded CPU receipt follows the
[sequential optional update proof](nuis-native-stateful-sequential-updates-v1.md).
The starting Git baseline remains beta-0.17.1, commit 99ebfcb2.
A runtime-counted loop now carries the actual current scalar through selected
pure work, then returns it to the common effectful tail and owned State
reconstruction. No production lowering change, new ABI, allocation mechanism,
entering-State write authority or relaxed pure-helper admission is added.

## Runtime Loop And Current Values

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop.ns)
advances the previous State.total and calls a noinline walk with runtime trips.
Its explicit mutable current and induction index survive as a YIR loop and LLVM
loop/backedge with stack-carried loads/stores before optimization, rather than a
source-unrolled three-call fixture. The source uses a trailing
index update. Each iteration's pure update helper tests event gate XOR previous
bool XOR current parity. Selected refine work computes twice the current value
plus a checked proof and the zero-based index; either path advances by one.
Zero trips retain the advanced entering value. A common finish adds three,
answer consumes the finished count and previous bool, and reconstructed State
retains the prior floating marker.

[The independent arithmetic oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_cases.rs)
crosses 48 signed seed/gate/flag/trip cases, four raw markers and two proof modes:
384 three-event sequences per source/inventory order. Negative and zero limits
both execute zero iterations; one and three trips exercise the inclusive limit.
All-valid runs have identical complete event arguments and history-dependent
results. Separate state-aware poisoned runs select a safe hole index. The
selected argument divides one by index minus hole, so the inactive hole must
not evaluate its zero divisor. Holes at indices zero, one and two are covered.
Changing a hole also changes other valid proofs, so poisoned runs have their own
oracle and are not claimed equivalent to all-valid masks.

Later real callee inputs must equal the preceding iteration's current output.
Freezing predicates or work operands at the loop entry has arithmetic witnesses
with different routes or values. These are independent oracle counterfactuals,
not mutated-compiler execution. Four original/mirrored plain/+0-wrapped sources
run under normal/reversed YIR inventories.

## Execution, Limits And Failure

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_loop.rs)
check actual state handoff over open, three events and close. The generic probe
copies actual prior callback output using volatile loads/stores, never expected
golden state. It observes the real choose and refine calls, including current
count, checked proof and zero-based index, and the real answer arguments.
Instrumentation does not replace computation. Status, canaries, raw output
words and once-only traces are checked. Positive/negative zero, minimum
subnormal and a signed quiet-NaN payload retain exact State words.

Iteration condition/work helpers are pure source computations: observer markers
10/20 are native test instrumentation, not admitted source I/O. Source prints
5/35/40 remain outside the loop. Reference traces contain only those real
source effects. Transitive iteration prints remain rejected in both predicate
and work controls. This is not effectful-iteration acceptance.

Eight main healthy native processes contribute 15360 healthy accepted callbacks.
The 13 selected arithmetic failures per source/inventory order include work
failures at indices zero, one and two, condition, advance, common finish and
answer arguments. They contribute 104 fatal processes and 312 earlier accepted
callbacks. The prefix is open State(12,true), event State(16,false), then event
State(21,false), all retaining the opening negative-zero marker. Native fatal
runs check exact flushed prefixes and no later publication, not post-abort memory inspection.
Reference sessions retain the complete last published State, enter Faulted,
reject another event and cannot complete successfully after close.

The explicit main producer policy is three loop-work reservations and 100
helper entries per callback. Repeated three-trip callbacks prove counters reset
per lifecycle entry; they do not share a session-wide counter. Separate native
runs use limits zero, two and three. Zero trips succeed even with a zero root
divisor and zero loop budget. Counts above the chosen budget, 65537 and i64::MAX
fail after advance but before the first iteration condition/work. These add six
healthy processes /30 accepted callbacks and 24 fatal processes /24 prior
accepted callbacks. The existing independent finite non-wrapping induction
proof and ten-second child deadline remain in force.
Budget-negative inputs use valid root and work arguments: bypassing reservation
would reach the observed first iteration, rather than an indistinguishable
ordinary argument trap.

[AOT tests](../../tools/nuisc/src/aot_application_effect_call_state_loop_tests.rs)
retain runtime loop/backedge/guard structure, exact State layout, no owned aggregate
allocation/drop, explicit print permission, Effect-order rejection and a distinct
budget-bearing packaging identity. The old probe and CLI fixture adapters still
default to zero loop work and 100 helper entries. The new generic limit entry
points do not widen production defaults or whitelist these test helper names.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop.rs)
checks sixteen images: four sources, each fresh/cache/two source-free restores.
Each image runs four three-event sessions before and after seven failures:
all five selected hole failures and two common-tail failures. It checks exactly
the three earlier published states, no failed/next/close state and no successful
completion. Buffered fatal CLI stdout is not effect-prefix proof; the flushed
native probe supplies that evidence.

Four finite marker/trip/proof profiles cover zero, one, three and negative trips.
Printed markers are parsed back and compared by bits. This is finite decimal
CLI input evidence, not full raw-marker cross-product or CLI NaN payload input.
The family contributes 640 healthy accepted CLI callbacks and 336 prior accepted
callbacks, 112 fatal events and sixteen oversized 65-event rejections before
open under the unchanged 64-event script bound.

The packaging token explicitly retains loop limit three /entry limit 100.
Cache checks compare executable, host objects, envelope, lifecycle and mode.
Source, manifest, project cache and original output are removed before restoring
twice. Sources stay absent, and restores/runs leave artifact/container/executable
bytes unchanged.

## Acceptance And Lineage

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join state_skip state_sequence state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_typed_exit_prefix state_join state_skip state_sequence state_loop:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The focused macOS CPU cohort passes 503 distinct tests: 457 compiler, fifteen
native/reference/oracle, one CLI, three host and 27 tensor. Seven tests are new:
two AOT, four oracle/reference/native/budget and one CLI. Preliminary reruns
are not counted again. Native predecessor replay covers sequential updates,
joins, skipped-arm retention and the older typed exit-prefix paired-row adapter.
Older CLI receipts remain historical, not replayed here.
The initial compiler selection passed 456/457; the sole new structural assertion
incorrectly required SSA phis from a stack-based pre-optimization emitter.
After checking the actual loop/backedge implementation, that assertion was
corrected and both new AOT tests passed on focused replay. No production
compiler behavior was changed to satisfy it.

All 3273 drift checks /22688 patterns, changed Rust formatting, UTF-8,
changed file caps/relative links and diff checks pass. The same four untouched
whole-workspace formatting failures remain. Strict Clippy/full-workspace tests
and fresh Linux/GPU/Windows/performance acceptance are not claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor values remain unabridged, with their hashes below.
Command history is a collision-checked quoted no-op heredoc; shell syntax and
harmless current-front replay prove history remains nonexecuting data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `08adf4d8a9d1f819165217b24157b357d719bc17c0cafd50471bb9c6514dbb6b` |
| `next_step` | `b332b69e80f1e4ad99848004d32e06a6a4fb5523f5a75a239dfd13ecdbcf9b46` |
| `blocker` | `2370d5af26292fb61237070e91029367f2679fb27ed6a5cd64be6c9c6235912f` |
| `next_action` | `9418197defad497ed768e0060ad137fa4e313c5404ccc99bde5917638f9332a1` |
| `validation_command` | `adf3dec81a3734ed8a9dcd1c3c4f668f56058140b2b0e625a06bf17fa712ccfa` |
| `expected_artifact` | `9ad19c8d9779a87274fee0a7cb1b3ac0059af9eaae39df0cdb89ba49eab500c5` |

The rebuilt ordinary CLI exposes six source-equal values and complete predecessor
hashes. Coverage, manifest/milestone coverage, hierarchy, task-card lineage,
the selected ready task card and drift remain clean.

## Next Boundary

The subsequent [literal-print iteration-effect proof](nuis-native-stateful-loop-effects-v1.md)
advances selected source-call effects through an isolated profile and explicit
backend grants. The preceding receipt's rejected effectful experiment and
counts remain historical, not retroactively expanded.

Make bounded iteration effects an explicit admission/ordering capability,
starting with selected literal-print work, rather than falsely classifying it
as pure. Keep shared helper/loop budgets, zero-trip non-evaluation, actual
per-iteration arguments, fault-before-publication and source-free identity.
The failed initial effectful experiment is not accepted support. Arbitrary
effects/resources, deeper exits, generalized entering-State writes, global
immutability, stable ABI, formal safety, measured performance, generalized GPU
closure and completed self-hosting remain separate.
