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

## Legacy Names Kept For Continuity

These files still matter, but their names reflect older architecture wording.

* [nsnova_ball_frame.yir](/Users/Shared/chroot/dev/nuislang/examples/legacy/nsnova_ball_frame.yir)
  kept because it was an important bridge demo during the `ns-nova` discussion
* [npu_tensor_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/legacy/npu_tensor_demo.yir)
  kept as a historical bridge from older `npu` naming into the newer `kernel` surface

## Output Bundles

Generated build outputs live under:

* [examples/bins](/Users/Shared/chroot/dev/nuislang/examples/bins)

Use the bundle folder whose name matches the source path you care about:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo/window_controls_demo)
  handwritten `YIR` route
* [window_controls_demo_ns](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_ns/window_controls_demo)
  single-file `.ns` route
* [window_controls_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
  multi-file project route
