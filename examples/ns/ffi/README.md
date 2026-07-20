# FFI `.ns` Examples

This folder contains source-level CPU host-bridge examples.

Use it for:

* narrow `extern "c"` and `extern "nurs"` host facade shapes
* scalar host ABI width probes, currently `i64` and `i32`
* std-owned `@host_symbol("...")` logical host bridge declarations
* source-level mirrors of current `std` host/runtime recipes
* small focused probes before moving to project-form examples

Canonical short map:

* [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
  Use that file first when you want the shortest current route.
* [docs/examples-freshness-audit.md](../../../docs/examples-freshness-audit.md)
  Use that file when the question is whether a facade example is still a
  frontdoor anchor or only companion detail.

Current role rule:

* this subtree is a narrow source-side facade mirror layer
* it should not compete with project-form tooling/filesystem/task onboarding
* during `alpha-0.4.*`, the goal is to keep one short host bridge ladder, one
  short task/runtime ladder, and one short path/runtime ladder obvious while
  project-form examples carry the broader workflow story
* when a facade belongs to the std-owned host surface, prefer
  `extern "c" @host_symbol("...") fn ...;` over hard-coding raw `host_*`
  symbol names in frontdoor examples

## Current Frontdoor Ladders

If you only want the shortest current FFI-side route, start with these ladders.

Host bridge ladder:

* [hello_ffi.ns](hello_ffi.ns)
* [hello_c_ffi.ns](hello_c_ffi.ns)
* [hello_c_i32_ffi.ns](hello_c_i32_ffi.ns)
* [libc_usleep_demo.ns](libc_usleep_demo.ns)
* [libc_puts_demo.ns](libc_puts_demo.ns)
* [libc_strlen_demo.ns](libc_strlen_demo.ns)
* [libc_write_demo.ns](libc_write_demo.ns)
* [libc_close_demo.ns](libc_close_demo.ns)
* [libc_read_buffer_demo.ns](libc_read_buffer_demo.ns)

Runnable libc smoke:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_usleep_demo.ns "$TMPDIR/nuis_libc_usleep_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_usleep_demo"
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_puts_demo.ns "$TMPDIR/nuis_libc_puts_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_puts_demo"
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_strlen_demo.ns "$TMPDIR/nuis_libc_strlen_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_strlen_demo"
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_write_demo.ns "$TMPDIR/nuis_libc_write_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_write_demo"
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_close_demo.ns "$TMPDIR/nuis_libc_close_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_close_demo"
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/ns/ffi/libc_read_buffer_demo.ns "$TMPDIR/nuis_libc_read_buffer_demo"
cargo run -p nuis -- run-artifact "$TMPDIR/nuis_libc_read_buffer_demo"
```

Task/runtime ladder:

* [hello_task_runtime_facades.ns](hello_task_runtime_facades.ns)
* [hello_task_cli_facades.ns](hello_task_cli_facades.ns)
* [hello_input_runtime_facades.ns](hello_input_runtime_facades.ns)

Path/runtime ladder:

* [hello_path_runtime_facades.ns](hello_path_runtime_facades.ns)
* [hello_file_runtime_facades.ns](hello_file_runtime_facades.ns)
* [hello_directory_runtime_facades.ns](hello_directory_runtime_facades.ns)

## Companion Detail Map

Use these as the shortest current `recipe -> facade` mirrors.

* task-facing
  - [hello_task_runtime_facades.ns](hello_task_runtime_facades.ns)
  - [hello_task_cli_facades.ns](hello_task_cli_facades.ns)
  - [hello_clock_runtime_facades.ns](hello_clock_runtime_facades.ns)
  - [hello_clock_domain_runtime_facades.ns](hello_clock_domain_runtime_facades.ns)
  - [hello_clock_test_facades.ns](hello_clock_test_facades.ns)
  - [hello_task_scheduler_facades.ns](hello_task_scheduler_facades.ns)
* host I/O and tooling
  - [hello_argv_runtime_facades.ns](hello_argv_runtime_facades.ns)
  - [hello_env_runtime_facades.ns](hello_env_runtime_facades.ns)
  - [hello_process_runtime_facades.ns](hello_process_runtime_facades.ns)
  - [hello_command_runtime_facades.ns](hello_command_runtime_facades.ns)
  - [hello_subprocess_runtime_facades.ns](hello_subprocess_runtime_facades.ns)
  - [hello_host_text_runtime_facades.ns](hello_host_text_runtime_facades.ns)
  - [hello_json_runtime_facades.ns](hello_json_runtime_facades.ns)
  - [hello_text_report_json_facades.ns](hello_text_report_json_facades.ns)
  - [hello_time_report_facades.ns](hello_time_report_facades.ns)
  - [hello_benchmark_report_facades.ns](hello_benchmark_report_facades.ns)
  - [hello_benchmark_report_count_facades.ns](hello_benchmark_report_count_facades.ns)
  - [hello_text_format_runtime_facades.ns](hello_text_format_runtime_facades.ns)
  - [hello_error_runtime_facades.ns](hello_error_runtime_facades.ns)
  - [hello_result_runtime_facades.ns](hello_result_runtime_facades.ns)
  - [hello_diagnostic_runtime_facades.ns](hello_diagnostic_runtime_facades.ns)
  - [hello_time_runtime_facades.ns](hello_time_runtime_facades.ns)
  - [hello_sleep_runtime_facades.ns](hello_sleep_runtime_facades.ns)
  - [hello_clock_runtime_facades.ns](hello_clock_runtime_facades.ns)
  - [hello_clock_domain_runtime_facades.ns](hello_clock_domain_runtime_facades.ns)
  - [hello_stdin_runtime_facades.ns](hello_stdin_runtime_facades.ns)
  - [hello_tty_runtime_facades.ns](hello_tty_runtime_facades.ns)
  - [hello_input_runtime_facades.ns](hello_input_runtime_facades.ns)
  - [hello_io_runtime_facades.ns](hello_io_runtime_facades.ns)
  - [hello_terminal_io_facades.ns](hello_terminal_io_facades.ns)
  - [hello_native_command_runtime.ns](hello_native_command_runtime.ns)
* state / location / persistence
  - [hello_cwd_runtime_facades.ns](hello_cwd_runtime_facades.ns)
  - [hello_temp_runtime_facades.ns](hello_temp_runtime_facades.ns)
  - [hello_home_runtime_facades.ns](hello_home_runtime_facades.ns)
  - [hello_location_runtime_facades.ns](hello_location_runtime_facades.ns)
  - [hello_kv_runtime_facades.ns](hello_kv_runtime_facades.ns)
  - [hello_cache_runtime_facades.ns](hello_cache_runtime_facades.ns)
  - [hello_config_runtime_facades.ns](hello_config_runtime_facades.ns)
  - [hello_config_cache_facades.ns](hello_config_cache_facades.ns)
* path / filesystem
  - [hello_window_runtime_facades.ns](hello_window_runtime_facades.ns)
  - [hello_pipe_runtime_facades.ns](hello_pipe_runtime_facades.ns)
  - [hello_fabric_runtime_facades.ns](hello_fabric_runtime_facades.ns)
  - [hello_handle_table_runtime_facades.ns](hello_handle_table_runtime_facades.ns)
  - [hello_directory_runtime_facades.ns](hello_directory_runtime_facades.ns)
  - [hello_file_runtime_facades.ns](hello_file_runtime_facades.ns)
  - [hello_fs_metadata_runtime_facades.ns](hello_fs_metadata_runtime_facades.ns)
  - [hello_stat_runtime_facades.ns](hello_stat_runtime_facades.ns)
  - [hello_path_runtime_facades.ns](hello_path_runtime_facades.ns)
  - [hello_path_parent_facades.ns](hello_path_parent_facades.ns)
  - [hello_path_depth_facades.ns](hello_path_depth_facades.ns)
  - [hello_path_filename_facades.ns](hello_path_filename_facades.ns)
  - [hello_path_stem_facades.ns](hello_path_stem_facades.ns)
  - [hello_path_extension_facades.ns](hello_path_extension_facades.ns)
  - [hello_file_output_facades.ns](hello_file_output_facades.ns)
  - [hello_file_roundtrip_facades.ns](hello_file_roundtrip_facades.ns)
  - [hello_benchmark_report_file_facades.ns](hello_benchmark_report_file_facades.ns)
  - [hello_directory_create_facades.ns](hello_directory_create_facades.ns)
  - [hello_directory_remove_facades.ns](hello_directory_remove_facades.ns)
  - [hello_directory_stat_facades.ns](hello_directory_stat_facades.ns)

## Reading Rule

* use the frontdoor ladders first
* use the companion detail map after you know which facade lane you care about
* use [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
  for the shortest repo-level route
* use [stdlib/std/README.md](../../../stdlib/std/README.md)
  when you want recipe-side grouping
* use [examples/projects/README.md](../../../examples/projects/README.md)
  when you want project-form companions
* treat the long tail of narrower path-specific facade files as local detail
  unless you are actively working in that subsystem

## Notes

* `mod` is a top-level builtin declaration, not a nested construct
* `cpu` is currently the only domain that can declare `async fn`
