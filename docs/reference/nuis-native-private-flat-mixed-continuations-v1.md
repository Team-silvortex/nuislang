# Native Private Flat Mixed Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This extends the [terminal mixed-update proof](nuis-native-private-mixed-return-updates-v1.md).
No version bump, commit, push, new ABI or common YIR/backend effect authority
is implied.

## Branch-Current Continuations

A mixed prefix selection may update the already authorized exact owned
unsealed bool ancestor and one already prefix-defined exact owned unsealed bool
child absent from the entering scope. The existing private classifier/token
and root actual-write evidence remain mandatory. Other entering targets
and extra eligible children remain read-only or reject atomically.

The first selected extension permits flat let/const source branches and a
nonempty flat let/const continuation. The suffix is proved once in its original
lexical scope, then its immutable scalar/logical recipes run after each guarded
branch's original writes. Later reads therefore use that branch's current
ancestor and child versions, including retained incoming values on skipped paths.
Only the final ancestor is returned. No second outward result, tuple carrier,
new public instruction or ABI change is introduced.

Fresh branch declarations cannot become continuation inputs. Fresh declaration
collisions between branch and suffix reject rather than flattening separate
lexical scopes into a name/type/seal collision. Current captures distinguish
old reads, direct source definitions and mere delegated permission.

## Bounded Copying

Only a proved flat tail is copied. Selection trees and another continuation
cannot be copied. Source suffixes containing a selection, loop, print or return,
and mixed prefix branches containing selections, remain outside this extension.
This prevents recursive continuation expansion rather than hiding it behind a
larger memory budget.

A suffix has at most 16 original statements; a source branch has at most
16 prefix statements plus one terminal statement. The common source tree still
shares 64 nodes, 256 expressions, 32 logical edges, eight selection levels,
below-32 expression depth and 31 captures plus a gate.
Suffix work is charged once to original source budgets, not once per generated
branch recipe. Original stage/check/call order and unused work remain intact.

Six NIR units check both source orders/polarities, nested terminal contexts,
captures and lexical isolation, atomic extra-target/hidden-work/seal/collision/
recursive-copy vetoes, exact ten nodes / 21 expressions / four logical edges
and one-less failure. Original prefix 16/17, source nodes 64/65, logical edges
32/33, selection depth 8/9 and capture 31/32 boundaries remain checked.
Flat suffix 16/17, source branch 17/18 and attempted selection-recipe copying
also have independent vetoes.

## Real Execution

The [prefix-mixed fixture](../../tools/nuisc/tests/native_application_bridge/return_prefix_mixed_logical_effectful_scalar_selections.ns)
uses the existing exact eighteen-field state and thirteen checked helpers.
Six mixed prefix sites have selected or skipped bool gates, both source write
orders, retained paths and post-prefix child reads/seals/unused work.
The ancestor and child deliberately differ on selected and skipped paths.

An independent imperative oracle replays all 16 truth variants with poisoned
inactive denominators, complete state/stdout, caller continuation and source
exit checks. Fresh/cache/three source-free restorations cover 240 lifecycle
transitions, policy and exact executable bytes after source/manifest/cache
removal. Thirteen selected checked sites must trap before later effects or
public state. Artifact tests retain exact state, guarded checked work,
function-local transitive Effect paths and default/empty-policy/order vetoes.

## Acceptance

The 2026-10-11 fresh selected acceptance passes **639 distinct tests**: 377
compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control units,
eight typed selected-return/effect-boundary units, 173 LLVM units including
ignored, 50 native bridge units, four CLI workflow units and 27 tensor units.
Compiler/typed names are disjoint and all four CLI names are unique. The prior
631-test receipt remains a complete frozen predecessor in all six task-card
fields, not fresh acceptance for this extension.

The mixed terminal/prefix CLI cohort passes 480 lifecycle transitions and
26 selected traps. This fixture contributes 240 transitions and thirteen traps,
with full state/stdout/source-exit/caller and source-free policy/byte checks
passing. Native truth variants, lifecycle transitions and repeated tests
are not extra test units.
Run the selected cohort serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_effectful_scalar_selections native_ordered_effectful_scalar_selections ordered_effectful_scalar_selections native_repeated_effectful_scalar_selections repeated_effectful_scalar_selections native_continued_effectful_scalar_selections continued_effectful_scalar_selections native_staged_effectful_scalar_selections staged_effectful_scalar_selections sequential_effectful_scalar_regions native_sequential_effectful_scalar_regions native_nested_effectful_scalar_selections nested_effectful_scalar_selections native_effectful_selected_calls native_effectful_scalar_rebindings effectful_scalar_rebindings effectful_selection aot_application_bundle tests_direct_calls tests_guard_buffer_order tests_recursive_composed_calls tests_higher_order_direct_calls tests_branch_helpers tests_branch_host_calls tests_async_runtime::tail_recursion tests_async_runtime::recursive_helpers tests_async_runtime::compound_flow tests_loops_terminal scalar_control::tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_typed_handoff conditional_return_effect_boundaries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy conditional_return_typed_handoff reachable_helpers_reject selected_call_graph aggregate_admission:: typed_helper_admission:: helper_entries:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_mixed_update_logical_effectful_scalar_selections native_return_prefix_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

After final tensor publication, the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift and 2382/2382 drift checks with zero
failures. All six current fields equal source, all six complete predecessor
hashes remain intact and the selected cell remains `active/99`, with nested or
sequential mixed prefixes as its next task. The post-publication tensor rerun
passes all 27 tests and adds no distinct units to the 639-test receipt.

## Remaining Boundary

The later [nested branch-tree proof](nuis-native-private-nested-mixed-continuations-v1.md)
supports original terminal nested selections before one shared flat suffix.
This document's 639-test receipt remains historical for that later extension.

Nested or sequential mixed prefix selections with observed child continuations
need a separately bounded proof, explicit lexical isolation and an expansion
strategy that cannot recursively copy tails. Root no-write authority, arbitrary
entering writes, hidden ordinary logical leaves, mixed/partial exits, resources
and generalized effects remain separate.
This scalar-host acceptance does not establish new GPU/Linux/Windows execution,
measured speedups, authentication, formal memory safety or completed self-hosting.
Full workspace and strict Clippy acceptance remain separate.

