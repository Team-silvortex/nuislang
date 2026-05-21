# Shader / Kernel Project-First Contract

This file captures the current repository contract for the checked-in `shader`
and `kernel` lanes.

It exists because these two lanes are now clearly part of the mainline reading
path, but they do not currently grow like the host-I/O, task, or
data/window/fabric lanes.

The important current truth is simple:

* `shader` and `kernel` are currently **project-first lanes**
* they are not currently `std`-first lanes
* they are not currently standalone single-file `check` lanes

## Current Lane Shape

The current `shader` / `kernel` lane prefers this order:

```text
YIR/reference truth
-> source-shaped domain module or demo stub
-> project-form companion with explicit ABI loading
-> wider showcase project
```

For checked-in repository practice, that currently means:

```text
surface_shader.ns / kernel_unit.ns style profile source
-> project-first domain companions
-> window_controls_demo / kernel_tensor_demo
```

## Why This Is Project-First

These lanes already have real current meaning through:

* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* checked-in showcase projects such as
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  and
  [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

But they currently differ from the `std` pure-layer lanes in one important way:

* standalone `nuis check <single-file.ns>` still depends on loaded `nustar`
  lowering implementations for these domains
* the current single-file bootstrap path does not provide a compatibility shim
  for:
  * `shader.yir.lowering.v1`
  * `kernel.yir.lowering.v1`

That means the most honest checked-in route today is:

* keep source-shaped examples
* validate the lanes canonically through project manifests with explicit domain
  ABI loading

## Current Checked-In Project-First Routes

The current narrow project-form companions are:

* [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo)
* [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo)
* [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo)
* [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo)
* [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo)
* [shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo)
* [shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo)
* [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo)
* [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo)
* [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
* [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo)
* [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
* [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo)
* [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo)
* [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo)
* [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo)
* [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)
* [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
* [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo)
* [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo)
* [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo)
* [kernel_tensor_slice_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_slice_demo)
* [kernel_tensor_reshape_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reshape_demo)
* [kernel_tensor_broadcast_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_broadcast_demo)
* [kernel_tensor_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_reduce_demo)
* [kernel_tensor_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_select_demo)
* [kernel_tensor_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_order_demo)
* [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo)
* [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo)
* [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo)
* [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo)
* [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo)
* [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo)
* [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo)
* [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo)
* [kernel_tensor_map_zip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_map_zip_demo)
* [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)

Current role split:

* `shader_profile_demo` is the narrow checked-in route for shader profile
  metadata such as target, viewport, pipeline, packet shape, and inline WGSL
* `shader_surface_profile_demo` is the next narrow route where surface-facing
  metadata such as target, viewport, pipeline, vertex count, instance count,
  material mode, and pass kind are read together without pulling in packet
  shaping or data/render bridge concerns
* `shader_surface_material_profile_demo` is the next narrow route where that
  same surface-facing metadata is explicitly joined with material-facing seed
  helpers such as `shader_profile_color_seed(...)`,
  `shader_profile_speed_seed(...)`, and `shader_profile_radius_seed(...)`,
  while still staying outside packet shaping and data/render bridge concerns
* `shader_surface_material_pass_profile_demo` is the next narrow route where
  those surface-facing and material-facing seed helpers are explicitly joined
  with `shader_profile_begin_pass(...)`, `shader_pass_ready(...)`, and the
  smallest checked-in `shader_value(pass_result)` consumer, while still
  staying outside the data/render bridge lanes
* `shader_surface_material_packet_profile_demo` is the next narrow route where
  those surface-facing and material-facing seed helpers are explicitly joined
  with packet slots, packet tag, packet field count, and
  `shader_profile_packet(...)`, while still staying outside the
  data/render bridge lanes
* `shader_surface_material_panel_profile_demo` is the next narrow route where
  those surface-facing and material-facing seed helpers are explicitly joined
  with `shader_profile_panel_packet(...)` and the richer `NovaPanelPacket`
  payload fields such as accent, toggle state, and focus index, while still
  staying outside the data/render bridge lanes
* `shader_surface_state_profile_demo` is the next narrow route where those
  same surface-facing and material-facing seed helpers are explicitly joined
  with a compact scene/material state set made of
  `nova_header_packet(...)`, `nova_theme_packet(...)`,
  `nova_surface_packet(...)`, `nova_viewport_packet(...)`,
  `nova_layer_packet(...)`, `nova_scene_packet(...)`,
  `nova_camera_packet(...)`, and `nova_material_packet(...)`, while still
  staying outside the data/render bridge lanes
* `shader_surface_state_packet_profile_demo` is the next narrow route where
  that compact scene/material state set is explicitly joined with packet slots,
  packet tag, packet field count, and `shader_profile_packet(...)`, while
  still staying outside the data/render bridge lanes
* `shader_surface_material_flow_profile_demo` is the next narrow route where
  those surface-facing and material-facing seed helpers are explicitly joined
  with packet shaping, `shader_profile_begin_pass(...)`,
  `shader_pass_ready(...)`, and the smallest checked-in draw consumer, while
  still staying outside the data/render bridge lanes
* `shader_surface_packet_profile_demo` is the next narrow route where
  surface-facing metadata is explicitly joined with packet slots and
  `shader_profile_packet(...)`, while still staying outside the
  data/render bridge lanes
* `shader_surface_pass_profile_demo` is the next narrow route where
  surface-facing metadata is explicitly joined with
  `shader_profile_begin_pass(...)`, `shader_pass_ready(...)`, and the smallest
  checked-in `shader_value(pass_result)` consumer
* `shader_packet_profile_demo` is the next narrow route where packet-contract
  metadata such as packet slots, packet field count, packet tag, material
  mode, and pass kind are read together with `shader_profile_packet(...)`
* `shader_packet_bridge_demo` is the next narrow route where packet-contract
  shaping is explicitly joined with `data_profile_send_uplink(...)` and
  `data_profile_send_downlink(...)`, while still keeping a minimal
  `shader_profile_begin_pass(...)` / `shader_profile_render(...)` compatibility
  path visible for the current project link contract
* `shader_pass_profile_demo` is the next narrow route where
  `shader_profile_begin_pass(...)`, `shader_pass_ready(...)`, and
  `shader_value(...)` are the main focus, with `shader_profile_draw_instanced(...)`
  kept as the smallest checked-in consumer of the pass value and
  `shader_profile_render(...)` retained for current project-link compatibility
* `shader_frame_profile_demo` is the next narrow route where
  `shader_result(shader_profile_render(...))`, `shader_frame_ready(...)`, and
  `shader_value(frame_result)` are the main focus, while packet shaping stays
  explicit and the downlink/present bridge remains visible
* `shader_result_profile_demo` is the next narrow route where shader profile
  metadata is explicitly joined with packet-slot inspection,
  `shader_profile_begin_pass(...)`, `shader_profile_draw_instanced(...)`, and
  `shader_result(...)` observers such as `shader_pass_ready(...)`,
  `shader_frame_ready(...)`, and `shader_value(...)`
* `shader_draw_profile_demo` is the next narrow route where the checked-in
  project lane visibly crosses the explicit draw bridge:
  `packet -> begin_pass -> draw_instanced -> downlink -> present`,
  while still keeping `shader_profile_render(...)` present to satisfy the
  current project link contract
* `shader_render_profile_demo` is the next narrow route where shader profile
  metadata is already joined with explicit `data` uplink/downlink and
  `shader_profile_render(...)`
* `kernel_profile_demo` is the narrow checked-in route for kernel profile
  metadata such as bind-core, queue depth, and batch lanes
* `kernel_result_profile_demo` is the next narrow route where kernel profile
  metadata is explicitly wrapped into `KernelResult<T>` and then observed
  through `kernel_config_ready(...)` and `kernel_value(...)`
* `kernel_tensor_profile_demo` is the next narrow route where kernel profile
  metadata is already joined with source-facing tensor primitives such as
  `kernel_tensor(...)`, `kernel_matmul(...)`, `kernel_add_bias(...)`,
  `kernel_relu(...)`, and `kernel_reduce_sum(...)`
* `kernel_tensor_inspect_demo` is the next narrow route where tensor layout and
  scalar inspection helpers such as `kernel_shape(...)`, `kernel_rows(...)`,
  `kernel_cols(...)`, and `kernel_element_at(...)` become visible at source level
* `kernel_tensor_slice_demo` is the next narrow route where first-slice helpers
  such as `kernel_row(...)` and `kernel_col(...)` become visible at source
  level with the current first-row / first-col semantics
* `kernel_tensor_reshape_demo` is the next narrow route where
  `kernel_reshape(...)` becomes visible at source level as the first explicit
  shape-transform helper
* `kernel_tensor_broadcast_demo` is the next narrow route where
  `kernel_broadcast(...)` becomes visible at source level as the first explicit
  shape-alignment helper
* `kernel_tensor_reduce_demo` is the next narrow route where
  `kernel_reduce_sum(...)`, `kernel_reduce_max(...)`, and
  `kernel_reduce_mean(...)` become visible together as the first explicit
  reduction cluster
* `kernel_tensor_select_demo` is the next narrow route where
  `kernel_argmax(...)` and `kernel_argmin(...)` become visible together as the
  first explicit global selection cluster
* `kernel_tensor_order_demo` is the next narrow route where
  `kernel_sort(...)` and `kernel_topk(...)` become visible together as the
  first explicit ordered-selection cluster
* `kernel_tensor_axis_reduce_demo` is the next narrow route where
  `kernel_reduce_sum_axis(..., "rows|cols")` becomes visible as the first
  explicit axis-aware reduction cluster
* `kernel_tensor_axis_family_demo` is the next narrow route where
  `kernel_reduce_max_axis(...)` and `kernel_reduce_mean_axis(...)` become
  visible together as the next explicit axis-aware reduction family
* `kernel_tensor_axis_select_demo` is the next narrow route where
  `kernel_argmax_axis(...)` and `kernel_argmin_axis(...)` become visible
  together as the first explicit axis-aware selection family
* `kernel_tensor_axis_sort_demo` is the next narrow route where
  `kernel_sort_axis(...)` becomes visible as the first explicit axis-aware
  full-order helper
* `kernel_tensor_axis_order_demo` is the next narrow route where
  `kernel_topk_axis(...)` becomes visible as the first explicit axis-aware
  ordered-selection helper
* `kernel_tensor_axis_map_demo` is the next narrow route where
  `kernel_map_axis(...)` becomes visible as the first explicit axis-aware
  transform helper
* `kernel_tensor_axis_pipeline_demo` is the next narrow route where axis-aware
  reduction, transform, and ordered-selection helpers are composed into a
  small checked-in operator flow
* `kernel_tensor_axis_roundtrip_demo` is the next narrow route where that
  axis-aware mini-flow is joined with explicit `data` uplink/downlink
* `kernel_tensor_map_zip_demo` is the next narrow route where light tensor
  transform helpers such as `kernel_map(...)` and `kernel_zip(...)` become
  visible at source level while still lowering into the existing kernel op set
* `kernel_roundtrip_profile_demo` is the next narrow route where kernel profile
  metadata is already joined with explicit `data` uplink/downlink

These are intentionally narrower than the showcase projects.

## Current Shader Branches

The shader ladder is now easier to read as three local branches after
[shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo):

* surface branch:
  [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo) ->
  [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo) ->
  [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo) ->
  [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo) ->
  [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo) ->
  [shader_surface_state_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_profile_demo) ->
  [shader_surface_state_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_state_packet_profile_demo) ->
  [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo) ->
  [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo) ->
  [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
* packet branch:
  [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo) ->
  [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
* bridge branch:
  [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo) ->
  [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo) ->
  [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo) ->
  [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo) ->
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)

Recommended reading order inside shader is now:

* start with `shader_profile_demo`
* read the surface branch
* then the packet branch
* then the bridge branch
* only then move to [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)

## Current Axis-Aware Kernel Lane

The current axis-aware tensor lane is now explicit enough to read as its own
subgroup inside the broader `kernel` ladder:

* [kernel_tensor_axis_reduce_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_reduce_demo)
  introduces `kernel_reduce_sum_axis(..., "rows|cols")`
* [kernel_tensor_axis_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_family_demo)
  expands that into `kernel_reduce_max_axis(...)` and
  `kernel_reduce_mean_axis(...)`
* [kernel_tensor_axis_select_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_select_demo)
  introduces `kernel_argmax_axis(...)` and `kernel_argmin_axis(...)`
* [kernel_tensor_axis_sort_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_sort_demo)
  introduces `kernel_sort_axis(...)`
* [kernel_tensor_axis_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_order_demo)
  introduces `kernel_topk_axis(...)`
* [kernel_tensor_axis_map_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_map_demo)
  introduces `kernel_map_axis(...)`
* [kernel_tensor_axis_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_pipeline_demo)
  composes `kernel_map_axis(...)`, `kernel_reduce_mean_axis(...)`, and
  `kernel_topk_axis(...)` into a narrow checked-in mini-flow
* [kernel_tensor_axis_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_axis_roundtrip_demo)
  joins that mini-flow with `data_profile_send_uplink(...)` and
  `data_profile_send_downlink(...)`

Current reading rule inside this subgroup:

* start with axis reduction
* then read axis selection
* then read axis full-order and axis top-k
* then read axis transform
* then read one composed mini-flow
* then read one data-bridge route

That keeps the growth shape aligned with the existing non-axis ladder:

* reduction
* selection
* ordered selection
* transform
* composition
* bridge

## Current Source-Shaped Mirrors

The repository still keeps source-shaped mirrors for these lanes:

* [shader_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_profile_demo.ns)
* [shader_surface_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_profile_demo.ns)
* [shader_surface_material_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_profile_demo.ns)
* [shader_surface_material_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_pass_profile_demo.ns)
* [shader_surface_material_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_packet_profile_demo.ns)
* [shader_surface_material_panel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_panel_profile_demo.ns)
* [shader_surface_material_flow_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_material_flow_profile_demo.ns)
* [shader_surface_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_packet_profile_demo.ns)
* [shader_surface_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_surface_pass_profile_demo.ns)
* [shader_packet_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_packet_profile_demo.ns)
* [shader_packet_bridge_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_packet_bridge_demo.ns)
* [shader_pass_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_pass_profile_demo.ns)
* [shader_frame_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_frame_profile_demo.ns)
* [shader_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_result_profile_demo.ns)
* [shader_draw_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_draw_profile_demo.ns)
* [kernel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_profile_demo.ns)
* [kernel_result_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_result_profile_demo.ns)
* [kernel_tensor_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_profile_demo.ns)
* [kernel_tensor_inspect_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_inspect_demo.ns)
* [kernel_tensor_slice_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_slice_demo.ns)
* [kernel_tensor_reshape_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_reshape_demo.ns)
* [kernel_tensor_broadcast_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_broadcast_demo.ns)
* [kernel_tensor_reduce_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_reduce_demo.ns)
* [kernel_tensor_select_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_select_demo.ns)
* [kernel_tensor_order_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_order_demo.ns)
* [kernel_tensor_axis_reduce_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_reduce_demo.ns)
* [kernel_tensor_axis_family_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_family_demo.ns)
* [kernel_tensor_axis_select_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_select_demo.ns)
* [kernel_tensor_axis_sort_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_sort_demo.ns)
* [kernel_tensor_axis_order_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_order_demo.ns)
* [kernel_tensor_axis_map_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_map_demo.ns)
* [kernel_tensor_axis_pipeline_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_pipeline_demo.ns)
* [kernel_tensor_axis_roundtrip_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_axis_roundtrip_demo.ns)
* [kernel_tensor_map_zip_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_tensor_map_zip_demo.ns)

Their current role is:

* to show the domain-local source shape
* to keep the profile-facing surface visible outside project manifests
* to mirror the project companions conceptually

Their current role is **not**:

* to serve as the canonical validation entrypoint for these domains

Today that validation role still belongs to the project-form companions.

## Current Showcase Routes

The current wider showcase projects are:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

Current role split:

* `window_controls_demo` is the current end-to-end `cpu + data + shader` route
  with explicit project links and render-profile shaping
* `kernel_tensor_demo` is the current end-to-end `cpu + data + kernel` route
  with explicit project links and kernel-profile shaping

These are the right place to read:

* multi-module domain interaction
* manifest `links`
* explicit ABI locking
* project-level lowering and artifact packaging

They are not the narrowest possible entrypoint for the domain profile surfaces.

## Current Reading Rule

The safe current reading order is:

1. read the current `YIR`/tooling truth first
2. read the source-shaped domain stub if you want the local source form
3. validate understanding through the project-form companion
4. move to the showcase project only after the narrow companion is clear

Concretely:

* shader:
  [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
  ->
  [shader_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_profile_demo.ns)
  ->
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
  ->
  surface branch
  ->
  packet branch
  ->
  bridge branch
  ->
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* kernel:
  [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
  ->
  [kernel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_profile_demo.ns)
  ->
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
  ->
  [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo)
  ->
  [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo)
  ->
  [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo)
  ->
  [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)
  ->
  [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)

## What This Contract Does Not Promise

This file does not promise that:

* `shader` and `kernel` will stay project-first forever
* source-shaped stubs will remain in `examples/ns/demos`
* a future `std` pure layer for these domains will never exist
* the current project companions already define the final package boundary for
  these lanes

It only captures the current repository truth about the safest readable and
verifiable route today.

## Current Guidance

If you are extending `shader` or `kernel` today:

* prefer adding the narrow project-form companion first
* keep explicit ABI loading visible in the project manifest
* use source-shaped stubs as mirrors, not as the primary validation route
* only propose a `std` pure-layer lane once the standalone bootstrap/lowering
  path is truly ready

If you are reading `shader` or `kernel` today:

* start with [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
  for current domain semantics
* use [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
  and
  [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo)
  and
  [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo)
  and
  [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo)
  and
  [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo)
  and
  [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo)
  and
  [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo)
  and
  [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo)
  and
  [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
  and
  [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo)
  and
  [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
  and
  [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo)
  and
  [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo)
  and
  [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo)
  and
  [shader_draw_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_draw_profile_demo)
  and
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)
  as the narrow checked-in shader validation route
* use [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
  and
  [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo)
  and
  [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo)
  and
  [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo)
  and
  [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)
  as the narrow checked-in kernel validation route
* move to the showcase projects only after the profile companion is clear

## Related References

* [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* [shader_surface_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_profile_demo)
* [shader_surface_material_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_profile_demo)
* [shader_surface_material_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_pass_profile_demo)
* [shader_surface_material_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_packet_profile_demo)
* [shader_surface_material_panel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_panel_profile_demo)
* [shader_surface_material_flow_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_material_flow_profile_demo)
* [shader_surface_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_packet_profile_demo)
* [shader_surface_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_surface_pass_profile_demo)
* [shader_packet_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_profile_demo)
* [shader_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_packet_bridge_demo)
* [shader_pass_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_pass_profile_demo)
* [shader_frame_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_frame_profile_demo)
* [shader_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_result_profile_demo)
* [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)
* [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
* [kernel_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_result_profile_demo)
* [kernel_tensor_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_profile_demo)
* [kernel_tensor_inspect_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_tensor_inspect_demo)
* [kernel_roundtrip_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_roundtrip_profile_demo)
* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
