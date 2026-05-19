# YIR Hot Sync Contraction Sketch

This document is a forward-looking sketch for one specific `nuis` direction:

* `nuis` is async-first by default
* but some local hot paths should later be allowed to contract from explicit
  async state-machine shape into a provably equivalent sync shape

This is not a current implementation claim.

## Why This Matters

The repository’s current direction makes more and more semantics explicit:

* task/state boundaries
* observer roles
* timeout/cancel paths
* lane/scheduler hints
* clock-domain bridges

That explicitness is good for correctness and hetero visibility.
But it also means some local paths may eventually carry more async machinery
than they need once the compiler can prove the surrounding region is “hot and
closed.”

So the long-term goal is not:

* “stop modeling async”

It is:

* “model async by default, then safely contract some verified local regions
  into sync form”

## Intended Meaning

Current sketch term:

* **hot sync contraction**

Current intended reading:

* a verified local transformation
* starting from explicit async/task/state shape
* lowering to a semantically equivalent sync region
* while preserving the observable contract that still matters outside that
  region

This is closer to:

* task/state inlining
* local scheduler erasure
* local observer elimination

than to a simple function-inline optimization.

## What It Would Remove

If a region qualifies, future contraction might erase or simplify some
combination of:

* local `Task<T>` handles
* local `TaskResult<T>` wrappers
* local `join_result(...)` observation scaffolding
* local timeout bookkeeping
* local lane/scheduler indirection
* local clock bridge overhead

The exact amount of erasure should be proof-driven, not heuristic-only.

## What It Must Not Pretend

This sketch does **not** say that a region may be contracted whenever it is
small.

The repository’s async-first model means contraction should only happen when
the compiler can prove that the region does not rely on still-visible async
semantics such as:

* task observation from outside the region
* cancellation-sensitive behavior
* timeout-sensitive behavior
* cross-lane scheduling differences
* cross-domain timing assumptions
* ownership/visibility behavior that still needs explicit task structure

## Minimum Proof Conditions

The current likely proof direction is that a future contraction pass would need
to establish a conjunction of conditions like:

* **closed ownership region**
  * task payloads and values do not escape into externally observed task state
* **no required external observer**
  * no later path depends on retaining `join_result(...)`-style state rather
    than a direct value/result
* **no cancellation-sensitive branch**
  * removing the async envelope would not hide meaningful `cancel(...)`
    behavior
* **no timeout-sensitive branch**
  * removing the async envelope would not change the interpretation of
    `timeout(...)`
* **no required cross-lane distinction**
  * the region does not depend on different scheduler-lane behavior
* **no required cross-domain timing distinction**
  * the region does not rely on timing differences that still require explicit
    bridge semantics

These names are sketches, not frozen verifier rules yet.

## Why This Is Harder Than Ordinary Inlining

Ordinary inlining usually reasons about:

* call boundaries
* expression structure
* local value substitution

Hot sync contraction must additionally reason about:

* task lifecycle
* observer roles
* timeout/cancel meaning
* lane placement
* clock-domain interpretation
* visible side effects

So the safe reading is:

* this is a semantic contraction problem, not just a code-motion problem

## Relationship To YIR

This sketch belongs at `YIR` level because `YIR` is where the repository is
trying to keep the async/hetero contract visible.

That means future contraction should likely consume signals from:

* task operation families
* `GLM` lifetime/ownership structure
* scheduler/lane metadata
* clock/bridge metadata
* effect ordering

In other words:

* the frontend may still be async-first
* `YIR` may still preserve explicit async semantics
* contraction would happen only after enough `YIR`-level analysis has made it
  safe

## Relationship To Performance

The motivation is performance, but performance is not the contract.

The contract is:

* preserve async-first semantics by default
* remove local async overhead only when the compiler can prove that the
  preserved visible contract is still correct

That is why the repository should prefer:

* verified contraction

over:

* speculative de-asyncification

## Current Working Direction

The most likely healthy future order is:

1. strengthen task/`GLM`/clock contracts
2. strengthen lane/scheduler and observer analysis
3. define a narrow first contraction region
4. validate that region on hotspot examples
5. only then generalize

So this sketch is intentionally early and narrow.

## Related References

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
