# Domain Project Companions

This folder contains project-form companions for current non-CPU helper lanes
such as `shader`, `kernel`, and `network`.

This subtree is especially easy to let drift into “everything is important”.
It is better read as a set of ladders than as one giant inventory.

Current role rule:

* only the short profile/recipe ladders should be treated as frontdoor
* most deeper domain variants are companion-only compile coverage
* `*_probe_demo` routes are validation-only
* `*_experiment_*` routes are exploration-only

## Shared Helpers

Shared async helper modules used across shader/kernel/network companions live
under:

* [shared](shared)
* [shader_task_async_shapes.ns](shared/shader_task_async_shapes.ns)
* [kernel_task_async_shapes.ns](shared/kernel_task_async_shapes.ns)
* [network_task_async_shapes.ns](shared/network_task_async_shapes.ns)

## Start Here

If you only want the current front-door examples, start with:

* [shader_profile_demo](shader_profile_demo)
* [kernel_profile_demo](kernel_profile_demo)
* [network_profile_demo](network_profile_demo)

## Pick By Ladder

* shader profile and async ladder:
  [shader_profile_demo](shader_profile_demo),
  [shader_async_policy_profile_demo](shader_async_policy_profile_demo),
  [shader_async_windowed_batch_profile_demo](shader_async_windowed_batch_profile_demo)
* shader branch reading order:
  `profile -> surface branch -> packet branch -> bridge branch`
  current narrow branch anchors:
  [pixelmagic_profile_demo](pixelmagic_profile_demo),
  [pixelmagic_packet_bridge_demo](pixelmagic_packet_bridge_demo),
  [pixelmagic_texture_resource_demo](pixelmagic_texture_resource_demo),
  [pixelmagic_pipeline_demo](pixelmagic_pipeline_demo),
  [shader_surface_profile_demo](shader_surface_profile_demo),
  [shader_packet_profile_demo](shader_packet_profile_demo),
  [shader_pass_profile_demo](shader_pass_profile_demo)
* kernel profile and async tensor ladder:
  [kernel_profile_demo](kernel_profile_demo),
  [kernel_async_tensor_policy_profile_demo](kernel_async_tensor_policy_profile_demo),
  [kernel_async_tensor_windowed_batch_profile_demo](kernel_async_tensor_windowed_batch_profile_demo)
* network frontdoor recipe ladder:
  [net_http_request_recipe_demo](net_http_request_recipe_demo),
  [net_http_client_get_recipe_demo](net_http_client_get_recipe_demo),
  [net_http_client_lane_recipe_demo](net_http_client_lane_recipe_demo),
  [net_http_service_lane_recipe_demo](net_http_service_lane_recipe_demo),
  [net_http_session_loop_bridge_recipe_demo](net_http_session_loop_bridge_recipe_demo)
* network companion result/session ladder:
  [network_result_profile_demo](network_result_profile_demo),
  [network_result_task_policy_demo](network_result_task_policy_demo),
  [network_result_session_bridge_demo](network_result_session_bridge_demo)
* network companion transport ladder:
  [network_owned_transport_result_demo](network_owned_transport_result_demo),
  [network_transport_result_demo](network_transport_result_demo),
  [network_transport_result_session_bridge_demo](network_transport_result_session_bridge_demo)
* network validation-only cluster:
  [network_loopback_runtime_demo](network_loopback_runtime_demo),
  [network_host_handle_runtime_probe_demo](network_host_handle_runtime_probe_demo),
  [net_tcp_send_runtime_probe_demo](net_tcp_send_runtime_probe_demo),
  [net_tcp_socket_runtime_probe_demo](net_tcp_socket_runtime_probe_demo),
  [net_http_status_runtime_probe_demo](net_http_status_runtime_probe_demo),
  [net_http_client_runtime_probe_demo](net_http_client_runtime_probe_demo),
  [net_http_roundtrip_runtime_probe_demo](net_http_roundtrip_runtime_probe_demo)
* network exploration-only route:
  [net_protocol_experiment_recipe_demo](net_protocol_experiment_recipe_demo)

Practical network rule:

* start with the frontdoor recipe ladder
* only move into result/transport ladders when the implementation question is
  specifically about those contract families
* treat the validation-only cluster as host/runtime verification material
* treat the exploration-only route as design-space material, not current
  onboarding

Practical shader rule:

* start with `shader_profile_demo`
* only then choose one local branch:
  `surface branch` or `packet branch` or `bridge branch`
* treat the async branch as bridge-branch continuation, not as an independent
  first-stop ladder
* for future `PixelMagic` image-processing work, treat the current host-side
  closure as a prep path:
  `filesystem_io_report -> shader profile/render lanes`
* the first checked-in `PixelMagic` report-file workload that reuses the std
  host report lane is:
  [pixelmagic_report_file_demo](pixelmagic_report_file_demo)
* the first checked-in `PixelMagic` seed scaffold is:
  [pixelmagic_profile_demo](pixelmagic_profile_demo)
* the first checked-in `PixelMagic` packet consumer scaffold is:
  [pixelmagic_packet_bridge_demo](pixelmagic_packet_bridge_demo)
* the first checked-in `PixelMagic` texture-resource handoff scaffold is:
  [pixelmagic_texture_resource_demo](pixelmagic_texture_resource_demo)
* the first checked-in `PixelMagic` project-shaped pipeline scaffold is:
  [pixelmagic_pipeline_demo](pixelmagic_pipeline_demo)
* the first checked-in `PixelMagic` single-binary render scaffold is:
  [pixelmagic_render_demo](pixelmagic_render_demo)
* the first checked-in `WitSage` report-file workload that reuses the std host
  report lane is:
  [witsage_report_file_demo](witsage_report_file_demo)
* the current prep sketch for that future lane is:
  [galaxy-frontdoor-prep-sketch.md](../../../docs/reference/galaxy-frontdoor-prep-sketch.md)
* the current next-step texture handoff note is:
  [galaxy-texture-handoff-contract.md](../../../docs/reference/galaxy-texture-handoff-contract.md)

## Retired In Current Cleanup

The older zero-reference `multidomain_*` and broad `net_*` recipe swarm that
no longer carried current doc, script, or mainline-map responsibility has been
retired from the checked-in examples tree.

## Reading Rule

* do not start by reading the whole network subtree
* pick one ladder that matches the surface you are touching
* treat `*_probe_demo` and `*_experiment_*` routes as validation or exploration
  material, not as default front-door reading
* treat most shader/kernel/network variants beyond the named frontdoor ladders
  as companion-only unless they are the exact lane you are implementing
* for network specifically:
  `recipe ladder -> result/transport companions -> validation-only cluster -> exploration-only route`
* many domain companions are intentionally narrow compile anchors
* for repo-level routing, prefer
  [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
