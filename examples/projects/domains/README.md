# Domain Project Companions

This folder contains project-form companions for current non-CPU helper lanes
such as `shader`, `kernel`, and `network`.

This subtree is especially easy to let drift into “everything is important”.
It is better read as a set of ladders than as one giant inventory.

## Shared Helpers

Shared async helper modules used across shader/kernel/network companions live
under:

* [shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared)
* [shader_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/shader_task_async_shapes.ns)
* [kernel_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/kernel_task_async_shapes.ns)
* [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)

## Start Here

If you only want the current front-door examples, start with:

* [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
* [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)

## Pick By Ladder

* shader profile and async ladder:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo),
  [shader_async_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_policy_profile_demo),
  [shader_async_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_async_windowed_batch_profile_demo)
* kernel profile and async tensor ladder:
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo),
  [kernel_async_tensor_policy_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_policy_profile_demo),
  [kernel_async_tensor_windowed_batch_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_async_tensor_windowed_batch_profile_demo)
* multidomain orchestration ladder:
  [multidomain_profile_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/multidomain_profile_probe_demo),
  [multidomain_async_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/multidomain_async_probe_demo),
  [multidomain_async_orchestration_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/multidomain_async_orchestration_demo)
* network recipe/front-door ladder:
  [net_endpoint_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_endpoint_recipe_demo),
  [net_http_request_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_recipe_demo),
  [net_http_client_get_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_get_recipe_demo),
  [net_http_service_lane_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_service_lane_recipe_demo)
* network runtime validation probes:
  [network_loopback_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_loopback_runtime_demo),
  [network_host_handle_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_probe_demo),
  [net_tcp_send_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_send_runtime_probe_demo),
  [net_tcp_socket_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_runtime_probe_demo),
  [net_http_status_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_status_runtime_probe_demo),
  [net_http_client_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_runtime_probe_demo)
* network result/session ladder:
  [network_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_profile_demo),
  [network_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_policy_demo),
  [network_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_session_bridge_demo)
* network transport ladder:
  [network_owned_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_demo),
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo),
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)
* HTTP/session bridge anchor:
  [net_http_session_loop_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo)
* exploratory protocol route:
  [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo)

## Reading Rule

* do not start by reading the whole network subtree
* pick one ladder that matches the surface you are touching
* treat `*_probe_demo` and `*_experiment_*` routes as validation or exploration
  material, not as default front-door reading
* many domain companions are intentionally narrow compile anchors
* for repo-level routing, prefer
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
