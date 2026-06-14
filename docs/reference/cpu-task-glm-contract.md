# CPU Task GLM Contract

This document records the current `GLM` interpretation of the `cpu` task line.

It is intentionally conservative.

Right now the goal is **not** to claim a finished concurrent ownership model.
The goal is to say, as clearly as possible, how `Task<T>` and
`TaskResult<T>` are currently treated by the repository's graph-lifetime
machinery.

## Scope

This document is about the `GLM`-visible meaning of:

* `spawn(...)`
* `cancel(...)`
* `timeout(...)`
* `join(...)`
* `join_result(...)`
* `task_completed(...)`
* `task_timed_out(...)`
* `task_cancelled(...)`
* `task_value(...)`

It is not yet a full answer for:

* worker-to-worker memory visibility
* shared mutable task payloads
* thread safety
* executor fairness
* cross-task alias propagation

## Current Position

Today the repository has two useful but still different layers:

* a meaningful `cpu` task contract
* a minimal explicit `GLM` ownership/lifetime layer

Those two layers are beginning to align, but they are not yet the final
unified concurrent memory model.

So this document should be read as:

* "how tasks currently fit into `GLM`"
* not "the final concurrency ownership story"

## Current GLM Reading Of CPU Tasks

At the current `YIR`/`GLM` level:

* async/task primitives are visible as semantic CPU operations
* `join_result(...)` is the current task observation root
* `task_*` helpers are observers or payload extractors on that observation root

This means the repository already distinguishes:

* raw task handles
* task-result observation handles
* direct payload extraction

That distinction matters for ownership reasoning even before real parallel
execution exists.

## Present Graph Classification

Today `glm_profile_for_operation(...)` classifies async core task operations
through the generic async-core path.

That means the current graph-level approximation is:

* task primitives are still `val`-shaped
* their arguments are currently treated as `val Read`
* no special task-only `res` lifetime edge class exists yet

This is intentionally smaller than the final model.

So, for now:

* `Task<T>` is a typed async handle at the language/runtime layer
* but not yet a full `res`-style ownership object in `GLM`

## Current Spawn Classification

`spawn(...)` is the current origin point for task handles, but it should also
be read carefully.

At the language/task-contract layer:

* `spawn(async_fn(...)) -> Task<T>`
* it creates the task handle that later flows into `join(...)`,
  `join_result(...)`, `cancel(...)`, and `timeout(...)`

At the current graph-lifetime layer:

* lowering treats task creation through the async-core pair
  * `cpu.async_call`
  * `cpu.spawn_task`
* that pair is still classified through the generic async-core approximation
* its inputs are currently modeled as `val Read`
* it is **not** yet modeled as a dedicated task-resource allocation origin
* it does **not** yet introduce task-specific lifetime edges

So, for now, treat `spawn(...)` as:

* the semantic origin of a `Task<T>` handle
* not yet a finalized `GLM` ownership-origin boundary

That gap is deliberate. It keeps the repository honest about the fact that task
syntax and task runtime meaning are ahead of the final ownership model.

## Observation Boundary

Even with that minimal graph classification, one boundary is already stable:

* `join(...)` is the direct payload path
* `join_result(...)` is the observation path
* `task_completed/result/timed_out/cancelled/value` sit on the observation path

The frontend already rejects obvious misuse such as:

* `task_completed(task)`
* `task_value(join(task))`

And `YIR` verifier already enforces the deeper rule:

* `cpu.task_value` must consume a `cpu.join_result`-shaped source

So the current repository already treats task observation as a distinct
graph-visible boundary, even if it has not yet elevated tasks to a richer GLM
resource class.

## Current TaskResult Observation Classification

Today `TaskResult<T>` is best read as the current observation handle.

At the current `GLM` layer:

* `task_completed(...)`
* `task_timed_out(...)`
* `task_cancelled(...)`
* `task_value(...)`

are all modeled as read-only observation uses on that handle.

So, for now:

* `join_result(...)` is the consuming boundary for the raw task handle
* the returned `TaskResult<T>` is the reusable observation boundary
* `task_*` helpers are read-only probes/extractors on that observation handle

This matches the current repository examples, where code often does:

1. `let result = join_result(task);`
2. `if task_completed(result) { ... }`
3. `task_value(result)`

That shape is intentionally still legal today.

## Current Join Classification

Today `join(...)` should be read carefully.

At the language/task-contract layer:

* `join(Task<T>) -> T`
* it is the immediate payload extraction path

But at the current graph-lifetime layer:

* `cpu.join` is still only approximated through the generic async-core path
* its task input is now modeled as a `res Own` consume
* it is now treated as a finalized task-handle consume boundary
* it still does **not** imply a dedicated task-only lifetime-end edge class

That is an intentional staging choice.

It keeps the current repository honest about where it is today:

* `join(...)` already matters semantically
* but it is not yet the final ownership-transfer rule for tasks

So, for now, treat `join(...)` as:

* a task payload boundary
* now a finalized `GLM` consume boundary for task handles

That tighter rule is one of the main reasons this area should still stay
conservative until the concurrent memory model is stronger.

One useful current probe shape is:

* direct `join(task)` for payload extraction
* followed later by `join_result(task)` for observation

That shape is intentionally no longer possible today because both `join(...)`
and `join_result(...)` now consume the same task handle. It remains useful as
an explicit negative probe for stricter task ownership semantics.

Current concrete probes:

* [hello_task_glm_join_nonconsuming_probe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_join_nonconsuming_probe.ns)
* [task_join_nonconsuming_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_join_nonconsuming_probe_demo)
* [FUTURE_CONSUME_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_join_nonconsuming_probe_demo/FUTURE_CONSUME_SKETCH.md)
* [FUTURE_LIFECYCLE_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo/FUTURE_LIFECYCLE_SKETCH.md)
* [FUTURE_CANCEL_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo/FUTURE_CANCEL_SKETCH.md)

## Current Cancel And Timeout Classification

`cancel(...)` and `timeout(...)` should be read with the same caution.

At the language/task-contract layer:

* `cancel(Task<T>) -> Task<T>`
* `timeout(Task<T>, i64) -> Task<T>`
* both change later lifecycle interpretation

But at the current graph-lifetime layer:

* `cpu.cancel` now consumes its incoming task handle as `res Own`
* `cpu.timeout` now consumes its incoming task handle as `res Own`
* both are now treated as task-handle ownership-transfer boundaries
* both currently return a new `Task<T>`-shaped handle, so the strongest current
  reading is “consume old handle, produce replacement handle”
* neither is yet modeled as a dedicated task-only lifetime-end effect

So, for now, treat both as:

* lifecycle-shaping task operations
* finalized task-handle consume/replace boundaries
* not yet finalized task-only lifetime-end boundaries

This is another intentional staging choice: task lifecycle semantics already
exist, but their final ownership/lifetime consequences are still open design
space.

## What Is Missing Today

The current task/GLM alignment is still incomplete.

That incompleteness matters even more for future thread/lock work: the current
repository should not pretend that a future `Thread<T>` or `Mutex<T>` family
can reuse the present task approximation unchanged. See the staged split in
[cpu-thread-lock-staging-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-staging-sketch.md).

In particular, the repository does **not** yet fully specify:

* whether task payloads are copied, transferred, or wrapped
* whether `spawn(...)` should count as a dedicated ownership-origin event in final `GLM`
* whether `join(...)` should count as a consuming event in final `GLM`
* whether `cancel(...)` or `timeout(...)` should eventually carry task-specific
  lifetime-end meaning beyond the current consume/replace boundary
* whether `Task<T>` should eventually become a `res`-class graph object
* what lifetime and alias rules hold across worker/lane/runtime boundaries

These are the next big missing pieces.

## Practical Reading For Today

If you want code that fits the current system well:

* treat `Task<T>` as a typed async handle, not as a stable shared object model
* treat `TaskResult<T>` as the current observation boundary
* use `join_result(...)` when control flow depends on lifecycle
* use `task_value(...)` only from the completed-result path
* do not assume the current `GLM` task approximation implies safe threading

## Relationship To Other References

Read this together with:

* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)
* [cpu-task-external-handle-glm-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-glm-sketch.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
