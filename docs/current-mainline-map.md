# Current Mainline Map

This file is the canonical short reading map for the current repository spine.

If several READMEs seem to overlap, prefer this file first, then drill into the
local README for the area you are actively touching.

## Start Here

* repo status and current toolchain spine:
  [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
* implementation-truth docs:
  [reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* repo structure:
  [repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

## Current Truth By Layer

* source examples:
  [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* project examples:
  [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* `std` growth path:
  [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)

## Task-Facing `std`

Read in this order:

* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)

Best companions:

* source-level:
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
* project-level:
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_status_observe_demo),
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo),
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo),
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo),
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)

## Host I/O Mainline

Read in this order:

* execution:
  [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns),
  [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns),
  [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns)
* input observation:
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns),
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns),
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* terminal shaping:
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns),
  [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns),
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)

Best companions:

* source-level:
  [hello_argv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns),
  [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns),
  [hello_process_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns),
  [hello_stdin_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stdin_runtime_facades.ns),
  [hello_tty_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_tty_runtime_facades.ns),
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns),
  [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
* project-level:
  [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/argv_runtime_demo),
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/env_runtime_demo),
  [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/process_runtime_demo),
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/stdin_runtime_demo),
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tty_runtime_demo),
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo),
  [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/terminal_io_demo)

## State / Location / Persistence

Read in this order:

* location roots:
  [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns),
  [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns),
  [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns)
* bundle:
  [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
* persistence:
  [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns),
  [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns),
  [config_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime_recipe.ns),
  [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)

Best companions:

* source-level:
  [hello_cwd_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns),
  [hello_temp_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns),
  [hello_home_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_home_runtime_facades.ns),
  [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns),
  [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns),
  [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns),
  [hello_config_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_runtime_facades.ns),
  [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)
* project-level:
  [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cwd_runtime_demo),
  [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/temp_runtime_demo),
  [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/home_runtime_demo),
  [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/location_runtime_demo),
  [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kv_runtime_demo),
  [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cache_runtime_demo),
  [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_runtime_demo),
  [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_cache_demo)

## Path / Filesystem

Use local READMEs for the long tail, but start from these anchors:

* path naming:
  [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
* path structure:
  [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns),
  [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
* path name parts:
  [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns),
  [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns),
  [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
* filesystem mutate/read/write:
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns),
  [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns),
  [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns),
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

## Cleanup Rule

When a local README and this file differ:

* use this file for the shortest current entry path
* use the local README only for area-specific detail
* treat anything outside these paths as secondary unless you are actively
  working in that subsystem
