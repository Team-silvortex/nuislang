# Native State-Derived Branch Selection V1

This bounded CPU receipt follows the
[typed state reconstruction checkpoint](nuis-native-stateful-field-reconstruction-v1.md).
It composes existing read-only field extraction, typed predicate lowering,
ordinary bool constants and complete return trees. No production lowering,
runtime ABI, owned-allocation mechanism or entering-field/root write authority
is changed for this family.

## State Selects Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_state_routing.ns)
retains State{result:bool,total:i64,marker:f64}. Advance computes the new count.
The root gate is event gate XOR previous State.result; the child gate is event
first XOR the new count's even parity. Thus previous bool and newly computed
i64 control which calls run, not merely which bool is returned.
Marker is retained unchanged in the newly reconstructed owned State.

[The arithmetic oracle](../../tools/nuisc/tests/native_application_bridge/return_state_routing_cases.rs)
resolves expected gates from prior expected state, independently of compiler
execution. It reuses the preceding leaf-result/trace oracle, not YIR output.
Wrong prior bool reverses the root gate; a one-event stale count reverses the
child gate. These are arithmetic counterfactuals, not compiler mutation runs.
Plain/wrapped original/mirrored sources retain the same behavior.

Two different claims have deliberately separate tests:

- With every proof set to one, identical complete event arguments are repeated
  six times. Histories select different root and child work; actual traces
  and returned state, not only the oracle's routing labels, must agree.
- With state-aware poisoned arguments, masks are recomputed from the independent
  prior-state oracle. Inactive work receives zero divisors. These full argument
  vectors may change with state, so they are not claimed to be identical inputs.

All four nested routes occur across sixteen input cases. Native/reference
coverage crosses four raw markers and both proof modes for 128 sequences per
source/inventory ordering. The event marker differs from the retained field.
Each sequence opens, accepts six events and closes.

## Reference And Native Execution

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/return_state_routing.rs)
compare every published State as three raw words. Positive/negative zero,
minimum subnormal and a signed quiet-NaN payload survive untouched.
Normal/reversed YIR inventories and four source variants are executed.

The existing generic sequence probe copies the actual prior callback output
into the next callback input with volatile loads/stores, never golden expected
state. It checks status, canaries, all three output words and flushed traces.
Real selected answer-callee owner/previous-bool/proof arguments are observed
without replacing the computation.

Eight healthy native processes account for 8192 healthy accepted callbacks.
Nine selected argument traps per variant/ordering produce 72 fatal processes
and 216 accepted callbacks before those failures. Advance can fail before any
print; later traps occur after local work but before new State publication.
Two accepted events leave a bool/count different from open and retain the
negative-zero marker. Exact flushed prefixes and no later publication are
checked. This is observable failure evidence, not post-abort memory inspection.

Reference failures retain that complete last State, enter Faulted, refuse
subsequent events and cannot report successful completion after close.
[AOT checks](../../tools/nuisc/src/aot_application_effect_call_return_state_routing_tests.rs)
retain the exact layout, guard-first private helpers, semantic source-entry
i64-before-bool/transitive Effect order, explicit print policy and no owned
aggregate allocation/drop. Removing Effect order or print permission rejects.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_state_routing.rs)
checks sixteen images: four source variants, each fresh/cache/two source-free
restores. Each image runs eight six-event sessions before and after five
selected traps. Those sessions separately exercise identical inputs and
state-aware poison masks across four finite decimal-round-trippable markers.
Each printed marker is parsed back and compared by bits; this is not raw NaN
payload input coverage through the CLI.

The family contributes 2288 accepted CLI callbacks, 80 fatal events and sixteen
65-event script rejections before open under the unchanged 64-event bound.
Fatal CLI runs publish exactly the three previously accepted states and no
failed/next/close state or successful completion. Buffered fatal CLI stdout is
not prefix evidence; the flushed native probe supplies that proof.

Cache preserves executable, host-object, envelope, lifecycle and packaging
identities. Each source and manifest, project cache and initial output are
removed before restoring twice. Restores and execution leave container/input
artifact and executable bytes unchanged; absent sources remain absent.

## Acceptance And Lineage

The serial macOS CPU cohort passes 505 distinct tests: 452 compiler,
21 native/reference/oracle, two CLI, three host and 27 tensor. Five tests are
new: one AOT, three oracle/reference/native and one CLI. The earlier focused
three-test run is not counted again. Native predecessors replay through the
same probe; only the typed state reconstruction CLI predecessor is freshly
replayed here. Other CLI receipts remain historical.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_stateful_fields return_state_routing --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3218 drift checks /22398 patterns, changed Rust formatting, UTF-8,
changed file caps/relative links and diff checks pass. Four untouched historical
whole-workspace formatting failures remain; strict Clippy and full-workspace
tests are not run. No fresh Linux/GPU/Windows or measured-performance acceptance
is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Six complete preceding fields are retained unabridged with their hashes.
Command history is a collision-checked quoted no-op heredoc. Shell syntax and
a harmless current-front replay prove historical commands remain literal data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `2b1a543b3b7d1fa1ca3732a91bd5c24f14cc5bb2ab81e0347a5024ea2e862365` |
| `next_step` | `91a0873e87f397cdfdc7ff51ebffe2b5833c9c5144c8398aa208f50d22350f3a` |
| `blocker` | `90179dfb081131c26196b2115e033736691cf5afcb9b33f12f3cc48d48bab7b7` |
| `next_action` | `ad6789b201a9d9dd7e72158e962e3b756892fcbfcec942f6f9601e2ff3e61ad9` |
| `validation_command` | `6ebb57e1b7af12100d8142ba668a0316782fea863fb691dfaffa94ff8239a044` |
| `expected_artifact` | `debc717788a766d866171d51cebbc6675d7cc6ca739140fb2658d4785a2e1eba` |

The rebuilt ordinary CLI exposes all six source-equal fields and complete
predecessor hashes. Coverage, manifest/milestone coverage, hierarchy, drift,
task-card lineage and selected ready task card remain clean.

## Next Boundary

The following [stateful continuing branch join proof](nuis-native-stateful-branch-joins-v1.md)
advances that boundary; the paragraph below records this receipt's original task.

The selected helper still has a complete-return tree. Next exercise a small
state-derived continuing selection: selected local work must rejoin a common
tail that consumes its current value and reconstructs State. Retain exact
types, read-only entering-state authority, seals, lexical scope, budgets,
selected-only effects and failure-before-publication/source-free checks.
Generalized in-place field/root writes, resource transport, arbitrary deeper
exits, global immutability, stable ABI, formal safety, measured performance,
general heterogeneous closure and completed self-hosting remain separate.
