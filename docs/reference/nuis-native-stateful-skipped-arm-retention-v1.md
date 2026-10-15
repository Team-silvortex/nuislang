# Native Stateful Skipped-Arm Retention V1

This bounded CPU receipt follows the
[stateful branch join proof](nuis-native-stateful-branch-joins-v1.md).
A one-sided selected region now rejoins common computation while preserving
the actual pre-branch local when work is skipped. Existing retained-leaf,
private-region, typed predicate and read-only field lowering suffice.
No production lowering, runtime ABI, allocation mechanism or entering-State
write authority is changed.

## Retained Means The Current Local

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_skip.ns)
advances previous State.total before initializing an explicitly mutable current.
Previous bool XOR event gate selects private work. The source omits else;
its mirrored variant uses an empty then and an effective else. Skipped work
retains that advanced local, not a neutral guard zero or the stale State.total.
A once-only common finish adds three, then answer consumes the finished count
and previous bool before owned State is reconstructed with unchanged marker bits.

[The arithmetic oracle](../../tools/nuisc/tests/native_application_bridge/state_skip_cases.rs)
expects count increments of seven when selected and four when skipped.
Twenty seed/gate/flag cases include negative, zero and positive starting values.
Actual retained locals include negative, zero and nonzero values. Nonzero
retention exposes neutral-zero substitution; legitimate zero retention remains
accepted. Substituting stale State.total changes the skipped count and bool.
Using the pre-finish value flips bool parity. These are arithmetic
counterfactuals, not compiler mutation runs.

For seed eight with false gate/flag, repeated identical events produce counts
12, 19, 26, 30 and both selected/skipped histories. Separate all-proofs-valid
runs establish identical complete event arguments.
Separate state-aware poisoned arguments supply a zero work divisor only when
private work should be skipped;
those masks may change and are not claimed to be identical complete inputs.

Four plain/+0-wrapped original/mirrored sources retain behavior. Both source
polarities use the same existing bounded retained-leaf proof. Entering State
fields remain read-only; branch-local private names do not escape, and no
source budget, scalar type, seal or guard authority is relaxed.

## Reference And Native Execution

[Execution tests](../../tools/nuisc/tests/native_application_bridge/state_skip.rs)
cross twenty cases, four raw markers and two proof modes for 160 sequences per
source/inventory ordering. Each opens, accepts six events and closes.
Normal/reversed YIR inventories are checked, with positive/negative zero,
minimum subnormal and a signed quiet-NaN payload retained as exact raw words.
Event marker inputs differ from the retained State field.

The unchanged generic sequence probe carries actual prior callback output
into the next callback, never golden expected state. It compares status,
canaries, raw State words, once-only selector/private-work/common-tail/answer
traces and actual answer-callee finished-count/previous-bool/proof arguments.

Eight healthy native processes contribute 10240 healthy accepted callbacks.
Nine trap cases per source/ordering produce 72 fatal processes and
216 accepted callbacks before those failures. Root and advance failures are
checked under both intended branch polarities before selection. Common-tail
and answer-argument failures are checked after selected and skipped work.
The private-work trap exists only on the selected route.

After two earlier events, bool and count differ from open while negative-zero
marker bits remain. Reference failures preserve the complete last State,
enter Faulted, refuse subsequent events and cannot complete successfully after
close. Fatal native runs establish exact flushed prefixes and no later
publication, not post-abort memory inspection.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_skip_tests.rs)
retain exact layout, guard-first helpers, transitive Effect order, semantic
i64-first/bool-last entry calls, explicit print policy and no owned aggregate
allocation/drop. Removing Effect order or print permission rejects. Expanded
helper counts are not fixed by the test.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_skip.rs)
checks sixteen images: four source variants, each fresh/cache/two source-free
restores. Each image runs four six-event sessions before and after six expected
failures. Four finite marker/proof-mode pairs cover all-valid and poisoned
modes with two markers each, not their full cross-product; native/reference
tests cover the complete raw marker/proof-mode cross-product.

Each displayed CLI marker is parsed back and compared by bits. This is finite
decimal-input coverage, not raw NaN payload entry through CLI. Tail, answer
and advance argument failures in selected/skipped histories publish exactly
the three earlier states, no failed/next/close state or successful completion.
Buffered fatal CLI stdout is not prefix proof; the native probe supplies it.

The family contributes 1312 accepted CLI callbacks, 96 fatal events and sixteen
65-event rejections before open under the unchanged 64-event bound.
Fresh/cache identities include executable, host objects, envelope, lifecycle
and packaging mode. Source, manifest, project cache and original output are
removed before restoring twice. Sources remain absent; restore/execution leave
input artifact/container/executable bytes unchanged.

## Acceptance And Lineage

The serial macOS CPU cohort passes 494 distinct tests: 454 compiler,
nine native/reference/oracle, one CLI, three host and 27 tensor. Five are new:
one AOT, three oracle/reference/native and one CLI. The earlier focused run is
not counted again. Native predecessors are state-derived routing and two-sided
stateful branch joins. Earlier CLI receipts remain historical rather than
being replayed in this cohort.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join state_skip --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_state_routing state_join state_skip --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_skip --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3244 drift checks /22531 patterns, changed Rust formatting, UTF-8,
changed file caps/relative links and diff checks pass. Four untouched historical
whole-workspace formatting failures remain. Strict Clippy and full-workspace
tests are not run. No fresh Linux/GPU/Windows or measured-performance acceptance
is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Six complete preceding fields are preserved unabridged with their hashes.
Command history is a collision-checked quoted no-op heredoc. Shell syntax and
harmless current-front replay prove historical commands remain literal data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `c22327ac3172611e6db3af8dc853e999d34b3145d2703c81caaae74e40712d48` |
| `next_step` | `4c3b0a980b6cd684464a052a10021224470972d3013e3d1a9695e59f126ea3ba` |
| `blocker` | `e13528c7bdb42c0f2864124e306864ad7d659157ba7d4a89ffd46535e8275e22` |
| `next_action` | `ba6273cb32295ca8b58635179cfc81cc4967c9074a6015d32c19457bd34bbdef` |
| `validation_command` | `e1841f4759402e9631b5510320072ec6bcf8a4967d53bba6844e2a48132b4090` |
| `expected_artifact` | `05670d9ebe9966af33c54b7afedc855efbae2e2cd351b944ec4a8d7d40347270` |

The rebuilt ordinary CLI exposes all six source-equal fields and matching
complete predecessor hashes. Coverage, manifest/milestone coverage, hierarchy,
task-card lineage, selected ready task card and drift are clean.

## Next Boundary

This session has one optional update before the common tail. Next compose two
sequential state-driven optional updates on the same owned local: the second
work must consume the first current-or-retained value, not rediscover its
original input. Keep exact scalar types, explicit mutability, read-only entering
fields, seals, scopes, source budgets, selected-only effects, failure before
publication and source-free identities. Generalized field/root writes,
resources, arbitrary deeper exits, global immutability, stable ABI, formal
safety, measured performance, generalized heterogeneous closure and completed
self-hosting remain separate.
