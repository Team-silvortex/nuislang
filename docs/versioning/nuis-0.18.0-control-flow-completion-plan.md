# `nuis` 0.18.0 Control-Flow Completion Plan

This file turns the `0.18.0` control-flow goal into a concrete work map.

It is intentionally narrower than a full release snapshot.

The question here is:

`what still has to become clearer and more integrated before we can honestly say control flow is a mature compiler mainline?`

## Current Truth We Already Have

Today’s checked-in control-flow story is already meaningfully better than it
used to be.

Current strong points include:

* frontend `if` expression lowering in many value positions
* frontend `match` expression lowering in many value positions
* `match` lowering inside `while` bodies
* loop-family lowering for counted/carry routes
* flow/post-flow loop lowering for branch-driven routes
* project-backed state anchors for:
  `loop_while_i64_cond_chain`,
  `loop_while_i64_flow_cond_chain`,
  `loop_while_i64_post_flow_cond_chain`

Current working anchors:

* frontend:
  [tests_control_flow.rs](../../tools/nuisc/src/frontend/tests_control_flow.rs)
* lowering:
  [tests_loop_flow.rs](../../tools/nuisc/src/lowering/tests_loop_flow.rs)
* lowering:
  [tests_loop_post_flow.rs](../../tools/nuisc/src/lowering/tests_loop_post_flow.rs)
* project:
  [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)
* project:
  [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)

Current frontdoor project routes:

* sync control-flow route:
  `chained_while_demo -> match_branching_while_demo -> flow_continuing_while_demo -> post_flow_breaking_while_demo -> post_flow_branching_continuing_while_demo`
* async control-flow route:
  `task_async_observer_bridge_demo -> task_async_if_expression_positions_demo -> task_async_await_match_operand_demo -> task_async_match_call_argument_demo -> task_async_struct_field_match_demo -> task_async_method_receiver_match_demo -> task_async_helper_expanded_match_demo -> task_async_while_flow_cond_demo -> task_async_while_post_flow_demo -> task_async_while_post_flow_cond_demo -> task_async_while_post_flow_compound_demo`
* generic/control-flow route:
  `generic_method_bound_if_binding_demo -> generic_method_bound_nested_match_demo -> generic_method_bound_guarded_nested_match_demo`

First landed `0.18.0` control-flow project anchors:

* state:
  `match_branching_while_demo`
* state:
  `match_expr_branching_while_demo`
* state:
  `bool_match_branching_while_demo`
* task:
  `task_result_policy_branch_demo`
* task:
  `task_lifecycle_branch_demo`

## What Still Feels Incomplete

The main incompleteness is no longer “there is no control flow”.

The main incompleteness is that support still feels split across layers.

The most important remaining categories are:

### 1. Expression-Position Completion

We should keep tightening routes where control-flow expressions appear inside:

* nested call arguments
* binary operands
* struct field initializers
* method receivers
* `await` operands
* branch-local helper-expanded values

Goal:

`expression-position control flow should stop being surprising`

### 2. Loop Family Closure

We already have several loop families, but they still need to feel more like
one coherent set.

Important current families:

* `loop_while_i64_chain`
* `loop_while_i64_cond_chain`
* `loop_while_i64_flow_cond_chain`
* `loop_while_i64_post_flow_chain`
* `loop_while_i64_post_flow_cond_chain`
* `loop_while_i64_async_chain`
* `loop_while_i64_async_post_flow_chain`
* `loop_while_i64_async_post_flow_cond_chain`

Current practical mapping:

* ordinary `while + carry`:
  `chained_while_demo`
* `match`-driven branching inside `while`:
  `match_branching_while_demo`
* source `continue` before post-flow:
  `flow_continuing_while_demo`
* source `break` after post-flow:
  `post_flow_breaking_while_demo`
* source post-flow conditional `continue`:
  `post_flow_branching_continuing_while_demo`
* async observer bridge into async loop families:
  `task_async_observer_bridge_demo`
* async `await if` expression-position family:
  `task_async_if_expression_positions_demo`
* async `await match` operand in expression position:
  `task_async_await_match_operand_demo`
* async `await match` inside call argument:
  `task_async_match_call_argument_demo`
* async `await match` inside struct field initializer:
  `task_async_struct_field_match_demo`
* async `await match` inside method receiver:
  `task_async_method_receiver_match_demo`
* async `await match` inside nested helper-expanded values:
  `task_async_helper_expanded_match_demo`
* async source `continue` family:
  `task_async_while_flow_cond_demo`
* async source post-flow `break`:
  `task_async_while_post_flow_demo`
* async source post-flow conditional `break`:
  `task_async_while_post_flow_cond_demo`
* async source post-flow compound `continue`:
  `task_async_while_post_flow_compound_demo`

Goal:

`the supported loop families should be explicit, stable, and easier to map from source shapes`

Near-term closure rule:

* new control-flow work should prefer extending one of the named families
  above instead of adding isolated examples with no route position
* when a lowering family gains a new meaningful source shape, it should gain a
  project anchor and a place in one of the frontdoor routes

### 3. Branch-Local Carry Truth

One of the most important real maturity questions is whether branch-local state
updates stay coherent when they come from:

* lowered `if`
* lowered `match`
* pure helper wrappers
* nested helper-expanded values
* async/task result selection paths

Goal:

`carry updates should survive branch-local rewrites without becoming shape-fragile`

### 4. Async/Task Control-Flow Crossover

The repository already proves many async routes independently.

What still needs stronger closure is their relationship to control flow:

* `if` around awaited values
* `match` around awaited values
* control-flow-local task result observation
* loop-family lowering around async observer steps
* project-backed examples where async and control-flow are both central

Goal:

`async control flow should increasingly read like ordinary control flow with async effects, not a side system`

### 5. Project-Backed Control-Flow Anchors

The next phase should keep promoting real examples into control-flow anchors
instead of treating control flow as mainly a unit-test topic.

Priority anchor classes:

* state/control-flow-heavy projects
* task/control-flow-heavy projects
* domain projects where control-flow shapes are central to the real example

Goal:

`real project anchors should increasingly be the place where control-flow claims become believable`

Current line note:

`this has already started: the first 0.18.0 work landed as real state/task project anchors, not only lowering-local snippet probes`

Memory/carry companion note:

* the current loop-memory carry boundary and next blocker breakdown now live in
  [nuis-0.18.0-loop-memory-read-contract-sketch.md](nuis-0.18.0-loop-memory-read-contract-sketch.md)
  and
  [nuis-0.18.0-loop-memory-carry-blockers.md](nuis-0.18.0-loop-memory-carry-blockers.md)

## Practical 0.18.0 Work Order

The most useful order for the line is:

1. tighten the current control-flow capability map
2. identify the most important still-missing project-backed anchors
3. close frontend/lowering mismatches for those routes
4. update release-facing docs only after the tests are real

## Candidate Near-Term Tasks

These are good `0.18.0` tasks because each one can improve multiple layers:

* strengthen `match` + loop + carry crossover anchors beyond the first landed
  state trio
* strengthen async control-flow crossover anchors beyond the first landed task
  pair
* promote one or two task-heavy control-flow projects into stronger shape
  anchors
* document the real supported loop families more explicitly
* trim or rewrite any test that protects an internal accident instead of a real
  source-level behavior

## Regression Gate For 0.18.0

By the end of the line, control-flow claims should be harder to make casually.

A believable `0.18.0` control-flow gate should include all of:

* frontend control-flow probes
* lowering loop-family probes
* async/task control-flow probes
* project-backed control-flow anchors

The first practical gate now already includes:

* frontend:
  [tests_control_flow.rs](../../tools/nuisc/src/frontend/tests_control_flow.rs)
* lowering:
  [tests_loop_flow.rs](../../tools/nuisc/src/lowering/tests_loop_flow.rs)
* lowering:
  [tests_loop_post_flow.rs](../../tools/nuisc/src/lowering/tests_loop_post_flow.rs)
* state projects:
  [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)
* task projects:
  [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)

Short rule:

`if only one layer is green, control flow is not mature enough`

## Honest Success Statement

If `0.18.0` goes well, the honest short claim should sound like this:

`nuis control flow is no longer just a set of supported forms; it is becoming a test-backed, lowering-backed, and project-backed mainline`
