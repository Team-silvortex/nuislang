# Demo `.ns` Examples

This folder contains higher-signal end-to-end demos and source-shaped mirrors
of the current project-first `shader` / `kernel` lanes.

Use this folder when you want the single-file story.
Use [README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)
when you want the canonical multi-file project route.

## Start Here

* showcase mirror:
  [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
* shader spine:
  [shader_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_profile_demo.ns) ->
  surface branch ->
  packet branch ->
  bridge branch
* kernel spine:
  [kernel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_profile_demo.ns) ->
  async base ->
  async tensor ->
  tensor lane
* network edge:
  [network_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_demo.ns) ->
  [network_endpoint_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_endpoint_profile_demo.ns) ->
  [network_host_control_runtime_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_host_control_runtime_demo.ns) ->
  [network_host_handle_runtime_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_host_handle_runtime_demo.ns) ->
  [network_host_handle_transport_runtime_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_host_handle_transport_runtime_demo.ns) ->
  [network_owned_transport_result_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_owned_transport_result_demo.ns) ->
  [network_owned_transport_result_task_policy_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_owned_transport_result_task_policy_demo.ns) ->
  [network_owned_transport_result_task_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_owned_transport_result_task_batch_demo.ns) ->
  [network_owned_transport_result_task_windowed_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_owned_transport_result_task_windowed_batch_demo.ns) ->
  [network_owned_transport_result_session_bridge_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_owned_transport_result_session_bridge_demo.ns) ->
  [network_host_transport_runtime_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_host_transport_runtime_demo.ns) ->
  [network_transport_result_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_demo.ns) ->
  [network_transport_result_task_policy_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_task_policy_demo.ns) ->
  [network_transport_result_policy_split_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_policy_split_demo.ns) ->
  [network_transport_result_batch_split_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_batch_split_demo.ns) ->
  [network_transport_result_windowed_split_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_windowed_split_demo.ns) ->
  [network_transport_result_task_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_task_batch_demo.ns) ->
  [network_transport_result_task_windowed_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_task_windowed_batch_demo.ns) ->
  [network_transport_result_session_bridge_split_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_session_bridge_split_demo.ns) ->
  [network_transport_result_session_bridge_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_transport_result_session_bridge_demo.ns) ->
  result ladder ->
  session/task ladder
* short network rule:
  `profile core -> endpoint/timing -> host control/runtime transport -> result observe -> session -> result-policy/result-batch/result-windowed/policy/fallback -> batch/windowed`
* transport ladder rule:
  `transport result -> transport policy -> transport split -> transport batch split -> transport windowed split -> transport batch -> transport windowed -> transport/session bridge`
* connect/accept control rule:
  `connect result -> accept result -> connect/accept policy -> connect/accept batch -> connect/accept windowed`
* network result ladder:
  [network_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_result_profile_demo.ns) ->
  [network_connect_result_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_connect_result_demo.ns) ->
  [network_accept_result_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_accept_result_demo.ns) ->
  [network_connect_accept_task_policy_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_connect_accept_task_policy_demo.ns) ->
  [network_connect_accept_task_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_connect_accept_task_batch_demo.ns) ->
  [network_connect_accept_task_windowed_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_connect_accept_task_windowed_batch_demo.ns) ->
  [network_result_task_policy_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_result_task_policy_demo.ns) ->
  [network_result_task_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_result_task_batch_demo.ns) ->
  [network_result_task_windowed_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_result_task_windowed_batch_demo.ns) ->
  [network_result_session_bridge_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_result_session_bridge_demo.ns)
* network session/task ladder:
  [network_profile_summary_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_summary_demo.ns) ->
  [network_profile_session_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_session_demo.ns) ->
  [network_profile_task_policy_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_task_policy_demo.ns) ->
  [network_profile_task_fallback_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_task_fallback_demo.ns) ->
  [network_profile_task_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_task_batch_demo.ns) ->
  [network_profile_task_windowed_batch_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/network_profile_task_windowed_batch_demo.ns)

## Shader Mirrors

* surface branch:
  [shader_surface_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_profile_demo.ns),
  [shader_surface_material_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_profile_demo.ns),
  [shader_surface_material_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_pass_profile_demo.ns),
  [shader_surface_material_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_packet_profile_demo.ns),
  [shader_surface_material_panel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_panel_profile_demo.ns),
  [shader_surface_state_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_state_profile_demo.ns),
  [shader_surface_state_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_state_packet_profile_demo.ns),
  [shader_surface_state_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_state_pass_profile_demo.ns),
  [shader_surface_state_flow_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_state_flow_profile_demo.ns),
  [shader_surface_material_flow_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_flow_profile_demo.ns),
  [shader_surface_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_packet_profile_demo.ns),
  [shader_surface_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_pass_profile_demo.ns)
* packet branch:
  [shader_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_packet_profile_demo.ns),
  [shader_packet_bridge_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_packet_bridge_demo.ns)
* bridge branch:
  [shader_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_pass_profile_demo.ns),
  [shader_frame_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_frame_profile_demo.ns),
  [shader_async_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_result_profile_demo.ns),
  [shader_async_fanin_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_fanin_profile_demo.ns),
  [shader_async_schedule_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_schedule_profile_demo.ns),
  [shader_async_policy_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_policy_profile_demo.ns),
  [shader_async_fallback_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_fallback_profile_demo.ns),
  [shader_async_batch_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_batch_profile_demo.ns),
  [shader_async_windowed_batch_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_async_windowed_batch_profile_demo.ns),
  [shader_result_family_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_result_family_profile_demo.ns),
  [shader_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_result_profile_demo.ns),
  [shader_draw_render_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_draw_render_profile_demo.ns),
  [shader_draw_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_draw_profile_demo.ns)

Short shader reading rule:

* surface:
  `metadata -> material seeds -> state set -> state+packet / state+pass -> state mini-flow`
* packet:
  `packet contract -> packet bridge`
* bridge:
  `pass -> frame -> async result consume -> async fan-in -> async scheduling -> async policy -> async fallback -> async batch -> async windowed batch -> result family -> draw/render split`

## Kernel Mirrors

* async base:
  [kernel_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_result_profile_demo.ns),
  [kernel_async_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_result_profile_demo.ns),
  [kernel_async_batch_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_batch_profile_demo.ns),
  [kernel_async_roundtrip_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_roundtrip_profile_demo.ns)
* async tensor:
  [kernel_async_tensor_batch_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_tensor_batch_profile_demo.ns),
  [kernel_async_tensor_policy_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_tensor_policy_profile_demo.ns),
  [kernel_async_tensor_fallback_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_tensor_fallback_profile_demo.ns),
  [kernel_async_tensor_windowed_batch_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_tensor_windowed_batch_profile_demo.ns),
  [kernel_async_tensor_roundtrip_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_async_tensor_roundtrip_profile_demo.ns)
* tensor lane:
  [kernel_tensor_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_profile_demo.ns),
  [kernel_tensor_inspect_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_inspect_demo.ns),
  [kernel_tensor_slice_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_slice_demo.ns),
  [kernel_tensor_reshape_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_reshape_demo.ns),
  [kernel_tensor_broadcast_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_broadcast_demo.ns),
  [kernel_tensor_reduce_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_reduce_demo.ns),
  [kernel_tensor_select_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_select_demo.ns),
  [kernel_tensor_order_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_order_demo.ns),
  [kernel_tensor_axis_reduce_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_reduce_demo.ns),
  [kernel_tensor_axis_family_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_family_demo.ns),
  [kernel_tensor_axis_select_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_select_demo.ns),
  [kernel_tensor_axis_sort_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_sort_demo.ns),
  [kernel_tensor_axis_order_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_order_demo.ns),
  [kernel_tensor_axis_map_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_map_demo.ns),
  [kernel_tensor_axis_pipeline_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_pipeline_demo.ns),
  [kernel_tensor_axis_roundtrip_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_roundtrip_demo.ns),
  [kernel_tensor_map_zip_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_map_zip_demo.ns)

Short kernel reading rule:

* async base:
  `result -> batch -> roundtrip`
* async tensor:
  `batch -> policy -> fallback -> windowed -> roundtrip`
* tensor lane:
  `profile -> inspect -> slice -> reshape -> broadcast -> reduce -> select -> order -> axis subgroup -> map/zip`

## Mirror Rule

These files are source-shaped mirrors of project-first lanes.
They are useful for reading, but today they are not the canonical validation
route for `shader` or `kernel`.

The current honest validation route is still the project companion under
[README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md),
because standalone single-file `shader` and `kernel` lowering still depends on
loaded `nustar` implementations rather than a bootstrap compatibility shim.

## Useful Commands

```bash
cargo run -p nuis -- check examples/ns/demos/window_controls_demo.ns
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns /tmp/window_controls_demo_ns

cargo run -p nuis -- check examples/projects/domains/shader_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_profile_demo
```
