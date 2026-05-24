# Shared Domain Helpers

This folder holds project-local helper modules that are intentionally shared by
multiple `shader` / `kernel` project-first demos.

These are not `std` wrappers.
They are the current checked-in reuse point for project-local `cpu` helper
shapes that would be too repetitive to keep duplicating in each async demo.

Current shared helpers:

* [shader_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/shader_task_async_shapes.ns)
  task-shaped async control helpers reused by shader async policy/fallback
  companions
* [kernel_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/kernel_task_async_shapes.ns)
  task-shaped async control helpers reused by kernel async tensor
  policy/fallback companions

Current stable helper reading rule:

* keep `task_*` names as the low-level compatibility layer
* prefer the newer `async_*_summary_*` exports when a sample is meant to read
  directly against the scheduler/result/summary contract stack

Current reading rule:

* start with the domain project route in
  [README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)
* use this folder when you want to see where shared task-shaped naming and
  helper logic are actually factored out
* use
  [std-shader-kernel-project-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-shader-kernel-project-contract.md)
  when you want the higher-level contract that explains why these helpers live
  here instead of under `std`
