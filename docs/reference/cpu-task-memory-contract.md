# CPU Task Memory Contract

This document records the current ownership and memory boundary for `cpu` task
operations.

It is intentionally conservative.

Right now the goal is **not** to promise full concurrent memory semantics.
The goal is to make async/task code less likely to smuggle unsafe ownership
shapes across task boundaries before the memory model is mature enough.

## Scope

This document is about task inputs and task ownership around:

* `spawn(...)`
* `join(...)`
* `cancel(...)`
* `timeout(...)`
* `join_result(...)`

It focuses on:

* what may enter a task today
* what is currently rejected
* why that restriction exists

## Current Position

`nuis` already has meaningful async/task syntax and `YIR` task observation
semantics, but it does **not** yet have a finalized concurrent memory model.

That means task syntax should currently be read as:

* a typed async/task contract
* not a proof of safe shared-memory concurrency

## Current Spawn Input Boundary

The current frontend now rejects the most obviously dangerous task inputs:

* borrowed inputs such as `borrow(x)`
* values whose inferred type is `ref ...`

In other words, this is currently rejected:

```ns
let head: ref Node = alloc_node(1, null());
let head_ref: ref Node = borrow(head);
spawn(ping(head_ref));
```

And this is also rejected:

```ns
let head: ref Node = alloc_node(1, null());
spawn(ping(head));
```

The current error shape is intentionally explicit:

* borrowed task inputs are not allowed
* `ref` task inputs are not allowed

## Current Async Payload Boundary

The repository already has a second, broader async-boundary check:

* `async fn` parameters must be async-boundary-safe
* `async fn` return types must be async-boundary-safe

Today that means async boundaries reject types like:

* `ref ...`
* resource-bearing `Window<...>`, `WindowMut<...>`, and `Pipe<...>`
* control-plane `Marker<...>` and `HandleTable<...>`
* optional `?...`
* `Instance<...>`
* `Task<...>`
* result families such as `TaskResult<...>`, `DataResult<...>`,
  `ShaderResult<...>`, and `KernelResult<...>`

That rule now also applies recursively through nominal struct payloads.

So a payload like:

* `ScalarPacket { lhs: i64, rhs: i64 }`

is still allowed, but a payload like:

* `RefPacket { head: ref Node }`

is rejected even though the outer payload is a named struct, because the
frontend now inspects nested struct fields instead of trusting only the
top-level nominal wrapper.

Current mental model:

* scalar/value-like payloads are the safe side of the boundary for now
* pointer-like, nullable, staged-instance, nested-task, and result-family
  payloads are the unsafe side for now

Current positive examples:

* [hello_task_glm_scalar_payload.ns](../../examples/ns/memory/hello_task_glm_scalar_payload.ns)
* [hello_task_glm_struct_payload.ns](../../examples/ns/memory/hello_task_glm_struct_payload.ns)
* [hello_task_glm_nested_struct_payload.ns](../../examples/ns/memory/hello_task_glm_nested_struct_payload.ns)
* [hello_task_glm_text_payload.ns](../../examples/ns/memory/hello_task_glm_text_payload.ns)
* [hello_task_glm_nested_text_struct_payload.ns](../../examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns)

For the compact current-state view, also see:

* [cpu-task-payload-matrix.md](cpu-task-payload-matrix.md)

Useful concrete negative examples:

* [hello_task_glm_nested_ref_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_ref_struct_payload_invalid.ns)
* [hello_task_glm_nested_window_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_window_struct_payload_invalid.ns)
* [hello_task_glm_nested_marker_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_marker_struct_payload_invalid.ns)
* [hello_task_glm_optional_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_optional_payload_invalid.ns)
* [hello_task_glm_nested_optional_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_optional_struct_payload_invalid.ns)
* [hello_task_glm_instance_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_instance_payload_invalid.ns)
* [hello_task_glm_nested_instance_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_instance_struct_payload_invalid.ns)
* [hello_task_glm_result_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_result_payload_invalid.ns)
* [hello_task_glm_nested_result_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_result_struct_payload_invalid.ns)

This matters for task payloads because `spawn(...)` ultimately depends on the
callee's async boundary shape. Even before a real parallel runtime exists, the
language is already trying to keep task payloads on the safer side of the
boundary.

## Why This Restriction Exists

Before real concurrency arrives, the most dangerous ambiguity is:

* whether a task receives a copied value
* a moved value
* a shared reference
* or a still-live alias into mutable state

If the answer is not stable yet, allowing `ref` and borrowed values into
`spawn(...)` would make later thread or worker semantics much harder to trust.

So the current rule is simple:

* task inputs should stay on the safer value side for now
* pointer-like and borrowed inputs are blocked early

## What This Does Not Yet Solve

This guardrail is useful, but it is not the full memory model.

It does **not** yet fully answer questions like:

* whether moved heap-backed values are task-safe
* whether task payloads are copied or transferred
* how cancellation interacts with ownership
* what memory visibility `join(...)` implies
* how true parallel workers will interact with `ref/borrow/move/free`

So this is a **first barrier**, not the finished story.

That also means future thread/lock work should not loosen this barrier casually.
If concurrency grows beyond today’s task line, thread handles and lock handles
should enter through an explicit staged contract instead of by silently letting
shared-mutable shapes cross `spawn(...)`.

## Join and Observation Boundary

The current task result line remains:

* `join(Task<T>) -> T`
* `join_result(Task<T>) -> TaskResult<T>`
* `task_*` observers work from `TaskResult<T>`

That separation matters for memory reasoning too:

* `join(...)` is currently the direct payload extraction path
* `join_result(...)` is the observation path
* `task_value(...)` is only meaningful when the result is actually completed

This helps keep lifecycle observation separate from raw payload extraction.

Today that separation is guarded in two places:

* the frontend rejects obvious observer misuse on non-`TaskResult<...>` values
* `YIR` verifier rules keep `cpu.task_value` tied to a `cpu.join_result` source

## Current Guidance

If you want task code that fits the current system well:

* prefer plain value inputs to `spawn(...)`
* avoid `ref` and borrowed task parameters
* use `join_result(...)` when control flow depends on task state
* treat real shared-memory concurrency as not-yet-promised

## Relationship To Other References

Read this together with:

* [cpu-task-contract.md](cpu-task-contract.md)
* [cpu-task-external-handle-contract.md](cpu-task-external-handle-contract.md)
* [cpu-thread-lock-staging-sketch.md](cpu-thread-lock-staging-sketch.md)
* [nir-memory-model.md](nir-memory-model.md)
* [nir-optimization-contract.md](nir-optimization-contract.md)
