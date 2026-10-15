# Native Stateful Branch Joins V1

This bounded CPU receipt follows the
[state-derived branch selection proof](nuis-native-state-derived-branch-selection-v1.md).
Selected arms now continue into common work instead of each returning.
Existing scalar selection, private staging, typed read-only state extraction
and owned State reconstruction suffice. No production lowering, runtime ABI,
owned-allocation mechanism or entering-field/root write authority is changed.

## Selected Local Work Rejoins

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_join.ns)
keeps State{result:bool,total:i64,marker:f64}. Advance reads the previous count.
Previous bool XOR event gate selects either arm of update. Each arm computes
a private local and assigns the explicitly mutable current scalar. The common
finish call runs once, consumes that selected value and adds three. The final
answer call consumes the finished count and previous bool before State is
reconstructed with its untouched marker.

The first fixture incorrectly used immutable let for current. Both execution
tests rejected it before lowering. The fixture was corrected to let mut;
compiler mutability checks were not weakened. Failed attempts are not part of
the final accepted test count.

[The independent arithmetic oracle](../../tools/nuisc/tests/native_application_bridge/state_join_cases.rs)
expects count increments of seven or two, depending on the selected arm.
Using the entering/advanced value or the opposite branch is observably wrong.
Using the selected pre-finish value for the bool flips its expected parity.
These are arithmetic counterfactuals, not mutated-compiler execution.

Separate runs distinguish identical complete event arguments with all proofs
valid from state-aware poisoned arguments whose inactive proof masks depend on
the expected prior state. The latter are not claimed to be identical full input
vectors. Eight seed/gate/flag cases cross four raw markers and both proof modes,
giving 64 sequences per source/inventory ordering. Four original/mirrored and
plain/+0-expression-wrapped sources preserve behavior and lexical scope.

## Execution And Failure Publication

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_join.rs)
open, accept six events and close each sequence under normal/reversed YIR
inventories. Three raw output words retain positive/negative zero, minimum
subnormal and a signed quiet-NaN payload. The event marker differs from the
retained state field.

The unchanged generic sequence probe copies actual prior callback output
into the next callback input, not golden expected state. Status, canaries,
once-only selector/selected-arm/common-tail/answer traces and real answer-callee
finished-count/previous-bool/proof arguments are compared.

Eight healthy native processes produce 4096 healthy accepted callbacks.
Ten selected traps per variant/ordering produce 80 fatal processes and
240 accepted callbacks before failure. Root, common-tail, answer and advance
argument failures are checked for both branch polarities; each selected arm
also has its own argument trap. Advance fails before any print; common-tail
failure occurs after the selected work but before finish; answer-argument
failure occurs after finish but before answer or new State publication.

Two accepted events have changed count and bool from open while preserving
negative-zero marker bits. Reference failures retain the complete last State,
enter Faulted, reject later events and cannot complete successfully after close.
Native fatal runs check exact flushed prefixes and no later publication, not
post-abort memory inspection.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_join_tests.rs)
require the exact State layout, guard-first private helpers, transitive Effect
order in helpers/entries and update if retained, i64-first/bool-last entry calls,
explicit print policy and no owned aggregate allocation/drop. Removing Effect
order or print permission still rejects. No fixed expanded-helper count is used.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_join.rs)
checks sixteen images: four sources, each fresh/cache/two source-free restores.
Each image runs four six-event sessions before and after six selected failures.
CLI uses four marker/proof-mode pairs, not their full cross-product: all-valid
and poisoned modes each cover two finite round-trippable markers. Native and
reference tests cover the complete raw marker/proof-mode cross-product.

Each published CLI marker is parsed back to f64 and checked by bits. This is
finite decimal-input evidence, not raw NaN payload entry through CLI.
Tail/answer/advance argument failures after either branch publish exactly
the three prior states, no failed/next/close state or successful completion.
Fatal CLI stdout may be buffered; the native probe supplies exact prefix proof.

This family contributes 1312 accepted CLI callbacks, 96 fatal events and sixteen
65-event rejections before open under the unchanged 64-event script bound.
Fresh/cache identities include executable, host objects, envelope, lifecycle
and packaging mode. Source, manifest, project cache and initial output are
removed before two restores. Sources remain absent; execution and restoration
leave input artifact/container/executable bytes unchanged.

## Acceptance And Lineage

The serial macOS CPU cohort passes 493 distinct tests: 453 compiler,
nine native/reference/oracle, one CLI, three host and 27 tensor. Five tests are
new: one AOT, three oracle/reference/native and one CLI. The corrected focused
three-test run is not counted again. Native predecessors are the typed state
reconstruction and state-derived routing families; the generic sequence probe
is unchanged. Earlier CLI receipts are historical, not replayed in this cohort.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_stateful_fields return_state_routing state_join --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_join --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3231 drift checks /22464 patterns, changed Rust formatting, UTF-8,
changed file caps/relative links and diff checks pass. Four untouched historical
whole-workspace formatting failures remain. Strict Clippy and full-workspace
tests are not run; no fresh Linux/GPU/Windows or measured-performance acceptance
is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Six complete preceding fields remain unabridged with their hashes. Command
history is enclosed in a collision-checked quoted no-op heredoc. Shell syntax
and harmless current-front replay establish that historical commands remain
literal data rather than executing.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `58953e906c2e0e11214618b38cdb68e61155ace55fe918c34f43a04e19c729e6` |
| `next_step` | `f9366563ef46550e52a97a2a357e64eb7376575d21ed0fe1558fd58884d6108a` |
| `blocker` | `5f87122ebda25b15828af0b75205414597ec844e6326442e6d60e994bb3087bf` |
| `next_action` | `5ecf55bb12cf37eb74428405df0540e15c182e6e6e2c46719eb557163751ced0` |
| `validation_command` | `84fb36f3e17c11d95cf2f0b533f08bc3c2c84a472e7e811e0742c922af6f2ac5` |
| `expected_artifact` | `b87203481bb43f050f7f31d2a8dcf464d2fc88e45bf0d3d6eec45f50ac75d26e` |

The rebuilt ordinary CLI exposes all six source-equal fields and matching
complete predecessor hashes. Coverage, manifest/milestone coverage, hierarchy,
task-card lineage, selected ready task card and drift remain clean.

## Next Boundary

The following [stateful skipped-arm retention proof](nuis-native-stateful-skipped-arm-retention-v1.md)
advances that boundary; the paragraph below retains this receipt's original task.

Both selected arms currently write the same owned local. Next exercise a
one-sided state-driven continuing selection: skipping work must retain the
previous local value before common-tail computation rather than substitute
an inactive guard's neutral zero. Keep exact types, read-only entering fields,
explicit mutability, seals, lexical scopes, budgets, selected-only effects,
failure-before-publication and source-free identities. Generalized field/root
writes, resources, arbitrary deeper exits, global immutability, stable ABI,
formal safety, measured performance, generalized heterogeneous closure and
completed self-hosting remain separate.
