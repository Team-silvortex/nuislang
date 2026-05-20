# FFI `.ns` Examples

This folder contains CPU host-bridge examples:

* `hello_ffi.ns`
* `hello_c_ffi.ns`
* `hello_cli_host_facades.ns`
* `hello_result_diagnostic_facades.ns`
* `hello_native_command_runtime.ns`
* `hello_argv_runtime_facades.ns`
* `hello_path_runtime_facades.ns`
* `hello_path_is_empty_facades.ns`
* `hello_path_is_dot_facades.ns`
* `hello_path_is_dotdot_facades.ns`
* `hello_path_parent_facades.ns`
* `hello_path_has_parent_facades.ns`
* `hello_path_depth_facades.ns`
* `hello_path_is_basename_only_facades.ns`
* `hello_path_basename_matches_facades.ns`
* `hello_path_filename_matches_facades.ns`
* `hello_path_parent_matches_facades.ns`
* `hello_path_stem_matches_facades.ns`
* `hello_path_filename_facades.ns`
* `hello_path_stem_facades.ns`
* `hello_path_extension_facades.ns`
* `hello_path_has_extension_facades.ns`
* `hello_path_matches_extension_facades.ns`
* `hello_path_extension_is_facades.ns`
* `hello_path_starts_with_dot_facades.ns`
* `hello_path_is_hidden_facades.ns`
* `hello_path_is_relative_facades.ns`
* `hello_path_is_root_facades.ns`
* `hello_path_ends_with_slash_facades.ns`
* `hello_path_rename_facades.ns`
* `hello_path_copy_facades.ns`
* `hello_path_remove_facades.ns`
* `hello_file_output_facades.ns`
* `hello_line_input_facades.ns`
* `hello_terminal_io_facades.ns`
* `hello_env_runtime_facades.ns`
* `hello_process_runtime_facades.ns`
* `hello_text_json_facades.ns`
* `hello_input_runtime_facades.ns`
* `hello_native_workflow_runtime.ns`
* `hello_clock_test_facades.ns`
* `hello_task_scheduler_facades.ns`
* `hello_task_cli_facades.ns`
* `hello_cwd_runtime_facades.ns`
* `hello_temp_runtime_facades.ns`
* `hello_home_runtime_facades.ns`
* `hello_directory_create_facades.ns`
* `hello_directory_remove_facades.ns`
* `hello_directory_stat_facades.ns`
* `hello_location_runtime_facades.ns`
* `hello_kv_runtime_facades.ns`
* `hello_cache_runtime_facades.ns`
* `hello_config_runtime_facades.ns`
* `hello_config_cache_facades.ns`

Reading guidance:

* `hello_ffi.ns`
  current `extern "nurs" interface`-style host bridge
* `hello_c_ffi.ns`
  plain `extern "c"` route kept as the lower-level baseline
* `hello_cli_host_facades.ns`
  a tooling-oriented `extern "c"` example that groups argv/env/cwd/stdout/diagnostic
  style host facades in one place; it now mirrors both
  [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  ,
  [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  , and
  [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  from the current `stdlib/std` host-backed tooling direction
* `hello_result_diagnostic_facades.ns`
  a narrower result/diagnostic facade example that mirrors
  [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
  and keeps `result_is_ok/value/error` plus `error_code/message/severity` and
  `diag_label/span/emit` on their own source-level staging path
* `hello_native_command_runtime.ns`
  a focused native-backed command example that shows the current early
  `command/subprocess` staging contract directly:
  `program_handle <- argv`, `argv_handle <- shell-style argv tail built from
  multiple source arguments`, `env_handle <- KEY=VALUE prefix text`; it is the
  repo-local example that most directly mirrors
  [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  and now has the narrower project-shaped companion
  [command_shell_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/command_shell_demo)
* `hello_process_runtime_facades.ns`
  a narrower process/runtime facade example that mirrors
  [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns)
  and keeps `process_id/status/exit_code` on their own source-level staging
  path
* `hello_stdin_runtime_facades.ns`
  a narrower stdin/runtime facade example that mirrors
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  and keeps repeated `stdin_read` observation on its own source-level staging
  path
* `hello_argv_runtime_facades.ns`
  a narrower argv/runtime facade example that mirrors
  [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns)
  and keeps `argv_count -> argv_at(0/1)` on their own source-level staging path
* `hello_env_runtime_facades.ns`
  a narrower env/runtime facade example that mirrors
  [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns)
  and keeps `env_has/env_get` on their own source-level staging path
* `hello_path_runtime_facades.ns`
  a narrower path/runtime facade example that mirrors
  [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  and keeps `path_join/is_absolute/basename` on their own source-level staging path
* `hello_path_is_empty_facades.ns`
  a narrower path/is-empty facade example that mirrors
  [path_is_empty_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_empty_recipe.ns)
  and keeps `path_is_empty/path_is_absolute/path_is_relative` on their own
  source-level staging path
* `hello_path_is_dot_facades.ns`
  a narrower path/is-dot facade example that mirrors
  [path_is_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dot_recipe.ns)
  and keeps `path_is_empty/path_is_dot/path_is_relative` on their own
  source-level staging path
* `hello_path_is_dotdot_facades.ns`
  a narrower path/is-dotdot facade example that mirrors
  [path_is_dotdot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_dotdot_recipe.ns)
  and keeps `path_is_empty/path_is_dotdot/path_is_relative` on their own
  source-level staging path
* `hello_path_parent_facades.ns`
  a narrower path/parent facade example that mirrors
  [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  and keeps `path_parent/is_absolute/basename` on their own source-level
  staging path
* `hello_path_has_parent_facades.ns`
  a narrower path/has-parent facade example that mirrors
  [path_has_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_parent_recipe.ns)
  and keeps `path_parent/path_has_parent/path_depth` on their own source-level
  staging path
* `hello_path_depth_facades.ns`
  a narrower path/depth facade example that mirrors
  [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  and keeps `path_parent/path_depth/is_absolute` on their own source-level
  staging path
* `hello_path_is_basename_only_facades.ns`
  a narrower path/is-basename-only facade example that mirrors
  [path_is_basename_only_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_basename_only_recipe.ns)
  and keeps `path_is_empty/path_basename/path_is_basename_only` on their own
  source-level staging path
* `hello_path_basename_matches_facades.ns`
  a narrower path/basename-matches facade example that mirrors
  [path_basename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_basename_matches_recipe.ns)
  and keeps `path_basename/path_is_basename_only/path_basename_matches` on
  their own source-level staging path
* `hello_path_filename_matches_facades.ns`
  a narrower path/filename-matches facade example that mirrors
  [path_filename_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_matches_recipe.ns)
  and keeps `path_filename/path_extension/path_filename_matches` on their own
  source-level staging path
* `hello_path_parent_matches_facades.ns`
  a narrower path/parent-matches facade example that mirrors
  [path_parent_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_matches_recipe.ns)
  and keeps `path_parent/path_has_parent/path_parent_matches` on their own
  source-level staging path
* `hello_path_stem_matches_facades.ns`
  a narrower path/stem-matches facade example that mirrors
  [path_stem_matches_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_matches_recipe.ns)
  and keeps `path_stem/path_extension/path_stem_matches` on their own
  source-level staging path
* `hello_path_filename_facades.ns`
  a narrower path/filename facade example that mirrors
  [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  and keeps `path_filename/path_stem/path_extension` on their own
  source-level staging path
* `hello_path_stem_facades.ns`
  a narrower path/stem facade example that mirrors
  [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  and keeps `path_parent/path_stem/is_absolute` on their own source-level
  staging path
* `hello_path_extension_facades.ns`
  a narrower path/extension facade example that mirrors
  [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  and keeps `path_stem/path_extension/is_absolute` on their own source-level
  staging path
* `hello_path_has_extension_facades.ns`
  a narrower path/has-extension facade example that mirrors
  [path_has_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_has_extension_recipe.ns)
  and keeps `path_stem/path_extension/path_has_extension` on their own
  source-level staging path
* `hello_path_matches_extension_facades.ns`
  a narrower path/matches-extension facade example that mirrors
  [path_matches_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_matches_extension_recipe.ns)
  and keeps `path_extension/path_has_extension/path_matches_extension` on their
  own source-level staging path
* `hello_path_extension_is_facades.ns`
  a narrower path/extension-is facade example that mirrors
  [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
  and keeps `path_extension/path_has_extension/path_extension_is` on their own
  source-level staging path
* `hello_path_starts_with_dot_facades.ns`
  a narrower path/starts-with-dot facade example that mirrors
  [path_starts_with_dot_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_starts_with_dot_recipe.ns)
  and keeps `path_basename/path_starts_with_dot/path_is_hidden` on their own
  source-level staging path
* `hello_path_is_hidden_facades.ns`
  a narrower path/is-hidden facade example that mirrors
  [path_is_hidden_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_hidden_recipe.ns)
  and keeps `path_basename/path_extension/path_is_hidden` on their own
  source-level staging path
* `hello_path_is_relative_facades.ns`
  a narrower path/is-relative facade example that mirrors
  [path_is_relative_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_relative_recipe.ns)
  and keeps `path_is_absolute/path_is_relative/path_basename` on their own
  source-level staging path
* `hello_path_is_root_facades.ns`
  a narrower path/is-root facade example that mirrors
  [path_is_root_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_is_root_recipe.ns)
  and keeps `path_is_absolute/path_is_root/path_parent` on their own
  source-level staging path
* `hello_path_ends_with_slash_facades.ns`
  a narrower path/ends-with-slash facade example that mirrors
  [path_ends_with_slash_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_ends_with_slash_recipe.ns)
  and keeps `path_is_root/path_ends_with_slash/path_depth` on their own
  source-level staging path
* `hello_path_rename_facades.ns`
  a narrower path/rename facade example that mirrors
  [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
  and keeps `temp_file_handle -> path_rename -> fs_exists` on their own
  source-level staging path
* `hello_path_copy_facades.ns`
  a narrower path/copy facade example that mirrors
  [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
  and keeps `temp source -> file_write -> path_copy -> fs_exists` on their own
  source-level staging path
* `hello_path_remove_facades.ns`
  a narrower path/remove facade example that mirrors
  [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
  and keeps `temp_file_handle -> path_remove -> fs_exists` on their own
  source-level staging path
* `hello_file_output_facades.ns`
  a narrower file/output facade example that mirrors
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
  and keeps `temp_file_handle -> file_open/write/close` on their own
  source-level staging path

Path facade fast map:

* shape
  - `hello_path_is_empty_facades.ns`
  - `hello_path_is_dot_facades.ns`
  - `hello_path_is_dotdot_facades.ns`
  - `hello_path_is_relative_facades.ns`
  - `hello_path_is_root_facades.ns`
  - `hello_path_ends_with_slash_facades.ns`
  - `hello_path_starts_with_dot_facades.ns`
  - `hello_path_is_hidden_facades.ns`
* structure
  - `hello_path_parent_facades.ns`
  - `hello_path_has_parent_facades.ns`
  - `hello_path_depth_facades.ns`
  - `hello_path_is_basename_only_facades.ns`
* name parts
  - `hello_path_filename_facades.ns`
  - `hello_path_stem_facades.ns`
  - `hello_path_extension_facades.ns`
  - `hello_path_has_extension_facades.ns`
* matches
  - `hello_path_basename_matches_facades.ns`
  - `hello_path_filename_matches_facades.ns`
  - `hello_path_parent_matches_facades.ns`
  - `hello_path_stem_matches_facades.ns`
  - `hello_path_matches_extension_facades.ns`
  - `hello_path_extension_is_facades.ns`

Tooling facade fast map:

* io
  - `hello_argv_runtime_facades.ns`
  - `hello_env_runtime_facades.ns`
  - `hello_process_runtime_facades.ns`
  - `hello_stdin_runtime_facades.ns`
  - `hello_tty_runtime_facades.ns`
  - `hello_input_runtime_facades.ns`
  - `hello_terminal_io_facades.ns`
  - `hello_line_input_facades.ns`
  - `hello_file_output_facades.ns`
* shell and process
  - `hello_native_command_runtime.ns`
  - `hello_native_workflow_runtime.ns`
* cli and reporting
  - `hello_cli_host_facades.ns`
  - `hello_result_diagnostic_facades.ns`

State/persistence facade fast map:

* location
  - `hello_location_runtime_facades.ns`
* kv
  - `hello_kv_runtime_facades.ns`
* cache
  - `hello_cache_runtime_facades.ns`
* config and cache bridge
  - `hello_config_cache_facades.ns`
* `hello_line_input_facades.ns`
  a narrower line-input facade example that mirrors
  [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
  and keeps `line_read/line_len` on their own source-level staging path
* `hello_terminal_io_facades.ns`
  a narrower terminal/io facade example that mirrors
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  and keeps `stdout/stderr/stdin/tty` on their own source-level staging path
* `hello_text_json_facades.ns`
  a narrower text/json facade example that mirrors
  [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
  and keeps `text_len/concat/measure` plus `json_pair/object/array` on their
  own source-level staging path
* `hello_input_runtime_facades.ns`
  a narrower `argv/file/stdin/tty` facade example that mirrors
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  and keeps the current native input/runtime staging separate from the wider
  command/process pipeline examples
* `hello_stdin_runtime_facades.ns`
  a narrower stdin/runtime facade example that mirrors
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  and keeps repeated `stdin_read` observation separate from the wider
  `argv/file/stdin/tty` staging path
* `hello_tty_runtime_facades.ns`
  a narrower tty/runtime facade example that mirrors
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
  and keeps `isatty/width/height` observation on its own source-level staging
  path
* `hello_native_workflow_runtime.ns`
  a native-backed workflow example that leans on the current AOT shim batch for
  `cwd/directory/temp/process/command/subprocess/stdout`, so it is the best
  repo-local source example when you want to see how the current
  [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  direction starts to touch real host workflow primitives; note that the
  current `command/subprocess` path still uses a small shell-oriented staging
  contract where `argv_handle` is a raw argument-tail text handle and
  `env_handle` is a raw environment-prefix text handle
* `hello_clock_test_facades.ns`
  a clock/timing-oriented `extern "c"` example that mirrors
  [clock_domain_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime.ns)
  ,
  [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  and the current `nuis test` time semantics; inside the task-facing `std`
  line it is also the narrowest single-file mirror for
  [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns).
  It includes a `should_fail=true` async test with `clock_domain="global"` so
  the front-door runner prints the resolved host clock domain during execution
  Future direction note:
  [examples/ns/ffi/FUTURE_CLOCK_NEGOTIATION_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/FUTURE_CLOCK_NEGOTIATION_SKETCH.md)
* `hello_task_scheduler_facades.ns`
  a task/scheduler-oriented `extern "c"` example that mirrors
  [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  and combines `cpu_bind_core(0)`, `cpu_tick_i64`, `timeout`, `join_result`,
  `task_completed`, and monotonic host timing in one source-level example
* `hello_task_cli_facades.ns`
  a task/tooling-oriented `extern "c"` example that mirrors
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  from the current `stdlib/std` task-facing recipe family; it combines
  `spawn/timeout/join_result/task_*` with stdout/stderr, diagnostic emit, and
  monotonic host timing in one source-level example
* `hello_config_cache_facades.ns`
  a narrower config/cache facade example that mirrors
  [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
  and keeps `config_open/get/close` plus `cache_open/lookup/store/close` on
  their own source-level staging path

* `hello_directory_create_facades.ns`
  a narrower directory/create facade example that mirrors
  [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
  and keeps `temp_file_handle -> dir_create -> fs_exists` on their own
  source-level staging path
* `hello_directory_remove_facades.ns`
  a narrower directory/remove facade example that mirrors
  [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
  and keeps `temp_file_handle -> dir_create -> dir_remove -> fs_exists` on
  their own source-level staging path
* `hello_directory_stat_facades.ns`
  a narrower directory/stat facade example that mirrors
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
  and keeps `dir_open/entry_count/close` plus `fs/stat` inspection on their
  own source-level staging path
* `hello_location_runtime_facades.ns`
  a narrower location/runtime facade example that mirrors
  [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
  and keeps `cwd/temp/home/config-dir` on their own source-level staging path
* `hello_cwd_runtime_facades.ns`
  a narrower cwd/runtime facade example that mirrors
  [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns)
  and keeps `cwd_handle/cwd_len/chdir` on their own source-level staging path
* `hello_temp_runtime_facades.ns`
  a narrower temp/runtime facade example that mirrors
  [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns)
  and keeps `temp_dir/temp_path_len/temp_file_handle` on their own
  source-level staging path
* `hello_home_runtime_facades.ns`
  a narrower home/runtime facade example that mirrors
  [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns)
  and keeps `home_dir/home_len/config_dir` on their own source-level staging
  path
* `hello_kv_runtime_facades.ns`
  a narrower kv/runtime facade example that mirrors
  [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
  and keeps `kv_open/put/get/close` on their own source-level staging path
* `hello_cache_runtime_facades.ns`
  a narrower cache/runtime facade example that mirrors
  [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
  and keeps `cache_open/lookup/store/close` on their own source-level staging path
* `hello_config_runtime_facades.ns`
  a narrower config/runtime facade example that mirrors
  [config_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime_recipe.ns)
  and keeps `config_open/get/close` on their own source-level staging path

Systems mirror map:

Filesystem mini-map:

* naming
  - [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
  - [hello_path_is_empty_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_empty_facades.ns)
  - [hello_path_is_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dot_facades.ns)
  - [hello_path_is_dotdot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dotdot_facades.ns)
  - [hello_path_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns)
  - [hello_path_has_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_parent_facades.ns)
  - [hello_path_depth_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns)
  - [hello_path_is_basename_only_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_basename_only_facades.ns)
  - [hello_path_basename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_basename_matches_facades.ns)
  - [hello_path_filename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_matches_facades.ns)
  - [hello_path_parent_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_matches_facades.ns)
  - [hello_path_stem_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_matches_facades.ns)
  - [hello_path_filename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_facades.ns)
  - [hello_path_stem_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_facades.ns)
  - [hello_path_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_facades.ns)
  - [hello_path_has_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_extension_facades.ns)
  - [hello_path_matches_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_matches_extension_facades.ns)
  - [hello_path_extension_is_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_is_facades.ns)
  - [hello_path_starts_with_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_starts_with_dot_facades.ns)
  - [hello_path_is_hidden_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_hidden_facades.ns)
  - [hello_path_is_relative_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_relative_facades.ns)
  - [hello_path_is_root_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_root_facades.ns)
  - [hello_path_ends_with_slash_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_ends_with_slash_facades.ns)
* mutation
  - [hello_path_rename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_rename_facades.ns)
  - [hello_path_copy_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_copy_facades.ns)
  - [hello_path_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_remove_facades.ns)
  - [hello_directory_create_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_create_facades.ns)
  - [hello_directory_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_remove_facades.ns)
* output
  - [hello_file_output_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_output_facades.ns)
* inspection
  - [hello_directory_stat_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_stat_facades.ns)

* input/runtime
  - [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
* command/shell
  - [hello_native_command_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_command_runtime.ns)
* path/runtime
  - [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
* path/is-empty
  - [hello_path_is_empty_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_empty_facades.ns)
* path/is-dot
  - [hello_path_is_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dot_facades.ns)
* path/is-dotdot
  - [hello_path_is_dotdot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_dotdot_facades.ns)
* path/parent
  - [hello_path_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns)
* path/has-parent
  - [hello_path_has_parent_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_parent_facades.ns)
* path/depth
  - [hello_path_depth_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns)
* path/is-basename-only
  - [hello_path_is_basename_only_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_basename_only_facades.ns)
* path/basename-matches
  - [hello_path_basename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_basename_matches_facades.ns)
* path/filename-matches
  - [hello_path_filename_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_matches_facades.ns)
* path/parent-matches
  - [hello_path_parent_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_matches_facades.ns)
* path/stem-matches
  - [hello_path_stem_matches_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_matches_facades.ns)
* path/filename
  - [hello_path_filename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_filename_facades.ns)
* path/stem
  - [hello_path_stem_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_stem_facades.ns)
* path/extension
  - [hello_path_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_facades.ns)
* path/has-extension
  - [hello_path_has_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_has_extension_facades.ns)
* path/matches-extension
  - [hello_path_matches_extension_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_matches_extension_facades.ns)
* path/extension-is
  - [hello_path_extension_is_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_extension_is_facades.ns)
* path/starts-with-dot
  - [hello_path_starts_with_dot_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_starts_with_dot_facades.ns)
* path/is-hidden
  - [hello_path_is_hidden_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_hidden_facades.ns)
* path/is-relative
  - [hello_path_is_relative_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_relative_facades.ns)
* path/is-root
  - [hello_path_is_root_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_is_root_facades.ns)
* path/ends-with-slash
  - [hello_path_ends_with_slash_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_ends_with_slash_facades.ns)
* path/rename
  - [hello_path_rename_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_rename_facades.ns)
* path/copy
  - [hello_path_copy_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_copy_facades.ns)
* path/remove
  - [hello_path_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_remove_facades.ns)
* file/output
  - [hello_file_output_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_output_facades.ns)
* line-input
  - [hello_line_input_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_line_input_facades.ns)
* terminal/io
  - [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
* text/json
  - [hello_text_json_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_json_facades.ns)
* cli/runtime
  - [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
* result/diagnostic
  - [hello_result_diagnostic_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_diagnostic_facades.ns)
* report/diagnostic
  - [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
* directory/create
  - [hello_directory_create_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_create_facades.ns)
* directory/remove
  - [hello_directory_remove_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_remove_facades.ns)
* directory/stat
  - [hello_directory_stat_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_stat_facades.ns)
* automation/workflow
  - [hello_native_workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_native_workflow_runtime.ns)
* cwd/runtime
  - [hello_cwd_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns)
* temp/runtime
  - [hello_temp_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns)
* home/runtime
  - [hello_home_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_home_runtime_facades.ns)
* location/runtime
  - [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns)
* kv/runtime
  - [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns)
* cache/runtime
  - [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns)
* config/runtime
  - [hello_config_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_runtime_facades.ns)
* config/cache
  - [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)

Task-facing recipe map:

* [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  and
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  are the closest direct mirrors for
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  is the closest direct mirror for
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  and that file is the current narrowest single-file clock companion in the
  task-facing `std` sequence
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  is the closest direct mirror for
  [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  is the closest direct mirror for
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
* [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  is the closest direct mirror for
  [hello_stdin_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stdin_runtime_facades.ns)
* [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
  is the closest direct mirror for
  [hello_tty_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_tty_runtime_facades.ns)

Recommended reading order for the task-facing FFI examples:

* start with
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  to read the smallest task/tooling observer path
* then read
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  when you want the task/clock bridge and timeout-facing timing vocabulary
  in its narrowest single-file form
* then read
  [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
  when you want the narrowest lane-hint plus monotonic-tick task path
* finish with
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  when you want task/tooling reporting on top of those earlier shapes

Current task-facing example boundaries:

* [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  is the clearest source-level mirror for current task/tooling observation
  shape, but it is still a host-facade example rather than a promise of a live
  native task executor path
* [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  is the clearest source-level mirror for current timeout/clock bridge
  vocabulary, but it still reflects staging metadata rather than a finalized
  multi-domain time negotiation protocol
* [hello_task_scheduler_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_scheduler_facades.ns)
  is the clearest source-level mirror for current lane-hint plus monotonic-tick
  task context, but it should not be read as a promise that `std` already
  exposes a mature executor or fairness-aware scheduler runtime

Current note:

* the source language already distinguishes the Rust-oriented `NURS` surface from the raw C ABI bridge, even though today the concrete bridge is still C-compatible underneath
