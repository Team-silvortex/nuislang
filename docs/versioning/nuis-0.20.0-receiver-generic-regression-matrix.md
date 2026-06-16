# `nuis` `0.20.0` Receiver-Generic Regression Matrix

This file is the short regression map for the current receiver-side explicit
generic specialization surface entering the `0.20.*` line.

It answers one practical question:

`which receiver-generic method-call routes are already intentionally defended, which test file owns them, and which shapes should still be treated as active follow-up work?`

Read this together with:

* [nuis-0.20.0-generic-validation-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-generic-validation-regression-matrix.md)
* [nuis-0.20.0-branch-runtime-lowering-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md)
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
* [../reference/control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)

## Short Rule

For `0.20.*`, explicit receiver generic arguments should be read as a connected
compile-chain surface, not as an isolated parser feature.

The current proof spine now expects explicit receiver generic specialization to
survive:

* direct receiver calls on generic struct literals
* payload-style constructor and alias-constructor receivers
* generic helper-return receivers
* nested field-access receiver chains
* `await` over generic helper-return receivers
* `spawn` / `join` task wrapping and unwrapping
* `Result<Task<...>>` plus `?`
* branch-local `if` and `match` control-flow routes

Short rule:

`if an explicit receiver generic call only works in a straight line but breaks after await/task/error/control-flow wrapping, that is a regression`

## Current Regression Owner

Primary file:

* [tests_generic_structs.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_structs.rs)

Current responsibility:

* receiver-side explicit generic parsing and specialization anchors
* constructor-style receiver expected-type reconstruction
* helper-return receiver specialization
* nested field-access receiver expected-type back-propagation
* async/task/result/control-flow receiver-chain preservation

Short rule:

`if the question is “does receiver.method<T...>(...) still specialize correctly after this wrapper shape?”, this file is now the first owner`

## Current Proven Families

### 1. Direct constructor and literal receiver anchoring

Proven today:

* generic struct literals can anchor explicit receiver generic calls
* payload-style struct constructors can anchor explicit receiver generic calls
* payload-style alias constructors can anchor explicit receiver generic calls

Primary anchors:

* `lowers_receiver_method_call_with_explicit_generic_args_anchoring_struct_literal`
* `lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_constructor`
* `lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_alias_constructor`

### 2. Helper-return receiver anchoring

Proven today:

* a generic helper returning the receiver type can be specialized from the
  receiver-side explicit generic arguments

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_generic_helper_call`

### 3. Nested field-access receiver anchoring

Proven today:

* one-layer field chains keep explicit receiver generic specialization
* multi-layer field chains keep explicit receiver generic specialization

Primary anchors:

* `lowers_receiver_method_call_with_explicit_generic_args_through_helper_field_chain`
* `lowers_receiver_method_call_with_explicit_generic_args_through_nested_helper_field_chain`

### 4. Async awaited receiver anchoring

Proven today:

* `await` can sit under the receiver chain without losing explicit receiver
  generic specialization
* the current lowered shape still reaches the specialized impl helper through
  awaited payload extraction plus field access

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_awaited_nested_helper_chain`

### 5. Task runtime wrapping

Proven today:

* `spawn(...)` preserves helper specialization on the producing side
* `join(...)` / task-unwrapping preserves receiver specialization on the
  consuming side
* the current backend-facing shape is now explicitly checked through
  `CpuSpawn` and `CpuJoin`, not only source-level calls

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_spawn_join_nested_helper_chain`

### 6. Error-routing through `Result<Task<...>>`

Proven today:

* `fetch(...) -> Result<Task<Nest<i64, bool>>, Error>` can flow through `?`
  and `await` without losing receiver-side explicit generic specialization
* success and error branches are both preserved in the current lowered shape

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_result_task_chain`

### 7. `if` branch-local receiver chains

Proven today:

* branch-local `Result<Task<...>>` routes keep separate try payload/error
  ownership per branch
* both sides of the outer `if` independently preserve the same explicit
  receiver generic specialization

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_if_result_task_chain`

### 8. `match` branch-local receiver chains

Proven today:

* `match`-routed `Result<Task<...>>` chains preserve explicit receiver generic
  specialization after control-flow lowering
* current lowering into an outer equality-guarded `if` still preserves the
  same receiver specialization shape in both branches

Primary anchor:

* `lowers_receiver_method_call_with_explicit_generic_args_through_match_result_task_chain`

## Current Practical Reading Order

If you only need the shortest believable route for this surface, use:

1. direct constructor/literal anchors
2. helper-return receiver anchor
3. nested field chain anchor
4. awaited nested receiver anchor
5. spawn/join task anchor
6. result-task straight-line anchor
7. `if` result-task branch anchor
8. `match` result-task branch anchor

Short rule:

`do not jump straight to the deepest task/result/control-flow case unless the simpler constructor/helper/field cases are already known-good`

## What This Matrix Does Not Yet Claim

This matrix should stay honest about the remaining frontier.

Not claimed yet:

* receiver explicit generic specialization through lambda-captured receiver
  chains
* receiver explicit generic specialization through higher-order callback
  return-position chains
* receiver explicit generic specialization through deeper `Result<Result<...>>`
  or mixed `Option<Result<Task<...>>>` style nesting
* receiver explicit generic specialization through project-level std/http
  examples rather than frontend-only regression families

Those may work in some cases already, but they are not yet the intentionally
owned `0.20.*` proof spine.

## Follow-Up Rule

When a new receiver-generic regression appears, use this order:

1. decide whether the failure belongs to constructor anchoring, helper
   specialization, field-chain propagation, async/task wrapping, or
   branch-local error routing
2. place the regression in
   [tests_generic_structs.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_structs.rs)
   if the issue is still fundamentally a frontend receiver-shape problem
3. only move it to a broader task/network/project regression family when the
   primary failure is no longer receiver specialization itself

The current `0.20.*` ownership rule should stay simple:

`receiver explicit generic specialization is now broad enough that wrapper depth alone should not be treated as a new feature family`
