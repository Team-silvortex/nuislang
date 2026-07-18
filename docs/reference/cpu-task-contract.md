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
* `join(...)`, `cancel(...)`, `timeout(...)`, `ready_after(...)`, and
  `join_result(...)` are only allowed inside `mod cpu <Unit>`
* `join(...)`, `cancel(...)`, and `join_result(...)` each expect exactly one
  `Task<...>`-like argument
* `timeout(...)` expects exactly two arguments:
  * a `Task<...>`
  * an explicit integer timeout limit
* `ready_after(...)` expects exactly two arguments:
  * a `Task<...>`
  * an explicit integer delay in scheduler ticks

If the frontend cannot prove a task-like input, it rejects the expression
rather than guessing.

## Type-Level Contract

Today the CPU task line has these conceptual types:

* `spawn(async_fn(...)) -> Task<T>`
* `timeout(Task<T>, i64) -> Task<T>`
* `ready_after(Task<T>, i64) -> Task<T>`
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

* scalar `bool`, `i32`, `i64`, `f32`, and `f64` task payloads now cross one
  packed native scheduler ABI:
  * `nuis_scheduler_task_spawn_i64_v1`
  * `nuis_scheduler_task_spawn_invoker_i64_v1`
  * `nuis_scheduler_task_timeout_v1`
  * `nuis_scheduler_task_cancel_v1`
  * `nuis_scheduler_task_join_state_v1`
  * `nuis_scheduler_task_value_i64_v1`
* the AOT shim registers a pending task, advances the lifecycle clock while
  joining, commits the completed state from task polling, and reads the payload
  through the runtime task handle
* arbitrary-arity scalar async calls consumed by `spawn_task` are emitted as
  deferred helper thunks; LLVM-generated `i64(ptr context)` wrappers decode
  contiguous eight-byte slots into typed arguments and normalize typed returns
  back into the shared payload slot; floating-point values use bit-preserving
  packing rather than numeric integer conversion
* timeout limits are stored on the native task slot; `limit <= 0` transitions
  the pending slot directly to `TimedOut`, while positive deadlines continue
  through lifecycle polling and preserve completed thunk execution
* `task_timed_out(...)` therefore observes a runtime-produced terminal state
  in native binaries rather than a lowering-time approximation
* cancellation transitions a pending native slot directly to `Cancelled`;
  later joins observe state code `3`, and completed/timeout terminal states are
  not overwritten
* thunk storage is normalized as one `NuisSchedulerTaskThunkPacket` per slot,
  carrying a common invoker and opaque context; completion, timeout,
  cancellation, startup failure, reset, and shutdown release owned contexts
* source `ready_after(task, ticks)` lowers through `NIR` to `cpu.ready_after`
  and updates the pending slot's ready tick through
  `nuis_scheduler_task_ready_after_v1`
* positive deadline ordering completes when `ready_delay <= timeout_limit` and
  times out when readiness is later; non-positive timeout limits remain
  immediate timeouts
* flat non-empty structs containing only `bool`, `i32`, `i64`, `f32`, and
  `f64` fields are materialized into stable eight-byte slots and cross the
  scheduler through the owned payload ABI; floating fields retain their bits,
  and aggregate-producing helpers execute from the lifecycle poll invoker

The native shim now defines the first owned aggregate payload boundary through
`NuisSchedulerOwnedPayloadV1`:

* descriptors carry `data`, byte `size`, power-of-two `alignment`, a non-zero
  `type_id`, and optional move plus mandatory drop hooks
* `nuis_scheduler_task_spawn_owned_v1` accepts a descriptor pointer and
  transfers a valid descriptor into one task slot, applying the move hook first
  when one is present; a full task table consumes the valid payload through its
  drop hook rather than leaking it
* `nuis_scheduler_task_take_owned_v1` is a one-shot ownership transfer through
  an out-parameter; it is not a borrowed view
* timeout, cancellation, lifecycle reset, and shutdown drop an untaken payload
  exactly through the slot's registered hook
* completed but untaken payloads remain owned by the scheduler until take or
  shutdown

Source `Task<Struct>` lowering is enabled for flat scalar-only structs. LLVM
materializes one eight-byte slot per field, derives a deterministic non-zero
type identity from the struct/field layout, and reconstructs virtual field SSA
only after the one-shot take. Both direct `join` and
`join_result`/`task_value` consume that runtime-owned payload. Empty, nested,
pointer-bearing, text, enum, and buffer aggregates remain outside this first
layout contract.

Today `nuis` does **not** yet have:

* a mature parallel executor
* a stable worker-pool or lane scheduler contract for tasks
* a queue-backed timer wheel or mature delayed-work executor beyond the
  current deterministic task ready-tick model
* an explicit failed terminal state when an owned helper cannot materialize a
  non-null payload (for example after allocation failure)
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
* treat `TaskResult<T>` as the current reusable observation handle, not a
  consume-once payload object
* if future thread/lock work begins, do not read it as an in-place redefinition
  of `Task<T>`; read the staging split in
  [cpu-thread-lock-staging-sketch.md](cpu-thread-lock-staging-sketch.md)

## Related References

* [yir-langref.md](yir-langref.md)
* [yir-tools-reference.md](yir-tools-reference.md)
* [nir-memory-model.md](nir-memory-model.md)
* [nir-optimization-contract.md](nir-optimization-contract.md)
* [cpu-task-scheduler-clock.md](cpu-task-scheduler-clock.md)
* [cpu-thread-lock-staging-sketch.md](cpu-thread-lock-staging-sketch.md)
