# CPU Thread/Lock Boundary

This file records the practical boundary for staged `Thread<T>`, `Mutex<T>`,
and `MutexGuard<T>` work that entered the mainline before `alpha-0.0.1`.

For present-tense `alpha-0.4.*` routing, read this as a still-relevant
contract note under the current system inventory, not as the repo entrypoint:

* [../versioning/nuis-alpha-0.4-system-inventory.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-system-inventory.md)

It is not the final thread runtime or memory-visibility design.

It is the shortest current answer to:

`what thread/lock shapes are already real, what shapes are still intentionally blocked, and which checked-in examples/tests prove that boundary today?`

## Short Rule

Read the current line this way:

* `Thread<T>` / `Mutex<T>` / `MutexGuard<T>` are real staged frontend,
  lowering, and verifier-visible families
* they already have compile-closure anchors and `GLM` ownership rules
* they still do **not** imply a finalized concurrent runtime or visibility
  contract

## What Is Real Today

### 1. Single-file source anchors now exist

Current source anchors:

* [hello_thread_mutex_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_observe.ns)
  straight-line staged thread/lock observation
* [hello_thread_mutex_branch_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_observe.ns)
  branch-selected guard/thread plus shared observer suffix
* [hello_thread_mutex_branch_suffix.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_suffix.ns)
  branch-selected guard/thread plus shared observer and shared pure suffix

Current source compile regression surface:

* [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)

Short rule:

* thread/lock work is no longer only a project-form demo lane
* there is now a small single-file compile-closure spine for it

### 2. Project-form anchor still exists

Current project anchor:

* [task_thread_mutex_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_thread_mutex_demo)

Short rule:

* use the project demo when you want generic helper/facade shape
* use the single-file `.ns` anchors when you want the shortest boundary

### 3. Shared observer control-flow paths are real

Today the checked-in control-flow boundary already accepts:

* branch-selected `MutexGuard<T>` followed by shared `mutex_value(...)`
* branch-selected `Thread<T>` followed by shared `thread_join_result(...)`
* shared `task_completed(...)` / `task_value(...)` observers after that
* small shared pure suffixes after those observer paths

Current lowering contract:

* [control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)

Current lowering regression surface:

* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)

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
* `task_value(...)` on thread-produced `TaskResult<T>` still requires a
  completed path, just like ordinary task results

Current verifier regression surface:

* [glm_verify.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/glm_verify.rs)

Important checked-in examples in that file:

* completed-branch thread result reads are accepted
* else/timed-out/cancelled task-value misuse is rejected
* post-unlock guard reuse is rejected
* branch-local guard reads may remain legal when no consuming step happened

## What Is Still Intentionally Blocked

### Branch-local consuming thread/lock runtime work inside `if` / lowered `match`

Current rejection rule:

* the branch itself still may not hide deeper consuming task/thread/mutex work
  as an arbitrary branch-local mini-program
* current invalid anchors intentionally place the consuming step and its
  observer chain inside each branch

Current invalid anchors:

* [hello_thread_mutex_if_lock_branch_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_thread_mutex_if_lock_branch_invalid.ns)
* [hello_thread_mutex_match_join_result_branch_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_thread_mutex_match_join_result_branch_invalid.ns)

Current diagnostic contract:

* `conditional if/lowered-match lowering does not yet support branch-local consuming task/thread/mutex runtime primitives`
* `hoist those effects before the branch or reduce each branch to pure/select-compatible values`

Short rule:

* selecting a handle/guard before a shared observer suffix is supported
* burying the consuming runtime effect chain separately inside each branch is
  still intentionally rejected

### Final concurrent visibility claims

Still not promised today:

* a mature worker runtime
* a final memory visibility model
* a finished synchronization contract
* a claim that thread/lock families are already semantically complete

For that broader positioning, read:

* [cpu-thread-lock-staging-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-staging-sketch.md)

## Practical Reading Rule Before `alpha-0.0.1`

Before `alpha`, read the thread/lock line in this order:

1. source anchors in [examples/ns/memory/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
2. invalid anchors in [examples/invalid/ns/memory/README.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/README.md)
3. ownership/lifecycle truth in [glm_verify.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/glm_verify.rs)
4. larger staging intent in [cpu-thread-lock-staging-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-staging-sketch.md)

That keeps the line honest:

* positive examples show what compiles
* negative examples show what is still blocked
* verifier tests show what the ownership story actually means

## Why This Matters For `alpha`

For the `0.20.* -> alpha-0.0.1` handoff, this lane is now strong enough to
say:

* thread/lock syntax is not just aspirational
* compile-closure anchors exist
* invalid boundary anchors exist
* `GLM` ownership truth exists

But it is **not** yet strong enough to say:

* the concurrent runtime model is final
* the visibility/synchronization story is complete

That is the right pre-`alpha` posture:

* explicit enough to build on
* still honest about what remains staged
