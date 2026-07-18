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
* an owned invoker that cannot materialize a non-null payload transitions to
  `Failed`; `join_result(...)` preserves runtime state code `4`, and
  `task_failed(...)` observes it without exposing an invalid payload
* direct `join(...)` requires state code `1` before extracting any scalar or
  owned payload and terminates deterministically on every other terminal state
* thunk storage is normalized as one `NuisSchedulerTaskThunkPacket` per slot,
  carrying a common invoker and opaque context; completion, timeout,
  cancellation, startup failure, reset, and shutdown release owned contexts
* source `ready_after(task, ticks)` lowers through `NIR` to `cpu.ready_after`
  and updates the pending slot's ready tick through
  `nuis_scheduler_task_ready_after_v1`
* positive deadline ordering completes when `ready_delay <= timeout_limit` and
  times out when readiness is later; non-positive timeout limits remain
  immediate timeouts
* recursive non-empty structs containing `bool`, `i32`, `i64`, `f32`, `f64`,
  and `String` use self-describing aggregate slots and cross the scheduler
  through the owned payload ABI; floating fields retain their bits, text bytes
  are copied into GLM-tokened blobs, and aggregate-producing helpers execute
  from the lifecycle poll invoker

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
* timeout and cancellation release a deferred owned-invoker context before its
  helper executes; native coverage uses a visibly printing helper and requires
  empty stdout on both terminal paths
* completed but untaken payloads remain owned by the scheduler until take or
  shutdown
* `NuisSchedulerOwnedBlobV1` is the first GLM-bearing dynamic leaf protocol:
  it deep-copies borrowed bytes, rejects a zero GLM token, validates identity
  moves, and exposes one drop hook compatible with the scheduler descriptor
* `NuisSchedulerOwnedAggregateV1` tags each flattened slot as scalar or blob;
  its common drop hook walks every slot before freeing the aggregate
* aggregate construction is transactional: every slot starts unset and must be
  written exactly once; duplicate, invalid, or incomplete construction poisons
  the aggregate, while `finish` drops all attached blobs and returns null
* a null finalized pointer makes a deferred owned invoker enter `Failed`; the
  immediate await lane requires a finalized aggregate and exits with status 71
  instead of exposing partially initialized fields
* compiled C coverage moves one blob through join/take, verifies that mutating
  the borrowed source cannot affect owned bytes, and proves cancellation drops
  an aggregate containing both scalar and text-blob slots

Source `Task<Struct>` lowering is enabled for non-empty recursive structs whose
leaves are `bool`, `i32`, `i64`, `f32`, `f64`, `String`, or `Bytes`. The protocol encodes
the full tree as `Type{field:kind;nested:Nested{...}}`, while LLVM flattens its
leaves in declaration order into one self-describing slot allocation.
Type identity hashes the complete nested shape, and unpacking reconstructs
virtual nested field SSA only after the one-shot take. A `String` leaf copies
its UTF-8 bytes into a task-owned blob with a shape-derived GLM token; unpacking
re-interns those bytes before the aggregate drop hook releases the blob. Text
registration uses strict Rust-compatible UTF-8 validation, so malformed,
overlong, surrogate, truncated, and out-of-range sequences cannot become a
Nuis `String`; arbitrary bytes remain valid only as binary blobs. Both
direct `join` and
`join_result`/`task_value` consume the runtime-owned aggregate payload.
Recursive type cycles, empty structs, enums, pointers, and other task-owned
dynamic resources remain outside this layout contract. Source Nuis now exposes
`copy_bytes(ref Buffer) -> Bytes`; NIR/YIR lower it to
`cpu.copy_buffer_owned`, and interpreted execution proves the value remains
independent after source mutation or release. LLVM copies the buffer into a
GLM-tokened blob, transfers that blob into the aggregate slot, and takes it back
out before aggregate destruction. The remaining native closure is a typed
source operation for observing and deterministically dropping the extracted
`Bytes` value, followed by a runnable source-level task smoke. That explicit
closure now exists as `bytes_len(Bytes) -> i64` and `drop_bytes(Bytes)`; GLM
rejects use after drop, and the native recursive task smoke returns 24 after
copying three `i64` Buffer elements. Straight-line functions now synthesize
reverse-declaration-order cleanup at explicit returns and normal fallthrough,
after preserving return-expression evaluation in a compiler-owned temporary.
Explicit drops and ownership transfer through returned aggregates are not
duplicated. Path-sensitive `if` cleanup now releases branch-local values before
merge, accepts only equal live-owner state across two continuing paths, and
handles early returns through conditional YIR guard/branch drop-return
operations whose LLVM drops execute inside the selected basic block. `while`
functions are no longer rejected wholesale: ownership-neutral conditions and
bodies may carry outer `Bytes` unchanged across every backedge, after which
normal function cleanup releases them. Per-iteration owned locals are supported
for the first linear counted-loop shape. At NIR level, linear loop bodies
synthesize reverse-order
cleanup for per-iteration locals before fallthrough, direct `break`, and direct
`continue`, while requiring the surviving owner set to equal loop entry. The
generated edge cleanup passes GLM verification; resource-aware structured-loop
YIR/LLVM lowering now covers the first direct-break form through
`cpu.loop_owned_bytes_copy_drop_break`. LLVM evaluates the condition, copies the
source Buffer into a GLM-tokened blob inside the selected loop body, drops it,
and reaches the break exit. Iterative fallthrough and tail `continue` use the
extensible `cpu.loop_while_i64_effect` contract. Its action descriptor carries
module, instruction, arity, and operands; `cpu.owned_bytes_copy_drop` is the
first registered action. The generic counted-loop backend owns induction update,
condition re-evaluation, and the backedge, while the action backend owns only
per-iteration resource work. Native task coverage executes two iterations before
transferring the outer payload. A direct guarded `break` now uses
`cpu.loop_while_i64_effect_flow`: NIR duplicates cleanup onto the selected break
edge and natural backedge, YIR records the scalar control predicate separately
from the registered resource action, and LLVM stores the updated induction value
before cleanup and branch selection. The task returns that final value through
its owned aggregate, so native exit 26 proves the break occurs after iteration
two. The effect-flow payload now carries an explicit scalar-carry list. A
mid-body guarded `continue` stores the updated current value, releases the
iteration blob, and jumps directly to condition re-evaluation; the normal edge
applies `add_current` carries before releasing the same blob. Rebinding a fresh
same-name local also clears the prior GLM moved identity. Native payload fields
observe break iteration 2 and post-continue carry score 7. Multiple ordered
linear carries are also supported: `add_carryN` may reference an earlier carry's
new value from the same update edge, while forward references are rejected by
both compiler encoding and YIR validation. Native `weighted += score` therefore
produces 10 and raises the observable exit to 43. Effect-flow control metadata is
now length-delimited and accepts the shared recursive `and`/`or` condition tree
when every terminal selects the same action. LLVM evaluates that tree after the
induction update while retaining one registered cleanup on each final action or
update edge; the native exit 43 path now exercises a compound continue guard.
Effect-flow carry records now advance by the shared carry-kind payload arity
instead of a fixed pair width. `mul_current_plus_invariant` now composes the
updated induction value with its payload before multiplying the carry. The two
normal update edges use factors 4 and 5, produce 20, and raise the aggregate
native exit to 63; guarded continue still bypasses the carry update. Grouped
multi-state sources now use the shared `current/prev_current/carryN/prev_carryN`
resolver. `weighted += current + carry0` consumes the earlier same-edge score,
reaches 17, and raises native exit to 70. Scaled records now reuse the canonical
factor/source resolver. `scaled *= (current + 1) * 2` emits
`mul_scaled_current_plus_invariant` and reaches 80. The invariant-factor record
stores an already-scaled additive offset, so lowering must calculate
`terms * factor + scaled_offset`, not `(terms + scaled_offset) * factor`; the
native aggregate exit 130 locks that ABI boundary. State-factor carries are
additionally accepted as ordered edge dependencies: a source-to-YIR test
emits `mul_scaled_by_carry0_current_plus_invariant`, and LLVM resolves `carry0`
from the earlier same-edge new carry rather than its previous iteration value.
Multi-state factor groups are now shared with the async-post-flow carry grammar
instead of being copied into effect-flow lowering. The native grouped delta uses
two affine state groups,
reads the earlier same-edge score, contributes 55, and raises aggregate exit to
185. Mixed-action resource control trees now carry terminal-local actions and
lower through ordered short-circuit blocks; recursive cleanup rewriting emits
one drop on each break, continue, or normal update edge. Cleanup now also treats
nested loops as ownership scopes: inner edges release inner locals while outer
owners remain live. Registered `cpu.scoped_call` loop actions now outline a
scalar helper into a static function lane, pass the current iteration through
`$current`, and let that helper own and lower another resource loop without a
fixed rank-two opcode. A `ref Buffer` capture is one logical YIR parameter with
an explicit Lifetime edge and expands to LLVM `(ptr, len)`, so the helper can
perform length-aware Bytes copies while async task invokers remain closed to the
borrowed kind. Owned Bytes copy/move capture and ownership return across this
helper boundary remain future work.

Today `nuis` does **not** yet have:

* a mature parallel executor
* a stable worker-pool or lane scheduler contract for tasks
* a queue-backed timer wheel or mature delayed-work executor beyond the
  current deterministic task ready-tick model
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
* use `task_failed(...)` to distinguish owned helper materialization failure
  from timeout and cancellation
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
