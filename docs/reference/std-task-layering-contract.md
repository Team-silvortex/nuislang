# `std` Task Layering Contract

This file captures the current layering contract for the checked-in `std`
task-facing lanes.

It sits one level below
[std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md):
that file explains the global `std` rule of thumb, while this file explains
what the task-facing lane currently means in repository practice.

## Current Lane Shape

The current task-facing lane prefers this order:

```text
current cpu task semantics
-> narrow task/runtime and task-observation recipes
-> clock/scheduler or CLI-facing composition recipes
-> source/project companions
```

For checked-in `std`, that currently means:

```text
task_runtime
-> task_status
-> task_value
-> task_compare
-> task_lifecycle
-> task_fallback
-> task_policy
-> task_batch
-> task_windowed_batch
-> task_result_family
-> task_result_policy
-> task_result_batch
-> task_result_windowed_batch
-> task_clock / task_scheduler
-> task_cli
```

## Semantic Boundary First

This lane should be read against the current CPU task semantic contract first,
not as proof of a finished concurrency runtime.

Current semantic references:

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)

The practical rule today is:

* task syntax already has real frontend, `NIR`, `YIR`, and built-in CPU-domain
  meaning
* the checked-in `std` pure task recipes are the narrow readable source-level
  contract for those semantics
* the scheduler and clock branches are meaningful today, but they are still
  explicit bridges rather than a final executor/runtime model

That means the pure `std` task layer is not “the final runtime”.
It is the current checked-in front door for readable task structure and
observation.

## Pure Task Layers

These are the current narrow checked-in task routes.

* [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns)
* [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
* [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns)
* [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
* [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns)
* [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns)
* [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns)
* [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)

Current role split:

* `task_runtime` is the narrow entry surface:
  `spawn -> join_result -> task_completed/task_timed_out/task_cancelled -> task_value`
* `task_status` isolates result-state observation
* `task_value` isolates payload extraction after a valid result path
* `task_compare` isolates branchable comparison over task outcomes
* `task_lifecycle` bundles the first wider but still task-local lifecycle route
* `task_fallback` isolates timeout-first fallback chaining before wider policy
* `task_policy` bundles the first explicit task-local policy/fallback route
* `task_batch` isolates the first explicit multi-task fan-in and summary route
* `task_windowed_batch` splits batch summary into preview/final windows before domain bridges
* `task_result_family` bundles completed/timed-out/cancelled observation with first value-family summary
* `task_result_policy` combines result-family observation with task-local policy selection
* `task_result_batch` combines result-family observation with explicit multi-result fan-in
* `task_result_windowed_batch` splits result-batch summary into preview/final windows

These are the narrowest readable contracts for the checked-in task semantics
before timing, scheduler, or CLI concerns are layered on top.

## Current Reading Order

The current task-facing lane reads best in three short groups:

* semantic core:
  `task_runtime -> task_status -> task_value -> task_compare -> task_lifecycle`
* async control:
  `task_fallback -> task_policy -> task_batch -> task_windowed_batch`
* async result:
  `task_result_family -> task_result_policy -> task_result_batch -> task_result_windowed_batch`

That means the shortest practical route today is:

```text
task semantic core
-> async control
-> async result
-> task_clock / task_scheduler
-> task_cli
```

## Timing And Scheduler Branches

These recipes intentionally extend the pure task lane into timing and lane-aware
observation.

* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)

Current role split:

* `task_clock` combines task observation with the current time/clock bridge
* `task_scheduler` combines task observation with scheduler/lane metadata and
  timing probes

These are not the narrowest task semantics.
They are the current checked-in branch points where the task lane meets
clock/tick and lane/bind-core concerns.

## Wider Composition Layer

The current wider CLI-facing composition layer for this lane is:

* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)

Its current role is to combine:

* task lifecycle observation
* timeout-sensitive branching
* host I/O writes
* diagnostic/report-style output
* monotonic tick capture

The current rule is the same as the global `std` rule:

* `task_cli` should not be the first place task structure becomes readable if
  the narrower pure task layers can reasonably exist on their own

## Current Task Cluster

```text
task_runtime
-> task_status
-> task_value
-> task_compare
-> task_lifecycle
-> task_fallback
-> task_policy
-> task_batch
-> task_windowed_batch
-> task_result_family
-> task_result_policy
-> task_result_batch
-> task_result_windowed_batch
-> task_clock / task_scheduler
-> task_cli
```

Concrete sources:

* [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns)
* [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
* [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns)
* [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
* [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns)
* [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns)
* [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns)
* [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)

This cluster should currently be read as:

* `task_runtime` explains the first semantic route
* `task_status` and `task_value` explain observation vs extraction
* `task_compare` and `task_lifecycle` finish the semantic core
* `task_fallback`, `task_policy`, `task_batch`, and `task_windowed_batch` are the current async-control subgroup
* `task_result_family`, `task_result_policy`, `task_result_batch`, and `task_result_windowed_batch` are the current async-result subgroup
* `task_clock` and `task_scheduler` explain timing and lane-aware extensions
* `task_cli` explains the first practical host-facing combined route

## Current Reading Rule For Clock And Scheduler

The task lane should currently be read with this split in mind:

* task semantics are already real
* scheduler/lane surfaces are already named and visible
* clock/tick surfaces are already named and visible
* the final unified executor/runtime that would make them one finished system
  does not yet exist

That is why:

* `task_clock` should be read through
  [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
  [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
  and
  [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* `task_scheduler` should be read through
  [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
  and
  [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)

## Companion Expectation

The current checked-in task lane is expected to have direct mirrors in:

* `examples/ns/ffi` for the source-level facade view
* `examples/projects/task` for the project-form route

Examples:

* [hello_task_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns)
* [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
* [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo)
* [task_clock_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo)
* [task_scheduler_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo)
* [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo)
* [task_fallback_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo)
* [task_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_policy_branch_demo)
* [task_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo)
* [task_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo)
* [task_result_family_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo)
* [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo)
* [task_result_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo)
* [task_result_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo)

## What This Contract Does Not Promise

This file does not promise that:

* the current task cluster order is a frozen ABI
* `task_clock` and `task_scheduler` already imply a final worker runtime
* `task_cli` is the final long-term package boundary for task-facing CLI work
* current checked-in task examples prove real parallel execution

It only captures the current repository truth about how the checked-in
task-facing lanes are meant to stack today.

## Current Guidance

If you are extending this lane today:

* add the narrow task/runtime recipe first when the surface can stand on its own
* keep task observation and payload extraction explicit before widening the lane
* only add clock, scheduler, or CLI bundling after the pure task layer is
  already readable
* do not treat timing or lane metadata as proof of a finished executor contract

If you are reading this lane today:

* start with [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
  if you need the semantic truth
* start with the pure `*_recipe.ns` files if you need the narrow checked-in
  source contract
* move to `task_clock`, `task_scheduler`, or `task_cli` only after the pure
  task layers are clear

## Related References

* [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [std-host-io-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-host-io-layering-contract.md)
* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
