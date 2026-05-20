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
* [native_cli_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/native_cli_pipeline_demo)
  one-file `cpu`-only native CLI/tooling demo:
  `main.ns`
  showing the current project-form AOT host-backed path for
  `argv`, `stdout`, `file`, `stdin`, `command`, and `subprocess`.
  This is the repo's current canonical project-shaped sample for the native
  `input -> process -> child command -> exit code` pipeline.
* [native_tool_runner_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/native_tool_runner_demo)
  one-file `cpu`-only command runner demo:
  `main.ns`
  showing the current project-form AOT host-backed path for
  `argv`, `stdout`, `command`, `subprocess`, and direct exit observers.
  This is the lighter-weight sibling route when you want a native tool runner
  without the extra file/stdin input path.
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
* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
  one-file `cpu`-only CLI/runtime staging demo:
  `main.ns`
  showing the current project-form bridge for
  `argv/env/cwd/config/cache -> stdout + diag + monotonic time`.
  This is the narrowest project-shaped companion to
  [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns).
* [native_branch_cli_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/native_branch_cli_demo)
  one-file `cpu`-only branch/usage demo:
  `main.ns`
  showing the current project-form real CPU control-flow half-step for
  `if { print(...); return ... } else { print(...); return ... }`.
  This is the smallest canonical sample for native CLI usage/error vs ok paths.
* [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
  one-file `cpu`-only native input/runtime demo:
  `main.ns`
  showing the current project-form AOT host-backed path for
  `argv`, `file`, `stdin`, and `tty`.
  This is the narrowest project-shaped companion to
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns).
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

* input/runtime
  - [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
* command/shell
  - [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
* cli/runtime
  - [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
* report/diagnostic
  - [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
* automation/workflow
  - [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)

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
cargo run -p nuis -- check examples/projects/native_cli_pipeline_demo
cargo run -p nuis -- build examples/projects/native_cli_pipeline_demo /private/tmp/native_cli_pipeline_demo_out
cargo run -p nuis -- check examples/projects/native_tool_runner_demo
cargo run -p nuis -- build examples/projects/native_tool_runner_demo /private/tmp/native_tool_runner_demo_out
cargo run -p nuis -- check examples/projects/command_shell_demo
cargo run -p nuis -- build examples/projects/command_shell_demo /private/tmp/command_shell_demo_out
cargo run -p nuis -- check examples/projects/report_runtime_demo
cargo run -p nuis -- build examples/projects/report_runtime_demo /private/tmp/report_runtime_demo_out
cargo run -p nuis -- check examples/projects/automation_runtime_demo
cargo run -p nuis -- build examples/projects/automation_runtime_demo /private/tmp/automation_runtime_demo_out
cargo run -p nuis -- check examples/projects/cli_runtime_demo
cargo run -p nuis -- build examples/projects/cli_runtime_demo /private/tmp/cli_runtime_demo_out
cargo run -p nuis -- check examples/projects/native_branch_cli_demo
cargo run -p nuis -- build examples/projects/native_branch_cli_demo /private/tmp/native_branch_cli_demo_out
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
