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
| `i64`, `bool`, other small scalar values | Allowed | Small value-like task payloads are the current safest path. | [hello_task_glm_scalar_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_scalar_payload.ns) |
| Small structs made only from value-like fields | Allowed | Useful current shape when the task input needs a little structure without introducing aliasing. | [hello_task_glm_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_struct_payload.ns) |
| Plain text / `String`-like value payloads | Allowed | Currently treated on the safe/value-like side of the boundary. | [hello_task_glm_text_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_text_payload.ns) |
| `ref ...` | Rejected | Too close to shared aliasing before the memory model is stronger. | [hello_task_glm_ref_spawn_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_ref_spawn_invalid.ns) |
| `borrow(...)` task input | Rejected | Explicit borrowed task input is blocked at the spawn boundary. | [hello_task_glm_borrowed_spawn_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_borrowed_spawn_invalid.ns) |
| Optional `?...` payloads | Rejected | Nullable payload identity and lifecycle are still treated as unsafe for task crossing. | [hello_task_glm_optional_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_optional_payload_invalid.ns) |
| `Instance<...>` | Rejected | Staged domain-instance handles are not yet allowed across the current async/task boundary. | [hello_task_glm_instance_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_instance_payload_invalid.ns) |
| `Task<...>` | Rejected | Nested task payloads are intentionally blocked for now. | See [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md) |
| `TaskResult<...>` and other `*Result<...>` families | Rejected | Result-family payloads stay on the observation side, not the task-input side. | [hello_task_glm_result_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_result_payload_invalid.ns) |

## Future Watch List

The following families are not good candidates for casual loosening. They are
the ones most likely to change only after `GLM`, task ownership, and concurrent
memory semantics become much sharper:

* `ref ...`
* borrowed task inputs
* optional payloads
* `Instance<...>`
* `Task<...>`
* `*Result<...>` families

If any of these are reconsidered later, update the matrix together with:

* the task-memory contract
* the task-GLM contract
* the positive/negative example set

## Related References

* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
