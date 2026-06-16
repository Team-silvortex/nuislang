# Memory `.ns` Examples

This folder contains ownership, lifetime, and task-payload front-end examples.

Use it for:

* local borrow / move / lifetime shape
* current structural and buffer address shape
* current task payload and observation boundaries
* the narrowest single-file companions to task-facing `std` recipes

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file first when you want the shortest current route.

Related current contracts:

* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)

## First Anchors

Start here:

* [hello_glm.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns)
  smallest ownership-oriented baseline
* [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
  explicit local borrow/lifetime closure
* [hello_buffer_addressing.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns)
  smallest direct `ref Buffer` indexed-address baseline
* [hello_task_glm_origin.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_origin.ns)
  smallest direct `spawn -> join` payload path
* [hello_task_result_control_flow.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns)
  smallest direct `if` / `match` + `Result<Task<_>, _>` + `await` + `?` path,
  now also promoted to source compile-closure coverage
* [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
  narrowest status-only task path,
  now also promoted to source compile-closure coverage
* [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
  narrowest timeout/cancel lifecycle task path,
  now also promoted to source compile-closure coverage
* [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
  narrowest completed-value task path,
  now also promoted to source compile-closure coverage
* [hello_thread_mutex_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_observe.ns)
  narrowest staged `Thread<T>` / `Mutex<T>` observation path,
  now also promoted to source compile-closure coverage
* [hello_thread_mutex_branch_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_observe.ns)
  narrowest staged thread/lock branch-selected observer path,
  now also promoted to source compile-closure coverage
* [hello_thread_mutex_branch_suffix.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_suffix.ns)
  staged thread/lock branch-selected observer path with shared pure suffix,
  now also promoted to source compile-closure coverage

## Short Task Map

* payload shape
  - [hello_task_glm_scalar_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_scalar_payload.ns)
  - [hello_task_glm_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_struct_payload.ns)
  - [hello_task_glm_nested_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_struct_payload.ns)
  - [hello_task_glm_text_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_text_payload.ns)
  - [hello_task_glm_nested_text_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns)
* task observation
  - [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
  - [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
  - [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
  - [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
  - [hello_task_result_control_flow.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns)
* wider local probes
  - [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
  - [hello_task_glm_boundary_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_boundary_compare.ns)
  - [hello_task_glm_lifecycle_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_compare.ns)
  - [hello_task_glm_join_nonconsuming_probe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_join_nonconsuming_probe.ns)
    - now best read as a negative probe for `join(...)` / `join_result(...)`
      double-consume, not as a still-legal shape
* staged thread/lock observation
  - [hello_thread_mutex_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_observe.ns)
  - [hello_thread_mutex_branch_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_observe.ns)
  - [hello_thread_mutex_branch_suffix.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_suffix.ns)

## Current Compile-Closure Set

These examples now survive real source compile coverage under
[tools/nuisc/tests/memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs):

* [hello_task_result_control_flow.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns)
* [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
* [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
* [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
* [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
* [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
* [hello_task_glm_boundary_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_boundary_compare.ns)
* [hello_thread_mutex_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_observe.ns)
* [hello_thread_mutex_branch_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_observe.ns)
* [hello_thread_mutex_branch_suffix.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_suffix.ns)

Read this set as:

* the current checked-in source-level task/GLM observation spine
* plus the first short single-file staged thread/lock observation anchors
* not yet the whole task/thread example inventory

## Reading Rule

* use this README for the shortest memory/task anchor set
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use [docs/reference/cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
  when you want the current allowed/rejected payload split
* use [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  when you want project-form task companions
* treat the wider local probes as secondary unless you are actively working on
  task/GLM boundary behavior
