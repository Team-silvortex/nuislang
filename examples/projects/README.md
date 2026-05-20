# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by `nuis.toml`.
This is the current canonical route for reading real `.ns` programs in this repo.

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

Also included:

* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  three-file `cpu + data + kernel` demo:
  `main.ns`, `kernel_unit.ns`, `fabric_plane.ns`
  with project links:
  `cpu.Main -> kernel.KernelUnit via data.FabricPlane`
  `kernel.KernelUnit -> cpu.Main via data.FabricPlane`
  and kernel profile slots consumed from CPU via
  `kernel_profile_bind_core/kernel_profile_queue_depth/kernel_profile_batch_lanes`.
  Its `FabricPlane` now only declares the `cpu_to_kernel/kernel_to_cpu` sync
  markers required by that route.
* [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
  one-file `cpu`-only command/subprocess staging demo:
  `main.ns`
  showing the current project-form shell-oriented bridge for
  `program/argv/env -> command/subprocess observers`.
  This is the narrowest project-shaped companion to
  [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns).
* [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
  one-file `cpu`-only report/diagnostic staging demo:
  `main.ns`
  showing the current project-form bridge for
  `path/fs/json -> diag_emit + stdout`.
  This is the narrowest project-shaped companion to
  [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns).
* [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)
  one-file `cpu`-only automation/workflow staging demo:
  `main.ns`
  showing the current project-form bridge for
  `cwd/temp/cache -> subprocess + monotonic time`.
  This is the narrowest project-shaped companion to
  [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns).
* [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cwd_runtime_demo)
  one-file `cpu`-only cwd/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `cwd_handle/cwd_len/chdir`.
  This is the narrowest project-shaped companion to
  [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns).
* [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/temp_runtime_demo)
  one-file `cpu`-only temp/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `temp_dir/temp_path_len/temp_file_handle`.
  This is the narrowest project-shaped companion to
  [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns).
* [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/home_runtime_demo)
  one-file `cpu`-only home/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `home_dir/home_len/config_dir`.
  This is the narrowest project-shaped companion to
  [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns).
* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
  one-file `cpu`-only CLI/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `argv/env/cwd/config/cache -> stdout + diag + monotonic time`.
  This is the narrowest project-shaped companion to
  [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns).
* [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
  one-file `cpu`-only native input/runtime demo:
  `main.ns`
  showing the current project-form AOT host-backed path for
  `argv`, `file`, `stdin`, and `tty`.
  This is the narrowest project-shaped companion to
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns).
* [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_runtime_demo)
  one-file `cpu`-only config/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `config_open/get/close`.
  This is the narrowest project-shaped companion to
  [config_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime_recipe.ns).
* [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/env_runtime_demo)
  one-file `cpu`-only env/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `env_has/env_get`.
  This is the narrowest project-shaped companion to
  [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns).
* [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/process_runtime_demo)
  one-file `cpu`-only process/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `process_id/status/exit_code`.
  This is the narrowest project-shaped companion to
  [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns).
* [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/stdin_runtime_demo)
  one-file `cpu`-only stdin/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  repeated `stdin_read`.
  This is the narrowest project-shaped companion to
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns).
* [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tty_runtime_demo)
  one-file `cpu`-only tty/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `isatty/width/height`.
  This is the narrowest project-shaped companion to
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns).
* [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/argv_runtime_demo)
  one-file `cpu`-only argv/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `argv_count -> argv_at(0/1)`.
  This is the narrowest project-shaped companion to
  [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns).
* [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  one-file `cpu`-only async/task lifecycle demo:
  `main.ns`
  showing the current project-form bridge between
  `spawn/timeout/join_result/task_timed_out`
  and real CPU branch control flow.
  This is the current canonical project-shaped sample for task observation plus
  branch/return behavior, while payload extraction still remains easier to read
  in the single-file memory examples.
  Current note:
  the project route already validates this shape through
  `.ns -> NIR -> YIR -> LLVM`,
  but native CPU task execution in the LLVM/AOT path is still deferred, so this
  sample is currently strongest as a compile/contract example rather than a
  fully live runtime task demo.
  Future direction note:
  [examples/projects/task_lifecycle_branch_demo/FUTURE_LIFECYCLE_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo/FUTURE_LIFECYCLE_SKETCH.md)
* [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  one-file `cpu`-only completed-result demo:
  `main.ns`
  showing the current project-form positive observation path for
  `spawn -> join_result -> task_completed -> task_value`.
  This is the smallest project-shaped sample for payload extraction from a
  completed task result.
  Future direction note:
  [examples/projects/task_completed_observe_demo/FUTURE_HOT_SYNC_CONTRACTION_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo/FUTURE_HOT_SYNC_CONTRACTION_SKETCH.md)
* [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  one-file `cpu`-only direct-vs-observed compare demo:
  `main.ns`
  showing the current project-form comparison between
  `spawn -> join`
  and
  `spawn -> join_result -> task_completed -> task_value`.
  This is the smallest project-shaped companion to
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns).
* [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
  one-file `cpu`-only status observer demo:
  `main.ns`
  showing the current project-form narrow status path for
  `join_result -> task_completed/task_timed_out/task_cancelled`.
  This is the smallest project-shaped companion to
  [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns).
* [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
  one-file `cpu`-only async tooling demo:
  `main.ns`
  showing the current project-form bridge between
  `spawn/timeout/join_result/task_completed/task_value`
  and host-facing CLI reporting surfaces like
  `host_argv_count`, `host_stdout_write`, `host_stderr_write`, and
  `host_monotonic_time_ns`.
  This is the current canonical project-shaped companion to
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns).
  Like the other current task samples, it is strongest today as a
  compile/contract example while native CPU task execution remains deferred in
  LLVM/AOT.
* [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
  one-file `cpu`-only cancel lifecycle demo:
  `main.ns`
  showing the current project-form bridge between
  `cancel -> join_result -> task_cancelled`
  and real CPU branch control flow.
  Like the timeout sibling, this is currently strongest as a compile/contract
  example while native CPU task execution remains deferred in LLVM/AOT.
  Future direction note:
  [examples/projects/task_cancel_branch_demo/FUTURE_CANCEL_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo/FUTURE_CANCEL_SKETCH.md)
* [task_join_nonconsuming_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_join_nonconsuming_probe_demo)
  one-file `cpu`-only join-boundary probe:
  `main.ns`
  showing a shape that is currently legal because `join(...)` is still treated
  as a direct payload boundary rather than a final graph-level consume.
  It deliberately performs `join(task)` and later `join_result(task)` in the
  same flow, so it acts as a future regression probe if task-GLM ownership
  rules become stricter.
  See also:
  [examples/projects/task_join_nonconsuming_probe_demo/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_join_nonconsuming_probe_demo/README.md)
  for the future-tightening note, and
  [examples/projects/task_join_nonconsuming_probe_demo/FUTURE_CONSUME_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_join_nonconsuming_probe_demo/FUTURE_CONSUME_SKETCH.md)
  for the likely migration sketch if `join(...)` later becomes consuming.

Narrow systems companions:

Filesystem mini-map:

* naming
  - [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_runtime_demo)
  - [path_is_empty_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_empty_demo)
  - [path_is_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dot_demo)
  - [path_is_dotdot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dotdot_demo)
  - [path_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_demo)
  - [path_depth_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_depth_demo)
  - [path_filename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_demo)
  - [path_stem_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_demo)
  - [path_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_demo)
  - [path_has_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_extension_demo)
  - [path_matches_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_matches_extension_demo)
  - [path_starts_with_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_starts_with_dot_demo)
  - [path_is_hidden_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_hidden_demo)
  - [path_is_relative_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_relative_demo)
  - [path_is_root_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_root_demo)
  - [path_ends_with_slash_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_ends_with_slash_demo)
* mutation
  - [path_rename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_rename_demo)
  - [path_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_copy_demo)
  - [path_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_remove_demo)
  - [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_create_demo)
  - [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_remove_demo)
* output
  - [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/file_output_demo)
* inspection
  - [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_stat_demo)

Path project fast map:

* shape
  - [path_is_empty_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_empty_demo)
  - [path_is_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dot_demo)
  - [path_is_dotdot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dotdot_demo)
  - [path_is_relative_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_relative_demo)
  - [path_is_root_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_root_demo)
  - [path_ends_with_slash_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_ends_with_slash_demo)
  - [path_starts_with_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_starts_with_dot_demo)
  - [path_is_hidden_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_hidden_demo)
* structure
  - [path_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_demo)
  - [path_has_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_parent_demo)
  - [path_depth_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_depth_demo)
  - [path_is_basename_only_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_basename_only_demo)
* name parts
  - [path_filename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_demo)
  - [path_stem_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_demo)
  - [path_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_demo)
  - [path_has_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_extension_demo)
* matches
  - [path_basename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_basename_matches_demo)
  - [path_filename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_matches_demo)
  - [path_parent_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_matches_demo)
  - [path_stem_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_matches_demo)
  - [path_matches_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_matches_extension_demo)
  - [path_extension_is_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_is_demo)

Tooling project fast map:

* io
  - [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/argv_runtime_demo)
  - [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/process_runtime_demo)
  - [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/env_runtime_demo)
  - [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
  - [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/terminal_io_demo)
  - [line_input_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/line_input_demo)
  - [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/file_output_demo)
* shell and process
  - [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
  - [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)
* cli and reporting
  - [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
  - [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
  - [result_diagnostic_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/result_diagnostic_demo)

State/persistence project fast map:

* location
  - [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/location_runtime_demo)
* kv
  - [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kv_runtime_demo)
* cache
  - [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cache_runtime_demo)
* config and cache bridge
  - [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_cache_demo)

* input/runtime
  - [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
* command/shell
  - [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
* path/runtime
  - [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_runtime_demo)
* path/is-empty
  - [path_is_empty_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_empty_demo)
* path/is-dot
  - [path_is_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dot_demo)
* path/is-dotdot
  - [path_is_dotdot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dotdot_demo)
* path/parent
  - [path_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_demo)
* path/has-parent
  - [path_has_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_parent_demo)
* path/depth
  - [path_depth_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_depth_demo)
* path/is-basename-only
  - [path_is_basename_only_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_basename_only_demo)
* path/basename-matches
  - [path_basename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_basename_matches_demo)
* path/filename-matches
  - [path_filename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_matches_demo)
* path/parent-matches
  - [path_parent_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_matches_demo)
* path/stem-matches
  - [path_stem_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_matches_demo)
* path/filename
  - [path_filename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_demo)
* path/stem
  - [path_stem_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_demo)
* path/extension
  - [path_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_demo)
* path/has-extension
  - [path_has_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_extension_demo)
* path/matches-extension
  - [path_matches_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_matches_extension_demo)
* path/extension-is
  - [path_extension_is_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_is_demo)
* path/starts-with-dot
  - [path_starts_with_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_starts_with_dot_demo)
* path/is-hidden
  - [path_is_hidden_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_hidden_demo)
* path/is-relative
  - [path_is_relative_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_relative_demo)
* path/is-root
  - [path_is_root_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_root_demo)
* path/ends-with-slash
  - [path_ends_with_slash_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_ends_with_slash_demo)
* path/rename
  - [path_rename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_rename_demo)
* path/copy
  - [path_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_copy_demo)
* path/remove
  - [path_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_remove_demo)
* file/output
  - [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/file_output_demo)
* line-input
  - [line_input_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/line_input_demo)
* terminal/io
  - [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/terminal_io_demo)
* text/json
  - [text_json_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/text_json_demo)
* cli/runtime
  - [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
* result/diagnostic
  - [result_diagnostic_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/result_diagnostic_demo)
* report/diagnostic
  - [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
* directory/create
  - [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_create_demo)
* directory/remove
  - [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_remove_demo)
* directory/stat
  - [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_stat_demo)
* automation/workflow
  - [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)
* cwd/runtime
  - [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cwd_runtime_demo)
* temp/runtime
  - [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/temp_runtime_demo)
* home/runtime
  - [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/home_runtime_demo)
* location/runtime
  - [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/location_runtime_demo)
* kv/runtime
  - [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kv_runtime_demo)
* cache/runtime
  - [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cache_runtime_demo)
* config/runtime
  - [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_runtime_demo)
* config/cache
  - [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_cache_demo)

Task-facing `std` companions:

* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  is mirrored most directly by
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  is mirrored most directly by
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
* [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  is reflected most directly in
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  ,
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  ,
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
  ,
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  , and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  is mirrored most directly by
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  is mirrored most directly by
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  is closest to
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  as the current compile/contract timeout-lifecycle companion
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  is currently closest in spirit to
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  and
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
  because they stay value-like, observer-local, and monotonic-time aware
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  is mirrored most directly by
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  is mirrored most directly by
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
* [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  is mirrored most directly by
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/stdin_runtime_demo)
* [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
  is mirrored most directly by
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tty_runtime_demo)

Recommended reading order for the current task projects:

* start with
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
  for the narrowest status-only observation path
* then read
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  for the smallest positive observation path
* then read
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  for the narrowest project-form direct-vs-observed comparison
* then read
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
  for timeout/cancel lifecycle shaping
* finish with
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
  when you want the current async/tooling reporting companion

Current task project boundaries by reading stage:

* [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo)
  is the cleanest project-shaped status-only observation path, but it should
  still be read as a compile/contract sample rather than proof that task status
  observation already implies a full native runtime lifecycle model
* [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  is the cleanest project-shaped positive observation path, but it should still
  be read mainly as a compile/contract sample while native CPU task execution
  remains deferred
* [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  is the cleanest project-shaped direct-vs-observed comparison path, but it
  should still be read as a current contract probe rather than proof that the
  present non-consuming `join(...)` shape is final
* [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
  are the clearest lifecycle-shaping samples, but they should still be read as
  branch/control-flow companions rather than proof of a completed cancellation
  or timeout runtime model
* [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
  is the clearest task/tooling project companion, but it should still be read
  as a project-form contract and reporting sample rather than a finished async
  native CLI runtime

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
cargo run -p nuis -- check examples/projects/command_shell_demo
cargo run -p nuis -- build examples/projects/command_shell_demo /private/tmp/command_shell_demo_out
cargo run -p nuis -- check examples/projects/report_runtime_demo
cargo run -p nuis -- build examples/projects/report_runtime_demo /private/tmp/report_runtime_demo_out
cargo run -p nuis -- check examples/projects/automation_runtime_demo
cargo run -p nuis -- build examples/projects/automation_runtime_demo /private/tmp/automation_runtime_demo_out
cargo run -p nuis -- check examples/projects/cli_runtime_demo
cargo run -p nuis -- build examples/projects/cli_runtime_demo /private/tmp/cli_runtime_demo_out
cargo run -p nuis -- check examples/projects/input_runtime_demo
cargo run -p nuis -- build examples/projects/input_runtime_demo /private/tmp/input_runtime_demo_out
cargo run -p nuis -- check examples/projects/task_lifecycle_branch_demo
cargo run -p nuis -- build examples/projects/task_lifecycle_branch_demo /private/tmp/task_lifecycle_branch_demo_out
cargo run -p nuis -- check examples/projects/task_completed_observe_demo
cargo run -p nuis -- build examples/projects/task_completed_observe_demo /private/tmp/task_completed_observe_demo_out
cargo run -p nuis -- check examples/projects/task_compare_observe_demo
cargo run -p nuis -- build examples/projects/task_compare_observe_demo /private/tmp/task_compare_observe_demo_out
cargo run -p nuis -- check examples/projects/task_status_observe_demo
cargo run -p nuis -- build examples/projects/task_status_observe_demo /private/tmp/task_status_observe_demo_out
cargo run -p nuis -- check examples/projects/task_cli_tooling_demo
cargo run -p nuis -- build examples/projects/task_cli_tooling_demo /private/tmp/task_cli_tooling_demo_out
cargo run -p nuis -- check examples/projects/task_cancel_branch_demo
cargo run -p nuis -- build examples/projects/task_cancel_branch_demo /private/tmp/task_cancel_branch_demo_out
cargo run -p nuis -- check examples/projects/task_join_nonconsuming_probe_demo
cargo run -p nuis -- build examples/projects/task_join_nonconsuming_probe_demo /private/tmp/task_join_nonconsuming_probe_demo_out
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
