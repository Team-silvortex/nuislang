# Native Stateful Sequential Optional Updates V1

This bounded CPU receipt follows the
[skipped-arm retention proof](nuis-native-stateful-skipped-arm-retention-v1.md).
The starting Git baseline is beta-0.17.1, commit 99ebfcb2.
Two optional updates now hand the current-or-retained scalar to a later
predicate and actual callee. Existing lowering suffices; no production
compiler special case, runtime ABI, allocation mechanism or entering-State
write authority is added.

## Two Current-Value Boundaries

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/state_sequence.ns)
advances the previous State.total, then uses previous bool XOR event gate to
select first private work. The first selected value is advanced count plus
three, or the retained advanced count when skipped. The second predicate
consumes that current value's parity. Selected refine work doubles its current
argument and subtracts one through a private intermediate. Skipping it retains
the first result. A common finish adds three before answer and owned State
reconstruction, with unchanged floating marker bits.

[The independent arithmetic oracle](../../tools/nuisc/tests/native_application_bridge/state_sequence_cases.rs)
crosses 24 signed seed and gate/second/flag combinations, four
raw markers and two proof modes: 192 six-event sequences per source/inventory
ordering. All four execute/skip combinations occur. A first update changes
parity, so a stale second predicate chooses the opposite route. A stale
second-work argument changes the final count. Second-stage skipped locals
include negative, nonzero and legitimate zero values. Pre-finish bool parity
also differs from the final answer. These are arithmetic counterfactuals,
not mutated-compiler execution.

All-valid proof runs establish identical complete event arguments with
history-dependent routes. Separate state-aware poisoned arguments supply zero
divisors only to inactive work in each optional stage; those full masks may
change and are not claimed to be identical inputs. Four original/mirrored
plain/+0-wrapped sources retain the same behavior and original selection
type/scope/seal/guard budgets.

## Actual Callees And Publication

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_sequence.rs)
open, accept six events and close under normal/reversed YIR inventories.
Positive/negative zero, minimum subnormal and a signed quiet-NaN payload
survive as exact State words; event marker inputs differ from retained state.

The generic word-sequence probe now accepts a list of observed callees.
It observes actual refine count/proof and answer finished-count/previous-bool/
proof arguments without replacing either computation. Real prior callback
output is still copied into the next input, never expected golden state.
Status, canaries, raw output words and exact once-only effect traces are checked.

Eight healthy native processes contribute 12288 healthy accepted callbacks.
Seventeen traps per source/inventory order produce 136 fatal processes and
408 accepted callbacks before failure. First/second condition and selected-work
arguments, common-tail and answer arguments, and advance are independently
targeted. Tail/answer traps cover all four execute/skip routes. Root/advance
traps occur before selection; second-condition traps occur after first work
or retention; second-work traps occur after its condition but before entering
the callee. Common-tail failures may include a completed refine observation.

Two earlier accepted events leave bool/count different from open while retaining
negative-zero bits. Reference failures retain that complete State, enter Faulted,
reject later events and cannot complete successfully after close. Native fatal
runs establish exact flushed prefixes and no later publication, not post-abort
memory inspection.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_sequence_tests.rs)
retain the exact layout, guard-first private helpers, transitive Effect order,
semantic i64-first/bool-last entry calls, explicit print policy and no owned
aggregate allocation/drop. Missing Effect order or print permission rejects.

The old single-observer entry point still delegates to the generic observer
list. Fresh native predecessor replay covers two-sided joins, skipped-arm
retention and the older typed exit-prefix paired-row adapter. Production
lowering does not know these test callee names or oracle formulas.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_sequence.rs)
checks sixteen images: four sources, each fresh/cache/two source-free restores.
Every image runs four six-event sessions before and after eight expected
failures. Two second-input profiles expose all four execute/skip combinations.
Four finite marker/proof-mode pairs are not a full cross-product; native/
reference tests cover the complete raw marker/proof-mode cross-product.

Every printed CLI marker is parsed back and compared by bits. This is finite
decimal-input evidence, not raw NaN payload entry through CLI. Second-condition,
second-work and common-tail failures publish exactly the three earlier states,
no failed/next/close state or successful completion. Buffered fatal CLI stdout
is not prefix proof; the flushed native probe supplies it.

The family contributes 1408 accepted CLI callbacks, 128 fatal events and sixteen
65-event rejections before open under the unchanged 64-event script bound.
Cache identity checks include executable, host objects, envelope, lifecycle
and packaging mode. Source, manifest, project cache and original output are
removed before restoring twice. Sources remain absent; restoration/execution
leave input artifact/container/executable bytes unchanged.

## Acceptance And Lineage

The serial macOS CPU cohort passes 497 distinct tests: 455 compiler,
eleven native/reference/oracle, one CLI, three host and 27 tensor. Five are new:
one AOT, three oracle/reference/native and one CLI. The focused new-family run
is not counted again. Older CLI receipts remain historical, not replayed here.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join state_skip state_sequence --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_typed_exit_prefix state_join state_skip state_sequence --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_sequence --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3258 drift checks /22605 patterns, changed Rust formatting, UTF-8,
changed file caps/relative links and diff checks pass. Four untouched historical
whole-workspace formatting failures remain. Strict Clippy and full-workspace
tests are not run. No fresh Linux/GPU/Windows or measured-performance acceptance
is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Six complete preceding fields are preserved unabridged with their hashes.
Command history is a collision-checked quoted no-op heredoc. Shell syntax and
harmless current-front replay establish that historical commands remain data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `98644b08f6424919c798ecd8c86fd3de882a2535cdd7d85113a9988c1b98db2b` |
| `next_step` | `f263b7befe24b91e6af797eb48d02a3adace78e7b49bca98d438327bf04ab8b3` |
| `blocker` | `6a7bef941bd6ca5dc946130c126ffe7b2ab713eda4ba435947cbe91140384302` |
| `next_action` | `a43e1f6e8670e2506cbd5eb6fc496f2ea94d80375cd390a04f159b0709b630bf` |
| `validation_command` | `eac12499c4cab82ce27cada9711cc06fc6b38d22bd6f41d5d05ef9732ace9ed0` |
| `expected_artifact` | `34237292134e14fd1824ab6a0bcedb6e3bf37c690d9b378a4b354087f9006a3b` |

The rebuilt ordinary CLI exposes six source-equal fields and matching complete
predecessor hashes. Coverage, manifest/milestone coverage, hierarchy, task-card
lineage, selected ready task card and drift remain clean.

## Next Boundary

The subsequent [bounded per-event loop proof](nuis-native-stateful-bounded-loops-v1.md)
advances pure-work iteration carries and explicit work-budget/source-free evidence.
It does not admit iteration I/O. The preceding straight-line receipt and counts
remain historical, not retroactively expanded.

This source has two straight-line optional updates, not a loop. Next exercise
a bounded per-event loop that carries the current owned scalar across zero,
one and several iterations, with optional selected work and final State
reconstruction. Preserve exact types, explicit mutability, read-only entering
fields, seals/scopes, original proof budgets, selected-only effects, failure
before publication and source-free identities. Generalized entering-State
writes, resources, arbitrary deeper exits, global immutability, stable ABI,
formal safety, performance, generalized heterogeneous closure and completed
self-hosting remain separate.
