# CPU Task Payload Matrix

This document is a short companion to the broader task references.

Its job is simple:

* show which task payload families are currently allowed
* show which are currently rejected
* point at concrete examples

It is intentionally a **current-state matrix**, not a final concurrency design.

## Current Reading Rule

Right now the repository treats task payloads conservatively.

The practical split is:

* value-like payloads are the safer side of the current async/task boundary
* pointer-like, nullable, staged-instance, nested-task, and result-family
  payloads are the unsafe side for now

That split exists because the repository still does **not** yet have a final
concurrent memory model.

## Current Matrix

| Payload family | Current status | Notes | Example |
| --- | --- | --- | --- |
| `i64`, `bool`, other small scalar values | Allowed | Small value-like task payloads are the current safest path. | [hello_task_glm_scalar_payload.ns](../../examples/ns/memory/hello_task_glm_scalar_payload.ns) |
| Small structs made only from value-like fields | Allowed | Useful current shape when the task input needs a little structure without introducing aliasing. | [hello_task_glm_struct_payload.ns](../../examples/ns/memory/hello_task_glm_struct_payload.ns) |
| Nested structs made only from value-like fields | Allowed | The recursive async/task boundary is not a blanket ban on nominal wrappers; it still accepts nested value-only payload trees. | [hello_task_glm_nested_struct_payload.ns](../../examples/ns/memory/hello_task_glm_nested_struct_payload.ns) |
| Resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>` payloads | Rejected | Fabric/resource carriers are not currently treated as plain task values. | [hello_task_glm_nested_window_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_window_struct_payload_invalid.ns) |
| Control-plane `Marker<...>` / `HandleTable<...>` payloads | Rejected | Control-plane carriers are also kept off the current async/task payload path. | [hello_task_glm_nested_marker_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_marker_struct_payload_invalid.ns) |
| Nominal structs whose nested fields contain `ref`, `?`, `Instance<...>`, `Task<...>`, or `*Result<...>` | Rejected | The current async/task boundary now recursively inspects named struct fields instead of trusting only the outer nominal type. | [hello_task_glm_nested_ref_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_ref_struct_payload_invalid.ns) |
| Plain text / `String`-like value payloads | Allowed | Currently treated on the safe/value-like side of the boundary. | [hello_task_glm_text_payload.ns](../../examples/ns/memory/hello_task_glm_text_payload.ns) |
| Nested structs with plain text / value-like fields | Allowed | Safe text/value payloads remain allowed through nested named wrappers. | [hello_task_glm_nested_text_struct_payload.ns](../../examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns) |
| `ref ...` | Rejected | Too close to shared aliasing before the memory model is stronger. | [hello_task_glm_ref_spawn_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_ref_spawn_invalid.ns) |
| `borrow(...)` task input | Rejected | Explicit borrowed task input is blocked at the spawn boundary. | [hello_task_glm_borrowed_spawn_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_borrowed_spawn_invalid.ns) |
| Optional `?...` payloads | Rejected | Nullable payload identity and lifecycle are still treated as unsafe for task crossing. | [hello_task_glm_optional_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_optional_payload_invalid.ns) |
| Nominal structs with nested optional `?...` fields | Rejected | The same optional restriction applies recursively inside named struct payloads. | [hello_task_glm_nested_optional_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_optional_struct_payload_invalid.ns) |
| `Instance<...>` | Rejected | Staged domain-instance handles are not yet allowed across the current async/task boundary. | [hello_task_glm_instance_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_instance_payload_invalid.ns) |
| Nominal structs with nested `Instance<...>` fields | Rejected | Named wrappers do not currently sanitize staged instance handles. | [hello_task_glm_nested_instance_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_instance_struct_payload_invalid.ns) |
| `Task<...>` | Rejected | Nested task payloads are intentionally blocked for now. | See [cpu-task-memory-contract.md](cpu-task-memory-contract.md) |
| `TaskResult<...>` and other `*Result<...>` families | Rejected | Result-family payloads stay on the observation side, not the task-input side. | [hello_task_glm_result_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_result_payload_invalid.ns) |
| Nominal structs with nested `*Result<...>` fields | Rejected | Result-family payloads also stay out of named wrapper payloads for now. | [hello_task_glm_nested_result_struct_payload_invalid.ns](../../examples/invalid/ns/memory/hello_task_glm_nested_result_struct_payload_invalid.ns) |

## Future Watch List

The following families are not good candidates for casual loosening. They are
the ones most likely to change only after `GLM`, task ownership, and concurrent
memory semantics become much sharper:

* `ref ...`
* borrowed task inputs
* resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>` payloads
* control-plane `Marker<...>` / `HandleTable<...>` payloads
* optional payloads
* `Instance<...>`
* `Task<...>`
* `*Result<...>` families

If any of these are reconsidered later, update the matrix together with:

* the task-memory contract
* the task-GLM contract
* the positive/negative example set

## Related References

* [cpu-task-contract.md](cpu-task-contract.md)
* [cpu-task-memory-contract.md](cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](cpu-task-glm-contract.md)
* [cpu-task-external-handle-contract.md](cpu-task-external-handle-contract.md)
