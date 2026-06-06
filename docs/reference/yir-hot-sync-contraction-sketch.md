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

## Current Disqualifiers

Before the repository has a stronger verifier/runtime story, it is useful to
state the opposite side just as clearly:

*what should immediately disqualify a region from hot sync contraction?*

The current conservative answer is:

* if any of the following are true, the region should be treated as
  **non-contractible by default**

### 1. Borrowed or `ref`-shaped task inputs

If a region depends on task inputs that are:

* borrowed
* `ref`-typed
* alias-sensitive through nested payload shape

then the region should not be contracted.

Why:

* the repository explicitly does not yet claim a finished concurrent memory
  model for these shapes

### 2. Resource-bearing or external-handle payload families

If a region depends on payloads or nested payload fields from families like:

* `Window<...>` / `WindowMut<...>`
* `Pipe<...>`
* `Marker<...>`
* `HandleTable<...>`
* `Instance<...>`

then the region should not be contracted.

Why:

* those are exactly the families that the repository is currently treating as
  future external-handle or bridge-object design space, not settled value-like
  task payloads

### 3. Lifecycle-sensitive branches

If local behavior changes based on:

* `task_timed_out(...)`
* `task_cancelled(...)`
* timeout-specific branch structure
* cancellation-specific branch structure

then the region should not be contracted.

Why:

* timeout/cancel semantics are precisely the area where future task lifetime
  rules are still intentionally conservative

### 4. Regions that require later observation of the same task

If a region extracts a direct value or otherwise simplifies task state in a way
that would interfere with later observation of the same task handle, the region
should not be contracted.

This includes shapes that are currently legal only because `join(...)` is not
yet a final consume boundary.

Why:

* the repository already treats those shapes as future-tightening probes, not
  stable optimization targets

### 5. Explicit clock-bridge dependence

If a region depends on:

* `clock_domain`
* `clock_policy="bridge"`
* resolved bridge interpretation
* declared/resolved clock-domain distinction
* explicit global/local clock comparison intent

then the region should not be contracted.

Why:

* timing-sensitive contraction should wait until the global/local clock
  negotiation story is much stronger

### 6. Required cross-lane or cross-domain distinction

If the region’s meaning depends on:

* scheduler-lane placement
* lane-specific sequencing
* cross-domain timing/visibility differences
* explicit `xfer`/bridge semantics that are still semantically important

then the region should not be contracted.

Why:

* erasing local async machinery in these cases risks erasing exactly the
  hetero distinctions the repository is trying to preserve

## Safe Default Rule

Until stronger proof machinery exists, a healthy default is:

* value-like, observer-local, completed-only regions may become probe
  candidates
* everything resource-bearing, timeout-sensitive, cancel-sensitive,
  bridge-sensitive, or alias-sensitive should remain explicitly async

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

## Current Probe Candidates

The repository already has several samples that are good future probe
candidates for hot sync contraction work.

### Best Near-Term Candidates

These are good because they already expose task/lifecycle structure clearly,
while still staying relatively local:

* [hello_task_glm_origin.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_origin.ns)
  * smallest `spawn -> join` payload path
* [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
  * smallest `spawn -> timeout -> join_result -> task_completed -> task_value`
    observation path
* [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
  * direct-payload path beside observation path
* [hello_task_glm_lifecycle_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_compare.ns)
  * completed/timeout/cancel observation contrast

### Best Project-Shaped Candidates

These are especially useful because they already sit in a more realistic
project/front-door compilation shape:

* [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo)
  * best current small project candidate for “can a completed-observe path
    collapse locally?”
  * see also:
    [examples/projects/task/task_completed_observe_demo/FUTURE_HOT_SYNC_CONTRACTION_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo/FUTURE_HOT_SYNC_CONTRACTION_SKETCH.md)
* [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo)
  * useful for testing that timeout-sensitive paths do **not** contract too
    aggressively
* [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo)
  * useful for testing that cancellation-sensitive paths do **not** contract
    too aggressively
* [task_join_nonconsuming_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_join_nonconsuming_probe_demo)
  * useful current negative probe when local contraction interacts with the
    now-stricter `join(...)` consume boundary

### What To Avoid As First Probes

The following are currently better treated as boundary/negative samples, not as
first contraction targets:

* borrowed/ref payload invalids
* resource-bearing external-handle probes
* samples whose main purpose is `GLM` boundary conflict rather than local
  async-state simplification

That is because the first contraction work should stay on value-like task
regions before it touches resource-bearing or future-concurrency-sensitive
families.

## Related References

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
