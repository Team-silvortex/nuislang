# Native Sealed-Binding Preflight V1

This bounded CPU repair follows the
[ordinary bool const staging receipt](nuis-native-plain-bool-const-staging-v1.md).
It tightens one effectful-selection outliner boundary, not a global
immutability or memory-safety claim.

## Reproduced Bypass

Before the repair, this parsed Nuis body generated effectful-selection helpers:

```nuis
const sealed: bool = saved;
if gate { let sealed = work(saved); }
return saved;
```

The new regression first failed on that exact one-sided bool case. The outer
preflight only recognized writes whose RHS was Binary And/Or; an ordinary
effectful call reached a planner that did not independently know the outer
constant inventory. This establishes a pass-local admission gap, not successful
execution of an illegal program through the full compiler.

[The preflight](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs)
is now writes_constants. Both Let and Const declarations are checked by their
destination name, independently of RHS syntax or declared type. The outer
[selection dispatcher](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections.rs)
applies this veto before trying nested, returning or simple selection plans.
Previously installed constant destinations remain in the same seal inventory.

The source-statement traversal still uses an explicit worklist and the same
64-node bound across both arm bodies, including visited and pending siblings.
The enclosing selection's complete source/depth budgets remain with its
existing proof; this preflight is not an alternative admission proof.
An oversized candidate
with known constants fails closed. Empty constant inventories retain their
no-op fast path. This check does not walk RHS/condition/return expression trees,
consume/reset expression budgets, or grant unsupported statement forms any
lowering authority. Type, effect, scope and deeper write checks remain with
their existing proofs and the compiler verifier.

## Regression Matrix

[Seven unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_sealed_binding_tests.rs)
cover the following boundaries:

- bool/i32/i64/f32/f64 roots with one-sided, two-sided and nested ordinary-call writes, including both source polarities and repeated Const destinations. A previously accepted tree also vetoes atomically after a late NIR destination mutation in its last leaf.
- Literal, name, comparison, call, And and Or RHS forms for Let and Const; the opposite arm retains a real effectful call so the ordinary forms are not rejected merely as pure selections.
- A constant installed by an earlier successful selection, followed by a forbidden write. The exact expected module and complete name inventory retain only that earlier installation, with no partial suffix helpers or names.
- Legal read-only constant captures, unrelated mutable updates, fresh bool constants in staged regions and returning trees. These controls still outline, verify and retain guard-first helpers.
- Direct NIR destination checks independent of RHS validity/type, including unknown callees and a numeric value in a bool declaration; unrelated destinations are not vetoed by this preflight alone.
- Exact flat, depth and pending-sibling 64/65-node boundaries and the empty-seal fast path.
- A 64-level expression read through an unrelated fresh binding, condition and return, confirming that expression trees are not recursively traversed by the statement preflight. Each expression is constructed independently, without recursive clone work in the test fixture.

Rejected standalone candidates preserve both the whole NirModule and generated
name inventory exactly. A rejected later candidate does not undo a prior valid
installation. The old logical-write tests remain registered under the renamed
preflight, rather than dropping their seal/budget assertions.

## Delivery And Lineage

No new execution fixture is needed to run a forbidden write. The accepted
ordinary-const, complete-tree, typed-prefix and paired-exit fixtures are the
positive native/source-free regression controls; their distinct tests are
counted once. Existing fixtures transport previous State but do not consume
it in their business computation.

The 2026-10-15 serial macOS CPU cohort passes 495 distinct tests:
449 compiler, twelve native/reference, four CLI, three host and 27 tensor.
Seven tests are new units. The first interrupted compiler run hit recursive
Clone work in the new deep-expression test fixture, not preflight traversal.
After independent expression construction replaced that clone, all seven
focused tests and the complete selected cohort pass. The red reproduction,
interrupted run and focused replay are not extra accepted/distinct tests.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The rebuilt ordinary CLI exposes six source-equal current fields and six
matching complete predecessor hashes. All 3179 drift checks /22169 patterns,
coverage, manifest/milestone coverage, hierarchy and task-card lineage pass.
Shell history isolation, UTF-8, changed file caps/relative links, changed Rust
formatting and diff checks pass. Four untouched historical files still fail
whole-workspace formatting:

- tools/nuis/tests/native_session_workflow/effect_call_return_two_bool_field_vector.rs
- tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_bool_field_vector_tests.rs
- tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs
- tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs

Strict Clippy and full-workspace tests were not run. This is not fresh
Linux/GPU/Windows, measured performance or formal-safety acceptance.

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Six complete preceding ordinary-const fields are retained, not replaced with
summaries. The current validation command encloses its complete predecessor
in a new collision-checked quoted no-op heredoc. Historical nested commands
remain literal data, verified by shell syntax and a harmless-front replay.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `27f4064b7f68b0423579b8e626cbc7b26e3fecff2646e1cb4899c47ac88aeb21` |
| `next_step` | `887f73f3c12efe869ea9d667eeea183ed02380979e41d7a3b36bfd2314a97694` |
| `blocker` | `6a86ece8c5c6f73301faa7921ac4a2631dcd4850edb34261ddeace1c1b5f8a8e` |
| `next_action` | `a44722519d96e08f1600a33bbfef1fd8c4f1b22f5d0d13c7eaf7c6010fce5b97` |
| `validation_command` | `e2ccd6d7ba228d09ba3cb9750045a26befb89608cba734d361d9762cf378ec85` |
| `expected_artifact` | `b333dae4036333111cb00bf17616af9d5cba9fcc284a2def850b72471059e7a9` |

## Next Boundary

Prove actual prior bool-State consumption in native repeated events and
source-free delivery, rather than just ABI transport. Require a result witness
that distinguishes the latest accepted state from open/stale input, selected
argument traps before publication, read-only captures, sealed local bool
initializers and unchanged authority/budgets. Global const verification,
non-bool staged constants, arbitrary deeper field-arm exits, resources, stable
ABI, GPU/Linux/Windows acceptance, formal safety, performance and completed
self-hosting remain separate.

The [state-dependent bool session receipt](nuis-native-stateful-bool-sessions-v1.md)
follows this historical next-boundary audit with actual latest-State tests,
without expanding this preflight's write authority.
