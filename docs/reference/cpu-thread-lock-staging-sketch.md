# CPU Thread/Lock Staging Sketch

This document is the current staging sketch for one specific next-step question:

`if nuis grows from today’s cpu task contract toward real concurrency, how should thread encapsulation and lock semantics enter the system without breaking the current async/task honesty?`

It is intentionally a **staging sketch**. The repository now has staged
frontend/`NIR`/`YIR` families for `Thread<T>`, `Mutex<T>`, and
`MutexGuard<T>`, but this document still does **not** claim that the final
thread runtime or synchronization visibility contract is already stable.

## Why This Sketch Exists

Today the repository already has:

* a meaningful `cpu` task contract
* conservative task payload boundaries
* early `GLM` classification for task handles
* a verifier-backed local ownership model for `ref / borrow / move / free`

Today the repository also explicitly does **not** yet have:

* a mature parallel executor
* shared-memory synchronization primitives
* a final concurrent memory visibility contract
* a stable thread runtime

That means the next async step should **not** be:

* “just add threads”
* “just add a mutex builtin”

The safer next step is:

* make thread and lock shapes explicit as a new staged lane
* say how they differ from today’s `Task<T>` line
* say what they should probably be in frontend, `NIR`, `YIR`, and `GLM`
  before pretending they are ordinary values

## Current Working Split

The repository should currently be read with this three-way split:

### 1. `Task<T>`

Current meaning:

* typed async computation contract
* lifecycle observation through `join_result(...)`
* conservative payload boundary
* no promise of true worker parallelism

Short rule:

`Task<T>` is already a real async/task semantic line, but not yet a real thread runtime`

### 2. `Thread<T>` (staged family)

Current staged meaning:

* explicit thread/worker execution handle family
* distinct from `Task<T>` in frontend typing and lowering shape
* still awaiting final runtime-strength and visibility semantics

Likely long-term meaning:

* explicit thread/worker execution handle
* stronger join/ownership boundary than today’s task approximation
* likely closer to a real resource handle than current `Task<T>`

Short rule:

`Thread<T>` should eventually mean “real concurrent execution handle”, not “alternate spelling for Task<T>”`

### 3. `Mutex<T>` / `MutexGuard<T>` lock families (staged family)

Current staged meaning:

* explicit coordination resource family
* separate mutex object and acquired guard object
* staged lock/value/unlock frontend and lowering surface

Likely long-term meaning:

* explicit shared-state coordination object
* not a plain value
* not something that should silently cross task/thread boundaries as if it were
  just another payload field

Short rule:

`locks should enter the language as explicit coordination resources, not as accidental value wrappers`

## Staging Rule

The safest current staging order is:

```text
current Task<T> contract
-> explicit future thread-handle family
-> explicit future lock/coordination family
-> stronger GLM/resource classification
-> final memory visibility / synchronization semantics
```

This ordering matters because:

* tasks already have syntax, docs, examples, and verifier-visible observer roles
* threads should not retroactively change what current task code means
* locks should not arrive before the repository can explain who owns them, who
  can share them, and what `join`/visibility means

## Likely Semantic Direction

### Thread handles

Best current intuition:

* `Thread<T>` should be modeled more strongly than current `Task<T>`
* spawn/join on threads should probably be explicit consume/replace boundaries
* thread handles should likely become real resource-like objects in `GLM`

Why:

* they imply true concurrent execution rather than only task lifecycle
* they should eventually carry stronger visibility and ownership consequences

### Mutex / lock handles

Best current intuition:

* `Mutex<T>` should not be plain `val`
* `Mutex<T>` is a coordination bridge/resource
* lock acquisition should likely produce a temporary guard-like authority rather
  than direct unrestricted value duplication

Why:

* a lock is not only data, it is synchronization authority
* treating it like plain value data would erase exactly the semantics we need
  it to protect

## GLM Direction

Current best staged reading:

### `Task<T>`

* current approximation may stay smaller/earlier
* observation boundary remains centered on `join_result(...)`

### `Thread<T>`

* likely future `res`-style execution handle
* stronger consume/join semantics than task approximation

### `Mutex<T>` / lock family

* likely future bridge/resource object
* probably enters `GLM` before full visibility semantics are finalized
* may later split into:
  * lock object
  * acquired guard/lease object

Short rule:

`threads probably harden toward res-objects; locks probably begin as explicit bridge/resources and only later gain final guard semantics`

## Payload Boundary Rule

Current task payload rules are already conservative about:

* `ref ...`
* borrowed values
* resource-bearing families
* control-plane handles

That existing rule should be read as a strong hint for thread/lock work too:

* thread handles should not be normal payload scalars
* lock handles should not be normal payload scalars
* any future crossing of these families should go through an explicit handle
  contract, not through accidental allow-list expansion

## Suggested Implementation Order

If this line becomes active implementation work, the safest order is:

1. define frontend vocabulary and type families for thread/lock handles
2. define current reject/accept boundary before adding runtime claims
3. define `NIR`/`YIR` semantic roles for thread spawn/join and lock
   acquire/release
4. define `GLM` classification for thread handles and lock/guard objects
5. only then begin claiming stronger parallel/runtime semantics

## What Not To Do First

The repository should avoid starting with:

* pretending `Task<T>` is already the final thread abstraction
* allowing `ref`-like shared mutable payloads first and explaining ownership
  later
* making locks plain host handles with no frontend/type/GLM story
* introducing synchronization syntax before the ownership boundary is explicit

## Current Working Conclusion

The current best reading is:

* keep `Task<T>` as the current async/task contract
* keep thread handles as a separate stronger staged family
* keep locks as explicit staged coordination resources
* let both families continue through a staged handle/resource story before
  promising real concurrent visibility semantics

That keeps the repository honest:

* current async/task work remains real
* future thread/lock work has a clear lane
* the memory model is not forced to pretend it is already final

## Related References

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)
* [cpu-task-external-handle-glm-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-glm-sketch.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
