# Examples

This folder currently mixes three kinds of material:

* current canonical examples
* verifier/error examples
* legacy-named examples kept for continuity while the architecture is still moving

The goal is to keep the useful history, while still making it obvious which files
best reflect the current `nuis -> NIR -> YIR -> LLVM/AOT` progress.

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file first when you want the shortest current path through the
  active examples and `std` surfaces.

Subdirectory guides:

* [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)
* [examples/invalid/README.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/README.md)
* [examples/legacy/README.md](/Users/Shared/chroot/dev/nuislang/examples/legacy/README.md)
* [examples/bins/README.md](/Users/Shared/chroot/dev/nuislang/examples/bins/README.md)

## Shortest Current Routes

Use the canonical short map first:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)

Then branch by goal:

* source-language `.ns` route
  - [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
  - start with:
    [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns),
    [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns),
    [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns),
    [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
* multi-file project route
  - [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  - start with:
    [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo),
    [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo),
    [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
* handwritten `YIR` route
  - [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)
  - start with:
    [hello_yir.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/hello_yir.yir),
    [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir),
    [data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_fabric_demo.yir)

Current reading rule:

* use this file and [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest route
* use local READMEs for area detail
* treat wider native examples and older umbrella demos as secondary unless
  you're actively working in that subsystem

## Verifier / Negative Examples

These examples are intentionally supposed to fail.

### `.ns`

* [hello_bad_unit.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/core/hello_bad_unit.ns)
  invalid unit not registered by the selected `nustar`
* [hello_glm_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_glm_invalid.ns)
  invalid ownership use
* [hello_ref_struct_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_ref_struct_invalid.ns)
  invalid consume of a borrowed `struct` field
* [hello_task_glm_observer_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_observer_invalid.ns)
  invalid attempt to treat `task_value(...)` as a direct `join(...)`-style extractor
* [hello_task_glm_borrowed_spawn_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_borrowed_spawn_invalid.ns)
  invalid borrowed task input passed directly through `spawn(...)`
* [hello_task_glm_ref_spawn_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_ref_spawn_invalid.ns)
  invalid `ref`-typed task input crossing the current spawn boundary
* [hello_task_glm_nested_ref_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_ref_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries a `ref` across the current async/task boundary
* [hello_task_glm_nested_window_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_window_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries a resource-bearing `Window<...>` across the current async/task boundary
* [hello_task_glm_window_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_window_external_handle_probe_invalid.ns)
  design probe for a future task-external `Window<...>` handle packet shape; intentionally still invalid today
* [hello_task_glm_marker_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_marker_external_handle_probe_invalid.ns)
  design probe for a future task-external `Marker<...>` control-plane packet shape; intentionally still invalid today
* [hello_task_glm_handle_table_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_handle_table_external_handle_probe_invalid.ns)
  design probe for a future task-external `HandleTable<...>` routing/control packet shape; intentionally still invalid today
* [hello_task_glm_nested_marker_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_marker_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries a control-plane `Marker<...>` across the current async/task boundary
* [hello_task_glm_optional_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_optional_payload_invalid.ns)
  invalid optional task payload family crossing the current async/task boundary
* [hello_task_glm_nested_optional_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_optional_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries an optional `?...` across the current async/task boundary
* [hello_task_glm_instance_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_instance_payload_invalid.ns)
  invalid `Instance<...>` task payload family crossing the current async/task boundary
* [hello_task_glm_nested_instance_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_instance_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries `Instance<...>` across the current async/task boundary
* [hello_task_glm_result_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_result_payload_invalid.ns)
  invalid `TaskResult<...>` payload family crossing the current async/task boundary
* [hello_task_glm_nested_result_struct_payload_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_nested_result_struct_payload_invalid.ns)
  invalid nominal struct payload whose nested field still carries a `*Result<...>` family across the current async/task boundary
* [hello_nested_mod_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/core/hello_nested_mod_invalid.ns)
  nested `mod` definitions are forbidden

### `YIR`

* [cpu_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_borrow_write_invalid.yir)
* [cpu_buffer_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_buffer_borrow_write_invalid.yir)
* [cpu_glm_missing_lifetime_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_glm_missing_lifetime_invalid.yir)
* [cpu_move_while_borrowed_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_move_while_borrowed_invalid.yir)
* [cpu_owner_write_while_borrowed_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_owner_write_while_borrowed_invalid.yir)
* [cpu_use_after_free_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_use_after_free_invalid.yir)
* [data_invalid_handle_table.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/data_invalid_handle_table.yir)
* [data_invalid_input_pipe.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/data_invalid_input_pipe.yir)
* [data_invalid_move_window.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/data_invalid_move_window.yir)
* [data_move_source_reuse_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/data_move_source_reuse_invalid.yir)

## Historical Bridge Examples

These files still matter, but their names reflect older architecture wording.

* [nsnova_ball_frame.yir](/Users/Shared/chroot/dev/nuislang/examples/legacy/nsnova_ball_frame.yir)
  historical bridge from early `ns-nova` naming into the current
  `window_controls_demo` routes; prefer
  [examples/ns/demos/window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns),
  [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo),
  or [examples/yir/demos/window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
* [npu_tensor_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/legacy/npu_tensor_demo.yir)
  historical bridge from older `npu` naming into the newer `kernel` surface;
  prefer [examples/yir/kernel/kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_auto_broadcast_demo.yir),
  [examples/yir/kernel/kernel_topk_axis_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_topk_axis_demo.yir),
  or [examples/projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

## Output Bundles

Generated build outputs live under:

* [examples/bins](/Users/Shared/chroot/dev/nuislang/examples/bins)

Current checked-in canonical bundles:

* [window_controls_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
  canonical multi-file project route
* [kernel_tensor_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/kernel_tensor_demo_project/kernel_tensor_demo)
  canonical kernel-project route

Single-file `.ns` and handwritten `YIR` demo routes are still useful source
examples, but their generated bundles are now treated as local rebuild outputs
instead of checked-in reference artifacts.
