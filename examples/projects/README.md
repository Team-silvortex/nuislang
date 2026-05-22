# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by `nuis.toml`.
This is the current canonical route for reading real `.ns` programs in this repo.

Current layout:

* showcase projects stay at the root of this folder
* narrow one-file companions now live under:
  - [task](/Users/Shared/chroot/dev/nuislang/examples/projects/task)
  - [tooling](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling)
  - [state](/Users/Shared/chroot/dev/nuislang/examples/projects/state)
  - [filesystem](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem)
  - [domains](/Users/Shared/chroot/dev/nuislang/examples/projects/domains)

## What A Project Gives You

Compared with a single `.ns` file, project mode currently adds:

* `nuis.toml` manifest
* multi-file `mod cpu / mod data / mod shader / mod kernel` split
* project-level `links`
* project-level ABI locking or auto-resolution
* project metadata outputs during `build`
* compile-cache identity based on the whole project input set

Current project `links` are not only manifest hints anymore. They are checked
against final `YIR` as real `source -> data -> target` exchange structure.

Projects can lock required `nustar` ABI profiles per domain via:

```toml
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "data=data.fabric.macos.arm64.v1",
  "shader=shader.metal.msl2_4",
]
```

If `abi` is omitted, `nuisc/nuis` now auto-resolve a host-matching ABI set per
involved domain from the `abi_targets` registered by each `nustar` package.

Per-domain lane defaults are also declared by each `nustar` package through
`default_lanes = ["op.name=lane"]`, so project/profile lowering stays mod-owned
and `nuisc` only applies declared policy plus narrow fallback rules.

## Core Commands

Inspect project state:

```bash
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- project-lock-abi examples/projects/window_controls_demo
```

Validate and build:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project

cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/kernel_tensor_demo examples/bins/kernel_tensor_demo_project
```

Inspect cache and artifact metadata:

```bash
cargo run -p nuis -- cache-status examples/projects/window_controls_demo
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
```

Override CPU target when needed:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project

cargo run -p nuis -- build --target aarch64-apple-darwin \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project
```

Recommended starting point:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  three-file real-time ball demo:
  `main.ns`, `surface_shader.ns`, `fabric_plane.ns`
  with project links:
  `cpu.Main -> shader.SurfaceShader via data.FabricPlane`
  `shader.SurfaceShader -> cpu.Main via data.FabricPlane`
  and per-mod `profile()` hooks in shader/data files that now also emit
  concrete `YIR` setup nodes during project compilation.
  `SurfaceShader` now contributes target/viewport/pipeline plus draw budget constants,
  plus inline WGSL source blocks via:
  `shader_inline_wgsl("entry", wgsl { ... })`
  while `FabricPlane` contributes bind-core, handle table, sync markers, and
  explicit uplink/downlink window policy nodes that are stitched into the final
  data-plane graph. The data profile markers are now validated per link
  direction, so a `cpu <-> shader` fabric only needs its own sync pair.

`window_controls_demo` is also the current migration source for the first
checked-in `ns-nova` stdlib recipes. The relationship today is:

* runtime orchestration patterns are being extracted into
  [stdlib/ns-nova/core/window_controls_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/core/window_controls_runtime_recipe.ns)
* UI/selection/control assembly patterns are being extracted into
  [stdlib/ns-nova/ui/window_controls_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/ui/window_controls_recipe.ns)
* scene/render-world assembly patterns are being extracted into
  [stdlib/ns-nova/scene/window_controls_scene_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/scene/window_controls_scene_recipe.ns)

What still remains demo-local on purpose:

* the full end-to-end one-file assembly of all those routes together
* project-specific host/window wiring
* the exact demo tuning constants and packet mixes used to stress the current
  `ns -> NIR -> YIR -> build` chain

Core companion routes:

* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  the main `cpu + data + kernel` project route alongside
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* domain-profile companions:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo),
  [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo),
  [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo),
  [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo),
  [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo),
  [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo),
  [shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo),
  [shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo),
  [shader_surface_state_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_pass_profile_demo),
  [shader_surface_state_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_flow_profile_demo),
  [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo),
  [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo),
  [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo),
  [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo),
  [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo),
  [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo),
  [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo),
  [shader_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_result_profile_demo),
  [shader_async_fanin_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fanin_profile_demo),
  [shader_async_schedule_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_schedule_profile_demo),
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo),
  [shader_async_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fallback_profile_demo),
  [shader_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_batch_profile_demo),
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo),
  [shader_result_family_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_family_profile_demo),
  [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo),
  [shader_draw_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_render_profile_demo),
  [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo),
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo),
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo),
  [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo),
  [kernel_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_result_profile_demo),
  [kernel_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_batch_profile_demo),
  [kernel_async_tensor_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_batch_profile_demo),
  [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo),
  [kernel_async_tensor_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_fallback_profile_demo),
  [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo),
  [kernel_async_tensor_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_roundtrip_profile_demo),
  [kernel_async_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_roundtrip_profile_demo),
  [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo),
  [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo),
  [kernel_tensor_slice_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_slice_demo),
  [kernel_tensor_reshape_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reshape_demo),
  [kernel_tensor_broadcast_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_broadcast_demo),
  [kernel_tensor_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reduce_demo),
  [kernel_tensor_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_select_demo),
  [kernel_tensor_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_order_demo),
  [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo),
  [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo),
  [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo),
  [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo),
  [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo),
  [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo),
  [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo),
  [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo),
  [kernel_tensor_map_zip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_map_zip_demo),
  [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)
* task-facing companions:
  [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo),
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo),
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo),
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo),
  [task_clock_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo),
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo),
  [task_fallback_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo),
  [task_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_policy_branch_demo),
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo)
* tooling/runtime companions:
  [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/argv_runtime_demo),
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/env_runtime_demo),
  [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/process_runtime_demo),
  [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo),
  [subprocess_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo),
  [host_text_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/host_text_runtime_demo),
  [json_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/json_runtime_demo),
  [text_format_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_format_runtime_demo),
  [error_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_runtime_demo),
  [diagnostic_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/diagnostic_runtime_demo),
  [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
  [sleep_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/sleep_runtime_demo),
  [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
  [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo),
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/tty_runtime_demo),
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/input_runtime_demo),
  [io_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/io_runtime_demo),
  [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_shell_demo),
  [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* state/persistence companions:
  [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cwd_runtime_demo),
  [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/temp_runtime_demo),
  [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/home_runtime_demo),
  [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/location_runtime_demo),
  [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_runtime_demo),
  [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_cache_demo)
* filesystem companions:
  [window_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/window_runtime_demo),
  [pipe_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/pipe_runtime_demo),
  [fabric_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/fabric_runtime_demo),
  [handle_table_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/handle_table_runtime_demo),
  [directory_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_runtime_demo),
  [stat_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/stat_runtime_demo),
  [fs_metadata_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/fs_metadata_runtime_demo),
  [file_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_runtime_demo),
  [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_runtime_demo),
  [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_output_demo),
  [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_create_demo),
  [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_stat_demo)

Reading rule:

* use this README for project-mode meaning plus the smallest current anchor set
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  when you want the recipe-side grouping
* treat deeper project inventories as secondary unless you are actively working
  in that subsystem

## Migration Map

Current project examples and `stdlib/ns-nova` play different roles:

* `examples/projects/*`
  canonical end-to-end project workflow and the most realistic current build
  path
* `stdlib/ns-nova/*`
  the first reusable builder/helper/recipe source assets being extracted from
  those projects

Read them together like this:

* start with [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  when you want the current real project route
* jump to [stdlib/ns-nova/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/ns-nova/README.md)
  when you want to see which pieces have already started to move out of the
  demo and into reusable source assets

Use:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/kernel_tensor_demo examples/bins/kernel_tensor_demo_project
cargo run -p nuis -- check examples/projects/tooling/command_shell_demo
cargo run -p nuis -- build examples/projects/tooling/command_shell_demo /private/tmp/command_shell_demo_out
cargo run -p nuis -- check examples/projects/tooling/report_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/report_runtime_demo /private/tmp/report_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/automation_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/automation_runtime_demo /private/tmp/automation_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/cli_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/cli_runtime_demo /private/tmp/cli_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/input_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/input_runtime_demo /private/tmp/input_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/io_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/io_runtime_demo /private/tmp/io_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/command_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/command_runtime_demo /private/tmp/command_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/subprocess_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/subprocess_runtime_demo /private/tmp/subprocess_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/host_text_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/host_text_runtime_demo /private/tmp/host_text_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/json_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/json_runtime_demo /private/tmp/json_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/text_format_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/text_format_runtime_demo /private/tmp/text_format_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/error_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/error_runtime_demo /private/tmp/error_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/result_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/result_runtime_demo /private/tmp/result_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/diagnostic_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/diagnostic_runtime_demo /private/tmp/diagnostic_runtime_demo_out
cargo run -p nuis -- check examples/projects/tooling/sleep_runtime_demo
cargo run -p nuis -- build examples/projects/tooling/sleep_runtime_demo /private/tmp/sleep_runtime_demo_out
cargo run -p nuis -- check examples/projects/task/task_lifecycle_branch_demo
cargo run -p nuis -- check examples/projects/task/task_fallback_branch_demo
cargo run -p nuis -- build examples/projects/task/task_lifecycle_branch_demo /private/tmp/task_lifecycle_branch_demo_out
cargo run -p nuis -- build examples/projects/task/task_fallback_branch_demo /private/tmp/task_fallback_branch_demo_out
cargo run -p nuis -- check examples/projects/task/task_policy_branch_demo
cargo run -p nuis -- build examples/projects/task/task_policy_branch_demo /private/tmp/task_policy_branch_demo_out
cargo run -p nuis -- check examples/projects/task/task_runtime_demo
cargo run -p nuis -- build examples/projects/task/task_runtime_demo /private/tmp/task_runtime_demo_out
cargo run -p nuis -- check examples/projects/task/task_completed_observe_demo
cargo run -p nuis -- build examples/projects/task/task_completed_observe_demo /private/tmp/task_completed_observe_demo_out
cargo run -p nuis -- check examples/projects/task/task_compare_observe_demo
cargo run -p nuis -- build examples/projects/task/task_compare_observe_demo /private/tmp/task_compare_observe_demo_out
cargo run -p nuis -- check examples/projects/task/task_clock_observe_demo
cargo run -p nuis -- build examples/projects/task/task_clock_observe_demo /private/tmp/task_clock_observe_demo_out
cargo run -p nuis -- check examples/projects/task/task_scheduler_observe_demo
cargo run -p nuis -- build examples/projects/task/task_scheduler_observe_demo /private/tmp/task_scheduler_observe_demo_out
cargo run -p nuis -- check examples/projects/task/task_status_observe_demo
cargo run -p nuis -- build examples/projects/task/task_status_observe_demo /private/tmp/task_status_observe_demo_out
cargo run -p nuis -- check examples/projects/task/task_cli_tooling_demo
cargo run -p nuis -- build examples/projects/task/task_cli_tooling_demo /private/tmp/task_cli_tooling_demo_out
cargo run -p nuis -- check examples/projects/task/task_cancel_branch_demo
cargo run -p nuis -- build examples/projects/task/task_cancel_branch_demo /private/tmp/task_cancel_branch_demo_out
cargo run -p nuis -- check examples/projects/task/task_join_nonconsuming_probe_demo
cargo run -p nuis -- build examples/projects/task/task_join_nonconsuming_probe_demo /private/tmp/task_join_nonconsuming_probe_demo_out
cargo run -p nuis -- check examples/projects/filesystem/fs_metadata_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/fs_metadata_runtime_demo /private/tmp/fs_metadata_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/directory_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/directory_runtime_demo /private/tmp/directory_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/window_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/window_runtime_demo /private/tmp/window_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/pipe_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/pipe_runtime_demo /private/tmp/pipe_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/fabric_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/fabric_runtime_demo /private/tmp/fabric_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/handle_table_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/handle_table_runtime_demo /private/tmp/handle_table_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/stat_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/stat_runtime_demo /private/tmp/stat_runtime_demo_out
cargo run -p nuis -- check examples/projects/filesystem/file_runtime_demo
cargo run -p nuis -- build examples/projects/filesystem/file_runtime_demo /private/tmp/file_runtime_demo_out
```

Generated outputs to expect from a project build:

* [examples/bins/window_controls_demo_project/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
* [examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/nuis.project.host_ffi.txt)
  generated host-ffi contract index (abi/interface/symbol/signature) consumed by the project route
* `nuis.project.modules.txt`
  module index emitted by the project route
* `nuis.project.links.txt`
  link index emitted by the project route
* `nuis.project.abi.txt`
  effective ABI lock/auto-resolution summary
* `nuis.build.manifest.toml`
  build manifest including per-domain target/backend details
