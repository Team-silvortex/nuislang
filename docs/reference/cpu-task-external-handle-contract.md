# CPU Task External Handle Contract

This document records a current design direction for task payloads that are
*not* good candidates for plain value semantics.

It does **not** mean those payload families are already allowed across
`spawn(...)`.

Instead, it gives the repository a clearer intermediate mental model:

* some payloads behave like plain values
* some payloads behave more like external/resource handles
* if those handle-shaped payloads ever cross the task boundary later, they
  should probably do so through an explicit external-handle contract rather
  than by being silently treated as ordinary values

## Why This Document Exists

The repository now has a sharper split between:

* value-like task payloads that are currently allowed
* payloads that are currently rejected because they carry resource, control
  plane, or domain-instance meaning

That is already visible in:

* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)

But those documents are mostly about the **current allow/reject line**.

This file adds one more question:

* if some rejected families later become task-crossing candidates, what shape
  should they probably take?

## Current Split

### Plain Value Side

These are the families currently treated as the safer side of the task
boundary:

* scalars like `i64`, `bool`
* plain `String` / text-like values
* structs composed only from value-like fields

These already have positive examples such as:

* [hello_task_glm_scalar_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_scalar_payload.ns)
* [hello_task_glm_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_struct_payload.ns)
* [hello_task_glm_nested_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_struct_payload.ns)
* [hello_task_glm_text_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_text_payload.ns)
* [hello_task_glm_nested_text_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns)

### External / Resource-Bearing Side

These are currently rejected:

* `Window<...>` / `WindowMut<...>`
* `Pipe<...>`
* `Marker<...>`
* `HandleTable<...>`
* `Instance<...>`

These families already carry more meaning than “just another value”:

* fabric/resource routing
* control-plane identity
* staged domain ownership
* host/runtime bridge state

That is why the current frontend now rejects them not only directly, but also
when they are nested inside nominal struct payloads.

Useful current negative examples:

* [hello_task_glm_nested_window_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_window_struct_payload_invalid.ns)
* [hello_task_glm_nested_marker_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_marker_struct_payload_invalid.ns)
* [hello_task_glm_instance_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_instance_payload_invalid.ns)
* [hello_task_glm_nested_instance_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_instance_struct_payload_invalid.ns)

There is also a first explicit design probe:

* [hello_task_glm_window_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_window_external_handle_probe_invalid.ns)
* [hello_task_glm_window_external_handle_probe_invalid.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_window_external_handle_probe_invalid.md)
* [hello_task_glm_marker_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_marker_external_handle_probe_invalid.ns)
* [hello_task_glm_marker_external_handle_probe_invalid.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_marker_external_handle_probe_invalid.md)
* [hello_task_glm_handle_table_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_handle_table_external_handle_probe_invalid.ns)
* [hello_task_glm_handle_table_external_handle_probe_invalid.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_handle_table_external_handle_probe_invalid.md)

These probes are intentionally still invalid today, but their field layouts are
meant to hint at possible future task-external handle packet directions for:

* data/fabric windows
* control-plane markers
* control/routing handle tables

## Current Reading Rule

Right now, the safe interpretation is:

* value-like payloads may cross `spawn(...)`
* resource-bearing families may **not**

And the reason is not “they are permanently forbidden”.

The reason is:

* the repository does not yet have a final concurrent memory model
* these families look more like external handles than copied values
* treating them as plain values too early would make later `GLM`,
  ownership-transfer, and visibility rules much harder to trust

## If They Ever Become Allowed Later

If the repository later decides that some of these families should cross the
task boundary, the safest direction is probably **not**:

* “just allow them as normal payloads”

The safer direction is more like:

* define a task-external handle contract
* say whether the handle is:
  * copied
  * shared read-only
  * transferred
  * pinned to an external scheduler/lane/domain
* define what `join(...)`, `cancel(...)`, and `timeout(...)` mean for that
  handle family
* define whether `GLM` should treat the crossing as:
  * plain `val`
  * external `res`
  * ownership bridge
  * visibility bridge

In other words, these families should probably enter task semantics through a
more explicit “external handle” path, not through accidental plain-value
promotion.

## Candidate Families For Future External-Handle Work

The most likely future candidates are:

* `Window<...>` / `WindowMut<...>`
* `Pipe<...>`
* `Marker<...>`
* `HandleTable<...>`

These are all already meaningful in the broader hetero/data/runtime story.

If any of them later cross `spawn(...)`, they should probably be documented as:

* task-external data/fabric handles
* control-plane handles
* bridge handles

rather than being silently folded into the same bucket as scalars and strings.

## What This Does Not Claim

This document does **not** claim that:

* external-handle task payloads are already supported
* `GLM` already knows how to model them
* the runtime already has correct cross-task semantics for them

It only claims that the repository now has a better place to put them
conceptually when that work starts.

## Relationship To Other References

Read this together with:

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
* [cpu-task-external-handle-glm-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-glm-sketch.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
