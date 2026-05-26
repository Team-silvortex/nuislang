# `std`

`std` is the practical systems layer above `core`.

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file for the shortest current reading path across task, host I/O,
  persistence, and filesystem surfaces. Use this README for local detail only.

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
  - [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns)
  - [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns)
  - [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns)
  - [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns)
  - [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* CLI/tooling runtime
  - [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* net/runtime staging
  - [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
  - [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns)
  - [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns)
  - [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
  - [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns)
  - [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns)
  - [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
  - [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns)
  - [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns)
  - [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns)
  - [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns)
  - [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns)
  - [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns)
  - [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns)
  - [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)
  - [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns)
  - [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
  - [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns)
  - [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns)
  - [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns)
  - [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
  - [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns)
  - [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns)
  - [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns)
  - [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns)
  - [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns)
  - [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns)
  - [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns)
  - [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns)
  - [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns)
  - [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  - [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
  - [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  - [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  - [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  - [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  - [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns)
  - [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
  - [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns)
  - [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
  - [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns)
  - [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns)
  - [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns)
  - [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)
  - [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* checker/reporter tooling
  - [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
* result/diagnostic staging
  - [error_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime_recipe.ns)
  - [result_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime_recipe.ns)
  - [diagnostic_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime_recipe.ns)
  - [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
* directory/stat staging
  - [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns)
  - [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns)
  - [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns)
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
  - [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
  - [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [command_text_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_text_builder_recipe.ns)
* path/runtime staging
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
* path/rename staging
  - [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
* path/remove staging
  - [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
* file/output staging
  - [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns)
  - [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
* terminal/io staging
  - [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* line-input staging
  - [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)
* text/json staging
  - [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns)
  - [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
  - [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns)
  - [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* clock/test timing alignment
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  - [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  - [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
* time/clock
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)

## Local Detail

Use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
for the shortest route. Use this section only when you want the local shape of
`std` itself.

### First File Per Main Cluster

* task-facing async/task
  - start with [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* tooling-facing host/runtime
  - start with [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* state/location/persistence
  - start with [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
* filesystem read/write/mutate
  - start with [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
* inspection/formatting
  - start with [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

### Pure-To-Composite Clusters

Read these lanes as `pure layer -> wider composition layer`.

* task
  - [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
  - [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  - [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  - [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  - [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  - [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns)
  - [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
  - [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns)
  - [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
  - [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns)
  - [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns)
  - [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns)
  - [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)
  - [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  - [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  - [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* host I/O
  - [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
  - [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  - [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
  - [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* text/data
  - [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
  - [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns)
  - [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns)
  - [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* command/tooling
  - [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
  - [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [command_text_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_text_builder_recipe.ns)
* net
  - grouped rule:
    `profile core -> transport edge -> control edge -> protocol edge -> result spine -> task spine -> session`
  - profile core:
    [net_endpoint_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_endpoint_recipe.ns)
  - transport edge:
    [net_ip_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_ip_packet_recipe.ns),
    [net_tcp_stream_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_tcp_stream_recipe.ns),
    [net_udp_datagram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_udp_datagram_recipe.ns)
  - control edge:
    [net_connect_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_connect_recipe.ns),
    [net_listen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_listen_recipe.ns),
    [net_close_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_close_recipe.ns)
  - protocol edge:
    [net_protocol_experiment_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_experiment_recipe.ns),
    [net_line_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_line_protocol_recipe.ns),
    [net_datagram_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_protocol_recipe.ns),
    [net_dnsish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_protocol_recipe.ns),
    [net_httpish_protocol_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_protocol_recipe.ns),
    [net_httpish_request_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_request_recipe.ns),
    [net_httpish_response_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_response_recipe.ns),
    [net_httpish_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_roundtrip_recipe.ns)
  - result spine:
    [net_result_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_recipe.ns),
    [net_result_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_result_bridge_recipe.ns)
  - task spine:
    [net_task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_policy_recipe.ns),
    [net_task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_batch_recipe.ns),
    [net_task_windowed_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_recipe.ns),
    [net_task_windowed_bridge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_task_windowed_bridge_recipe.ns)
  - session:
    [net_control_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_control_session_recipe.ns),
    [net_transport_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_transport_session_recipe.ns),
    [net_protocol_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_protocol_session_recipe.ns),
    [net_datagram_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_session_recipe.ns),
    [net_datagram_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_exchange_session_recipe.ns),
    [net_datagram_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_datagram_pipeline_recipe.ns),
    [net_dnsish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_dnsish_exchange_session_recipe.ns),
    [net_httpish_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_session_recipe.ns),
    [net_httpish_exchange_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_exchange_session_recipe.ns),
    [net_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_session_recipe.ns)
  - companion validation:
    [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo),
    [net_ip_packet_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_ip_packet_recipe_demo),
    [net_tcp_stream_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_stream_recipe_demo),
    [net_udp_datagram_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_udp_datagram_recipe_demo),
    [net_connect_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_connect_recipe_demo),
    [net_listen_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_listen_recipe_demo),
    [net_close_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_close_recipe_demo),
    [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo),
    [net_line_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_line_protocol_recipe_demo),
    [net_datagram_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_protocol_recipe_demo),
    [net_dnsish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_protocol_recipe_demo),
    [net_httpish_protocol_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo),
    [net_httpish_request_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_request_recipe_demo),
    [net_httpish_response_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_response_recipe_demo),
    [net_httpish_roundtrip_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_roundtrip_recipe_demo),
    [net_result_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_recipe_demo),
    [net_result_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_result_bridge_recipe_demo),
    [net_task_policy_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_policy_recipe_demo),
    [net_task_batch_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_batch_recipe_demo),
    [net_task_windowed_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_recipe_demo),
    [net_task_windowed_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_task_windowed_bridge_recipe_demo),
    [net_control_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_control_session_recipe_demo),
    [net_transport_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_transport_session_recipe_demo),
    [net_protocol_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_session_recipe_demo),
    [net_datagram_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_session_recipe_demo),
    [net_datagram_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_exchange_session_recipe_demo),
    [net_datagram_pipeline_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_datagram_pipeline_recipe_demo),
    [net_dnsish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_dnsish_exchange_session_recipe_demo),
    [net_httpish_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_session_recipe_demo),
    [net_httpish_exchange_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_session_recipe_demo),
    [net_session_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo)
* time/clock
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
* filesystem metadata
  - [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns)
  - [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns)
  - [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns)
  - [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* data/window/fabric
  - [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns)
  - [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns)
  - [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns)
  - [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns)
  - [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)

### Local Mini-Maps

* path naming/inspection
  - [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  - [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
  - [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
  - [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
  - [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
  - [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
  - [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)
* host I/O and tooling
  - [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns)
  - [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns)
  - [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns)
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
  - [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
  - [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
  - [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
  - [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
* time/clock
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  - source mirrors:
    [hello_time_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_time_runtime_facades.ns),
    [hello_sleep_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_sleep_runtime_facades.ns),
    [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns),
    [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns),
    [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  - project mirrors:
    [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
    [sleep_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/sleep_runtime_demo),
    [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
    [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo)
* persistence
  - [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns)
  - [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns)
  - [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns)
  - [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns)
  - [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns)
  - [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)
* task-facing
  - [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
  - [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
  - [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
  - [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  - [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
  - [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns)
  - [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
  - [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns)
  - [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
  - [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns)
  - [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns)
  - [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns)
  - [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)
  - [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
  - [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
  - [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
  - timing bridge base:
    [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
    [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
    [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)

### Reading Rule

* use this README for module inventory and local clustering
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use the example READMEs for `recipe -> facade -> project` mirrors
* for concrete mirrors, use:
  - [examples/ns/ffi/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
  - [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/std/module.toml)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
