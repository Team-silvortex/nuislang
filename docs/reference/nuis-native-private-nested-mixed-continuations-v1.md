# Native Private Nested Mixed Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This extends the [flat mixed-continuation proof](nuis-native-private-flat-mixed-continuations-v1.md).
No version bump, commit, push, new ABI or common YIR/backend effect authority
is implied.

## Original Branch Tree

A mixed prefix may contain an original nested branch tree before its one
shared nonempty flat let/const suffix. Each branch permits a flat prefix and,
at most, a terminal nested selection. Each nested condition reads its preceding
branch-current source bindings, not the ancestor's original snapshot.

The existing root actual-write evidence, exact owned unsealed bool ancestor,
and one prefix-defined exact owned unsealed bool child absent from the entering
scope remain mandatory. Other entering targets remain read-only.
Only the final ancestor is returned. Empty arms retain current values and still
execute the shared suffix; no child export, tuple carrier or second outward
result is introduced.

Preflight checks every original branch for fresh-name collisions with the
suffix's lexical scope. The suffix is proved once in that original scope.
Branch-only names cannot become suffix or sibling inputs. Read-before-write
captures, const seals, guarded calls and unused work remain source ordered.

## Expansion Bound

Only flat recipes are copied at original leaves. Each original selection and
source statement is proved once. No selection plan or remaining selection
sequence is cloned. Sequential mixed prefix selections and nonterminal nested
selections followed by additional branch work remain separate.

The unchanged common bounds are 64 nodes, 256 expressions, 32 logical edges,
eight selection levels, below-32 expression depth and 31 captures plus a gate.
Each branch has at most 16 flat prefix statements plus one terminal statement;
the shared flat suffix has at most 16 statements. Its original work is charged
once, regardless of the number of leaf recipes.

For S original binary selections and T suffix statements, copied suffix recipes
are at most (S + 1) * T, not exponentially expanded combinations of sequential
selections. The conservative existing-cap bound is 1040 recipes.
A 64-node test contains 17 leaves and 272 flat recipes, with 16 original tree
leaves plus the root's retained arm. The suffix contains no selection recipe.

Six NIR units cover both orders/polarities, branch-current nested gates, retained
arms, lexical collisions/leaks, hidden work, extra writes and seals, exact
eight nodes / 16 expressions / three logical edges and one-less vetoes.
Independent adjacent node 64/65, expression 256/257, edge 32/33, selection
depth 8/9, prefix 16/17 and capture 31/32 tests retain atomic rejection.

## Real Execution

The [nested fixture](../../tools/nuisc/tests/native_application_bridge/return_nested_prefix_mixed_logical_effectful_scalar_selections.ns)
uses an exact fourteen-field state and nine checked helpers. Nested conditions
consume current child values; both write orders and empty live/retained arms
feed a common child-dependent seal, outward ancestor write and unused effect.
An independent imperative oracle checks all 16 input truth variants with
poisoned inactive denominators, complete state/stdout, source exits and caller
continuation. Fresh/cache/three source-free restorations cover 240 lifecycle
transitions with identical policy and executable bytes after source/manifest/
cache removal. Nine selected checked sites must trap before later effects or
state publication.

Two artifact units verify exact state, guarded work and function-local
transitive Effect paths, plus default/empty-policy and removed-order vetoes.

## Acceptance

The 2026-10-11 fresh selected six-cohort acceptance passes **647 distinct tests**:
385 compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control,
eight typed selected-return/effect-boundary, 173 LLVM including ignored,
50 native bridge, four CLI workflow and 27 tensor units. Actual compiler/typed
test lists are disjoint; all four CLI names are unique. The compiler list includes
six new NIR and two new artifact units. The earlier 24-test focus and later repeat
runs add no distinct units. The complete previous 639-test receipt is historical
evidence, frozen in all six task fields, not fresh acceptance for this extension.

The flat/nested CLI cohort passes 480 lifecycle transitions and 22 selected
traps. This nested fixture contributes 240 transitions and nine traps, with
complete state/stdout/source-exit/caller and source-free policy/byte checks.
Truth variants, lifecycle transitions and traps are not extra test units.
Run the selected cohort serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_effectful_scalar_selections native_ordered_effectful_scalar_selections ordered_effectful_scalar_selections native_repeated_effectful_scalar_selections repeated_effectful_scalar_selections native_continued_effectful_scalar_selections continued_effectful_scalar_selections native_staged_effectful_scalar_selections staged_effectful_scalar_selections sequential_effectful_scalar_regions native_sequential_effectful_scalar_regions native_nested_effectful_scalar_selections nested_effectful_scalar_selections native_effectful_selected_calls native_effectful_scalar_rebindings effectful_scalar_rebindings effectful_selection aot_application_bundle tests_direct_calls tests_guard_buffer_order tests_recursive_composed_calls tests_higher_order_direct_calls tests_branch_helpers tests_branch_host_calls tests_async_runtime::tail_recursion tests_async_runtime::recursive_helpers tests_async_runtime::compound_flow tests_loops_terminal scalar_control::tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_typed_handoff conditional_return_effect_boundaries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy conditional_return_typed_handoff reachable_helpers_reject selected_call_graph aggregate_admission:: typed_helper_admission:: helper_entries:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_prefix_mixed_logical_effectful_scalar_selections native_return_nested_prefix_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

After final tensor publication, the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift and 2394/2394 checks with zero failures.
All six current fields equal source and all six complete predecessor hashes
remain intact. The selected cell stays `active/99`, with sequential mixed prefixes
as its next task. The post-publication tensor rerun passes all 27 tests and adds
no distinct units to the 647-test receipt.

## Remaining Boundary

The later [sequential mixed-continuation follow-up](nuis-native-private-sequential-mixed-continuations-v1.md)
implements a separate linear route with a closed private scalar carrier. This
647-test receipt remains historical, not acceptance for that extension.
Sequential mixed prefix selections with observed child continuations need a
separately bounded version-transport strategy, not recursive tail copying.
Generalized nested branches with their own nonflat continuations, root no-write
authority, arbitrary entering writes, hidden ordinary logical leaves, mixed/
partial exits, resources and generalized effects remain separate.
This scalar-host proof does not establish new GPU/Linux/Windows execution,
measured speedups, authentication, formal memory safety or completed self-hosting.
Full workspace and strict Clippy acceptance remain separate.
