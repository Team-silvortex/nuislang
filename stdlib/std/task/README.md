# `std/task`

This directory is the reading router for the current `std task/runtime` lane.

Keep the actual recipe sources in
[stdlib/std](../../../stdlib/std) for now; this file
exists to give the lane a cluster-shaped front door before any higher-risk
filesystem reshuffle.

Canonical companions:

* cluster contract:
  [std-task-layering-contract.md](../../../docs/reference/std-task-layering-contract.md)
* semantic boundary:
  [cpu-task-contract.md](../../../docs/reference/cpu-task-contract.md)
* scheduler/clock note:
  [cpu-task-scheduler-clock.md](../../../docs/reference/cpu-task-scheduler-clock.md)
* global `std` rule:
  [std-mainline-layering-contract.md](../../../docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [current-mainline-map.md](../../../docs/current-mainline-map.md)
* project companions:
  [examples/projects/task/README.md](../../../examples/projects/task/README.md)
* source companions:
  [examples/ns/ffi/README.md](../../../examples/ns/ffi/README.md)

## Current Lane Shape

Read the current lane in this order:

```text
semantic core
-> async control
-> async result
-> clock and scheduler branches
-> cli-facing composition
```

Current rule:

* read this lane against the CPU task semantic contract first, not as a claim
  that the final executor/runtime already exists
* keep thread/lock staging beside this lane, not silently folded into it
* use `task_cli` as the first host-facing combined route, not as the first
  place task structure becomes readable

## Source Router

### Semantic Core

* [task_runtime_recipe.ns](../../../stdlib/std/task_runtime_recipe.ns)
* [task_status_recipe.ns](../../../stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](../../../stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](../../../stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](../../../stdlib/std/task_lifecycle_recipe.ns)

### Async Control

* [task_fallback_recipe.ns](../../../stdlib/std/task_fallback_recipe.ns)
* [task_policy_recipe.ns](../../../stdlib/std/task_policy_recipe.ns)
* [task_batch_recipe.ns](../../../stdlib/std/task_batch_recipe.ns)
* [task_windowed_batch_recipe.ns](../../../stdlib/std/task_windowed_batch_recipe.ns)

### Async Result

* [task_result_family_recipe.ns](../../../stdlib/std/task_result_family_recipe.ns)
* [task_result_policy_recipe.ns](../../../stdlib/std/task_result_policy_recipe.ns)
* [task_result_batch_recipe.ns](../../../stdlib/std/task_result_batch_recipe.ns)
* [task_result_windowed_batch_recipe.ns](../../../stdlib/std/task_result_windowed_batch_recipe.ns)

### Clock And Scheduler Branches

* [task_clock_recipe.ns](../../../stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](../../../stdlib/std/task_scheduler_recipe.ns)

### CLI-Facing Composition

* [task_cli_recipe.ns](../../../stdlib/std/task_cli_recipe.ns)

## Companion Validation Router

Use the FFI and project companions as grouped mirrors instead of browsing every
neighboring branch probe first.

Shortest grouped route:

* source-level anchors:
  [hello_task_runtime_facades.ns](../../../examples/ns/ffi/hello_task_runtime_facades.ns),
  [hello_task_scheduler_facades.ns](../../../examples/ns/ffi/hello_task_scheduler_facades.ns),
  [hello_clock_test_facades.ns](../../../examples/ns/ffi/hello_clock_test_facades.ns)
* project-form anchors:
  [task_runtime_demo](../../../examples/projects/task/task_runtime_demo),
  [task_status_observe_demo](../../../examples/projects/task/task_status_observe_demo),
  [task_result_policy_branch_demo](../../../examples/projects/task/task_result_policy_branch_demo),
  [task_clock_observe_demo](../../../examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](../../../examples/projects/task/task_scheduler_observe_demo)

Wider grouped route:

* lifecycle, fallback, and policy:
  [task_lifecycle_branch_demo](../../../examples/projects/task/task_lifecycle_branch_demo),
  [task_fallback_branch_demo](../../../examples/projects/task/task_fallback_branch_demo),
  [task_policy_branch_demo](../../../examples/projects/task/task_policy_branch_demo)
* batch and result families:
  [task_batch_branch_demo](../../../examples/projects/task/task_batch_branch_demo),
  [task_windowed_batch_branch_demo](../../../examples/projects/task/task_windowed_batch_branch_demo),
  [task_result_family_branch_demo](../../../examples/projects/task/task_result_family_branch_demo),
  [task_result_batch_branch_demo](../../../examples/projects/task/task_result_batch_branch_demo),
  [task_result_windowed_batch_branch_demo](../../../examples/projects/task/task_result_windowed_batch_branch_demo)
* async recursion and control-flow crossover:
  [task_recursive_async_demo](../../../examples/projects/task/task_recursive_async_demo),
  [task_recursive_async_shared_suffix_demo](../../../examples/projects/task/task_recursive_async_shared_suffix_demo),
  [task_async_observer_bridge_demo](../../../examples/projects/task/task_async_observer_bridge_demo),
  [task_async_while_flow_cond_demo](../../../examples/projects/task/task_async_while_flow_cond_demo)
* staged thread/lock route:
  [task_thread_mutex_demo](../../../examples/projects/task/task_thread_mutex_demo)
* host/tooling bridge:
  [task_cli_tooling_demo](../../../examples/projects/task/task_cli_tooling_demo)

## Current Reading Rule

If you only want one pass:

1. start with [task_runtime_recipe.ns](../../../stdlib/std/task_runtime_recipe.ns)
2. widen to [task_lifecycle_recipe.ns](../../../stdlib/std/task_lifecycle_recipe.ns)
3. then read [task_result_policy_recipe.ns](../../../stdlib/std/task_result_policy_recipe.ns)
4. end with [task_scheduler_recipe.ns](../../../stdlib/std/task_scheduler_recipe.ns) and [task_cli_recipe.ns](../../../stdlib/std/task_cli_recipe.ns)

Short rule:

* semantic core first
* async control/result second
* timing and scheduler after that
* cli-facing aggregation last
