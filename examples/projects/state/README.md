# State And Persistence Project Companions

This folder is the project-form companion area for:

* stateful control flow
* recursion and loop lowering
* pattern matching and destructuring
* generic method-bound validation routes
* small runtime location/config/cache probes
* current GLM/state ownership probes

Use this README as a router, not as a full inventory.

## Start Here

If you want the shortest current route, read:

* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)

Then use the focused clusters below.

## Focused Clusters

Generic method-bound route:

* [generic_method_bound_if_binding_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_if_binding_demo)
* [generic_method_bound_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_nested_match_demo)
* [generic_method_bound_guarded_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_guarded_nested_match_demo)

This is the shortest user-facing path for:

* alias-wrapped generic receivers
* `T: Addable` method calls
* binding visibility through `if`
* binding visibility through nested `match`
* guard-preserving nested `match`

Pattern / destructuring route:

* [destructure_let_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/destructure_let_state_demo)
* [destructure_nested_shorthand_let_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/destructure_nested_shorthand_let_state_demo)
* [match_payload_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_payload_struct_state_demo)
* [match_struct_binding_shorthand_guard_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_struct_binding_shorthand_guard_state_demo)
* [match_unit_struct_guard_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_unit_struct_guard_state_demo)

Recursion / higher-order route:

* [ordinary_mutual_recursive_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_mutual_recursive_demo)
* [ordinary_recursive_composed_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_composed_call_graph_demo)
* [ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo)
* [lambda_alias_fn3_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_alias_fn3_demo)

Loop / lowering route:

* [counted_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_demo)
* [guarded_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/guarded_while_demo)
* [flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo)
* [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
* [tail_recursive_branching_cross_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo)

GLM / state ownership route:

* [glm_borrow_end_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/glm_borrow_end_state_demo)
* [glm_buffer_roundtrip_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/glm_buffer_roundtrip_state_demo)

Runtime location/config route:

* [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cwd_runtime_demo)
* [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/home_runtime_demo)
* [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/location_runtime_demo)
* [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_runtime_demo)
* [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_cache_demo)
* [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/kv_runtime_demo)
* [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cache_runtime_demo)
* [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/temp_runtime_demo)

## Reading Rule

If you are exploring broadly:

* use the clusters above first
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md) for repo-level shortest paths
* browse the folder directly for wider sibling probes once you already know the cluster you care about
