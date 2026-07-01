# Demo `.ns` Examples

This folder contains higher-signal end-to-end demos and source-shaped mirrors
of the current project-first showcase, `shader`, and `kernel` lanes.

Use this folder when you want the small single-file mirror story.
Use [README.md](../../../examples/projects/domains/README.md)
when you want the canonical multi-file project route.

Current role rule:

* this subtree is mirror-only by default
* it is useful for reading compact source-shaped domain stories
* it should not be read as the canonical validation route for `shader`,
  `kernel`, or future network lanes when the project-form route already exists

## Current Frontdoor Mirrors

If you only want the shortest current single-file mirror route, use:

* showcase mirror:
  [window_controls_demo.ns](window_controls_demo.ns)
* shader mirror anchor:
  [shader_profile_demo.ns](shader_profile_demo.ns)
* kernel mirror anchor:
  [kernel_profile_demo.ns](kernel_profile_demo.ns)
* network note:
  the older single-file `net_*` / `network_*` mirrors have been retired from
  this folder; use
  [README.md](../../../examples/projects/domains/README.md)
  and the `examples/projects/domains/*` project routes for the current network
  path

Practical rule:

* stop at these anchors if you only need the compact mirror story
* continue into the longer shader/kernel branches only when you are already
  working inside that domain lane

## Shader Mirrors

* surface branch:
  [shader_surface_profile_demo.ns](shader_surface_profile_demo.ns),
  [shader_surface_material_profile_demo.ns](shader_surface_material_profile_demo.ns),
  [shader_surface_material_pass_profile_demo.ns](shader_surface_material_pass_profile_demo.ns),
  [shader_surface_material_packet_profile_demo.ns](shader_surface_material_packet_profile_demo.ns),
  [shader_surface_material_panel_profile_demo.ns](shader_surface_material_panel_profile_demo.ns),
  [shader_surface_state_profile_demo.ns](shader_surface_state_profile_demo.ns),
  [shader_surface_state_packet_profile_demo.ns](shader_surface_state_packet_profile_demo.ns),
  [shader_surface_state_pass_profile_demo.ns](shader_surface_state_pass_profile_demo.ns),
  [shader_surface_state_flow_profile_demo.ns](shader_surface_state_flow_profile_demo.ns),
  [shader_surface_material_flow_profile_demo.ns](shader_surface_material_flow_profile_demo.ns),
  [shader_surface_packet_profile_demo.ns](shader_surface_packet_profile_demo.ns),
  [shader_surface_pass_profile_demo.ns](shader_surface_pass_profile_demo.ns)
* packet branch:
  [shader_packet_profile_demo.ns](shader_packet_profile_demo.ns),
  [shader_packet_bridge_demo.ns](shader_packet_bridge_demo.ns)
* bridge branch:
  [shader_pass_profile_demo.ns](shader_pass_profile_demo.ns),
  [shader_frame_profile_demo.ns](shader_frame_profile_demo.ns),
  [shader_async_result_profile_demo.ns](shader_async_result_profile_demo.ns),
  [shader_async_fanin_profile_demo.ns](shader_async_fanin_profile_demo.ns),
  [shader_async_schedule_profile_demo.ns](shader_async_schedule_profile_demo.ns),
  [shader_async_policy_profile_demo.ns](shader_async_policy_profile_demo.ns),
  [shader_async_fallback_profile_demo.ns](shader_async_fallback_profile_demo.ns),
  [shader_async_batch_profile_demo.ns](shader_async_batch_profile_demo.ns),
  [shader_async_windowed_batch_profile_demo.ns](shader_async_windowed_batch_profile_demo.ns),
  [shader_result_family_profile_demo.ns](shader_result_family_profile_demo.ns),
  [shader_result_profile_demo.ns](shader_result_profile_demo.ns),
  [shader_draw_render_profile_demo.ns](shader_draw_render_profile_demo.ns),
  [shader_draw_profile_demo.ns](shader_draw_profile_demo.ns)

Short shader reading rule:

* surface:
  `metadata -> material seeds -> state set -> state+packet / state+pass -> state mini-flow`
* packet:
  `packet contract -> packet bridge`
* bridge:
  `pass -> frame -> async result consume -> async fan-in -> async scheduling -> async policy -> async fallback -> async batch -> async windowed batch -> result family -> draw/render split`

## Kernel Mirrors

* async base:
  [kernel_result_profile_demo.ns](kernel_result_profile_demo.ns),
  [kernel_async_result_profile_demo.ns](kernel_async_result_profile_demo.ns),
  [kernel_async_batch_profile_demo.ns](kernel_async_batch_profile_demo.ns),
  [kernel_async_roundtrip_profile_demo.ns](kernel_async_roundtrip_profile_demo.ns)
* async tensor:
  [kernel_async_tensor_batch_profile_demo.ns](kernel_async_tensor_batch_profile_demo.ns),
  [kernel_async_tensor_policy_profile_demo.ns](kernel_async_tensor_policy_profile_demo.ns),
  [kernel_async_tensor_fallback_profile_demo.ns](kernel_async_tensor_fallback_profile_demo.ns),
  [kernel_async_tensor_windowed_batch_profile_demo.ns](kernel_async_tensor_windowed_batch_profile_demo.ns),
  [kernel_async_tensor_roundtrip_profile_demo.ns](kernel_async_tensor_roundtrip_profile_demo.ns)
* tensor lane:
  [kernel_tensor_profile_demo.ns](kernel_tensor_profile_demo.ns),
  [kernel_tensor_inspect_demo.ns](kernel_tensor_inspect_demo.ns),
  [kernel_tensor_slice_demo.ns](kernel_tensor_slice_demo.ns),
  [kernel_tensor_reshape_demo.ns](kernel_tensor_reshape_demo.ns),
  [kernel_tensor_broadcast_demo.ns](kernel_tensor_broadcast_demo.ns),
  [kernel_tensor_reduce_demo.ns](kernel_tensor_reduce_demo.ns),
  [kernel_tensor_select_demo.ns](kernel_tensor_select_demo.ns),
  [kernel_tensor_order_demo.ns](kernel_tensor_order_demo.ns),
  [kernel_tensor_axis_reduce_demo.ns](kernel_tensor_axis_reduce_demo.ns),
  [kernel_tensor_axis_family_demo.ns](kernel_tensor_axis_family_demo.ns),
  [kernel_tensor_axis_select_demo.ns](kernel_tensor_axis_select_demo.ns),
  [kernel_tensor_axis_sort_demo.ns](kernel_tensor_axis_sort_demo.ns),
  [kernel_tensor_axis_order_demo.ns](kernel_tensor_axis_order_demo.ns),
  [kernel_tensor_axis_map_demo.ns](kernel_tensor_axis_map_demo.ns),
  [kernel_tensor_axis_pipeline_demo.ns](kernel_tensor_axis_pipeline_demo.ns),
  [kernel_tensor_axis_roundtrip_demo.ns](kernel_tensor_axis_roundtrip_demo.ns),
  [kernel_tensor_map_zip_demo.ns](kernel_tensor_map_zip_demo.ns)

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
route for `shader`, `kernel`, or `network`.

The current honest validation route is still the project companion under
[README.md](../../../examples/projects/domains/README.md),
because standalone single-file domain lowering still depends on loaded
`nustar` implementations rather than a bootstrap compatibility shim.

## Useful Commands

```bash
cargo run -p nuis -- check examples/ns/demos/window_controls_demo.ns
cargo run -p nuis -- build examples/ns/demos/window_controls_demo.ns /tmp/window_controls_demo_ns

cargo run -p nuis -- check examples/projects/domains/shader_profile_demo
cargo run -p nuis -- check examples/projects/domains/kernel_profile_demo
cargo run -p nuis -- check examples/projects/domains/network_profile_demo
```
