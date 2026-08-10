# CPU Thread/Lock Boundary

This file records the practical boundary for staged `Thread<T>`, `Mutex<T>`,
`MutexGuard<T>`, `SharedMutex<T>`, `MutexPermit<T>`, and `MutexLease<T>` work.

For hardening-baseline routing from the `alpha-0.4.*` line, read this as a
still-relevant contract note under the current system inventory, not as the
repo entrypoint:

* [../versioning/nuis-alpha-0.4-system-inventory.md](../../docs/versioning/nuis-alpha-0.4-system-inventory.md)

It is not the final thread runtime or memory-visibility design.

It is the shortest current answer to:

`what thread/lock shapes are already real, what shapes are still intentionally blocked, and which checked-in examples/tests prove that boundary today?`

## Short Rule

Read the current line this way:

* `Thread<T>` / `Mutex<T>` / `MutexGuard<T>` are real staged frontend,
  lowering, and verifier-visible families
* they already have compile-closure anchors and `GLM` ownership rules
* `Mutex<i64>` now reaches a cooperative scheduler handle/guard runtime with
  explicit acquire/release epoch evidence
* `SharedMutex<i64>` can authorize a statically declared `1..=64` task lanes
  through distinct, non-cloneable `MutexPermit<i64>` values; the one-argument
  `mutex_share(lock)` form remains a two-lane shorthand
* each worker consumes its permit into one `MutexLease<i64>` without receiving
  the raw scheduler handle, and a live lease can replace the central `i64`
  payload while retaining authority
* this foothold still does **not** imply a parallel executor, runtime-dynamic
  permit cardinality, generalized payloads, or a finalized visibility contract

## What Is Real Today

### 1. Single-file source anchors now exist

Current source anchors:

* [hello_thread_mutex_observe.ns](../../examples/ns/memory/hello_thread_mutex_observe.ns)
  straight-line staged thread/lock observation
* [hello_thread_mutex_branch_observe.ns](../../examples/ns/memory/hello_thread_mutex_branch_observe.ns)
  branch-selected guard/thread plus shared observer suffix
* [hello_thread_mutex_branch_suffix.ns](../../examples/ns/memory/hello_thread_mutex_branch_suffix.ns)
  branch-selected guard/thread plus shared observer and shared pure suffix
* [hello_thread_mutex_if_lock_branch.ns](../../examples/ns/memory/hello_thread_mutex_if_lock_branch.ns)
  branch-local mutex-lock syntax converged into one selected lock operation
* [hello_thread_mutex_match_join_result_branch.ns](../../examples/ns/memory/hello_thread_mutex_match_join_result_branch.ns)
  branch-local thread spawn/join-result syntax converged into one selected chain

Current source compile regression surface:

* [memory_compile.rs](../../tools/nuisc/tests/memory_compile.rs)

Short rule:

* thread/lock work is no longer only a project-form demo lane
* there is now a small single-file compile-closure spine for it

### 2. Project-form anchor still exists

Current project anchor:

* [task_thread_mutex_demo](../../examples/projects/task/task_thread_mutex_demo)
* [task_shared_mutex_permit_demo](../../examples/projects/task/task_shared_mutex_permit_demo)
* [task_shared_mutex_cardinality_demo](../../examples/projects/task/task_shared_mutex_cardinality_demo)
* [task_shared_mutex_replace_demo](../../examples/projects/task/task_shared_mutex_replace_demo)
* [task_branch_cancel_unlock_demo](../../examples/projects/task/task_branch_cancel_unlock_demo)

Short rule:

* use the project demo when you want generic helper/facade shape
* use the shared permit demo when you want the shortest native default-two-lane
  authorization path without copied mutex handles
* use the cardinality demo when you want three statically declared task lanes
* use the replace demo when you want release-published lease mutation observed
  by a permit issued before that mutation
* use the branch demo when you want native dynamic cancel/unlock closure with
  both source branches executed
* use the single-file `.ns` anchors when you want the shortest boundary

### 3. Shared observer control-flow paths are real

Today the checked-in control-flow boundary already accepts:

* branch-selected `MutexGuard<T>` followed by shared `mutex_value(...)`
* branch-selected `Thread<T>` followed by shared `thread_join_result(...)`
* shared `task_completed(...)` / `task_value(...)` observers after that
* small shared pure suffixes after those observer paths

Current lowering contract:

* [control-flow-lowering-contract.md](control-flow-lowering-contract.md)

Current lowering regression surface:

* [tests_branch_helpers.rs](../../tools/nuisc/src/lowering/tests_branch_helpers.rs)

Short rule:

* branch-local observer-safe thread/lock paths are part of the current mainline
* the branch can choose the handle/guard first, then a shared observer/pure
  suffix can continue

### 4. `GLM` ownership/lifecycle rules are real

Current `GLM`/verifier truth already includes:

* `thread_join_result(...)` consumes the thread handle
* `mutex_lock(...)` consumes the mutex handle and produces guard authority
* `mutex_unlock(...)` consumes the guard
* `mutex_value(...)` is a read, not a consume
* `mutex_share(...)` consumes the original mutex, reads its static cardinality,
  and produces shared authority
* `mutex_shared_close(...)` consumes shared authority, revokes pending permits,
  and returns the revoked permit count
* `mutex_permit(...)` reads shared authority and issues one lane-bound permit
* `mutex_permit_lock(...)` consumes a permit and produces lease authority
* `mutex_lease_value(...)` reads a live lease
* `mutex_lease_replace(...)` writes through a live lease without consuming it
* `mutex_lease_unlock(...)` consumes the lease without returning a raw handle
* `task_value(...)` on thread-produced `TaskResult<T>` still requires a
  completed path, just like ordinary task results

Current verifier regression surface:

* [glm_verify.rs](../../tools/nuisc/tests/glm_verify.rs)

Important checked-in examples in that file:

* completed-branch thread result reads are accepted
* else/timed-out/cancelled task-value misuse is rejected
* post-unlock guard reuse is rejected
* branch-local guard reads may remain legal when no consuming step happened

### 5. The first scheduler mutex runtime is real

The current native `Mutex<i64>` lane now lowers through opaque runtime
identities rather than LLVM-only value wrappers:

* `mutex_new` allocates a monotonic scheduler handle
* `mutex_lock` consumes that handle and yields a one-shot guard token
* each guard records the cooperative scheduler worker identity and the release
  epoch observed by acquisition
* `mutex_value` requires a live, generation-matching locked guard
* `mutex_unlock` performs a release fence, advances the visibility epoch,
  invalidates the guard, and returns the original handle
* contention, repeated unlock, forged tokens, stale generations, and exhausted
  tables fail closed
* lifecycle shutdown invalidates all live mutex and guard slots

Generated YIR records this boundary on every mutex node:

```text
mutex_contract=scheduler-handle-v1
visibility=acquire-release-epoch-v1
authority=linear-guard-v1
payload_policy=i64-native-staged-fallback
```

The CPU domain accepts the full ordered contract, rejects metadata drift, and
keeps only the first argument as the data dependency. Legacy single-input YIR
remains readable during this transition.

Current proof surfaces:

* [shim_mutex.rs](../../tools/nuisc/src/aot_tests/shim_mutex.rs) runs two
  cooperative workers through deterministic contention, release publication,
  reacquisition, token replay rejection, and lifecycle cleanup
* [concurrency_branch_native_smoke.rs](../../tools/nuis/tests/concurrency_branch_native_smoke.rs)
  proves generated Nuis YIR carries the metadata and native LLVM calls the
  scheduler mutex ABI while both dynamic source branches still execute

Honesty boundary:

* only the current `i64` payload lane is runtime-backed
* other typed mutex payloads retain the staged typed wrapper fallback
* the contention harness exercises scheduler workers at the runtime boundary;
  Nuis source still cannot duplicate or send one mutex authority into multiple
  workers
* this runtime is cooperative and does not yet claim OS-thread parallel safety

### 6. Shared permit/lease authority reaches native tasks

The source-level shared-mutex path now closes over a statically declared permit
cardinality:

```text
Mutex<i64>
  -> mutex_share(lock, N), where N is a literal in 1..=64
  -> SharedMutex<i64>
  -> MutexPermit<i64> for one lane in 0..N
  -> task boundary as an opaque scheduler scalar
  -> MutexLease<i64>
  -> value read or release-published replacement
  -> lease unlock
  -> shared close
```

`mutex_share(lock)` is normalized by the compiler to
`mutex_share(lock, 2)`. The two-argument form accepts only a literal cardinality
in `1..=64`. `mutex_permit(shared, lane)` likewise requires a literal lane in
`0..=63`; the interpreter and native runtime then enforce the exact relation
`lane < N` from the cardinality stored at share time.

The runtime keeps the mutex handle inside its permit table. A permit carries
only an unforgeable token, is bound to the mutex generation and one fixed lane,
and is consumed before locking. Duplicate lanes, lanes outside the declared
cardinality, stale generations, and token replay fail closed. Nuis only accepts
a freshly issued inline permit at `spawn`/`thread_spawn`; a stored permit
variable cannot escape across that boundary.

Every shared capability YIR node carries:

```text
mutex_contract=scheduler-handle-v1
visibility=acquire-release-epoch-v1
authority=linear-permit-lease-v1
permit_cardinality=share-literal-1-to-64-v1
permit_policy=one-shot-generation-bound-v1
payload_policy=i64-native-staged-fallback
lifecycle=explicit-close-revoke-v1
mutation=lease-replace-release-epoch-v1
```

Three native project smokes pin the current boundary:

* `task_shared_mutex_permit_demo` uses the default cardinality, sends two
  permits through two scheduler task invocations, and exits `34`
* `task_shared_mutex_cardinality_demo` declares cardinality `3`, admits lanes
  `0`, `1`, and `2`, sends all three permits through tasks, and exits `33`
* `task_shared_mutex_replace_demo` replaces `17` with `23` through one lease,
  exposes the replacement to a pre-issued second permit, and exits `65`

Close performs a release fence, rejects an active lease, revokes every pending
same-generation permit, invalidates the mutex slot, and returns the revocation
count. Lease replacement updates one central shared value and publishes a
release epoch without consuming the lease. The runtime harness separately
proves duplicate/out-of-range lane rejection, one-shot consumption, epoch
progression, active-lease close rejection, post-close token failure, and
lifecycle cleanup. The CPU interpreter tracks the same cardinality, close,
permit, lease, and mutation state rather than treating them as native-only
effects.

This is shared authorization, not copied ownership: ordinary `Mutex<T>` and
`MutexGuard<T>` retain their existing linear behavior.

## What Is Still Intentionally Blocked

### Arbitrary branch-local runtime mini-programs

Current boundary:

* matching branch-local lock or spawn/join-result prefixes are now supported
* lowering selects the resource or call arguments first, emits the common
  consuming operation once, and then lowers one shared or selectable suffix
* differently shaped operations, unrelated callees, or non-collapsible effect
  suffixes remain outside this route

Short rule:

* source syntax may place the matching operation inside each branch
* YIR still receives one post-selection consuming operation rather than eager
  execution of both source branches
* matching consuming prefixes may form a multi-stage chain; the native branch
  demo proves one `mutex_new -> lock -> unlock` chain and one
  `spawn -> cancel -> join_result` chain

### Final concurrent visibility claims

Still not promised today:

* a mature worker runtime
* an OS-thread-parallel mutex implementation
* runtime-dynamic permit cardinality
* generalized non-`i64` shared payloads and replacement
* branch-local shared-capability selected-prefix lowering
* a final memory visibility model beyond the current acquire/release epoch
  protocol
* a finished synchronization contract
* a claim that thread/lock families are already semantically complete

For that broader positioning, read:

* [cpu-thread-lock-staging-sketch.md](cpu-thread-lock-staging-sketch.md)

## Practical Reading Rule

Read the thread/lock line in this order:

1. source anchors in [examples/ns/memory/README.md](../../examples/ns/memory/README.md)
2. lowering regressions in [thread_mutex_branch_compile.rs](../../tools/nuisc/tests/thread_mutex_branch_compile.rs)
3. shared authority closure in [concurrency_branch_native_smoke.rs](../../tools/nuis/tests/concurrency_branch_native_smoke.rs)
4. ownership/lifecycle truth in [glm_verify.rs](../../tools/nuisc/tests/glm_verify.rs)
5. larger staging intent in [cpu-thread-lock-staging-sketch.md](cpu-thread-lock-staging-sketch.md)

That keeps the line honest:

* positive examples show what compiles
* lowering regressions prove exactly-once selected-prefix behavior
* verifier tests show what the ownership story actually means

## Why This Matters For Beta Hardening

For the current beta foundation line, this lane is now strong enough to say:

* thread/lock syntax is not just aspirational
* compile-closure anchors exist
* structured branch-local consuming prefixes have exactly-once YIR regressions
* `GLM` ownership truth exists
* `Mutex<i64>` reaches a scheduler-backed native runtime with auditable
  visibility metadata
* statically sized shared permits cross native Nuis task boundaries without raw
  handle copies, including a three-lane native proof
* lease replacement and explicit close/revocation share one interpreter/native
  contract

But it is **not** yet strong enough to say:

* the concurrent runtime model is final
* shared mutex authority is runtime-dynamic or OS-thread parallel
* the visibility/synchronization story is complete

That is the right beta-hardening posture:

* explicit enough to build on
* still honest about what remains staged
