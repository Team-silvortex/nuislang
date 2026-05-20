# `.ns` Examples

This folder contains the current front-end examples for:

* `mod <domain> <unit>` parsing
* `AST -> NIR -> YIR` lowering
* lazy `nustar` binding through `nuis / nuisc`
* current async/task surface at the source-language layer

Subdirectories:

* [core](/Users/Shared/chroot/dev/nuislang/examples/ns/core/README.md)
* [types](/Users/Shared/chroot/dev/nuislang/examples/ns/types/README.md)
* [data](/Users/Shared/chroot/dev/nuislang/examples/ns/data/README.md)
* [ffi](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
* [memory](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
* [demos](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/README.md)

Read these examples in roughly five bands:

* basic language and expression shape
  - [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
    minimal `mod cpu Main`
  - [hello_if.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_if.ns)
    current conditional shape
* type and ownership shape
  - [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)
    `struct` plus `ref` fields
  - [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
    explicit local lifetime closure
  - [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
    narrowest current completed-only task value path and the closest single-file
    memory companion to `std/task_value_recipe.ns`
  - [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
    narrowest current status-only task path and the closest single-file memory
    companion to `std/task_status_recipe.ns`
  - [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
    narrowest current lifecycle-only task path and the closest single-file
    memory companion to `std/task_lifecycle_recipe.ns`
  - [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
    clearest current direct-vs-observed task comparison path and the closest
    single-file memory companion to `std/task_compare_recipe.ns`
* data-path examples
  - [hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
    first front-end `data` link surface
  - [hello_data_window.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data_window.ns)
    local mutable `WindowMut<T>` and frozen `Window<T>` path
  - [hello_instantiate.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_instantiate.ns)
    `cpu`-side instantiation of another domain unit
* host facade examples
  - [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
    first `extern "nurs" interface` CPU-side host bridge example
  - [hello_c_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns)
    lower-level explicit `extern "c"` host route
  - [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
    tooling-shaped host example aligned with current `std` CLI/report/automation recipe thinking
  - [hello_result_diagnostic_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_diagnostic_facades.ns)
    narrow result/diagnostic host example aligned with current
    `std/result_diagnostic_recipe.ns`
  - [hello_native_cli_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_cli_runtime.ns)
    repo-local native CLI example aligned with the current AOT-backed `std` host shim batch
  - [hello_native_command_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_command_runtime.ns)
    repo-local native command example aligned with the current shell-oriented
    `program/argv/env` handle bridge in the AOT-backed `std` host shim batch
  - [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
    narrow path/runtime host example aligned with current
    `std/path_runtime_recipe.ns`
  - [hello_path_is_empty_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_empty_facades.ns)
    narrow path/is-empty host example aligned with current
    `std/path_is_empty_recipe.ns`
  - [hello_path_is_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dot_facades.ns)
    narrow path/is-dot host example aligned with current
    `std/path_is_dot_recipe.ns`
  - [hello_path_is_dotdot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dotdot_facades.ns)
    narrow path/is-dotdot host example aligned with current
    `std/path_is_dotdot_recipe.ns`
  - [hello_path_has_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_parent_facades.ns)
    narrow path/has-parent host example aligned with current
    `std/path_has_parent_recipe.ns`
  - [hello_path_depth_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns)
    narrow path/depth host example aligned with current
    `std/path_depth_recipe.ns`
  - [hello_path_is_basename_only_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_basename_only_facades.ns)
    narrow path/is-basename-only host example aligned with current
    `std/path_is_basename_only_recipe.ns`
  - [hello_path_basename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_basename_matches_facades.ns)
    narrow path/basename-matches host example aligned with current
    `std/path_basename_matches_recipe.ns`
  - [hello_path_filename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_matches_facades.ns)
    narrow path/filename-matches host example aligned with current
    `std/path_filename_matches_recipe.ns`
  - [hello_path_parent_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_matches_facades.ns)
    narrow path/parent-matches host example aligned with current
    `std/path_parent_matches_recipe.ns`
  - [hello_path_stem_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_matches_facades.ns)
    narrow path/stem-matches host example aligned with current
    `std/path_stem_matches_recipe.ns`
  - [hello_path_filename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_facades.ns)
    narrow path/filename host example aligned with current
    `std/path_filename_recipe.ns`
  - [hello_path_matches_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_matches_extension_facades.ns)
    narrow path/matches-extension host example aligned with current
    `std/path_matches_extension_recipe.ns`
  - [hello_path_extension_is_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_is_facades.ns)
    narrow path/extension-is host example aligned with current
    `std/path_extension_is_recipe.ns`
  - [hello_path_starts_with_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_starts_with_dot_facades.ns)
    narrow path/starts-with-dot host example aligned with current
    `std/path_starts_with_dot_recipe.ns`
  - [hello_path_is_root_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_root_facades.ns)
    narrow path/is-root host example aligned with current
    `std/path_is_root_recipe.ns`
  - [hello_path_ends_with_slash_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_ends_with_slash_facades.ns)
    narrow path/ends-with-slash host example aligned with current
    `std/path_ends_with_slash_recipe.ns`
  - [hello_path_rename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_rename_facades.ns)
    narrow path/rename host example aligned with current
    `std/path_rename_recipe.ns`
  - [hello_path_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_remove_facades.ns)
    narrow path/remove host example aligned with current
    `std/path_remove_recipe.ns`
  - [hello_file_output_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_output_facades.ns)
    narrow file/output host example aligned with current
    `std/file_output_recipe.ns`
  - [hello_line_input_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_line_input_facades.ns)
    narrow line-input host example aligned with current
    `std/line_input_recipe.ns`
  - [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
    narrow terminal/io host example aligned with current
    `std/terminal_io_recipe.ns`
  - [hello_text_json_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_json_facades.ns)
    narrow text/json host example aligned with current
    `std/text_json_recipe.ns`
  - [hello_native_input_tool.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_input_tool.ns)
    repo-local native input-driven example that reads one file path from `argv`
    and combines native file/stdin byte counts into its own result
  - [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
    narrower `argv/file/stdin/tty` facade mirror aligned directly with
    `std/input_runtime_recipe.ns`
  - [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns)
    narrow env/runtime host example aligned with current
    `std/env_runtime_recipe.ns`
  - [hello_native_cli_pipeline.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_cli_pipeline.ns)
    repo-local native pipeline example that combines input-driven reads with a
    child command/subprocess step in one front-door flow
  - [hello_native_tool_runner.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_tool_runner.ns)
    repo-local native tool-shaped example that launches a child command and
    decides its own result from direct-exit observers
  - [hello_native_workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_workflow_runtime.ns)
    repo-local native workflow example aligned with the current AOT-backed `std`
    directory/temp/process/command/subprocess shim batch
  - [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
    clock/timing host example aligned with current `std` clock/test recipe and `nuis test` timeout semantics
  - [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
    task/scheduler host example aligned with current `std` task scheduler recipe
    and lane-hint plus monotonic-tick task context
  - [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
    task/tooling host example aligned with current `std` task CLI recipe and observer-oriented async/task reporting
  - [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns)
    narrow location/runtime host example aligned with current
    `std/location_runtime_recipe.ns`
  - [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns)
    narrow kv/runtime host example aligned with current
    `std/kv_runtime_recipe.ns`
  - [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns)
    narrow cache/runtime host example aligned with current
    `std/cache_runtime_recipe.ns`
  - [hello_directory_create_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_create_facades.ns)
    narrow directory/create host example aligned with current
    `std/directory_create_recipe.ns`
  - [hello_directory_stat_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_stat_facades.ns)
    narrow directory/stat host example aligned with current
    `std/directory_stat_recipe.ns`
  - [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)
    narrow config/cache host example aligned with current
    `std/config_cache_recipe.ns`
* end-to-end demo path
  - [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
    current single-file `cpu + data + shader` real-time control/render demo
  - [projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
    current recommended multi-file project form once the demo grows beyond a single source file

Use:

```bash
cargo run -p nuis -- dump-ast examples/ns/core/hello_world.ns
cargo run -p nuis -- dump-nir examples/ns/types/hello_ref_struct.ns
cargo run -p nuis -- dump-yir examples/ns/data/hello_data.ns
cargo run -p nuis -- build examples/ns/data/hello_instantiate.ns /tmp/nuis_hello_instantiate
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns /tmp/window_controls_demo_ns
```

Project route is now preferred once a demo spans multiple domains or needs
real profile/link orchestration:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
```

Current reading notes:

* `mod` is a top-level builtin declaration, not a nested construct
* `cpu` is currently the only domain that can declare `async fn`
* current explicit task-style async surface is intentionally small:
  `spawn`, `join`, `cancel`, `timeout`, `join_result`, and `task_*`
* cross-domain interaction is expected to route through project links and `mod data`, not direct nested mod definitions
* current `ffi` examples are also where the host-backed `std` direction is easiest to see in source form before full import wiring exists
