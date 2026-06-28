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
* [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
  Use that file when the question is whether a memory example is still a
  frontdoor anchor, a compile-closure anchor, or only narrow probe detail.

Related current contracts:

* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)

Current role rule:

* this subtree is one of the strongest single-file semantic anchor sets in the
  repository
* during `alpha-0.4.*`, it should read as:
  ownership baseline -> task/GLM compile-closure spine -> thread/lock staged
  anchors -> narrow probes
* not every task-facing file here is equal-entry frontdoor material

## Current Frontdoor Ladders

If you only want the shortest current memory-side route, start with these
ladders.

Ownership and address ladder:

* [hello_glm.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns)
* [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
* [hello_buffer_addressing.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns)

This is the shortest source-facing route for:

* ownership-oriented baseline reading
* explicit local borrow/lifetime closure
* direct `ref Buffer` indexed-address reading

Task compile-closure ladder:

* [hello_task_result_control_flow.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns)
* [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
* [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
* [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)

This is the shortest source compile-closure route for:

* `if` / `match` + `Result<Task<_>, _>` + `await` + `?`
* status-only task observation
* timeout/cancel lifecycle observation
* completed-value observation

Thread/lock staged ladder:

* [hello_thread_mutex_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_observe.ns)
* [hello_thread_mutex_branch_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_observe.ns)
* [hello_thread_mutex_branch_suffix.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_thread_mutex_branch_suffix.ns)

This is the shortest staged thread/lock route for:

* basic `Thread<T>` / `Mutex<T>` observation
* branch-selected observer shaping
* branch-selected observer shaping with shared pure suffix

## Companion Detail Map

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
  - [hello_task_glm_origin.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_origin.ns)
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
* stronger than ordinary probes, but still narrower than the full project-route
  async/task story

## Reading Rule

* use the frontdoor ladders first
* use the compile-closure set when the question is "what is source-backed and
  currently survives real compile coverage?"
* use the companion detail map after you know which memory/task lane you care
  about
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use [docs/reference/cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
  when you want the current allowed/rejected payload split
* use [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  when you want project-form task companions
* treat the wider local probes as secondary unless you are actively working on
  task/GLM boundary behavior
