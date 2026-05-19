# CPU Task Contract

This document captures the current `cpu` task contract that `nuis` exposes
today through the frontend, `NIR`, `YIR`, and the built-in CPU domain.

It is intentionally narrower than a full concurrency or executor model.
`nuis` currently has meaningful async/task semantics, but it does **not** yet
have a mature general-purpose concurrency runtime.

## Scope

This contract is about the current semantics of:

* `spawn(...)`
* `join(...)`
* `cancel(...)`
* `timeout(...)`
* `join_result(...)`
* `task_completed(...)`
* `task_timed_out(...)`
* `task_cancelled(...)`
* `task_value(...)`

It is **not** a promise of:

* a stable executor design
* fairness or worker scheduling semantics
* shared-memory synchronization rules
* parallel execution guarantees

## Frontend Shape

In the current frontend:

* `spawn(...)` is only allowed inside `mod cpu <Unit>`
* `spawn(...)` expects exactly one async function call
* the callee passed to `spawn(...)` must already be known as `async fn`
* `join(...)`, `cancel(...)`, `timeout(...)`, and `join_result(...)` are only
  allowed inside `mod cpu <Unit>`
* `join(...)`, `cancel(...)`, and `join_result(...)` each expect exactly one
  `Task<...>`-like argument
* `timeout(...)` expects exactly two arguments:
  * a `Task<...>`
  * an explicit integer timeout limit

If the frontend cannot prove a task-like input, it rejects the expression
rather than guessing.

## Type-Level Contract

Today the CPU task line has these conceptual types:

* `spawn(async_fn(...)) -> Task<T>`
* `timeout(Task<T>, i64) -> Task<T>`
* `cancel(Task<T>) -> Task<T>`
* `join(Task<T>) -> T`
* `join_result(Task<T>) -> TaskResult<T>`
* `task_completed(TaskResult<T>) -> bool`
* `task_timed_out(TaskResult<T>) -> bool`
* `task_cancelled(TaskResult<T>) -> bool`
* `task_value(TaskResult<T>) -> T`

The important distinction is:

* `join(...)` is a direct payload operation
* `join_result(...)` is an observation boundary
* `task_*` helpers operate on `TaskResult<T>`, not on raw `Task<T>`
* the frontend already rejects observer misuse such as
  * `task_completed(task)`
  * `task_value(join(task))`

This is already verifier-visible in `YIR`: `cpu.task_value` is only valid when
its input is a `cpu.join_result`-shaped source.

## YIR Semantic Roles

At the `YIR` layer, the current CPU task family is split into semantic roles:

* `cpu.join_result`
  * result family: `Task`
  * role: result entry / observation root
* `cpu.task_completed`
  * result family: `Task`
  * role: state probe
  * probed state: `Completed`
* `cpu.task_timed_out`
  * result family: `Task`
  * role: state probe
  * probed state: `TimedOut`
* `cpu.task_cancelled`
  * result family: `Task`
  * role: state probe
  * probed state: `Cancelled`
* `cpu.task_value`
  * result family: `Task`
  * role: payload extractor

This means the current task line already has a meaningful observation model,
even though the runtime is still early.

## Current Built-In CPU Runtime Meaning

The built-in CPU domain currently interprets task handles conservatively:

* `spawn_task`
  * creates a `TaskHandle`
  * initial state is `Pending`
  * payload is already attached symbolically
* `cancel`
  * returns a new `TaskHandle` with state `Cancelled`
* `timeout`
  * returns a new `TaskHandle` carrying a timeout limit
* `join_result`
  * materializes a `TaskResultHandle`
  * exposes lifecycle state
  * only carries payload when the lifecycle is `Completed`
* `task_completed` / `task_timed_out` / `task_cancelled`
  * inspect the result state
* `task_value`
  * succeeds only when the result actually has a payload

The current built-in lifecycle rule is intentionally simple:

* `Pending` with `limit <= 0` resolves as `TimedOut`
* otherwise `Pending` resolves as `Completed`
* `Cancelled`, `TimedOut`, and `Completed` remain stable

That is a current built-in semantic approximation, not a final executor model.

## Join vs JoinResult

`join(...)` and `join_result(...)` are intentionally different:

### `join(Task<T>) -> T`

* direct payload extraction path
* errors if the task has already been cancelled or timed out

### `join_result(Task<T>) -> TaskResult<T>`

* observation path
* lets the program inspect lifecycle first
* allows later probes like:
  * `task_completed(result)`
  * `task_timed_out(result)`
  * `task_cancelled(result)`
  * `task_value(result)`

This is the current recommended shape whenever control flow depends on task
state.

## What This Means Today

Today `nuis` has:

* async/task expression support
* typed `Task<T>` and `TaskResult<T>` boundaries
* frontend checks around task-shaped inputs
* `YIR` semantic roles for task observation
* verifier-visible result-source rules
* a built-in CPU-domain interpretation for task lifecycle

Today the repository also still has one important runtime boundary:

* native LLVM/AOT lowering for CPU task primitives is not yet a full live task
  executor
* project/examples can already validate task semantics through
  `.ns -> NIR -> YIR -> LLVM`
* but that should not yet be read as proof that `spawn/join/timeout/cancel`
  already execute as a complete native task runtime

Today `nuis` does **not** yet have:

* a mature parallel executor
* a stable worker-pool or lane scheduler contract for tasks
* shared-state synchronization primitives
* a finalized concurrent memory model

So the right mental model is:

* `nuis` already has a real **CPU task contract**
* `nuis` does **not** yet have a full **concurrency runtime**

## Current Guidance

If you want code that fits the current system well:

* use `spawn(...)` only with explicit `async fn`
* use `timeout(...)` when you want lifecycle to influence later control flow
* use `join_result(...)` when you need to inspect outcome
* treat `task_value(...)` as valid only after a completed result path
* do not assume real parallel execution just because task syntax exists
* treat `task_*` helpers as observation-only APIs, not as alternate
  `join(...)` spellings

## Related References

* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [nir-optimization-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-optimization-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
