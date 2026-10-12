# Native Private Sequential Mixed Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This extends the [nested branch-tree receipt](nuis-native-private-nested-mixed-continuations-v1.md)
with a separate linear route. No release, commit, public tuple type or common
YIR/backend effect authority is implied.

## Private Transport

Sequential mixed prefix selections with flat arms can preserve both the exact
owned unsealed bool ancestor and its one prefix-defined private bool child.
The existing classifier, private child token and actual root source-write
evidence remain mandatory. Other entering bindings stay read-only.
The completed return boundary exports only the ancestor bool.

Each original branch is proved in its lexical scope and installed once. A
closed compiler-private scalar carrier encodes ancestor + 2 * child in 0..=3,
using the existing canonical bool-to-i64 cast and scalar helper ABI. After each
join, pure equality/XOR and comparison recipes restore both current versions.
There is no generated division, trapping decode, public tuple, source cast
admission expansion or second outward result. Internal helper transport changes;
the public result ABI does not. Carrier names are allocated during installation
against all source bindings, not guessed during proof.

Empty arms explicitly capture retained versions before creating joined versions.
Transport is not evidence granting source-write authority. Branch-only names
cannot become continuation inputs; fresh branch/flat-tail collisions, sealed
writes, extra entering writes, hidden ordinary logical leaves and partial exits
reject atomically. Original checked calls, short-circuit work and unused effects
remain guarded and source ordered.

## Linear Bound

No selection or continuation tree is cloned. For S selections this route installs
exactly 2 * S guarded branch helpers and one constant-size pack/join/decode per
selection. Flat statements occur once in their original arms or continuation.
The existing leaf-copy route is retained separately for terminal nested trees
followed by a single flat suffix; it cannot copy a private pair selection.

The unchanged common limits are 64 nodes, 256 expressions, 32 logical edges,
eight source selection levels, below-32 expression depth and 31 captures plus a gate.
Each sequential arm and continuation has at most 16 statements. Original nodes,
expressions and logical edges are charged once; generated constant-size transport
is separately bounded by the original selection count, not charged as source work.

Six NIR units check both write orders and outer polarities, retention/current
conditions, all four pair codes and collision-free names, lexical/seal/extra-write/
hidden-work vetoes, seven nodes / seven expressions / zero logical edges and
one-less rejection. Adjacent 64/65 nodes, 256/257 expressions, 32/33 edges,
16/17 arm/suffix sizes, 8/9 selection depth and 31/32 captures are tested.
Sequences of 2, 9 and 17 selections produce exactly 4, 18 and 34 branch helpers,
without recursive continuation expansion.

## Real Execution

The [native fixture](../../tools/nuisc/tests/native_application_bridge/return_sequential_mixed_logical_effectful_scalar_selections.ns)
uses an exact fourteen-field state and nine checked helper sites. Consecutive
child/ancestor gates consume current joined values, then execute a sealed tail
and unused checked work. Two artifact units verify exact state, ancestor guards,
function-local transitive Effect paths, policy/source-free integrity and veto
of removed Effect order.

An independent imperative oracle, not the compiler's carrier or an IR evaluator,
checks all 16 truth variants, poisoned inactive denominators, complete state/
stdout, source return suppression and caller continuation. Fresh/cache/three
source-free restorations cover 240 lifecycle transitions with identical policy
and executable bytes after source, manifest and project cache deletion.
Nine selected sites must trap before later effects or state publication.

## Acceptance

The 2026-10-11 fresh selected six-cohort acceptance passes **655 distinct tests**:
393 compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control,
eight typed-return/effect-boundary, 173 LLVM including ignored, 50 native bridge,
four CLI workflows and 27 tensor units. Actual test-name lists confirm zero
compiler/typed overlap, eight new compiler units (six NIR/two artifact) and four
unique CLI names. Focus/repeat runs add no distinct units. The complete preceding
647-test receipt is frozen in all six task fields as history, not new acceptance.

The nested/sequential CLI cohort passes 480 lifecycle transitions and 18 selected
traps; the sequential fixture contributes 240 transitions and nine traps.
Truth variants, lifecycle transitions and traps are not additional test units.
Run the selected cohort serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_effectful_scalar_selections native_ordered_effectful_scalar_selections ordered_effectful_scalar_selections native_repeated_effectful_scalar_selections repeated_effectful_scalar_selections native_continued_effectful_scalar_selections continued_effectful_scalar_selections native_staged_effectful_scalar_selections staged_effectful_scalar_selections sequential_effectful_scalar_regions native_sequential_effectful_scalar_regions native_nested_effectful_scalar_selections nested_effectful_scalar_selections native_effectful_selected_calls native_effectful_scalar_rebindings effectful_scalar_rebindings effectful_selection aot_application_bundle tests_direct_calls tests_guard_buffer_order tests_recursive_composed_calls tests_higher_order_direct_calls tests_branch_helpers tests_branch_host_calls tests_async_runtime::tail_recursion tests_async_runtime::recursive_helpers tests_async_runtime::compound_flow tests_loops_terminal scalar_control::tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_typed_handoff conditional_return_effect_boundaries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy conditional_return_typed_handoff reachable_helpers_reject selected_call_graph aggregate_admission:: typed_helper_admission:: helper_entries:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_sequential_mixed_logical_effectful_scalar_selections native_return_nested_prefix_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

After final tensor publication, the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift, 2406/2406 checks and zero failures.
All six fields equal source and all six complete predecessor hashes remain intact.
The selected cell remains `active/99`, with nested mixed branches and their own
sequential continuations next. The post-publication tensor rerun passes all 27
tests and adds no distinct units to the 655-test receipt.
The drift contract is registered through
`tools/nuis/src/dev_tensor_drift_data_native_return_sequential_mixed_logical.rs`.

## Remaining Boundary

At this checkpoint the linear route required flat arms and the earlier tree
route permitted only terminal nesting before one shared flat suffix. The later
[nested sequential extension](nuis-native-private-nested-sequential-mixed-continuations-v1.md)
composes branch-local sequences without changing this historical receipt.
General multi-value transport,
resources, root no-write authority, arbitrary entering writes, mixed/partial
exits and generalized effects are not admitted.
This host scalar proof does not establish new GPU/Linux/Windows execution,
measured speedups, authentication, formal memory safety or completed self-hosting.
Full workspace and strict Clippy acceptance remain separate.
