# Demo `.ns` Examples

This folder contains higher-signal end-to-end demos and source-shaped domain
stubs:

* `window_controls_demo.ns`
* `shader_profile_demo.ns`
* `shader_surface_profile_demo.ns`
* `shader_surface_material_profile_demo.ns`
* `shader_surface_material_pass_profile_demo.ns`
* `shader_surface_material_packet_profile_demo.ns`
* `shader_surface_material_panel_profile_demo.ns`
* `shader_surface_state_profile_demo.ns`
* `shader_surface_state_packet_profile_demo.ns`
* `shader_surface_state_pass_profile_demo.ns`
* `shader_surface_state_flow_profile_demo.ns`
* `shader_surface_material_flow_profile_demo.ns`
* `shader_surface_packet_profile_demo.ns`
* `shader_surface_pass_profile_demo.ns`
* `shader_packet_profile_demo.ns`
* `shader_packet_bridge_demo.ns`
* `shader_pass_profile_demo.ns`
* `shader_frame_profile_demo.ns`
* `shader_async_result_profile_demo.ns`
* `shader_async_fanin_profile_demo.ns`
* `shader_async_schedule_profile_demo.ns`
* `shader_async_policy_profile_demo.ns`
* `shader_async_fallback_profile_demo.ns`
* `shader_async_batch_profile_demo.ns`
* `shader_async_windowed_batch_profile_demo.ns`
* `shader_result_family_profile_demo.ns`
* `shader_result_profile_demo.ns`
* `shader_draw_render_profile_demo.ns`
* `shader_draw_profile_demo.ns`
* `kernel_profile_demo.ns`
* `kernel_result_profile_demo.ns`
* `kernel_async_result_profile_demo.ns`
* `kernel_async_batch_profile_demo.ns`
* `kernel_async_roundtrip_profile_demo.ns`
* `kernel_async_tensor_batch_profile_demo.ns`
* `kernel_async_tensor_policy_profile_demo.ns`
* `kernel_async_tensor_fallback_profile_demo.ns`
* `kernel_async_tensor_windowed_batch_profile_demo.ns`
* `kernel_async_tensor_roundtrip_profile_demo.ns`
* `kernel_tensor_profile_demo.ns`
* `kernel_tensor_inspect_demo.ns`
* `kernel_tensor_slice_demo.ns`
* `kernel_tensor_reshape_demo.ns`
* `kernel_tensor_broadcast_demo.ns`
* `kernel_tensor_reduce_demo.ns`
* `kernel_tensor_select_demo.ns`
* `kernel_tensor_order_demo.ns`
* `kernel_tensor_axis_reduce_demo.ns`
* `kernel_tensor_axis_family_demo.ns`
* `kernel_tensor_axis_select_demo.ns`
* `kernel_tensor_axis_sort_demo.ns`
* `kernel_tensor_axis_order_demo.ns`
* `kernel_tensor_axis_map_demo.ns`
* `kernel_tensor_axis_pipeline_demo.ns`
* `kernel_tensor_axis_roundtrip_demo.ns`
* `kernel_tensor_map_zip_demo.ns`

Current guidance:

* read this file when you want the single-file end-to-end story
* prefer [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo) when you want the canonical multi-file route with explicit project links, ABI state, and support-module profiles
* treat `shader_profile_demo.ns`, `shader_surface_profile_demo.ns`,
  `shader_surface_material_profile_demo.ns`,
  `shader_surface_material_pass_profile_demo.ns`,
  `shader_surface_material_packet_profile_demo.ns`,
  `shader_surface_material_panel_profile_demo.ns`,
  `shader_surface_state_profile_demo.ns`,
  `shader_surface_state_packet_profile_demo.ns`,
  `shader_surface_state_pass_profile_demo.ns`,
  `shader_surface_state_flow_profile_demo.ns`,
  `shader_surface_material_flow_profile_demo.ns`,
  `shader_surface_packet_profile_demo.ns`, `shader_surface_pass_profile_demo.ns`,
  `shader_packet_profile_demo.ns`,
  `shader_packet_bridge_demo.ns`, `shader_pass_profile_demo.ns`,
  `shader_frame_profile_demo.ns`, `shader_async_result_profile_demo.ns`,
  `shader_async_fanin_profile_demo.ns`,
  `shader_async_schedule_profile_demo.ns`,
  `shader_async_policy_profile_demo.ns`,
  `shader_async_fallback_profile_demo.ns`,
  `shader_async_batch_profile_demo.ns`,
  `shader_async_windowed_batch_profile_demo.ns`,
  `shader_result_family_profile_demo.ns`,
  `shader_result_profile_demo.ns`, `shader_draw_render_profile_demo.ns`,
  `shader_draw_profile_demo.ns`,
  `kernel_profile_demo.ns`,
  `kernel_result_profile_demo.ns`, `kernel_async_result_profile_demo.ns`,
  `kernel_async_batch_profile_demo.ns`,
  `kernel_async_roundtrip_profile_demo.ns`,
  `kernel_async_tensor_batch_profile_demo.ns`,
  `kernel_async_tensor_roundtrip_profile_demo.ns`,
  `kernel_tensor_profile_demo.ns`, and
  `kernel_tensor_inspect_demo.ns`, `kernel_tensor_slice_demo.ns`,
  `kernel_tensor_reshape_demo.ns`, `kernel_tensor_broadcast_demo.ns`,
  `kernel_tensor_reduce_demo.ns`, `kernel_tensor_select_demo.ns`,
  `kernel_tensor_order_demo.ns`, `kernel_tensor_axis_reduce_demo.ns`,
  `kernel_tensor_axis_family_demo.ns`, `kernel_tensor_axis_select_demo.ns`,
  `kernel_tensor_axis_sort_demo.ns`, `kernel_tensor_axis_order_demo.ns`,
  `kernel_tensor_axis_map_demo.ns`, `kernel_tensor_axis_pipeline_demo.ns`,
  `kernel_tensor_axis_roundtrip_demo.ns`, `kernel_tensor_map_zip_demo.ns` as
  current source-shaped mirrors of project-first lanes
* today those two domain stubs do not pass standalone `nuis check` because
  `shader` and `kernel` lowering still rely on loaded `nustar` implementations
  rather than a bootstrap compatibility shim

Useful commands:

```bash
cargo run -p nuis -- check examples/ns/demos/window_controls_demo.ns
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns /tmp/window_controls_demo_ns
cargo run -p nuis -- check examples/projects/domains/shader_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_material_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_material_pass_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_material_packet_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_material_panel_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_state_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_state_packet_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_state_pass_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_state_flow_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_material_flow_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_packet_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_surface_pass_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_packet_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_packet_bridge_demo
cargo run -p nuis -- check examples/projects/domains/shader_pass_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_frame_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_result_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_fanin_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_schedule_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_policy_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_fallback_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_batch_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_async_windowed_batch_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_result_family_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_result_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_draw_render_profile_demo
cargo run -p nuis -- check examples/projects/domains/shader_draw_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_result_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_result_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_batch_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_roundtrip_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_tensor_batch_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_tensor_policy_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_tensor_fallback_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_async_tensor_roundtrip_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_inspect_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_slice_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_reshape_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_broadcast_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_reduce_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_select_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_order_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_reduce_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_family_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_select_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_sort_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_order_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_map_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_pipeline_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_axis_roundtrip_demo
cargo run -p nuis -- check examples/projects/domains/kernel_tensor_map_zip_demo
```
