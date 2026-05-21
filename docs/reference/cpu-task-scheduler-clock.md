# CPU Task, Scheduler, and Clock Contract

This document explains the current relationship between three things that are
already visible in the repository, but are not yet a finished concurrency
runtime:

* `cpu` task semantics
* scheduler/lane surfaces
* clock and timeout surfaces

The goal is to keep those layers conceptually aligned while the implementation
is still evolving.

## Scope

This document is about the current shape of:

* `spawn(...)`, `join(...)`, `join_result(...)`, `cancel(...)`, `timeout(...)`
* `cpu_bind_core(...)` and lane defaults
* `cpu_tick_i64(...)`, test clock domains, and timeout resolution

It is **not** yet a promise of:

* a full executor model
* worker-pool fairness semantics
* a final task scheduler
* a final cross-domain clock synchronization model

## Current Split

Today the repository has three partial but real layers:

1. task semantics
2. scheduler/lane metadata
3. clock/timing bridges

Each layer is meaningful on its own.
What is still missing is the final runtime that would unify them into one
complete concurrent execution model.

## Task Layer

Current task semantics already exist through:

* `spawn(...)`
* `join(...)`
* `join_result(...)`
* `task_completed(...)`
* `task_timed_out(...)`
* `task_cancelled(...)`
* `task_value(...)`
* `cancel(...)`
* `timeout(...)`

Current references:

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)

Important current truth:

* these operations already have meaningful frontend, `NIR`, `YIR`, and built-in
  CPU-domain semantics
* they do **not** yet imply a finished native concurrent runtime

## Scheduler / Lane Layer

The repository already has a real scheduler-facing surface, but it is still
mod-owned policy rather than a full task executor.

Current sources:

* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* `/Users/Shared/chroot/dev/nuislang/nustar-packages/*.toml`
* [lowering.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering.rs)

Today the important pieces are:

* compiler-known host-read surface:
  * `SchedulerLane`
* compiler-known scheduler bridge:
  * `cpu_bind_core(0) -> host_main_lane`
  * `cpu_bind_core(n>0) -> worker_lane`
* mod-owned lane defaults in `nustar` manifests:
  * CPU has defaults like `cpu.alloc_node=mem`, `cpu.print=main`
  * data/shader/kernel also carry their own default lane maps

Current reading rule:

* lanes are already visible as lowering policy
* lanes are **not** yet a proof that tasks execute on a finished worker runtime

In other words:

* lane assignment exists
* final task scheduling semantics do not yet

## Clock / Timing Layer

The repository also has a meaningful timing surface, but it is still split
between compiler-known host reads and explicit bridge contracts.

Current sources:

* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
* [clock_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime.ns)
* [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
* [clock_domain_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime.ns)
* [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)

Current important pieces:

* compiler-known host-read surface:
  * `ClockTick`
* compiler-known timing bridge names:
  * `monotonic_tick`
  * `wall_deadline`
  * `global_to_monotonic_tick_bridge`
* `nuis test` clock-facing metadata:
  * `clock_domain`
  * `clock_policy="bridge"`
  * `resolved_clock_domain`
  * `resolved_clock_source`
  * `resolved_clock_bridge`
  * `resolved_clock_surface`

Current reading rule:

* clock and timeout semantics are already visible and named
* they are still not a final distributed/global runtime clock model

## How These Three Layers Relate Today

The safe current mental model is:

* task syntax gives lifecycle and observation semantics
* scheduler/lane surfaces give lowering and host-facing placement hints
* clock surfaces give timeout and timing interpretation hints

But:

* task semantics do not yet guarantee final worker behavior
* lane semantics do not yet guarantee final executor behavior
* clock semantics do not yet guarantee final cross-domain timing behavior

That is also why two future directions matter here:

* hotspot-local async-to-sync contraction must remain proof-driven
* global-clock / local-clock negotiation must become more explicit before
  timing-sensitive contraction is trusted

So the repository is currently strongest at:

* semantic structure
* verifier-visible boundaries
* bridge naming
* compile-time examples and probes

And still intentionally conservative about:

* real parallel execution
* real worker scheduling
* final concurrent memory visibility

## Current Guidance

If you want code that fits the current system well:

* treat `Task<T>` as semantic task structure, not finished thread semantics
* treat lane defaults as lowering/runtime hints, not final executor guarantees
* treat clock bridges as explicit timing contracts, not universal clock truth
* keep control flow depending on task state on the `join_result(...)` path
* avoid assuming that task + lane + clock already imply a full concurrent
  runtime model

## Related References

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [yir-hot-sync-contraction-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-hot-sync-contraction-sketch.md)
* [yir-global-clock-negotiation-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-global-clock-negotiation-sketch.md)
