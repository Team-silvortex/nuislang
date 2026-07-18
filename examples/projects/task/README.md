# Task Project Companions

This folder contains narrow project-form companions for the current task-facing
`std` line.

These are not all equal-entry examples. Many are intentionally narrow compile
or regression anchors.

Current role rule:

* this subtree is companion-first, not showcase-first
* only a short frontdoor slice should be treated as current onboarding
* most branch/payload/session variants below are best read as
  companion-only coverage unless they are the strongest current proof for a
  specific task contract

## Start Here

If you only want the current task mainline, start with:

* [task_runtime_demo](task_runtime_demo)
  smallest task/runtime anchor
* [task_thread_mutex_demo](task_thread_mutex_demo)
  current staged `Thread<T>` / `Mutex<T>` helper/facade project anchor
* [task_recursive_async_demo](task_recursive_async_demo)
  current recursive async anchor
* [task_recursive_async_shared_suffix_demo](task_recursive_async_shared_suffix_demo)
  recursive async anchor with branch-selected value plus shared suffix
* [task_result_policy_branch_demo](task_result_policy_branch_demo)
  current result-family branching anchor
* [task_status_observe_demo](task_status_observe_demo)
  current observe/status surface anchor

## Pick By Goal

* recursion and async shape:
  [task_recursive_async_demo](task_recursive_async_demo),
  [task_recursive_async_shared_suffix_demo](task_recursive_async_shared_suffix_demo),
  [task_mutual_recursive_async_demo](task_mutual_recursive_async_demo),
  [task_generic_recursive_async_demo](task_generic_recursive_async_demo),
  [task_generic_mutual_recursive_async_demo](task_generic_mutual_recursive_async_demo),
  [task_recursive_async_result_family_demo](task_recursive_async_result_family_demo),
  [task_recursive_async_payload_alias_hof_demo](task_recursive_async_payload_alias_hof_demo),
  [task_async_observer_bridge_demo](task_async_observer_bridge_demo)
* async/control-flow crossover:
  [task_async_if_expression_positions_demo](task_async_if_expression_positions_demo),
  [task_async_await_match_operand_demo](task_async_await_match_operand_demo),
  [task_async_match_call_argument_demo](task_async_match_call_argument_demo),
  [task_async_struct_field_match_demo](task_async_struct_field_match_demo),
  [task_async_method_receiver_match_demo](task_async_method_receiver_match_demo),
  [task_async_helper_expanded_match_demo](task_async_helper_expanded_match_demo),
  [task_async_post_flow_shared_suffix_loop_control_demo](task_async_post_flow_shared_suffix_loop_control_demo),
  [task_async_while_flow_cond_demo](task_async_while_flow_cond_demo),
  [task_async_while_post_flow_demo](task_async_while_post_flow_demo),
  [task_async_while_post_flow_cond_demo](task_async_while_post_flow_cond_demo),
  [task_async_while_post_flow_compound_demo](task_async_while_post_flow_compound_demo)
* result and branch control:
  [task_result_family_branch_demo](task_result_family_branch_demo),
  [task_result_policy_branch_demo](task_result_policy_branch_demo),
  [task_batch_branch_demo](task_batch_branch_demo),
  [task_windowed_batch_branch_demo](task_windowed_batch_branch_demo),
  [task_result_batch_branch_demo](task_result_batch_branch_demo),
  [task_result_windowed_batch_branch_demo](task_result_windowed_batch_branch_demo)
* timeout, fallback, lifecycle:
  [task_lifecycle_branch_demo](task_lifecycle_branch_demo),
  [task_ready_delay_ordering_demo](task_ready_delay_ordering_demo),
  [task_cancel_branch_demo](task_cancel_branch_demo),
  [task_fallback_branch_demo](task_fallback_branch_demo),
  [task_policy_branch_demo](task_policy_branch_demo)
* memory/session payload routes:
  [task_memory_roundtrip_demo](task_memory_roundtrip_demo),
  [task_memory_result_branch_demo](task_memory_result_branch_demo),
  [task_memory_result_batch_demo](task_memory_result_batch_demo),
  [task_memory_session_policy_demo](task_memory_session_policy_demo),
  [task_memory_session_packet_demo](task_memory_session_packet_demo)
* http-like session routes:
  [task_httpish_response_packet_demo](task_httpish_response_packet_demo),
  [task_httpish_response_slots_demo](task_httpish_response_slots_demo),
  [task_httpish_session_policy_demo](task_httpish_session_policy_demo),
  [task_httpish_header_session_demo](task_httpish_header_session_demo)
* observe/clock routes:
  [task_status_observe_demo](task_status_observe_demo),
  [task_completed_observe_demo](task_completed_observe_demo),
  [task_context_arity_demo](task_context_arity_demo),
  [task_scalar_context_demo](task_scalar_context_demo),
  [task_float_context_demo](task_float_context_demo),
  [task_compare_observe_demo](task_compare_observe_demo),
  [task_clock_observe_demo](task_clock_observe_demo),
  [task_scheduler_observe_demo](task_scheduler_observe_demo)
* task/tooling bridge:
  [task_cli_tooling_demo](task_cli_tooling_demo),
  [std_language_task_cli_demo](std_language_task_cli_demo)
  as the std-language-assisted task CLI bridge that feeds
  `StdLanguageOps.build_report` into `StdTaskContracts` and a real stdout path
* staged thread/lock route:
  [task_thread_mutex_demo](task_thread_mutex_demo)
  current note:
  explicit project smoke test is checked in and now runs through the staged
  AOT thread/lock lowering path, including generic helper-style
  `mutex_snapshot<T>` / `join_thread_*<T>` wrappers
* probe-only route:
  [task_join_nonconsuming_probe_demo](task_join_nonconsuming_probe_demo)

## Reading Rule

Use this folder like a companion matrix, not like a linear tutorial.

* pick one representative example for the feature you care about
* do not start by reading every neighboring branch demo
* treat most `*_branch_*`, `*_batch_*`, and `*_session_*` variants as
  companion-only unless you are actively working on that contract family
* treat `task_join_nonconsuming_probe_demo` as forward-looking probe material,
  not as current front-door reading
* if you want the repo-level shortest route, prefer
  [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
* if you want the project-level route, prefer
  [examples/projects/README.md](../../../examples/projects/README.md)
