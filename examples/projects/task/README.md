# Task Project Companions

This folder contains narrow project-form companions for the current task-facing
`std` line.

These are not all equal-entry examples. Many are intentionally narrow compile
or regression anchors.

## Start Here

If you only want the current task mainline, start with:

* [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo)
  smallest task/runtime anchor
* [task_thread_mutex_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_thread_mutex_demo)
  current staged `Thread<T>` / `Mutex<T>` helper/facade project anchor
* [task_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo)
  current recursive async anchor
* [task_recursive_async_shared_suffix_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_shared_suffix_demo)
  recursive async anchor with branch-selected value plus shared suffix
* [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo)
  current result-family branching anchor
* [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo)
  current observe/status surface anchor

## Pick By Goal

* recursion and async shape:
  [task_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo),
  [task_recursive_async_shared_suffix_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_shared_suffix_demo),
  [task_mutual_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_mutual_recursive_async_demo),
  [task_generic_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo),
  [task_generic_mutual_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_mutual_recursive_async_demo),
  [task_recursive_async_result_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_result_family_demo),
  [task_recursive_async_payload_alias_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_payload_alias_hof_demo),
  [task_async_observer_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo)
* async/control-flow crossover:
  [task_async_if_expression_positions_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_if_expression_positions_demo),
  [task_async_await_match_operand_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_await_match_operand_demo),
  [task_async_match_call_argument_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_match_call_argument_demo),
  [task_async_struct_field_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_struct_field_match_demo),
  [task_async_method_receiver_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_method_receiver_match_demo),
  [task_async_helper_expanded_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_helper_expanded_match_demo),
  [task_async_post_flow_shared_suffix_loop_control_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo),
  [task_async_while_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo),
  [task_async_while_post_flow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo),
  [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo),
  [task_async_while_post_flow_compound_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo)
* result and branch control:
  [task_result_family_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo),
  [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo),
  [task_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo),
  [task_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo),
  [task_result_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo),
  [task_result_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo)
* timeout, fallback, lifecycle:
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo),
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo),
  [task_fallback_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo),
  [task_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_policy_branch_demo)
* memory/session payload routes:
  [task_memory_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_roundtrip_demo),
  [task_memory_result_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_branch_demo),
  [task_memory_result_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_batch_demo),
  [task_memory_session_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo),
  [task_memory_session_packet_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_packet_demo)
* http-like session routes:
  [task_httpish_response_packet_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_packet_demo),
  [task_httpish_response_slots_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_slots_demo),
  [task_httpish_session_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_session_policy_demo),
  [task_httpish_header_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_header_session_demo)
* observe/clock routes:
  [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo),
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo),
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo),
  [task_clock_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo)
* task/tooling bridge:
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo)
* staged thread/lock route:
  [task_thread_mutex_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_thread_mutex_demo)
  current note:
  explicit project smoke test is checked in and now runs through the staged
  AOT thread/lock lowering path, including generic helper-style
  `mutex_snapshot<T>` / `join_thread_*<T>` wrappers
* future/probe route:
  [task_join_nonconsuming_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_join_nonconsuming_probe_demo)

## Reading Rule

Use this folder like a companion matrix, not like a linear tutorial.

* pick one representative example for the feature you care about
* do not start by reading every neighboring branch demo
* treat `task_join_nonconsuming_probe_demo` as forward-looking probe material,
  not as current front-door reading
* if you want the repo-level shortest route, prefer
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* if you want the project-level route, prefer
  [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
