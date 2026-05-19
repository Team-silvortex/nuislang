# Examples

This folder currently mixes three kinds of material:

* current canonical examples
* verifier/error examples
* legacy-named examples kept for continuity while the architecture is still moving

The goal is to keep the useful history, while still making it obvious which files
best reflect the current `nuis -> NIR -> YIR -> LLVM/AOT` progress.

Subdirectory guides:

* [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)
* [examples/invalid/README.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/README.md)
* [examples/legacy/README.md](/Users/Shared/chroot/dev/nuislang/examples/legacy/README.md)
* [examples/bins/README.md](/Users/Shared/chroot/dev/nuislang/examples/bins/README.md)

## Recommended `.ns` examples

These are the best current front-end examples to read first.

* [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
  minimal `mod cpu <unit>` entry
* [hello_let_expr.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_let_expr.ns)
  `let` + arithmetic expression lowering
* [hello_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_struct.ns)
  module-level `struct` plus field access
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)
  `struct` fields carrying `ref` values
* [hello_glm.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns)
  ownership/lifetime-flavored CPU memory path through `.ns -> NIR -> YIR`
* [hello_task_glm_scalar_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_scalar_payload.ns)
  smallest currently-safe scalar task payload path
* [hello_task_glm_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_struct_payload.ns)
  small struct-of-scalars payload path across the current async/task boundary
* [hello_task_glm_nested_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_struct_payload.ns)
  nested struct-of-scalars payload path showing that named wrappers are still allowed when their fields remain value-like
* [hello_task_glm_text_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_text_payload.ns)
  current plain text/value-like payload path across the current async/task boundary
* [hello_task_glm_nested_text_struct_payload.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns)
  nested text/value-like payload path showing that named wrappers with safe text fields remain allowed
* [hello_task_glm_origin.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_origin.ns)
  smallest current task-handle origin and direct payload extraction path: `spawn -> join`
* [hello_task_glm_lifecycle.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle.ns)
  lifecycle-shaping path through `timeout/cancel -> join_result -> task_timed_out/task_cancelled`
* [hello_task_glm_boundary_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_boundary_compare.ns)
  direct side-by-side task boundary sample:
  `spawn -> join` as origin/payload path
  versus `timeout/cancel -> join_result -> task_*` as lifecycle observation path
* [hello_task_glm_lifecycle_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_compare.ns)
  side-by-side lifecycle sample showing that completed tasks flow to `task_value(...)`, while timeout/cancel paths stay observation-only
* [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
  current positive task-observation path: `spawn -> timeout -> join_result -> task_completed -> task_value`
* [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
  side-by-side comparison of direct payload extraction with `join(...)` and lifecycle-aware observation with `join_result(...)`
* [hello_task_glm_join_nonconsuming_probe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_join_nonconsuming_probe.ns)
  design-probe sample showing a shape that is currently allowed because
  `join(...)` is not yet treated as a graph-level consume boundary:
  direct `join(task)` followed by `join_result(task)`
* [hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
  first front-end `data` link surface
* [hello_data_window.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data_window.ns)
  front-end `data` windows and handle table
* [hello_instantiate.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_instantiate.ns)
  `cpu`-side unit instantiation of another domain through lazy `nustar` binding
* [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
  front-end `cpu + data + shader` control/render demo that now builds to a live macOS bundle
* [projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  same live ball demo as a multi-file `nuis.toml` project with `main / shader / data` split
* [projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  current multi-file `cpu + data + kernel` project route

## Recommended `YIR` examples

These are the best current handwritten `YIR` examples to read first.

* [hello_yir.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/hello_yir.yir)
  smallest cross-domain `cpu + data + shader` flavor
* [window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
  current main `cpu + data + shader` control/render demo
* [host_ui_sphere.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/host_ui_sphere.yir)
  richer host-window/render path
* [shader_bindings_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_bindings_demo.yir)
  shader resource layout, geometry input, and render-state surface
* [shader_texture_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_texture_demo.yir)
  texture/sampler/UV sampling path
* [kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_auto_broadcast_demo.yir)
  current kernel broadcast path
* [kernel_topk_axis_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_topk_axis_demo.yir)
  current axis-selection path
* [data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_fabric_demo.yir)
  current typed Fabric/data surface
* [cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu/cpu_linked_list_rustish.yir)
  current Rust-ish CPU ownership model
* [cpu_types_struct.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu/cpu_types_struct.yir)
  typed scalar + struct value surface

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
