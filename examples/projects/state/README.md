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
* [generic_method_bound_payload_alias_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_payload_alias_demo)

This is the shortest user-facing path for:

* alias-wrapped generic receivers
* `T: Addable` method calls
* binding visibility through `if`
* binding visibility through nested `match`
* guard-preserving nested `match`

Pattern / destructuring route:

* [generic_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_struct_state_demo)
* [generic_alias_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_alias_struct_state_demo)
* [generic_param_alias_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_param_alias_struct_state_demo)
* [generic_struct_match_shorthand_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_struct_match_shorthand_state_demo)
* [generic_nested_shorthand_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_nested_shorthand_state_demo)
* [generic_nested_alias_shorthand_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_nested_alias_shorthand_state_demo)
* [generic_payload_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_struct_state_demo)
* [generic_alias_payload_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_alias_payload_struct_state_demo)
* [destructure_let_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/destructure_let_state_demo)
* [destructure_nested_shorthand_let_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/destructure_nested_shorthand_let_state_demo)
* [match_payload_struct_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_payload_struct_state_demo)
* [match_struct_binding_shorthand_guard_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_struct_binding_shorthand_guard_state_demo)
* [match_unit_struct_guard_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_unit_struct_guard_state_demo)

Recursion / higher-order route:

* [ordinary_mutual_recursive_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_mutual_recursive_demo)
* [ordinary_recursive_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_call_graph_demo)
* [ordinary_recursive_i32_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_i32_call_graph_demo)
* [ordinary_recursive_bool_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_bool_call_graph_demo)
* [ordinary_recursive_match_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_match_call_graph_demo)
* [ordinary_recursive_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_higher_order_call_graph_demo)
* [ordinary_recursive_fn2_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_fn2_higher_order_call_graph_demo)
* [ordinary_recursive_generic_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_higher_order_call_graph_demo)
* [ordinary_recursive_generic_fn2_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn2_higher_order_call_graph_demo)
* [ordinary_recursive_generic_fn3_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn3_higher_order_call_graph_demo)
* [ordinary_recursive_generic_alias_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_higher_order_call_graph_demo)
* [ordinary_recursive_composed_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_composed_call_graph_demo)
* [ordinary_recursive_lambda_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_lambda_call_graph_demo)
* [ordinary_recursive_mixed_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_mixed_call_graph_demo)
* [ordinary_recursive_generic_composed_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_composed_call_graph_demo)
* [ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo)
* [tail_recursive_sum_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_sum_demo)
* [tail_recursive_factorial_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_demo)
* [tail_recursive_cross_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_cross_carry_demo)
* [tail_recursive_branching_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_demo)
* [tail_recursive_multi_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_multi_carry_demo)
* [tail_recursive_carry_condition_multi_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_carry_condition_multi_carry_demo)
* [tail_recursive_branching_cross_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo)
* [tail_recursive_branching_multi_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_multi_carry_demo)
* [generic_callable_forwarding_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_callable_forwarding_hof_demo)
* [lambda_alias_fn3_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_alias_fn3_demo)
* [generic_payload_alias_higher_order_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_higher_order_demo)
* [generic_payload_alias_method_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_method_hof_demo)
* [generic_lambda_method_bound_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_lambda_method_bound_hof_demo)
* [generic_lambda_method_bound_fn3_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_lambda_method_bound_fn3_hof_demo)

This route now also carries:

* ordinary recursion through composed helper lanes
* ordinary recursion through lambda-mediated helper lanes
* ordinary recursion through named higher-order helper lanes
* ordinary recursion through plain scalar and `i32` helper lanes
* ordinary recursion through `Fn2` higher-order helper lanes
* ordinary recursion through pure bool helper lanes
* mixed scalar/bool recursive helper truth
* specialized generic `Fn2` / `Fn3` helper truth
* specialized generic higher-order helper truth for both direct and alias call surfaces
* specialized generic higher-order recursive helper truth
* generic callable forwarding through `Fn2` / `Fn3`
* nested `relay -> chain -> apply` helper specialization
* explicit generic arguments on project-shaped higher-order calls
* project-backed tail recursion lowering into `loop_while_i64_chain`
* project-backed tail recursion lowering into `loop_while_i64_cond_chain`
* project-backed multi-carry and cross-carry tail recursion truth
* project-backed branching multi-carry tail recursion truth

Loop / lowering route:

* [counted_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_demo)
* [accumulating_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/accumulating_while_demo)
* [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo)
* [bounded_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/bounded_while_demo)
* [equality_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_while_demo)
* [inequality_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/inequality_while_demo)
* [guarded_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/guarded_while_demo)
* [match_guarded_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guarded_while_demo)
* [branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/branching_while_demo)
* [match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo)
* [match_expr_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_expr_branching_while_demo)
* [bool_match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/bool_match_branching_while_demo)
* [lambda_match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_branching_while_demo)
* [flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo)
* [equality_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_branching_while_demo)
* [flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo)
* [lambda_match_flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_flow_continuing_while_demo)
* [lambda_match_or_flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_or_flow_continuing_while_demo)
* [post_flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_while_demo)
* [post_flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_continuing_while_demo)
* [post_flow_breaking_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo)
* [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
* [carried_breaking_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/carried_breaking_while_demo)
* [double_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/double_branching_while_demo)
* [tail_recursive_branching_cross_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo)

This route now also carries:

* project-backed basic `loop_while_i64` lowering truth
* project-backed single-carry `loop_while_i64_chain` lowering truth
* project-backed multi-carry `loop_while_i64_chain` lowering truth
* project-backed bounded `post_flow_chain` lowering truth
* project-backed equality-triggered `post_flow_chain` lowering truth
* project-backed inequality-driven basic loop lowering truth
* project-backed `match`-driven `cond_chain` lowering truth
* project-backed guarded `match` inside `while` return-shape truth
* project-backed plain branching `cond_chain` lowering truth
* project-backed expression-scrutinee `match` inside `while` lowering truth
* project-backed bool-scrutinee `match` inside `while` lowering truth
* project-backed lambda-driven `match` inside `while` `cond_chain` lowering truth
* project-backed `flow_cond_chain` lowering truth
* project-backed equality-driven `flow_cond_chain` lowering truth
* project-backed plain `continue` `flow_cond_chain` lowering truth
* project-backed lambda-driven `match` + `continue` `flow_cond_chain` lowering truth
* project-backed lambda `or`-composed `flow_cond_chain` lowering truth
* project-backed carried `break` `flow_chain` lowering truth
* project-backed double-branch carried `cond_chain` lowering truth
* project-backed plain `continue` `post_flow_chain` lowering truth
* project-backed plain `break` `post_flow_chain` lowering truth
* project-backed `post_flow_cond_chain` lowering truth for both `break` and
  `continue`
* one checked-in state compile gate for ordinary structured while loops, not
  only lowering-local snippet probes

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
