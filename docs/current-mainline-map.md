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

## Pure-To-Composite Clusters

Use these when you want the shortest explanation of how the current layers stack.

* task:
  [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns) ->
  [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns) ->
  [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns) ->
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns) ->
  [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns) ->
  [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns) ->
  [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns) /
  [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns) ->
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* host I/O:
  [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns) ->
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns) /
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns) ->
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns) ->
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
* text/data:
  [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns) ->
  [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns) ->
  [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns) ->
  [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* command/tooling:
  [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns) ->
  [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns) ->
  [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
* filesystem metadata:
  [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns) ->
  [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns) ->
  [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns) ->
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* data/window/fabric:
  [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns) ->
  [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns) ->
  [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns) ->
  [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns) ->
  [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* project-first domain profiles:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo) ->
  surface branch ->
  packet branch ->
  [shader_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_result_profile_demo) ->
  [shader_async_fanin_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fanin_profile_demo) ->
  [shader_async_schedule_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_schedule_profile_demo) ->
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo) ->
  [shader_async_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fallback_profile_demo) ->
  [shader_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_batch_profile_demo) ->
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo) ->
  bridge branch ->
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  and
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo) ->
  async base ->
  async tensor ->
  tensor lane
  where
  async base =
  [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo) ->
  [kernel_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_result_profile_demo) ->
  [kernel_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_batch_profile_demo) ->
  [kernel_async_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_roundtrip_profile_demo)
  and
  async tensor =
  [kernel_async_tensor_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_batch_profile_demo) ->
  [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo) ->
  [kernel_async_tensor_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_fallback_profile_demo) ->
  [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo) ->
  [kernel_async_tensor_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_roundtrip_profile_demo)
  with async alignment:
  shader = result -> policy -> fallback -> batch -> windowed
  kernel = result -> batch -> policy -> fallback -> windowed -> roundtrip
  and
  tensor lane =
  [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo) ->
  [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo) ->
  [kernel_tensor_slice_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_slice_demo) ->
  [kernel_tensor_reshape_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reshape_demo) ->
  [kernel_tensor_broadcast_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_broadcast_demo) ->
  [kernel_tensor_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reduce_demo) ->
  [kernel_tensor_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_select_demo) ->
  [kernel_tensor_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_order_demo) ->
  [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo) ->
  [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo) ->
  [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo) ->
  [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo) ->
  [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo) ->
  [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo) ->
  [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo) ->
  [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo) ->
  [kernel_tensor_map_zip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_map_zip_demo) ->
  [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo) ->
  [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

Axis-aware kernel reading hint:
reduction -> selection -> full-order/top-k -> transform

Shader surface-branch reading hint:
metadata -> material seeds -> state set -> state+packet / state+pass -> state mini-flow

Current surface-branch anchors:
[shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo) ->
[shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo) ->
[shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo) ->
[shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo) ->
[shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo) ->
[shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo) ->
[shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo) ->
[shader_surface_state_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_pass_profile_demo) ->
[shader_surface_state_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_flow_profile_demo) ->
[shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo) ->
[shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo) ->
[shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)

Shader bridge-branch reading hint:
pass -> frame -> async result consume -> async fan-in -> async scheduling -> async policy -> async fallback -> async batch -> async windowed batch -> result family -> draw/render split -> wider draw/render

Current bridge-branch anchors:
[shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo) ->
[shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo) ->
[shader_async_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_result_profile_demo) ->
[shader_async_fanin_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fanin_profile_demo) ->
[shader_async_schedule_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_schedule_profile_demo) ->
[shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo) ->
[shader_async_fallback_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_fallback_profile_demo) ->
[shader_async_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_batch_profile_demo) ->
[shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo) ->
[shader_result_family_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_family_profile_demo) ->
[shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo) ->
[shader_draw_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_render_profile_demo) ->
[shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo) ->
[shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)

## Task-Facing `std`

Read in this order:

* [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)

Timing bridge base for that lane:

* [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
* [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
* [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)

Best companions:

* source-level:
  [hello_task_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns),
  [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
* project-level:
  [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo),
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo),
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo),
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo),
  [task_clock_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo),
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo),
  [task_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_policy_branch_demo),
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo)
* time/clock mirrors for task timing:
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns),
  [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
  [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
  [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo)

## Host I/O Mainline

Read in this order:

* execution:
  [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns),
  [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns),
  [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns),
  [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns),
  [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns),
  [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns),
  [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns),
  [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns),
  [error_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime_recipe.ns),
  [result_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime_recipe.ns),
  [diagnostic_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime_recipe.ns),
  [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
  [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns),
  [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
  [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* input observation:
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns),
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns),
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* terminal shaping:
  [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns),
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns),
  [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns),
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)

Time/clock order inside this lane:

* [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
* [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
* [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
* [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)

Best companions:

* source-level:
  [hello_argv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns),
  [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns),
  [hello_process_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns),
  [hello_command_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_command_runtime_facades.ns),
  [hello_subprocess_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_subprocess_runtime_facades.ns),
  [hello_host_text_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns),
  [hello_json_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_json_runtime_facades.ns),
  [hello_text_format_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_format_runtime_facades.ns),
  [hello_error_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_error_runtime_facades.ns),
  [hello_result_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_runtime_facades.ns),
  [hello_diagnostic_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_diagnostic_runtime_facades.ns),
  [hello_time_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_time_runtime_facades.ns),
  [hello_sleep_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_sleep_runtime_facades.ns),
  [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns),
  [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns),
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns),
  [hello_stdin_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stdin_runtime_facades.ns),
  [hello_tty_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_tty_runtime_facades.ns),
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns),
  [hello_io_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns),
  [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
* project-level:
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
  [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/terminal_io_demo)

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
  [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cwd_runtime_demo),
  [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/temp_runtime_demo),
  [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/home_runtime_demo),
  [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/location_runtime_demo),
  [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/kv_runtime_demo),
  [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cache_runtime_demo),
  [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_runtime_demo),
  [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_cache_demo)

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
  [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns),
  [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns),
  [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns),
  [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns),
  [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns),
  [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns),
  [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns),
  [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns),
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
