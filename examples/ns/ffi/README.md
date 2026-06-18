# FFI `.ns` Examples

This folder contains source-level CPU host-bridge examples.

Use it for:

* narrow `extern "c"` and `extern "nurs"` host facade shapes
* source-level mirrors of current `std` host/runtime recipes
* small focused probes before moving to project-form examples

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file first when you want the shortest current route.
* [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
  Use that file when the question is whether a facade example is still a
  frontdoor anchor or only companion detail.

Current role rule:

* this subtree is a narrow source-side facade mirror layer
* it should not compete with project-form tooling/filesystem/task onboarding
* before `alpha-0.0.1`, the goal is to keep one short host bridge ladder, one
  short task/runtime ladder, and one short path/runtime ladder obvious

## Current Frontdoor Ladders

If you only want the shortest current FFI-side route, start with these ladders.

Host bridge ladder:

* [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
* [hello_c_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns)

Task/runtime ladder:

* [hello_task_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns)
* [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
* [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)

Path/runtime ladder:

* [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
* [hello_file_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_runtime_facades.ns)
* [hello_directory_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_runtime_facades.ns)

## Companion Detail Map

Use these as the shortest current `recipe -> facade` mirrors.

* task-facing
  - [hello_task_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns)
  - [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  - [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns)
  - [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns)
  - [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  - [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
* host I/O and tooling
  - [hello_argv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns)
  - [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns)
  - [hello_process_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns)
  - [hello_command_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_command_runtime_facades.ns)
  - [hello_subprocess_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_subprocess_runtime_facades.ns)
  - [hello_host_text_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns)
  - [hello_json_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_json_runtime_facades.ns)
  - [hello_text_format_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_format_runtime_facades.ns)
  - [hello_error_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_error_runtime_facades.ns)
  - [hello_result_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_runtime_facades.ns)
  - [hello_diagnostic_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_diagnostic_runtime_facades.ns)
  - [hello_time_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_time_runtime_facades.ns)
  - [hello_sleep_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_sleep_runtime_facades.ns)
  - [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns)
  - [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns)
  - [hello_stdin_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stdin_runtime_facades.ns)
  - [hello_tty_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_tty_runtime_facades.ns)
  - [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
  - [hello_io_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns)
  - [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
  - [hello_native_command_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_command_runtime.ns)
* state / location / persistence
  - [hello_cwd_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns)
  - [hello_temp_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns)
  - [hello_home_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_home_runtime_facades.ns)
  - [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns)
  - [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns)
  - [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns)
  - [hello_config_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_runtime_facades.ns)
  - [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)
* path / filesystem
  - [hello_window_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_window_runtime_facades.ns)
  - [hello_pipe_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_pipe_runtime_facades.ns)
  - [hello_fabric_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_fabric_runtime_facades.ns)
  - [hello_handle_table_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_handle_table_runtime_facades.ns)
  - [hello_directory_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_runtime_facades.ns)
  - [hello_file_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_runtime_facades.ns)
  - [hello_fs_metadata_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_fs_metadata_runtime_facades.ns)
  - [hello_stat_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stat_runtime_facades.ns)
  - [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
  - [hello_path_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns)
  - [hello_path_depth_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns)
  - [hello_path_filename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_facades.ns)
  - [hello_path_stem_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_facades.ns)
  - [hello_path_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_facades.ns)
  - [hello_file_output_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_output_facades.ns)
  - [hello_directory_create_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_create_facades.ns)
  - [hello_directory_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_remove_facades.ns)
  - [hello_directory_stat_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_stat_facades.ns)

## Reading Rule

* use the frontdoor ladders first
* use the companion detail map after you know which facade lane you care about
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  when you want recipe-side grouping
* use [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  when you want project-form companions
* treat the long tail of narrower path-specific facade files as local detail
  unless you are actively working in that subsystem

## Notes

* `mod` is a top-level builtin declaration, not a nested construct
* `cpu` is currently the only domain that can declare `async fn`
