# Native Private Ordered Return Updates V1

Status: worktree follow-up after Git baseline `8a760089` (`beta-0.16.3`).
No version bump, commit or push is implied. This extends the
[nested private return update checkpoint](nuis-native-effectful-scalar-selection-v1.md#nested-private-return-bool-updates),
not common YIR effect authority or an arbitrary control-flow route.

## Single-Target Authority

At the ordered-only checkpoint, a nested update selection first discovered matching terminal exports for one
exact owned bool already defined and unsealed in the enclosing private prefix.
Discovery has its own 64-statement and eight-selection bounds. Child declarations
cannot manufacture authority over an entering binding. An entirely update-free
terminal tree remained outside that proof, even if an earlier statement wrote.
The [retention-tail follow-up](nuis-native-private-retention-return-updates-v1.md)
now adds bounded discovery of those actual earlier writes under the same owner.

Only that proven owner constructs a private `Target` capability. Update arms pass
it to shared staging so source-ordered `let` writes and intervening child
selections may update that same target. Ordinary return staging passes no
capability: its entering bindings remain read-only. Constants, unrelated entering
bindings, different terminal targets and child-private exports are not authorized.
The capability exists only during proof, not in YIR, ABI, artifacts or backend
dispatch. No Nustar-specific route is added.

## Current Versions

Permission does not define a new version. Staging starts with an empty `defined`
set, proves every original RHS against the current scope and only then installs
the source write. First reads of the old target remain captures. A prior definite
replacement can remove that old-value capture; an earlier unused call that reads
it still requires it. Conditional updates before the first definite write keep
the incoming value available for retention, rather than pretending permission
already supplied a value.

Every original condition, checked argument, initializer, unused call, update and
complete return remains source-ordered and guarded. Empty branches transport the
actual current value without invented source work. Child-owned updates may
delegate their own target, not the ancestor's target, and child locals/seals do
not escape to siblings or ancestors. Failure remains atomic before generated
helper/name installation.

The whole return tree still shares 64 nodes, 256 expressions, 32 logical edges,
eight selection levels, below-32 expression depth and 31 captures plus a gate.
Each independent child prefix has at most 16 statements. Ordinary leaf, guarded
join, region and staged-nested plans are reused, with transitive function-local
Effect paths through guards, checks and calls.

## Real Execution

The [ordered-update fixture](../../tools/nuisc/tests/native_application_bridge/return_ordered_update_logical_effectful_scalar_selections.ns)
contains a conditional write before the first definite replacement, an observed
current value, an intervening child update, unused checked work and a final
logical update. The opposite outer arm checks retention after executed prefix
work before a nested terminal selection. Complete logical returns suppress the
callee continuation while the caller still completes.

An independent imperative oracle replays original source order for all 16
seed/first/second/reply combinations. Inactive checked arguments receive zero
denominators. Fresh/cache/three source-free restorations check complete state,
stdout, exact policy and executable-byte identity after deleting source,
manifest and project cache: 240 successful lifecycle transitions. Ten checked
helper sites have independently selected traps before state publication or
later effects.

NIR tests separately check incoming versus replacement captures, private seals,
both retention polarities, source-current child predicates, late invalid siblings
and exact original-work accounting (10 nodes / 21 expressions / four edges).
Boundary tests admit 16 prefix statements, 64 original nodes, 32 logical edges
and eight selection levels, rejecting the adjacent overflow cases. Artifact
tests check the exact state layout, guarded transitive Effect paths and unchanged
default/empty-policy and removed-order vetoes.

## Acceptance

The 2026-10-11 fresh selected acceptance passes **601 distinct tests**: 339
compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control units,
eight typed selected-return/effect-boundary units, 173 LLVM tests including
ignored cases, 50 native bridge regressions, four nested/ordered-update CLI
workflows and 27 tensor units. Exact compiler/typed names have zero overlap.
Focused reruns, truth variants and earlier receipts are not additional units.

The selected CLI cohort checks 480 successful lifecycle transitions and 26
selected traps. The ordered fixture contributes 240 transitions and ten traps,
with complete state/stdout, source exits and caller continuation, inactive
poisoned arguments and source-free policy/executable-byte identity checked.
Earlier 594-test and other checkpoint receipts remain historical. Reproduce
this selected cohort serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_effectful_scalar_selections native_ordered_effectful_scalar_selections ordered_effectful_scalar_selections native_repeated_effectful_scalar_selections repeated_effectful_scalar_selections native_continued_effectful_scalar_selections continued_effectful_scalar_selections native_staged_effectful_scalar_selections staged_effectful_scalar_selections sequential_effectful_scalar_regions native_sequential_effectful_scalar_regions native_nested_effectful_scalar_selections nested_effectful_scalar_selections native_effectful_selected_calls native_effectful_scalar_rebindings effectful_scalar_rebindings effectful_selection aot_application_bundle tests_direct_calls tests_guard_buffer_order tests_recursive_composed_calls tests_higher_order_direct_calls tests_branch_helpers tests_branch_host_calls tests_async_runtime::tail_recursion tests_async_runtime::recursive_helpers tests_async_runtime::compound_flow tests_loops_terminal scalar_control::tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_typed_handoff conditional_return_effect_boundaries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy conditional_return_typed_handoff reachable_helpers_reject selected_call_graph aggregate_admission:: typed_helper_admission:: helper_entries:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_nested_update_logical_effectful_scalar_selections native_return_ordered_update_logical_effectful_scalar_selections --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

At this checkpoint the ordinary CLI was rebuilt after publishing the receipt. Its live tensor
reported clean coverage, hierarchy, task-card lineage and drift, with 2326/2326
registered checks and zero failures. All six current fields match source and
all six complete nested-private-update predecessor SHA-256 hashes remain intact.
The cell remained `active/99`, with the separate retention-only discovery task
below. The 27 tensor tests passed again after publication, adding no distinct
units to the 601-test receipt.

## Remaining Boundary

Nonterminal-only current-target updates followed by terminal retention-only
selections needed a separate bounded discovery/authority proof, now recorded in
the [retention-tail contract](nuis-native-private-retention-return-updates-v1.md). Hidden logical
roots, arbitrary incoming writes, mixed/partial exits, loops and general
resources remain rejected here. GPU/Linux/Windows execution, measured speedups,
formal memory safety, publisher authentication and completed self-hosting are
not established by this scalar-host receipt. Full workspace and strict Clippy
acceptance remain separate from this selected cohort.
