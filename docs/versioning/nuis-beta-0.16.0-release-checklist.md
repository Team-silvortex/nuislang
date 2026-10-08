# Beta 0.16 Validation Checklist

This is the operating checklist for the `beta-0.16.*` line anchored at
`dff1bdbc` (`beta-0.16.0`, 2026-10-07). The version already exists in Git;
using this checklist does not authorize a new version, commit or push.
The [minor snapshot](nuis-beta-0.16.0-snapshot.md) records the historical baseline.
Checkboxes below are recurring gates, not a certification that all have passed.

## Documentation And Task Selection

* [ ] Confirm Git's current version and distinguish it from Cargo/protocol versions.
* [ ] Keep repository, documentation, reference and mainline entrypoints on beta-0.16.
* [ ] Keep old snapshots and their original receipt dates/counts historical.
* [ ] Inspect live tensor selection, drift, coverage, hierarchy and task-card lineage.
* [ ] Update selected evidence and next action without dropping complete prior history.
* [ ] Keep source/test/Markdown caps at 800/1000/2000 lines and use relative repository links.

## Bounded CPU And Application Gates

Run from the repository root, serially, with incremental build caching disabled:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo fmt --all --check
cargo test -p nuisc --lib --test native_application_bridge --locked -j1 -- unary_float_literals unary_float_negation conditional_return_typed_snapshots --test-threads=1
cargo test -p nuisc --lib --locked -j1 -- conditional_return_equal_effect_joins conditional_return_join_captures conditional_return_one_sided_effect_joins conditional_return_partial_effect_joins conditional_return_effect_joins if_expressions:: match_expressions:: --test-threads=1
cargo test -p nuisc --test native_application_bridge --locked -j1 -- frontend lowering optimize:: typed_sparse_ typed_nested_preheader_snapshots typed_materialized_ gain_guarded_lowering gain_shared_keep_lowering --test-threads=1
cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy conditional_return_typed_handoff reachable_helpers_reject selected_call_graph aggregate_admission:: typed_helper_admission:: helper_entries:: --test-threads=1
cargo test -p nuis --bin nuis --test native_session_workflow --locked -j1 -- dev_tensor closure_then_tensor native_stored_projections native_materialized_record_arguments --test-threads=1
cargo test -p nuisc --test ns_nova_application_session --test examples_mainline_compile --locked -j1 -- compiled_nuis_ compiled_window_selection compiled_registration registration_preserves_uncalled_helpers host_absolute_paths --test-threads=1
cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

* [ ] Verify original source conditions, selected work and exits, not just helper shapes.
* [ ] Check both source arms before equality, alias substitution or private publication.
* [ ] Keep condition-as-data captures and merged source-live result authority.
* [ ] Reject wrong types, borrowed/resource transports, unreachable suffixes and over-budget work atomically.
* [ ] Require exact native success stdout/normal completion and explicit selected-trap outcomes.
* [ ] Check child-to-suffix current versions, private staging and traps after both selected and retained child outcomes.
* [ ] Check two-child region merges before intervening staging and second predicates, with poisoned inactive arguments and selected-stage traps.
* [ ] Verify the unified ordered-region walk with adjacent/more/final children, empty stages, shared budgets and original single-target retention.
* [ ] Verify single-edge logical child predicates, both RHS truth values, skipped poisoned arguments, selected RHS/leaf traps, exact types and unchanged guard-seed authority.
* [ ] Verify computed child-left calls/comparisons and checked arguments run once at original sites, contribute left-only effects, preserve transitive Effect order and trap even when their result would skip the RHS; keep nested logical trees separate.
* [ ] Verify direct logical child trees keep every complete RHS selected, share region edge/expression/combined-depth and complete-subtree capture budgets, grow helpers linearly and reject hidden logical leaves or late invalid descendants atomically.
* [ ] Verify fresh bool let/const logical initializers in selected scalar leaves and prefix/middle/suffix stages, once-only selected work, exact original declarations, private bindings, cross-stage constant seals, shared condition/initializer budgets and source-free selected-stage trap evidence.
* [ ] Verify existing owned-bool let roots in selected scalar stages, read-before-write captures, private/sole-target updates, known outer logical constant vetoes, shared original budgets, source-free bool state/stdout/byte identity and selected-stage traps. Single-statement logical child leaves remain a separate task.
* [ ] Exercise source-free artifact restoration, identity and lifecycle contracts.
* [ ] Rebuild the ordinary CLI after tensor changes; static text checks alone are insufficient.

The focused command is a reproducer, not a full-frontend/workspace acceptance
command. Count distinct tests once; native variants and duplicate reruns are not
additional test units. Record freshly measured drift totals, not snapshot totals.

## Separate Acceptance

* [ ] Record Linux/GPU/provider results only after fresh runs on the named hardware/profile.
* [ ] Track full-workspace, socket and strict-Clippy gates independently.
* [ ] Measure performance separately from structural IR/capture reductions.
* [ ] Keep scalar native closure distinct from complete heterogeneous application closure.
* [ ] Verify explicit literal-print build/cache/launch policy binding, repeated source-free restoration and unchanged pure defaults; see the [build policy contract](../reference/nuis-native-literal-print-build-policy-v1.md).
* [ ] Verify guarded effectful scalar-call selection, sequential private staging and staging before child selections, inactive argument laziness, post-prefix retention, selected prefix/predicate/argument traps and source-free execution; see the [scalar selection contract](../reference/nuis-native-effectful-scalar-selection-v1.md).
* [ ] Verify one-sided existing-scalar rebinding in both directions, current-value retention across consecutive updates, capture limits, selected traps and fresh/cache/three source-free runs under the same guard and explicit-policy contract.
* [ ] Verify nested existing-scalar updates keep descendant predicates and argument work behind ancestor guards, preserve registered guard boundaries, and pass tree/expression/capture limits plus poisoned inactive-path and selected-trap execution checks.
* [ ] Keep self-hosting responsibility transfer distinct from compiling an application in Nuis.
