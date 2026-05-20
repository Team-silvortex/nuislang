# `std`

`std` is the practical systems layer above `core`.

## Current Status

At the current repository stage, `std` is also still mostly a layout/contract
layer, but it now has its first small checked-in `.ns` source set.

That means its role is already important for dependency boundaries, and it is
now starting to accumulate small reusable modules for data/window/pipe helper
patterns.

Intended scope:

* convenience APIs that still preserve the AOT-first and semantics-first nature of `nuis`
* data-plane and host-integration helper surfaces that are too opinionated for `core`
* common utilities that typical `nuis` projects should not have to rebuild each time

Expected areas:

* collections and builder-style helper surfaces
* host FFI helper facades for CPU-side integration
* common data/window/handle-table orchestration helpers
* project/runtime utility APIs that are broadly useful but not framework-specific

Relationship:

* `std` depends on `core`
* `ns-nova` will use `std` as its general-purpose support layer rather than duplicating systems helpers

Source modules are easiest to read in two groups.

Facade modules:

* data/window/pipe helpers
  - [window_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime.ns)
  - [pipe_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime.ns)
  - [fabric_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime.ns)
  - [handle_table_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime.ns)
* text / file / path
  - [host_text_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime.ns)
  - [file_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime.ns)
  - [path_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime.ns)
* CLI and process
  - [argv_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime.ns)
  - [env_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime.ns)
  - [process_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime.ns)
  - [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  - [command_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime.ns)
  - [subprocess_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime.ns)
* terminal and output
  - [io_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime.ns)
  - [stdin_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime.ns)
  - [line_input_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_runtime.ns)
  - [tty_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime.ns)
  - [json_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime.ns)
  - [text_format_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime.ns)
* filesystem inspection
  - [fs_metadata_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime.ns)
  - [directory_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime.ns)
  - [stat_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime.ns)
* environment and location
  - [cwd_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime.ns)
  - [temp_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime.ns)
  - [home_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime.ns)
* time and clock
  - [time_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime.ns)
  - [clock_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime.ns)
  - [clock_domain_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime.ns)
* error and reporting
  - [error_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime.ns)
  - [result_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime.ns)
  - [diagnostic_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime.ns)
* config and persistence
  - [config_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime.ns)
  - [kv_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime.ns)
  - [cache_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime.ns)

Recipe modules:

* data/window routing
  - [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* CLI/tooling runtime
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  - [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  - [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  - [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  - [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  - [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* checker/reporter tooling
  - [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
* result/diagnostic staging
  - [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
* directory/stat staging
  - [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* directory/create staging
  - [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
* automation/workflow tooling
  - [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
* location/runtime staging
  - [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
* kv/runtime staging
  - [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
* cache/runtime staging
  - [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
* config/cache staging
  - [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
* shell-oriented command bridge
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [command_text_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_text_builder_recipe.ns)
* path/runtime staging
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
* path/rename staging
  - [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
* path/remove staging
  - [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
* file/output staging
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
* terminal/io staging
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
* line-input staging
  - [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
* text/json staging
  - [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* clock/test timing alignment
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  - [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  - [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)

Current systems map:

High-level read map:

* task-facing async/task
  - [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  - [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  - [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  - [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  - [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  - [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  - [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* tooling-facing host/runtime
  - [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  - [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  - [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  - [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  - [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  - [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
  - [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  - [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  - [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  - [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  - [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
  - [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  - [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  - [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  - [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
  - [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  - [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  - [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
  - [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
  - [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
  - [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  - [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  - [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  - [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
  - [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
* state/location/persistence
  - [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
  - [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
  - [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
  - [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
* filesystem read/write/mutate
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  - [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  - [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  - [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  - [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  - [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
  - [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  - [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  - [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  - [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  - [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
  - [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  - [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  - [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  - [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
  - [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  - [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  - [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
  - [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
  - [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
  - [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
  - [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
  - [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
  - [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* inspection/formatting
  - [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
  - [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
  - [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)

Recommended first file per group:

* task-facing async/task
  - start with [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
    because it is the narrowest observer-only task path
* tooling-facing host/runtime
  - start with [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
    because it is the smallest native host I/O path
* state/location/persistence
  - start with [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
    because it is the cleanest state-free location surface
* filesystem read/write/mutate
  - start with [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
    because it is the smallest filesystem naming surface before mutation
* inspection/formatting
  - start with [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
    because it is the narrowest filesystem inspection path

Filesystem read/write/mutate mini-map:

* naming
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  - [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  - [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  - [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  - [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  - [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
  - [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  - [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  - [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  - [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  - [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
  - [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  - [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  - [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  - [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
  - [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  - [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  - [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
* mutation
  - [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
  - [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
  - [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
  - [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
  - [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
* output
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
* inspection
  - [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

Path naming/inspection fast map:

* shape
  - [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  - [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  - [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  - [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  - [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  - [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
  - [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  - [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
* structure
  - [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  - [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
* name parts
  - [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
* matches
  - [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  - [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  - [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  - [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  - [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  - [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)

Recommended path read order:

* start with [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  for the smallest naming surface
* then read [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns) and [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  for structure
* then read [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns), [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns), and [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  for name-part extraction
* finish with the comparison helpers, especially [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns) and [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  once the extraction surfaces already feel natural

Tooling/runtime fast map:

* io
  - [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns)
  - [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
* shell and process
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
* cli and reporting
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  - [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  - [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
* location and persistence support
  - [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
  - [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
  - [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)

Recommended tooling read order:

* start with [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns)
  for the narrowest host presence/value surface
* then read [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  for the smallest host I/O surface
* then read [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns) and [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
  for interactive terminal staging
* then read [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  before the wider workflow layer
* finish with [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns) and [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  once argv/env/stdout/diagnostic grouping already feels natural

State/location/persistence fast map:

* location
  - [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
* kv
  - [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
* cache
  - [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
* config and cache bridge
  - [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)

Recommended persistence read order:

* start with [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
  for the cleanest state-free location surface
* then read [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
  for the narrowest explicit store/get path
* then read [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
  for cache-shaped persistence
* finish with [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
  once config plus persistence composition feels natural

* input/runtime
  - recipe:
    [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - single-file mirror:
    [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
  - narrow project companion:
    [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/input_runtime_demo)
* command/shell
  - recipe:
    [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - single-file mirror:
    [hello_native_command_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_command_runtime.ns)
  - narrow project companion:
    [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
* path/runtime
  - recipe:
    [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  - single-file mirror:
    [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
  - narrow project companion:
    [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_runtime_demo)
* path/is-empty
  - recipe:
    [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  - single-file mirror:
    [hello_path_is_empty_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_empty_facades.ns)
  - narrow project companion:
    [path_is_empty_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_empty_demo)
* path/is-dot
  - recipe:
    [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  - single-file mirror:
    [hello_path_is_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dot_facades.ns)
  - narrow project companion:
    [path_is_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dot_demo)
* path/is-dotdot
  - recipe:
    [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  - single-file mirror:
    [hello_path_is_dotdot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dotdot_facades.ns)
  - narrow project companion:
    [path_is_dotdot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_dotdot_demo)
* path/parent
  - recipe:
    [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - single-file mirror:
    [hello_path_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns)
  - narrow project companion:
    [path_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_demo)
* path/has-parent
  - recipe:
    [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  - single-file mirror:
    [hello_path_has_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_parent_facades.ns)
  - narrow project companion:
    [path_has_parent_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_parent_demo)
* path/depth
  - recipe:
    [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - single-file mirror:
    [hello_path_depth_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns)
  - narrow project companion:
    [path_depth_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_depth_demo)
* path/is-basename-only
  - recipe:
    [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
  - single-file mirror:
    [hello_path_is_basename_only_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_basename_only_facades.ns)
  - narrow project companion:
    [path_is_basename_only_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_basename_only_demo)
* path/basename-matches
  - recipe:
    [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  - single-file mirror:
    [hello_path_basename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_basename_matches_facades.ns)
  - narrow project companion:
    [path_basename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_basename_matches_demo)
* path/filename-matches
  - recipe:
    [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  - single-file mirror:
    [hello_path_filename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_matches_facades.ns)
  - narrow project companion:
    [path_filename_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_matches_demo)
* path/parent-matches
  - recipe:
    [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  - single-file mirror:
    [hello_path_parent_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_matches_facades.ns)
  - narrow project companion:
    [path_parent_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_parent_matches_demo)
* path/stem-matches
  - recipe:
    [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  - single-file mirror:
    [hello_path_stem_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_matches_facades.ns)
  - narrow project companion:
    [path_stem_matches_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_matches_demo)
* path/filename
  - recipe:
    [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - single-file mirror:
    [hello_path_filename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_facades.ns)
  - narrow project companion:
    [path_filename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_filename_demo)
* path/stem
  - recipe:
    [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - single-file mirror:
    [hello_path_stem_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_facades.ns)
  - narrow project companion:
    [path_stem_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_stem_demo)
* path/extension
  - recipe:
    [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - single-file mirror:
    [hello_path_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_facades.ns)
  - narrow project companion:
    [path_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_demo)
* path/has-extension
  - recipe:
    [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
  - single-file mirror:
    [hello_path_has_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_extension_facades.ns)
  - narrow project companion:
    [path_has_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_has_extension_demo)
* path/matches-extension
  - recipe:
    [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  - single-file mirror:
    [hello_path_matches_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_matches_extension_facades.ns)
  - narrow project companion:
    [path_matches_extension_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_matches_extension_demo)
* path/extension-is
  - recipe:
    [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  - single-file mirror:
    [hello_path_extension_is_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_is_facades.ns)
  - narrow project companion:
    [path_extension_is_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_extension_is_demo)
* path/starts-with-dot
  - recipe:
    [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  - single-file mirror:
    [hello_path_starts_with_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_starts_with_dot_facades.ns)
  - narrow project companion:
    [path_starts_with_dot_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_starts_with_dot_demo)
* path/is-hidden
  - recipe:
    [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
  - single-file mirror:
    [hello_path_is_hidden_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_hidden_facades.ns)
  - narrow project companion:
    [path_is_hidden_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_hidden_demo)
* path/is-relative
  - recipe:
    [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  - single-file mirror:
    [hello_path_is_relative_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_relative_facades.ns)
  - narrow project companion:
    [path_is_relative_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_relative_demo)
* path/is-root
  - recipe:
    [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  - single-file mirror:
    [hello_path_is_root_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_root_facades.ns)
  - narrow project companion:
    [path_is_root_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_is_root_demo)
* path/ends-with-slash
  - recipe:
    [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
  - single-file mirror:
    [hello_path_ends_with_slash_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_ends_with_slash_facades.ns)
  - narrow project companion:
    [path_ends_with_slash_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_ends_with_slash_demo)
* path/rename
  - recipe:
    [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
  - single-file mirror:
    [hello_path_rename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_rename_facades.ns)
  - narrow project companion:
    [path_rename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_rename_demo)
* path/copy
  - recipe:
    [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
  - single-file mirror:
    [hello_path_copy_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_copy_facades.ns)
  - narrow project companion:
    [path_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_copy_demo)
* path/remove
  - recipe:
    [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
  - single-file mirror:
    [hello_path_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_remove_facades.ns)
  - narrow project companion:
    [path_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/path_remove_demo)
* file/output
  - recipe:
    [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
  - single-file mirror:
    [hello_file_output_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_output_facades.ns)
  - narrow project companion:
    [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/file_output_demo)
* terminal/io
  - recipe:
    [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - single-file mirror:
    [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
  - narrow project companion:
    [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/terminal_io_demo)
* line-input
  - recipe:
    [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
  - single-file mirror:
    [hello_line_input_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_line_input_facades.ns)
  - narrow project companion:
    [line_input_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/line_input_demo)
* text/json
  - recipe:
    [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
  - single-file mirror:
    [hello_text_json_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_json_facades.ns)
  - narrow project companion:
    [text_json_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/text_json_demo)
* cli/runtime
  - recipe:
    [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  - single-file mirror:
    [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
  - narrow project companion:
    [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)
* result/diagnostic
  - recipe:
    [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
  - single-file mirror:
    [hello_result_diagnostic_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_diagnostic_facades.ns)
  - narrow project companion:
    [result_diagnostic_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/result_diagnostic_demo)
* report/diagnostic
  - recipe:
    [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  - single-file mirror:
    [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
  - narrow project companion:
    [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
* directory/stat
  - recipe:
    [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
  - single-file mirror:
    [hello_directory_stat_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_stat_facades.ns)
  - narrow project companion:
    [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_stat_demo)
* directory/create
  - recipe:
    [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
  - single-file mirror:
    [hello_directory_create_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_create_facades.ns)
  - narrow project companion:
    [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_create_demo)
* directory/remove
  - recipe:
    [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
  - single-file mirror:
    [hello_directory_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_remove_facades.ns)
  - narrow project companion:
    [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/directory_remove_demo)
* automation/workflow
  - recipe:
    [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  - single-file mirror:
    [hello_native_workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_workflow_runtime.ns)
  - narrow project companion:
    [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)
* location/runtime
  - recipe:
    [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
  - single-file mirror:
    [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns)
  - narrow project companion:
    [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/location_runtime_demo)
* kv/runtime
  - recipe:
    [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
  - single-file mirror:
    [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns)
  - narrow project companion:
    [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kv_runtime_demo)
* cache/runtime
  - recipe:
    [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
  - single-file mirror:
    [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns)
  - narrow project companion:
    [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cache_runtime_demo)
* config/cache
  - recipe:
    [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
  - single-file mirror:
    [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)
  - narrow project companion:
    [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/config_cache_demo)

Task-facing map:

Short task-facing summary:

* status
  [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  is the narrowest status-only observer path:
  `join_result -> task_completed/task_timed_out/task_cancelled`
* value
  [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  is the narrowest completed-only value path:
  `spawn -> join_result -> task_completed -> task_value`
* compare
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  is the narrowest direct-vs-observed comparison path:
  `spawn -> join` beside
  `spawn -> join_result -> task_completed -> task_value`
* lifecycle
  [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  is the narrowest timeout/cancel lifecycle path:
  `timeout/cancel -> join_result -> task_timed_out/task_cancelled`
* clock
  [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  adds timeout plus current clock-bridge staging to the observer path
* scheduler
  [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  adds lane-hint and monotonic-tick context without pretending `std` already
  exposes a finished executor runtime
* cli
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  adds task-facing reporting through argv/stdout/stderr/diagnostic/monotonic
  host surfaces

Recommended fast read:

* start with `status`
* then `value`
* then `compare`
* then `lifecycle`
* then `clock`
* then `scheduler`
* finish with `cli`

* [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  the smallest observer-oriented task source module:
  `spawn -> join_result -> task_completed/task_timed_out/task_cancelled -> task_value`
  Single-file companion:
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  Project companions:
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  ,
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  , and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  the narrowest status-only observer path:
  `join_result -> task_completed/task_timed_out/task_cancelled`
  Closest current companions:
  [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
  ,
  [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
  ,
  [hello_task_glm_lifecycle.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle.ns)
  ,
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  , and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  the smallest completed-only value path:
  `spawn -> join_result -> task_completed -> task_value`
  Closest current companions:
  [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
  ,
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  and
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  the narrowest direct-vs-observed comparison path:
  `spawn -> join` beside `spawn -> join_result -> task_completed -> task_value`
  Closest current companions:
  [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
  ,
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  and
  [hello_task_glm_boundary_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_boundary_compare.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  the narrowest timeout/cancel lifecycle path:
  `timeout/cancel -> join_result -> task_timed_out/task_cancelled`
  Closest current companions:
  [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
  ,
  [hello_task_glm_lifecycle.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle.ns)
  ,
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  , and
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  task observe plus timeout/clock bridge summary:
  `timeout -> join_result -> task_completed/task_timed_out`
  plus declared/resolved global clock code and host/global timing metadata
  Single-file companion:
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  Project companion:
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  lane-hint plus task observe plus host timing:
  `cpu_bind_core(0)`, `timeout`, `join_result`, `cpu_tick_i64`, `host_monotonic_time_ns`
  Single-file companion:
  [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
  Closest current project companions:
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  and
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  task observe plus CLI-facing reporting:
  `host_argv_count`, `stdout/stderr`, diagnostic emit, and monotonic timing
  Single-file companion:
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  Project companion:
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)
* [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns)
  narrow env/runtime staging:
  `env_has/env_get`
  Single-file companion:
  [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns)
  Narrow project companion:
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/env_runtime_demo)
* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  narrow native input/runtime staging:
  `argv -> file_read -> stdin_read -> tty_isatty`
  Single-file companions:
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
  and
  [hello_native_input_tool.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_input_tool.ns)
  Wider project companion:
  [native_cli_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/native_cli_pipeline_demo)
* [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  shell-oriented command/subprocess staging:
  `program/argv/env -> command_wait_exit + subprocess_join_exit`
  Single-file companion:
  [hello_native_command_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_command_runtime.ns)
  Narrow project companion:
  [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
  Wider project companion:
  [native_tool_runner_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/native_tool_runner_demo)
* [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  narrow report/diagnostic staging:
  `path/fs/json -> diag_emit + stdout`
  Single-file companion:
  [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
  Narrow project companion:
  [report_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/report_runtime_demo)
* [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  narrow automation/workflow staging:
  `cwd/temp/cache -> subprocess + monotonic time`
  Single-file companion:
  [hello_native_workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_workflow_runtime.ns)
  Narrow project companion:
  [automation_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/automation_runtime_demo)
* [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  narrow CLI/runtime staging:
  `argv/env/cwd/config/cache -> stdout + diag + monotonic time`
  Single-file companion:
  [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
  Narrow project companion:
  [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/cli_runtime_demo)

Recommended reading order for the current task-facing `std` line:

* start with [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  to read the smallest observer-oriented task contract first
* continue to [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  when you want the narrowest status-only observation path
* continue to [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  when you want the narrowest completed-only payload extraction path
* continue to [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  when you want the smallest direct-vs-observed task-path comparison
* continue to [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  when you want the narrowest timeout/cancel lifecycle path
* continue to [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  when you want timeout plus clock bridge semantics
* then read [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  when you want lane-hint plus monotonic-tick context
* finish with [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  when you want the task/tooling front-door reporting shape

Recommended example route for the same line:

* single-file source mirrors:
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  and
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
* project-shaped companions:
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_completed_observe_demo)
  ,
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_compare_observe_demo)
  ,
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_lifecycle_branch_demo)
  ,
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cancel_branch_demo)
  , and
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task_cli_tooling_demo)

Current task-facing boundaries by reading stage:

* [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  is the right place to learn the current observer-oriented task contract, but
  it should not be read as a promise that `Task<T>` already has final GLM
  ownership semantics or a finished native concurrency runtime
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  is the right place to learn the narrow status-only observer path, but it
  should not be read as a promise that those status observations already imply
  final lifetime-end or consuming GLM semantics
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  is the right place to learn the narrow completed-only value path, but it
  should not be read as a promise that `join(...)` or `join_result(...)` have
  already been frozen into final consuming ownership semantics
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  is the right place to compare the current direct and observed task paths, but
  it should not be read as a promise that the present non-consuming `join(...)`
  contract is final
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  is the right place to learn the current timeout/cancel observation path, but
  it should not be read as a promise that cancellation or timeout already carry
  final lifetime-end semantics in GLM or runtime
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  is the right place to learn timeout plus clock bridge staging, but it should
  not be read as a promise that cross-domain time negotiation has already been
  finalized beyond the current `global -> monotonic` front-door contract
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  is the right place to learn lane-hint plus monotonic tick context, but it
  should not be read as a promise that `std` already exposes a mature executor,
  fairness contract, or parallel scheduler runtime
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  is the right place to learn task/tooling reporting shape, but it should not
  be read as a promise that async task execution, timeout handling, and host
  reporting are already unified into a fully live native tooling runtime

Current boundaries:

* `std` is not yet a populated importable source module tree
* `std` is still much thinner than the future practical systems layer it is
  meant to become
* a first small native-backed AOT batch now exists in the compiler shim for:
  `argv`, `env`, `cwd`, `stdout/stderr`, `host_text_len`, basic
  `path/fs/stat`, `file/stdin/tty`, `directory/temp`, simple
  `process/command/subprocess`, and `wall/monotonic/sleep` time helpers, so
  those facades are beginning to become real host integration rather than pure
  placeholder shape
* current string/file direction is still host-backed and opaque; there is not
  yet a first-class native `String` or file-path/file-descriptor standard type
* a small native-backed `host_text_concat` bridge now exists for staging
  shell-oriented command text assembly; it is intentionally narrow and is not
  yet a full native string model
* current CLI-facing `argv/env/process` direction is also still host-backed and
  opaque; it is a facade over handles and status integers, not yet a stable
  native command-line/runtime API
* current `path/stdout/stderr` direction is likewise still host-backed and
  opaque; it is useful as a staging facade, but not yet a finalized portable
  path/stream standard library contract
* current `time/clock` direction is intentionally split between basic
  wall/monotonic time facades and a more explicit global clock-scale contract;
  both are still host-backed staging APIs rather than a finalized cross-domain
  timing standard
* current clock/test alignment also includes a small recipe that mirrors the
  front-door runner's current `global -> monotonic` timeout resolution; this
  is a staging contract for async test semantics rather than a final cross-domain
  clock bridge
* `clock_domain_runtime.ns` is the current canonical place where the staging
  clock-domain code mapping is written down explicitly: `0 = monotonic`,
  `1 = wall`, `2 = global`, with `global` currently resolving to `monotonic`
  inside the front-door test runner
* `clock_test_recipe.ns` and the FFI-facing clock test example now mirror that
  canonical mapping using the more explicit field names
  `declared_global_code` and `resolved_global_code`, so the bridge summary reads
  like a contract rather than a pair of unexplained integers
* current `command/subprocess` direction is likewise a host-backed facade over
  opaque process handles and integer status codes; it is useful for CLI/tooling
  experiments but not yet a finalized portable process-management contract;
  the current native-backed AOT shim now treats `program_handle` as the main
  shell command text, `argv_handle` as a raw shell argument-tail text handle,
  and `env_handle` as a raw `KEY=VALUE ...` environment-prefix text handle;
  [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  is the canonical source-level staging recipe for that bridge, and native
  direct-exit observers now exist for `command_wait_exit` and
  `subprocess_join_exit`
* current `metadata/directory/stat` direction is also still a host-backed file
  system facade; it is enough to sketch CLI and tooling flows, but not yet a
  finalized portable filesystem standard library contract
* current `stdin/line-input/tty` direction is also still a host-backed terminal
  facade; it is enough to sketch command-line interaction flows, but not yet a
  finalized portable terminal standard library contract
* current compiler-facing host-read recognition is narrower than the full `std`
  facade surface; only a small builtin set is currently classified as
  `HostReadOnly`, while most explicit `host_*` facade calls still lower through
  conservative `cpu.extern_call_*` paths
* current task-facing `std` direction is intentionally still value-like and
  observer-oriented; [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  , [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns),
  [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns),
  and [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  mirror the current `spawn/join_result/task_*` guidance plus timeout/clock,
  lane-hint, and task-facing CLI reporting shape, but they should not be read
  as a promise of a finished native concurrency runtime or executor standard
  library
* current `json/text-format` direction is also still a host-backed formatting
  facade; it is enough to sketch machine-readable and human-readable output
  flows, but not yet a finalized native text/serialization standard library
  contract
* current `cwd/temp/home` direction is likewise still a host-backed environment
  facade; it is enough to sketch common tooling location flows, but not yet a
  finalized portable runtime-environment standard library contract
* current `error/result/diagnostic` direction is also still a host-backed
  reporting facade; it is enough to sketch tool-facing failure and reporting
  flows, but not yet a finalized native error/diagnostic standard library
  contract
* current `config/kv/cache` direction is also still a host-backed persistence
  facade; it is enough to sketch tool-facing state and lookup flows, but not
  yet a finalized portable configuration/storage standard library contract

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/std/module.toml)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
